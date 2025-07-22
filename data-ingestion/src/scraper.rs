use ammonia::Builder;
use async_openai::types::{
    ChatCompletionRequestMessage, ChatCompletionRequestSystemMessage,
    ChatCompletionRequestSystemMessageContent, ChatCompletionRequestUserMessage,
    ChatCompletionRequestUserMessageContent, CreateChatCompletionRequestArgs,
};
use async_openai::{config::OpenAIConfig, Client};
use futures::stream::{FuturesUnordered, StreamExt};
use indicatif::{ProgressBar, ProgressStyle};
use playwright::api::Page;
use regex::Regex;
use rusqlite::{params, Connection};
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use std::env;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc, Mutex,
};
use tokio;
use url::Url;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
#[serde(tag = "action")]
enum GptAction {
    Click { selector: String },
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
        .tags(["p", "a", "h1", "h2", "h3", "ul", "li", "div", "span", "img"].into()) // adjust as needed
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

fn persist_artist_and_styles(
    conn: &Connection,
    artist: &Artist,
    location_id: i64,
) -> anyhow::Result<()> {
    if let Some(name) = &artist.name {
        let socials = artist.social_links.clone().unwrap_or_default();
        let email = artist.email.clone().unwrap_or_default();
        let phone = artist.phone.clone().unwrap_or_default();
        let years_experience = artist.years_experience.unwrap_or(0);

        conn.execute(
            "INSERT INTO artists (name, social_links, email, phone, years_experience, location_id) VALUES (?, ?, ?, ?, ?, ?)",
            params![name, socials, email, phone, years_experience, location_id],
        )?;
        let artist_id = conn.last_insert_rowid();

        if let Some(styles) = &artist.styles {
            for raw_style in styles {
                let style_name = normalize_style(raw_style);
                let style_id = match conn.query_row(
                    "SELECT id FROM styles WHERE LOWER(name) = LOWER(?)",
                    params![style_name],
                    |row| row.get(0),
                ) {
                    Ok(id) => id,
                    Err(_) => {
                        conn.execute("INSERT INTO styles (name) VALUES (?)", params![style_name])?;
                        conn.last_insert_rowid()
                    }
                };

                conn.execute(
                    "INSERT INTO artist_style (artist_id, style_id) VALUES (?, ?)",
                    params![artist_id, style_id],
                )?;
            }
        }
    }
    Ok(())
}

async fn handle_gpt_decision(
    decision: &GptAction,
    conn: &Arc<Mutex<Connection>>,
    page: &Page,
    client: &Client<OpenAIConfig>,
    cleaned_html: &str,
    location_id: i64,
    base_url: &str,
) -> anyhow::Result<bool> {
    match decision {
        GptAction::Click { selector } => {
            println!("Clicking selector: {}", selector);
            // Note: Playwright Page click method may not be available, using goto for now
            page.goto_builder(&format!(
                "javascript:document.querySelector('{}').click()",
                selector
            ))
            .goto()
            .await?;
        }
        GptAction::Navigate { url } => {
            // Validate that navigation stays within the same domain
            let base_domain = get_domain(base_url)?;
            let nav_domain = get_domain(url)?;

            if base_domain != nav_domain {
                println!(
                    "Navigation rejected: {} not in same domain as {}",
                    url, base_url
                );
                return Ok(false);
            }

            println!("Navigating to: {}", url);
            page.goto_builder(url).goto().await?;
        }
        GptAction::Extract => {
            println!("Extracting artist data via GPT-4...");
            let url = page.url()?.to_string();
            let artists = call_gpt_extract(client, &url, cleaned_html).await?;
            println!("Extracted {} artist(s):", artists.len());
            for artist in &artists {
                let conn_guard = conn.lock().unwrap();
                persist_artist_and_styles(&conn_guard, artist, location_id)?;
            }
        }
        GptAction::Done => {
            println!("Agent decided it is done.");
            let conn_guard = conn.lock().unwrap();
            conn_guard.execute(
                "UPDATE locations SET is_scraped = 1 WHERE id = ?",
                params![location_id],
            )?;
            return Ok(true);
        }
    }
    Ok(false)
}

async fn setup_playwright_page() -> anyhow::Result<Page> {
    let playwright = playwright::Playwright::initialize().await?;
    let browser = playwright
        .chromium()
        .launcher()
        .headless(true)
        .launch()
        .await?;
    let context = browser.context_builder().build().await?;
    let page = context.new_page().await?;
    Ok(page)
}

async fn call_gpt_action(
    client: &Client<OpenAIConfig>,
    url: &str,
    cleaned_html: &str,
) -> anyhow::Result<GptAction> {
    let sys_msg = ChatCompletionRequestMessage::System(ChatCompletionRequestSystemMessage {
        content: ChatCompletionRequestSystemMessageContent::Text("You're an autonomous web agent that navigates tattoo shop websites to find artist data. 
        
        CLICK RESTRICTIONS:
        - Only click elements that will change page state or navigate to new content
        - Examples: buttons, form submits, navigation links, tabs, dropdowns
        - Do NOT click generic elements based only on CSS classes like '.item' or '.card'
        - Focus on elements with meaningful actions: 'a[href]', 'button', '[role=button]', form elements
        
        NAVIGATION RESTRICTIONS:
        - Only navigate to URLs within the same domain as the current page
        - Do not navigate to external domains, social media, or third-party sites
        
        Decide what to do next based on the HTML and URL.".to_string()),
        name: None,
    });

    let user_msg = ChatCompletionRequestMessage::User(ChatCompletionRequestUserMessage {
        content: ChatCompletionRequestUserMessageContent::Text(format!(
            r#"URL: {}
            HTML:
            {}

            Respond with a JSON object in one of the following formats:
            
            To click an element (only for state-changing elements like buttons, links, tabs):
            {{ "action": "CLICK", "selector": "css-selector-string" }}
            
            To navigate to a new page (must be same domain):
            {{ "action": "NAVIGATE", "url": "https://example.com/path" }}
            
            To extract artist data from current page:
            {{ "action": "EXTRACT" }}
            
            When finished with this location:
            {{ "action": "DONE" }}"#,
            url, cleaned_html
        )),
        name: None,
    });

    let req = CreateChatCompletionRequestArgs::default()
        .model("gpt-3.5-turbo")
        .messages([sys_msg, user_msg])
        .temperature(0.2)
        .max_tokens(1000u32)
        .build()?;

    let res = client.chat().create(req).await?;
    let text = res.choices[0].message.content.as_ref().unwrap();

    let action: GptAction = serde_json::from_str(text.trim())?;
    Ok(action)
}

async fn call_gpt_extract(
    client: &Client<OpenAIConfig>,
    url: &str,
    cleaned_html: &str,
) -> anyhow::Result<Vec<Artist>> {
    let sys_msg = ChatCompletionRequestMessage::System(ChatCompletionRequestSystemMessage {
        content: ChatCompletionRequestSystemMessageContent::Text("Extract tattoo artist data from the HTML content. Return only valid artist entries in JSON format.".to_string()),
        name: None,
    });

    let user_msg = ChatCompletionRequestMessage::User(ChatCompletionRequestUserMessage {
        content: ChatCompletionRequestUserMessageContent::Text(format!(
            r#"URL: {}
            HTML:
            {}

            Extract artist data with fields:
            - name (string)
            - styles (array of strings)
            - email (optional string)
            - phone (optional string)
            - social_links (optional string, comma-delimited)
            - years_experience (optional number)

            Respond with a JSON array of artist objects."#,
            url, cleaned_html
        )),
        name: None,
    });

    let req = CreateChatCompletionRequestArgs::default()
        .model("gpt-4")
        .messages([sys_msg, user_msg])
        .temperature(0.2)
        .max_tokens(1500u32)
        .build()?;

    let res = client.chat().create(req).await?;
    let text = res.choices[0].message.content.as_ref().unwrap();

    let artists: Vec<Artist> = serde_json::from_str(text.trim())?;
    Ok(artists)
}

pub async fn scrape(conn: Connection) -> Result<(), Box<dyn std::error::Error>> {
    let conn = Arc::new(Mutex::new(conn));

    let openai_key = env::var("OPENAI_API_KEY")?;
    let num_threads: usize = env::var("NUM_THREADS").unwrap_or("12".into()).parse()?;
    let max_scrapes: usize = env::var("MAX_SCRAPES").unwrap_or("100".into()).parse()?;
    let max_page_visits: usize = env::var("MAX_PAGE_VISITS").unwrap_or("5".into()).parse()?;

    let config = OpenAIConfig::new().with_api_key(openai_key);
    let client = Arc::new(Client::with_config(config));

    let scrape_limit = Arc::new(AtomicUsize::new(0));
    let progress = Arc::new(ProgressBar::new(max_scrapes as u64));
    progress.set_style(
        ProgressStyle::default_bar()
            .template(
                "{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta})",
            )
            .unwrap()
            .progress_chars("##-"),
    );

    let mut tasks = FuturesUnordered::new();
    for _ in 0..num_threads {
        let conn = Arc::clone(&conn);
        let client = Arc::clone(&client);
        let scrape_counter = Arc::clone(&scrape_limit);
        let pb: Arc<ProgressBar> = Arc::clone(&progress);

        tasks.push(tokio::spawn(async move {
            let page = setup_playwright_page().await.unwrap();
            loop {
                if scrape_counter.load(Ordering::SeqCst) >= max_scrapes {
                    break;
                }

                let (id, url) = {
                    let conn_guard = conn.lock().unwrap();
                    let mut stmt = conn_guard
                        .prepare(
                            "SELECT id, website_uri FROM locations WHERE is_scraped = 0 AND website_uri NOT LIKE '%facebook%' LIMIT 1",
                        )
                        .unwrap();
                    let mut rows = stmt.query([]).unwrap();
                    if let Some(row) = rows.next().unwrap() {
                        let id: i64 = row.get(0).unwrap();
                        let url: String = row.get(1).unwrap();
                        (id, url)
                    } else {
                        break;
                    }
                };

                match page.goto_builder(&url).goto().await {
                    Ok(response) => {
                        if let Some(resp) = response {
                            let status = resp.status().unwrap_or(500);
                            if status >= 400 {
                                println!("Failed to load page with status: {}", status);
                                conn.lock()
                                    .unwrap()
                                    .execute(
                                        "UPDATE locations SET is_scraped = -1 WHERE id = ?",
                                        params![id],
                                    )
                                    .unwrap();
                            } else {
                                for _ in 0..max_page_visits {
                                    let raw_html = page.content().await.unwrap();
                                    let cleaned_html = preprocess_html(&raw_html);
                                    let decision = call_gpt_action(
                                        &client,
                                        &page.url().unwrap(),
                                        &cleaned_html,
                                    )
                                    .await
                                    .unwrap();

                                    let should_break = handle_gpt_decision(
                                        &decision,
                                        &conn,
                                        &page,
                                        &client,
                                        &cleaned_html,
                                        id,
                                        &url,
                                    )
                                    .await
                                    .unwrap();
                                    if should_break {
                                        break;
                                    }
                                }
                            }
                        }
                    }
                    Err(err) => {
                        println!("Navigation error: {:?}", err);
                        conn.lock()
                            .unwrap()
                            .execute(
                                "UPDATE locations SET is_scraped = -1 WHERE id = ?",
                                params![id],
                            )
                            .unwrap();
                    }
                }

                let current = scrape_counter.fetch_add(1, Ordering::SeqCst);
                pb.set_position((current + 1) as u64);
            }
        }));
    }

    while (tasks.next().await).is_some() {}

    progress.finish_with_message("Scraping complete.");
    Ok(())
}
