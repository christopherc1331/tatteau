use ammonia::Builder;
use async_openai::types::{
    ChatCompletionRequestMessage, ChatCompletionRequestSystemMessage,
    ChatCompletionRequestSystemMessageContent, ChatCompletionRequestUserMessage,
    ChatCompletionRequestUserMessageContent, CreateChatCompletionRequestArgs,
};
use async_openai::{config::OpenAIConfig, Client};
use chrono::Utc;
use futures::stream::{FuturesUnordered, StreamExt};
use indicatif::{ProgressBar, ProgressStyle};
use regex::Regex;
use reqwest::Client as HttpClient;
use sqlx::{PgPool, Row};
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::env;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};
use tokio;
use url::Url;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
#[serde(tag = "action")]
enum GptAction {
    Navigate { url: String },
    Extract,
    Done,
}

#[derive(Debug, Serialize, Deserialize)]
struct Artist {
    name: Option<String>,
    styles: Option<Vec<String>>,
    social_links: Option<String>,
    email: Option<String>,
    phone: Option<String>,
    years_experience: Option<u8>,
}

fn extract_body_html(raw_html: &str) -> String {
    let doc = Html::parse_document(raw_html);
    let selector = Selector::parse("body").unwrap();
    doc.select(&selector)
        .next()
        .map(|el| el.html())
        .unwrap_or_default()
}

fn sanitize_html(html: &str) -> String {
    Builder::new()
        .tags(["p", "a", "h1", "h2", "h3", "ul", "li", "div", "span", "img"].into())
        .clean(html)
        .to_string()
}

fn preprocess_html(raw_html: &str) -> String {
    let body = extract_body_html(raw_html);
    sanitize_html(&body)
}

fn normalize_style(style: &str) -> String {
    let re = Regex::new(r"[^a-zA-Z0-9]+").unwrap();
    re.replace_all(style.to_lowercase().trim(), " ").to_string()
}

fn get_domain(url_str: &str) -> anyhow::Result<String> {
    let url = Url::parse(url_str)?;
    Ok(url.host_str().unwrap_or("").to_string())
}

async fn log_scrape_action(pool: &PgPool, location_id: i64, action: &str) -> anyhow::Result<()> {
    let timestamp = Utc::now().to_rfc3339();
    sqlx::query(
        "INSERT INTO scrape_actions (location_id, action, timestamp) VALUES ($1, $2, $3)"
    )
    .bind(location_id)
    .bind(action)
    .bind(timestamp)
    .execute(pool)
    .await?;
    Ok(())
}

async fn get_existing_artists(pool: &PgPool, location_id: i64) -> anyhow::Result<Vec<String>> {
    let rows = sqlx::query("SELECT name FROM artists WHERE location_id = $1")
        .bind(location_id)
        .fetch_all(pool)
        .await?;

    let artists: Vec<String> = rows
        .into_iter()
        .map(|row| row.get("name"))
        .collect();

    Ok(artists)
}

async fn persist_artist_and_styles(
    pool: &PgPool,
    artist: &Artist,
    location_id: i64,
) -> anyhow::Result<()> {
    if let Some(name) = &artist.name {
        let socials = artist.social_links.clone().unwrap_or_default();
        let email = artist.email.clone().unwrap_or_default();
        let phone = artist.phone.clone().unwrap_or_default();
        let years_experience = artist.years_experience.unwrap_or(0) as i64;

        let row = sqlx::query(
            "INSERT INTO artists (name, social_links, email, phone, years_experience, location_id) VALUES ($1, $2, $3, $4, $5, $6) RETURNING id"
        )
        .bind(name)
        .bind(socials)
        .bind(email)
        .bind(phone)
        .bind(years_experience)
        .bind(location_id)
        .fetch_one(pool)
        .await?;

        let artist_id: i64 = row.get("id");

        if let Some(styles) = &artist.styles {
            for raw_style in styles {
                let style_name = normalize_style(raw_style);

                // Try to get existing style_id, or insert new style
                let style_id = match sqlx::query(
                    "SELECT id FROM styles WHERE LOWER(name) = LOWER($1)"
                )
                .bind(&style_name)
                .fetch_optional(pool)
                .await?
                {
                    Some(row) => row.get::<i64, _>("id"),
                    None => {
                        let row = sqlx::query(
                            "INSERT INTO styles (name) VALUES ($1) RETURNING id"
                        )
                        .bind(&style_name)
                        .fetch_one(pool)
                        .await?;
                        row.get("id")
                    }
                };

                sqlx::query(
                    "INSERT INTO artists_styles (artist_id, style_id) VALUES ($1, $2)"
                )
                .bind(artist_id)
                .bind(style_id)
                .execute(pool)
                .await?;
            }
        }
    }
    Ok(())
}

async fn handle_gpt_decision(
    decision: &GptAction,
    pool: &PgPool,
    gpt_client: &Client<OpenAIConfig>,
    cleaned_html: &str,
    location_id: i64,
    base_url: &str,
    artists_counter: &Arc<AtomicUsize>,
) -> anyhow::Result<(bool, Option<String>)> {
    match decision {
        GptAction::Navigate { url } => {
            let base_domain = get_domain(base_url)?;
            let nav_domain = get_domain(url)?;

            if base_domain != nav_domain {
                println!(
                    "Navigation rejected: {} not in same domain as {}",
                    url, base_url
                );
                return Ok((false, None));
            }

            println!("Navigating to: {}", url);
            let _ = log_scrape_action(pool, location_id, &format!("navigate:{}", url)).await;
            return Ok((false, Some(url.clone())));
        }
        GptAction::Extract => {
            let existing_artists = get_existing_artists(pool, location_id).await.unwrap_or_default();

            let artists =
                call_gpt_extract(gpt_client, base_url, cleaned_html, &existing_artists).await?;
            if artists.is_empty() {
                println!("⚠️  No new artists found on this page");
                let _ = log_scrape_action(pool, location_id, "extract:no_new_artists").await;
            } else {
                println!("💾 Saving {} new artist(s) to database", artists.len());
                artists_counter.fetch_add(artists.len(), Ordering::SeqCst);
                for artist in &artists {
                    persist_artist_and_styles(pool, artist, location_id).await?;
                }
                let _ = log_scrape_action(
                    pool,
                    location_id,
                    &format!("extract:new_artists_found:{}", artists.len()),
                ).await;
            }
        }
        GptAction::Done => {
            println!(
                "✅ Completed processing location ID: {} (marked as done)",
                location_id
            );
            let _ = log_scrape_action(pool, location_id, "done").await;
            sqlx::query("UPDATE locations SET is_scraped = 1 WHERE id = $1")
                .bind(location_id)
                .execute(pool)
                .await?;
            return Ok((true, None));
        }
    }
    Ok((false, None))
}

async fn fetch_html(http_client: &HttpClient, url: &str) -> anyhow::Result<String> {
    println!("Fetching HTML from: {}", url);
    let response = http_client
        .get(url)
        .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36")
        .timeout(std::time::Duration::from_secs(15))
        .send()
        .await?;

    if !response.status().is_success() {
        return Err(anyhow::anyhow!("HTTP error: {}", response.status()));
    }

    let html = response.text().await?;
    Ok(html)
}

async fn call_gpt_action(
    client: &Client<OpenAIConfig>,
    url: &str,
    cleaned_html: &str,
    visited: &HashSet<String>,
) -> anyhow::Result<GptAction> {
    println!("🤖 Calling GPT for action decision on: {}", url);
    let sys_msg = ChatCompletionRequestMessage::System(ChatCompletionRequestSystemMessage {
        content: ChatCompletionRequestSystemMessageContent::Text("You're an autonomous web agent that analyzes tattoo shop websites to find artist data.

        BUSINESS TYPE VALIDATION:
        - FIRST: Determine if this appears to be a tattoo shop/parlor website
        - Look for tattoo-related keywords: 'tattoo', 'ink', 'tattoo artist', 'body art', 'custom tattoos'
        - If this doesn't appear to be a tattoo-related business, respond with DONE immediately
        
        EXTRACTION PRIORITY (only for tattoo shops):
        1. Look for artist information on the current page (names, styles, contact info)
        2. If you find artist data, choose EXTRACT
        3. If no artist data found, look for navigation links that might lead to artist info
        
        NAVIGATION RESTRICTIONS:
        - Only navigate to URLs within the same domain as the current page
        - Look for links like 'Artists', 'Our Team', 'Meet the Staff', 'Gallery', 'About Us'
        - Consider link context and surrounding text when choosing navigation
        - Do not navigate to external domains, social media, or third-party sites
        - Do not revisit urls that have already been visited
        
        WHEN TO CHOOSE DONE:
        - If you've visited 3+ pages without finding clear artist profiles or contact information
        - If the current page has no promising navigation links to artist-related content
        - If you've extracted artist data from previous pages and current page adds nothing new
        - If the site structure suggests no dedicated artist information exists
        
        Always validate business type first, then prioritize extraction over navigation, and be decisive about finishing.".to_string()),
        name: None,
    });

    let user_msg = ChatCompletionRequestMessage::User(ChatCompletionRequestUserMessage {
        content: ChatCompletionRequestUserMessageContent::Text(format!(
            r#"URL: {}
            HTML:
            {}
            Visited urls: {}

            Respond with a JSON object in one of the following formats:
            
            To navigate to a new page (must be same domain, only if no artist data found):
            {{ "action": "NAVIGATE", "url": "https://example.com/path" }}
            
            To extract artist data from current page:
            {{ "action": "EXTRACT" }}
            
            When finished with this location:
            {{ "action": "DONE" }}"#,
            url,
            cleaned_html,
            visited
                .iter()
                .map(String::as_str)
                .collect::<Vec<&str>>()
                .join(",")
        )),
        name: None,
    });

    let req = CreateChatCompletionRequestArgs::default()
        .model("o4-mini")
        .messages([sys_msg, user_msg])
        .max_completion_tokens(1000u32)
        .build()?;

    let res = client.chat().create(req).await?;
    let text = res.choices[0].message.content.as_ref().unwrap();

    let action: GptAction = serde_json::from_str(text.trim())?;
    println!("✅ GPT decision: {:?}", action);
    Ok(action)
}

async fn call_gpt_extract(
    client: &Client<OpenAIConfig>,
    url: &str,
    cleaned_html: &str,
    existing_artists: &[String],
) -> anyhow::Result<Vec<Artist>> {
    println!("🎯 Extracting artist data from: {}", url);
    let sys_msg = ChatCompletionRequestMessage::System(ChatCompletionRequestSystemMessage {
        content: ChatCompletionRequestSystemMessageContent::Text("Extract tattoo artist data from the HTML content. Return only valid artist entries in JSON format.".to_string()),
        name: None,
    });

    let existing_artists_text = if existing_artists.is_empty() {
        "None".to_string()
    } else {
        existing_artists.join(", ")
    };

    let user_msg = ChatCompletionRequestMessage::User(ChatCompletionRequestUserMessage {
        content: ChatCompletionRequestUserMessageContent::Text(format!(
            r#"URL: {}
            HTML:
            {}

            EXISTING ARTISTS FOR THIS LOCATION: {}

            Extract artist data with fields:
            - name (string)
            - styles (array of strings)
            - email (optional string)
            - phone (optional string)
            - social_links (optional string, comma-delimited)
            - years_experience (optional number)

            IMPORTANT: Only return artists that are NOT already in the existing artists list above. 
            Skip any artists whose names are already present. 
            SUPER IMPORTANT:
            ONLY ASSIGN STYLES TO THE ARTIST THAT DOES THAT STYLE, DO NOT ASSIGN ALL STYLES TO THE PAGE TO ALL ARTISTS ON THE PAGE. IF YOU CANNOT TELL WHICH STYLE BELONGS TO WHICH ARTIST, THEN PUT NO STYLES FOR SAID ARTIST. NO STYLES ARE BETTER THAN INCORRECT ONES.

            Respond with a JSON array of artist objects."#,
            url, cleaned_html, existing_artists_text
        )),
        name: None,
    });

    let req = CreateChatCompletionRequestArgs::default()
        .model("o4-mini")
        .messages([sys_msg, user_msg])
        .max_completion_tokens(1500u32)
        .build()?;

    let res = client.chat().create(req).await?;
    let text = res.choices[0].message.content.as_ref().unwrap();

    let artists: Vec<Artist> = serde_json::from_str(text.trim())?;
    println!("📝 Extracted {} artists", artists.len());
    Ok(artists)
}

pub async fn scrape(pool: &PgPool) -> Result<(), Box<dyn std::error::Error>> {
    let openai_key = env::var("OPENAI_API_KEY")?;
    let num_threads: usize = env::var("NUM_THREADS").unwrap_or("12".into()).parse()?;
    let max_scrapes: usize = env::var("MAX_SCRAPES").unwrap_or("100".into()).parse()?;
    let max_page_visits: usize = env::var("MAX_PAGE_VISITS").unwrap_or("5".into()).parse()?;

    println!("🚀 Starting tattoo shop scraper");
    println!("📊 Configuration:");
    println!("   • Threads: {}", num_threads);
    println!("   • Max scrapes: {}", max_scrapes);
    println!("   • Max page visits per location: {}", max_page_visits);

    let config = OpenAIConfig::new().with_api_key(openai_key);
    let client = Arc::new(Client::with_config(config));
    let http_client = Arc::new(HttpClient::new());

    let scrape_limit = Arc::new(AtomicUsize::new(0));
    let artists_added = Arc::new(AtomicUsize::new(0));
    let progress = Arc::new(ProgressBar::new(max_scrapes as u64));
    progress.set_style(
        ProgressStyle::default_bar()
            .template(
                "{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta})",
            )
            .unwrap()
            .progress_chars("##-"),
    );

    let locations: Vec<(i64, String)> = {
        let rows = sqlx::query(
            "UPDATE locations SET is_scraped = -2 WHERE id IN (
                SELECT id FROM locations
                WHERE is_scraped = 0
                AND website_uri IS NOT NULL
                AND website_uri != ''
                AND website_uri NOT LIKE '%facebook%'
                AND website_uri NOT LIKE '%instagram%'
                LIMIT $1
            ) RETURNING id, website_uri"
        )
        .bind(max_scrapes as i32)
        .fetch_all(pool)
        .await?;

        rows.into_iter()
            .map(|row| (row.get::<i64, _>("id"), row.get::<String, _>("website_uri")))
            .collect()
    };

    println!("📍 Claimed {} locations for processing", locations.len());

    let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
    let rx = Arc::new(tokio::sync::Mutex::new(rx));

    for location in locations {
        tx.send(location).unwrap();
    }
    drop(tx);

    let mut tasks = FuturesUnordered::new();
    for thread_id in 0..num_threads {
        let pool = pool.clone();
        let client = Arc::clone(&client);
        let http_client = Arc::clone(&http_client);
        let scrape_counter = Arc::clone(&scrape_limit);
        let artists_counter = Arc::clone(&artists_added);
        let pb: Arc<ProgressBar> = Arc::clone(&progress);
        let rx = Arc::clone(&rx);

        tasks.push(tokio::spawn(async move {
            loop {
                let (id, url) = match rx.lock().await.recv().await {
                    Some(location) => location,
                    None => break,
                };

                println!(
                    "🏪 Thread {} processing location ID: {} - {}",
                    thread_id, id, url
                );

                let _ = log_scrape_action(&pool, id, &format!("start:{}", url)).await;

                let url = if !url.starts_with("http://") && !url.starts_with("https://") {
                    format!("https://{}", url)
                } else {
                    url
                };

                let mut current_url = url;
                let base_url = current_url.clone();

                let mut visited: HashSet<String> = HashSet::new();
                let mut location_completed = false;
                for _ in 0..max_page_visits {
                    match fetch_html(&http_client, &current_url).await {
                        Ok(raw_html) => {
                            visited.insert(current_url.clone());
                            let cleaned_html = preprocess_html(&raw_html);
                            match call_gpt_action(&client, &current_url, &cleaned_html, &visited)
                                .await
                            {
                                Ok(decision) => {
                                    match handle_gpt_decision(
                                        &decision,
                                        &pool,
                                        &client,
                                        &cleaned_html,
                                        id,
                                        &base_url,
                                        &artists_counter,
                                    )
                                    .await
                                    {
                                        Ok((should_break, next_url)) => {
                                            if should_break {
                                                location_completed = true;
                                                break;
                                            }
                                            if let Some(next) = next_url {
                                                current_url = next;
                                            } else {
                                                break;
                                            }
                                        }
                                        Err(e) => {
                                            println!("Error handling decision: {:?}", e);
                                            let _ = log_scrape_action(
                                                &pool,
                                                id,
                                                &format!("error:decision_handling:{}", e),
                                            ).await;
                                            break;
                                        }
                                    }
                                }
                                Err(e) => {
                                    println!("Error calling GPT: {:?}", e);
                                    let _ = log_scrape_action(
                                        &pool,
                                        id,
                                        &format!("error:gpt_call:{}", e),
                                    ).await;
                                    break;
                                }
                            }
                        }
                        Err(err) => {
                            println!(
                                "❌ Failed to fetch '{}' (ID: {}): {:?}",
                                current_url, id, err
                            );
                            let _ = log_scrape_action(
                                &pool,
                                id,
                                &format!("error:fetch_failed:{}", err),
                            ).await;
                            let _ = sqlx::query("UPDATE locations SET is_scraped = -1 WHERE id = $1")
                                .bind(id)
                                .execute(&pool)
                                .await;
                            break;
                        }
                    }
                }

                if !location_completed {
                    println!(
                        "⏰ Max page visits reached for location ID: {} - marking as complete",
                        id
                    );
                    let _ = log_scrape_action(&pool, id, "done:max_visits_reached").await;
                    let _ = sqlx::query("UPDATE locations SET is_scraped = 1 WHERE id = $1")
                        .bind(id)
                        .execute(&pool)
                        .await;
                }

                let current = scrape_counter.fetch_add(1, Ordering::SeqCst);
                pb.set_position((current + 1) as u64);
            }
        }));
    }

    while (tasks.next().await).is_some() {}

    progress.finish_with_message("🎉 Scraping complete!");

    let locations_processed = scrape_limit.load(Ordering::SeqCst);
    let artists_found = artists_added.load(Ordering::SeqCst);

    println!("📈 Final Results:");
    println!("   • Locations processed: {}", locations_processed);
    println!("   • Artists added this run: {}", artists_found);

    Ok(())
}
