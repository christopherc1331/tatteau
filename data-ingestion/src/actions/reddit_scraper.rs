// Reddit Artist Discovery
// Scrapes /r/tattoo subreddit to find artists and match them to existing shops
// Also scrapes shop Instagram bios to discover additional artists

use crate::repository::{self, CityStats, CityToScrape};
use regex::Regex;
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Row};
use std::collections::HashSet;
use std::env;

// ============================================================================
// Configuration
// ============================================================================

struct Config {
    max_posts: i32,
    min_images: i32,
    rescrape_days: i16,
    enable_shop_scrape: bool,
    city_filter: Option<String>,
    state_filter: Option<String>,
    max_cities: Option<i16>,
}

fn load_config_from_env() -> Config {
    Config {
        max_posts: env::var("REDDIT_MAX_POSTS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(100),
        min_images: env::var("REDDIT_MIN_IMAGES")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(1),
        rescrape_days: env::var("REDDIT_RESCRAPE_DAYS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(30),
        enable_shop_scrape: env::var("REDDIT_ENABLE_SHOP_SCRAPE")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(true),
        city_filter: env::var("REDDIT_CITY").ok(),
        state_filter: env::var("REDDIT_STATE").ok(),
        max_cities: env::var("REDDIT_MAX_CITIES")
            .ok()
            .and_then(|s| s.parse().ok()),
    }
}

// ============================================================================
// Data Structures
// ============================================================================

#[derive(Debug, Deserialize)]
struct RedditPost {
    #[serde(rename = "postUrl")]
    url: Option<String>,
    #[serde(rename = "dataType")]
    data_type: Option<String>,  // "post" or "comment"
    title: Option<String>,
    body: Option<String>,
    images: Option<Vec<String>>,
    #[serde(rename = "contentUrl")]
    content_url: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
struct ExtractedArtist {
    artist_name: Option<String>,
    instagram: Option<String>,
    shop: Option<String>,
}

struct CityProcessingResult {
    stats: CityStats,
    matched_shops: HashSet<i64>,
}

// ============================================================================
// Main Entry Point
// ============================================================================

pub async fn run_reddit_scraper(pool: &PgPool) -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Starting Reddit Artist Discovery");

    let config = load_config_from_env();
    let cities = select_cities_to_scrape(pool, &config).await?;

    println!("📍 Selected {} cities to scrape", cities.len());

    for city in cities {
        println!("\n🌆 Processing: {}, {}", city.city, city.state);

        match process_city(&city, pool, &config).await {
            Ok(_) => println!("✅ Successfully processed {}, {}", city.city, city.state),
            Err(e) => {
                eprintln!("❌ Failed to process {}, {}: {}", city.city, city.state, e);
                // Mark city as failed
                let error_stats = CityStats {
                    posts_found: 0,
                    artists_added: 0,
                    artists_updated: 0,
                    artists_pending: 0,
                    artists_added_from_shop_bios: 0,
                    shops_scraped: 0,
                };
                repository::mark_city_scraped(
                    pool,
                    &city.city,
                    &city.state,
                    "failed",
                    &error_stats,
                    Some(&e.to_string()),
                )
                .await?;
            }
        }
    }

    println!("\n🎉 Reddit Artist Discovery completed!");
    Ok(())
}

async fn select_cities_to_scrape(
    pool: &PgPool,
    config: &Config,
) -> Result<Vec<CityToScrape>, Box<dyn std::error::Error>> {
    let cities = repository::get_cities_for_scrape(
        pool,
        config.max_cities,
        config.city_filter.clone(),
        config.state_filter.clone(),
        config.rescrape_days,
    )
    .await?;

    Ok(cities)
}

async fn process_city(
    city: &CityToScrape,
    pool: &PgPool,
    config: &Config,
) -> Result<(), Box<dyn std::error::Error>> {
    // Phase 1: Reddit scraping & artist processing
    let result = scrape_reddit_for_city(city, pool, config).await?;

    // Phase 2: Shop Instagram bio scraping
    let mut final_stats = result.stats;
    if config.enable_shop_scrape && !result.matched_shops.is_empty() {
        println!(
            "🏪 Scraping {} shop Instagram bios...",
            result.matched_shops.len()
        );
        let shop_stats = scrape_shop_instagram_bios(&result.matched_shops, pool, city).await?;
        final_stats.artists_added_from_shop_bios = shop_stats.artists_added_from_shop_bios;
        final_stats.artists_updated += shop_stats.artists_updated;
        final_stats.shops_scraped = shop_stats.shops_scraped;
    }

    // Phase 3: Update city stats
    finalize_city_stats(city, &final_stats, pool).await?;

    Ok(())
}

// ============================================================================
// Phase 1: Reddit Scraping & Artist Processing
// ============================================================================

async fn scrape_reddit_for_city(
    city: &CityToScrape,
    pool: &PgPool,
    config: &Config,
) -> Result<CityProcessingResult, Box<dyn std::error::Error>> {
    // Search Reddit posts for this city
    let posts = call_apify_reddit_scraper(&city.city, config).await?;

    let mut stats = CityStats {
        posts_found: posts.len() as i32,
        artists_added: 0,
        artists_updated: 0,
        artists_pending: 0,
        artists_added_from_shop_bios: 0,
        shops_scraped: 0,
    };

    let mut matched_shops: HashSet<i64> = HashSet::new();

    // Filter for posts with images
    let posts_with_images = filter_posts_with_images(posts, config.min_images);
    println!("📸 Found {} posts with images", posts_with_images.len());

    // Process each post
    for post in posts_with_images {
        match process_reddit_post(&post, city, pool, &mut stats, &mut matched_shops).await {
            Ok(_) => {}
            Err(e) => {
                let url = post.url.as_deref().unwrap_or("unknown");
                eprintln!("⚠️  Error processing post {}: {}", url, e);
            }
        }
    }

    Ok(CityProcessingResult {
        stats,
        matched_shops,
    })
}

async fn call_apify_reddit_scraper(
    city: &str,
    config: &Config,
) -> Result<Vec<RedditPost>, Box<dyn std::error::Error>> {
    let api_token = env::var("APIFY_API_TOKEN")?;
    let actor_id = "harshmaur~reddit-scraper";
    let url = format!(
        "https://api.apify.com/v2/acts/{}/run-sync-get-dataset-items?token={}",
        actor_id, api_token
    );

    // Use subreddit search URL to search within /r/tattoos for city name
    // Get 1 comment per post for additional context
    let search_url = format!("https://www.reddit.com/r/tattoos/search/?q={}&sort=top&t=year",
        urlencoding::encode(city));
    let input = serde_json::json!({
        "startUrls": [{"url": search_url}],
        "crawlCommentsPerPost": true,
        "maxPostsCount": config.max_posts,
        "maxCommentsPerPost": 1,  // Get 1 comment per post for additional context
        "includeNSFW": true,
        "proxy": {
            "useApifyProxy": true,
            "apifyProxyGroups": ["RESIDENTIAL"]
        }
    });

    println!("🔍 Scraping Reddit posts...");
    println!("📋 Apify input:");
    println!("{}", serde_json::to_string_pretty(&input).unwrap_or_default());

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(300))
        .build()?;

    let response = client.post(&url).json(&input).send().await?;

    if !response.status().is_success() {
        let status = response.status();
        let error_text = response.text().await?;
        return Err(format!("Apify Reddit scraper failed: {} - {}", status, error_text).into());
    }

    let response_text = response.text().await?;

    // Log raw response for debugging
    println!("📦 Raw Apify response preview (first 500 chars):");
    println!("{}", &response_text[..std::cmp::min(500, response_text.len())]);

    let all_items: Vec<RedditPost> = serde_json::from_str(&response_text)
        .map_err(|e| {
            eprintln!("Failed to parse response. Full response: {}", response_text);
            format!("Failed to parse Reddit response: {}", e)
        })?;

    let total_items = all_items.len();

    // Filter to only posts with images/galleries and valid postUrl
    let posts: Vec<RedditPost> = all_items
        .into_iter()
        .filter(|item| {
            // Must be a post (not comment)
            item.data_type.as_deref() == Some("post") &&
            // Must have a postUrl
            item.url.is_some() &&
            // Must have images OR contentUrl (gallery)
            (item.images.as_ref().map_or(false, |imgs| !imgs.is_empty()) ||
             item.content_url.is_some())
        })
        .collect();

    println!("📥 Retrieved {} posts with images from Reddit (filtered from {} total items)", posts.len(), total_items);

    // Log sample post structure
    if let Some(first_post) = posts.first() {
        println!("📝 Sample post structure:");
        println!("  - URL: {:?}", first_post.url);
        println!("  - Title: {:?}", first_post.title.as_ref().map(|s| &s[..std::cmp::min(50, s.len())]));
        println!("  - Has body: {}", first_post.body.as_ref().map_or(false, |b| !b.is_empty()));
        println!("  - Images: {:?}", first_post.images.as_ref().map(|m| m.len()));
    }

    Ok(posts)
}

fn filter_posts_with_images(posts: Vec<RedditPost>, min_images: i32) -> Vec<RedditPost> {
    // Posts already have images/contentUrl at this point
    // This function checks if they meet the minimum image count requirement
    if min_images <= 1 {
        // No additional filtering needed
        return posts;
    }

    let total = posts.len();
    let mut filtered_out_too_few = 0;

    let result: Vec<RedditPost> = posts
        .into_iter()
        .filter(|post| {
            // Check if post has enough images
            let image_count = post.images.as_ref().map_or(0, |imgs| imgs.len());

            if image_count >= min_images as usize {
                true
            } else {
                // ContentUrl (gallery) posts count as having sufficient images
                if post.content_url.is_some() {
                    true
                } else {
                    filtered_out_too_few += 1;
                    let url = post.url.as_deref().unwrap_or("unknown");
                    println!("  ⊘ Filtered out (only {} images, need {}): {}", image_count, min_images, url);
                    false
                }
            }
        })
        .collect();

    if filtered_out_too_few > 0 {
        println!("🔍 Minimum image filtering:");
        println!("  - Posts before filtering: {}", total);
        println!("  - Filtered out (too few images): {}", filtered_out_too_few);
        println!("  - Posts with ≥{} images: {}", min_images, result.len());
    }

    result
}

async fn process_reddit_post(
    post: &RedditPost,
    city: &CityToScrape,
    pool: &PgPool,
    stats: &mut CityStats,
    matched_shops: &mut HashSet<i64>,
) -> Result<(), Box<dyn std::error::Error>> {
    // Extract artist info with OpenAI
    let extracted_artists = extract_artist_info_with_openai(post).await?;

    // If no artist info could be extracted, add to pending for manual review
    if extracted_artists.is_empty() {
        let title = post.title.as_deref().unwrap_or("");
        let body = post.body.as_deref().unwrap_or("");
        let post_context = format!("Title: {}\nBody: {}", title, body);
        let post_url = post.url.as_deref().unwrap_or("unknown");

        let pending = repository::PendingArtistData {
            reddit_post_url: Some(post_url.to_string()),
            artist_name: None,
            instagram_handle: None,
            shop_name_mentioned: None,
            city: city.city.clone(),
            state: city.state.clone(),
            post_context: Some(post_context),
            match_type: "no_artist_info_extracted".to_string(),
        };

        repository::insert_reddit_artist_pending(pool, &pending).await?;
        stats.artists_pending += 1;
        println!("📋 Added to pending review (no info extracted): {}", post_url);
        return Ok(());
    }

    // Process each extracted artist
    let post_url = post.url.as_deref().unwrap_or("unknown");
    for artist_data in extracted_artists {
        process_extracted_artist(&artist_data, city, post_url, pool, stats, matched_shops).await?;
    }

    Ok(())
}

async fn extract_artist_info_with_openai(
    post: &RedditPost,
) -> Result<Vec<ExtractedArtist>, Box<dyn std::error::Error>> {
    let api_key = env::var("OPENAI_API_KEY")?;

    // Build context from post title and body
    let title = post.title.as_deref().unwrap_or("");
    let post_body = post.body.as_deref().unwrap_or("");

    let prompt = format!(
        r#"Analyze this r/tattoos Reddit post and extract artist information:

TITLE: {}
BODY: {}

Extract:
1. Artist Instagram handles (e.g., @username or instagram.com/username)
2. Shop/studio names mentioned
3. Artist names if explicitly mentioned

Return JSON array:
[{{"artist_name": "...", "instagram": "@...", "shop": "..."}}]

If no artist information found, return empty array [].
All fields are optional - include only what you can confidently extract."#,
        title, post_body
    );

    let client = reqwest::Client::new();
    let response = client
        .post("https://api.openai.com/v1/chat/completions")
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .json(&serde_json::json!({
            "model": "gpt-4",
            "messages": [
                {"role": "system", "content": "You are a helpful assistant that extracts tattoo artist information from Reddit posts. Always return valid JSON."},
                {"role": "user", "content": prompt}
            ],
            "temperature": 0.3
        }))
        .send()
        .await?;

    let response_json: serde_json::Value = response.json().await?;

    let content = response_json["choices"][0]["message"]["content"]
        .as_str()
        .ok_or("No content in OpenAI response")?;

    // Parse JSON from response
    let artists: Vec<ExtractedArtist> =
        serde_json::from_str(content).unwrap_or_else(|_| Vec::new());

    Ok(artists)
}

async fn process_extracted_artist(
    artist_data: &ExtractedArtist,
    city: &CityToScrape,
    post_url: &str,
    pool: &PgPool,
    stats: &mut CityStats,
    matched_shops: &mut HashSet<i64>,
) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(instagram) = &artist_data.instagram {
        // Scenario A: Has Instagram handle
        process_artist_with_instagram(
            artist_data,
            instagram,
            city,
            post_url,
            pool,
            stats,
            matched_shops,
        )
        .await?;
    } else if artist_data.shop.is_some() || artist_data.artist_name.is_some() {
        // Scenario B: No Instagram but has name/shop
        process_artist_without_instagram(artist_data, city, post_url, pool, stats).await?;
    } else {
        // Scenario D: Nothing useful extracted - add to pending for manual review
        add_to_pending_review(
            pool,
            artist_data,
            city,
            post_url,
            "no_useful_info",
            stats,
        )
        .await?;
    }

    Ok(())
}

async fn process_artist_with_instagram(
    artist_data: &ExtractedArtist,
    instagram: &str,
    city: &CityToScrape,
    post_url: &str,
    pool: &PgPool,
    stats: &mut CityStats,
    matched_shops: &mut HashSet<i64>,
) -> Result<(), Box<dyn std::error::Error>> {
    let handle = normalize_instagram_handle(instagram);
    let instagram_url = format!("https://instagram.com/{}", handle);

    // Search globally for artist by Instagram
    if let Some(artist) = repository::find_artist_by_instagram_globally(pool, &handle).await? {
        // Artist exists - check if needs Instagram URL update
        if !has_valid_instagram_url(&artist.social_links) {
            repository::update_artist_add_instagram(
                pool,
                artist.id,
                &handle,
                &instagram_url,
                artist.social_links.clone(),
            )
            .await?;
            stats.artists_updated += 1;
            println!("✏️  Updated artist {} with Instagram", artist.id);
        }
    } else {
        // Artist doesn't exist - try to match shop and create
        if let Some(shop) = &artist_data.shop {
            if let Some(location_id) =
                repository::find_location_by_shop_and_city(pool, shop, &city.city, &city.state)
                    .await?
            {
                // Shop matched - create new artist
                let artist_id = repository::insert_artist_with_instagram(
                    pool,
                    artist_data.artist_name.as_deref(),
                    location_id,
                    &handle,
                    &instagram_url,
                )
                .await?;
                stats.artists_added += 1;
                matched_shops.insert(location_id);
                println!(
                    "➕ Created new artist {} at shop {}",
                    artist_id, location_id
                );
            } else {
                // Shop not matched - add to pending
                add_to_pending_review(pool, artist_data, city, post_url, "no_shop_match", stats)
                    .await?;
            }
        } else {
            // No shop provided - add to pending (Scenario C)
            add_to_pending_review(pool, artist_data, city, post_url, "no_shop_match", stats)
                .await?;
        }
    }

    Ok(())
}

async fn process_artist_without_instagram(
    artist_data: &ExtractedArtist,
    city: &CityToScrape,
    post_url: &str,
    pool: &PgPool,
    stats: &mut CityStats,
) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(shop) = &artist_data.shop {
        if let Some(location_id) =
            repository::find_location_by_shop_and_city(pool, shop, &city.city, &city.state).await?
        {
            // Shop found - try to find artist by name
            if let Some(name) = &artist_data.artist_name {
                let (first, last) = parse_name_components(name);
                if let Some(_artist) =
                    repository::find_artist_by_name_at_location(pool, &first, &last, location_id)
                        .await?
                {
                    // Found artist by name but no Instagram to add
                    add_to_pending_review(
                        pool,
                        artist_data,
                        city,
                        post_url,
                        "found_artist_no_ig",
                        stats,
                    )
                    .await?;
                } else {
                    // Shop found but not artist, no Instagram
                    add_to_pending_review(
                        pool,
                        artist_data,
                        city,
                        post_url,
                        "found_shop_no_artist",
                        stats,
                    )
                    .await?;
                }
            } else {
                // Shop found but no name or Instagram
                add_to_pending_review(
                    pool,
                    artist_data,
                    city,
                    post_url,
                    "found_shop_no_artist",
                    stats,
                )
                .await?;
            }
        } else {
            // Shop not found
            add_to_pending_review(pool, artist_data, city, post_url, "no_shop_match", stats)
                .await?;
        }
    } else {
        // No shop provided
        add_to_pending_review(pool, artist_data, city, post_url, "no_shop_match", stats).await?;
    }

    Ok(())
}

async fn add_to_pending_review(
    pool: &PgPool,
    artist_data: &ExtractedArtist,
    city: &CityToScrape,
    post_url: &str,
    match_type: &str,
    stats: &mut CityStats,
) -> Result<(), Box<dyn std::error::Error>> {
    // Build context string from what we extracted
    let context_parts: Vec<String> = vec![
        artist_data.artist_name.as_ref().map(|n| format!("Name: {}", n)),
        artist_data.instagram.as_ref().map(|ig| format!("Instagram: {}", ig)),
        artist_data.shop.as_ref().map(|s| format!("Shop: {}", s)),
    ]
    .into_iter()
    .flatten()
    .collect();

    let post_context = if !context_parts.is_empty() {
        Some(context_parts.join(", "))
    } else {
        None
    };

    let pending = repository::PendingArtistData {
        reddit_post_url: Some(post_url.to_string()),
        artist_name: artist_data.artist_name.clone(),
        instagram_handle: artist_data.instagram.clone(),
        shop_name_mentioned: artist_data.shop.clone(),
        city: city.city.clone(),
        state: city.state.clone(),
        post_context,
        match_type: match_type.to_string(),
    };

    repository::insert_reddit_artist_pending(pool, &pending).await?;
    stats.artists_pending += 1;
    println!("📋 Added to pending review ({}): {}", match_type, post_url);

    Ok(())
}

// ============================================================================
// Phase 2: Shop Instagram Bio Scraping
// ============================================================================

async fn scrape_shop_instagram_bios(
    matched_shops: &HashSet<i64>,
    pool: &PgPool,
    city: &CityToScrape,
) -> Result<CityStats, Box<dyn std::error::Error>> {
    let mut stats = CityStats {
        posts_found: 0,
        artists_added: 0,
        artists_updated: 0,
        artists_pending: 0,
        artists_added_from_shop_bios: 0,
        shops_scraped: 0,
    };

    for shop_id in matched_shops {
        match process_shop_bio(*shop_id, pool, city, &mut stats).await {
            Ok(_) => stats.shops_scraped += 1,
            Err(e) => eprintln!("⚠️  Error processing shop {}: {}", shop_id, e),
        }
    }

    Ok(stats)
}

async fn process_shop_bio(
    shop_id: i64,
    pool: &PgPool,
    city: &CityToScrape,
    stats: &mut CityStats,
) -> Result<(), Box<dyn std::error::Error>> {
    // Get shop details
    let shop_name = get_shop_name(pool, shop_id).await?;

    // Search for shop's Instagram profile
    let shop_profile = search_for_shop_instagram(&shop_name, &city.city).await?;

    if let Some(profile) = shop_profile {
        // Get full profile with bio
        let profile_info =
            crate::actions::apify_scraper::get_instagram_profile_info(&profile.username).await?;

        if let Some(bio) = profile_info.bio {
            // Parse bio for artist handles
            let artist_handles = extract_handles_from_bio(&bio).await?;

            println!(
                "   Found {} artist handles in @{}'s bio",
                artist_handles.len(),
                profile.username
            );

            // Process each artist handle
            for handle in artist_handles {
                match process_artist_from_shop_bio(&handle, shop_id, pool, stats).await {
                    Ok(_) => {}
                    Err(e) => eprintln!("      ⚠️  Error processing @{}: {}", handle, e),
                }
            }
        }
    }

    Ok(())
}

async fn get_shop_name(pool: &PgPool, shop_id: i64) -> Result<String, Box<dyn std::error::Error>> {
    let result = sqlx::query("SELECT name FROM locations WHERE id = $1")
        .bind(shop_id)
        .fetch_one(pool)
        .await?;

    Ok(result.get("name"))
}

async fn search_for_shop_instagram(
    shop_name: &str,
    city: &str,
) -> Result<Option<crate::actions::apify_scraper::InstagramSearchResult>, Box<dyn std::error::Error>>
{
    let query = format!("{} {} tattoo", shop_name, city);
    let results = crate::actions::apify_scraper::search_instagram_profiles(&query).await?;

    // Filter for profiles with "tattoo" in bio
    let filtered = results.into_iter().find(|r| {
        if let Some(bio) = &r.bio {
            bio.to_lowercase().contains("tattoo")
        } else {
            false
        }
    });

    Ok(filtered)
}

async fn extract_handles_from_bio(bio: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let api_key = env::var("OPENAI_API_KEY")?;

    let prompt = format!(
        r#"Extract all Instagram handles mentioned in this tattoo shop's Instagram bio.
Look for patterns like @username or instagram.com/username.

Bio text:
{}

Return a JSON array of Instagram handles (just the username, without @ or URLs):
["handle1", "handle2", ...]

If no handles found, return empty array []."#,
        bio
    );

    let client = reqwest::Client::new();
    let response = client
        .post("https://api.openai.com/v1/chat/completions")
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .json(&serde_json::json!({
            "model": "gpt-4",
            "messages": [
                {"role": "system", "content": "You are a helpful assistant that extracts Instagram handles from bios. Always return valid JSON."},
                {"role": "user", "content": prompt}
            ],
            "temperature": 0.3
        }))
        .send()
        .await?;

    let response_json: serde_json::Value = response.json().await?;

    let content = response_json["choices"][0]["message"]["content"]
        .as_str()
        .ok_or("No content in OpenAI response")?;

    let handles: Vec<String> = serde_json::from_str(content).unwrap_or_else(|_| Vec::new());

    Ok(handles)
}

async fn process_artist_from_shop_bio(
    handle: &str,
    shop_id: i64,
    pool: &PgPool,
    stats: &mut CityStats,
) -> Result<(), Box<dyn std::error::Error>> {
    let instagram_url = format!("https://instagram.com/{}", handle);

    // Check if artist exists at this shop
    if let Some(artist) =
        repository::find_artist_by_instagram_at_location(pool, handle, shop_id).await?
    {
        // Artist exists - check if needs Instagram URL
        if !has_valid_instagram_url(&artist.social_links) {
            repository::update_artist_add_instagram(
                pool,
                artist.id,
                handle,
                &instagram_url,
                artist.social_links.clone(),
            )
            .await?;
            stats.artists_updated += 1;
            println!("      ✏️  Updated @{}", handle);
        }
    } else {
        // Artist doesn't exist - get name from Instagram and try to match
        let profile_info =
            crate::actions::apify_scraper::get_instagram_profile_info(handle).await?;

        if let Some(name) = profile_info.full_name {
            let (first, last) = parse_name_components(&name);

            // Try to find by name
            if let Some(artist) =
                repository::find_artist_by_name_at_location(pool, &first, &last, shop_id).await?
            {
                // Found by name - update with Instagram
                repository::update_artist_add_instagram(
                    pool,
                    artist.id,
                    handle,
                    &instagram_url,
                    artist.social_links.clone(),
                )
                .await?;
                stats.artists_updated += 1;
                println!("      ✏️  Matched and updated @{}", handle);
            } else {
                // Not found by name - create new artist
                let artist_id = repository::insert_artist_with_instagram(
                    pool,
                    Some(&name),
                    shop_id,
                    handle,
                    &instagram_url,
                )
                .await?;
                stats.artists_added_from_shop_bios += 1;
                println!("      ➕ Created new artist {} (@{})", artist_id, handle);
            }
        } else {
            // No name from Instagram - skip
            println!("      ⏭️  Skipping @{} (no name in profile)", handle);
        }
    }

    Ok(())
}

// ============================================================================
// Phase 3: Finalize Stats
// ============================================================================

async fn finalize_city_stats(
    city: &CityToScrape,
    stats: &CityStats,
    pool: &PgPool,
) -> Result<(), Box<dyn std::error::Error>> {
    repository::mark_city_scraped(pool, &city.city, &city.state, "success", stats, None).await?;

    println!("\n📊 City Stats for {}, {}:", city.city, city.state);
    println!("   Posts found: {}", stats.posts_found);
    println!("   Artists added: {}", stats.artists_added);
    println!("   Artists updated: {}", stats.artists_updated);
    println!("   Artists pending review: {}", stats.artists_pending);
    println!(
        "   Artists from shop bios: {}",
        stats.artists_added_from_shop_bios
    );
    println!("   Shops scraped: {}", stats.shops_scraped);

    Ok(())
}

// ============================================================================
// Helper Functions
// ============================================================================

fn normalize_instagram_handle(handle: &str) -> String {
    let handle = handle.trim().trim_start_matches('@');

    if handle.contains("instagram.com/") {
        if let Some(extracted) = extract_instagram_handle_from_url(handle) {
            return extracted;
        }
    }

    handle.trim_end_matches('/').to_string()
}

fn extract_instagram_handle_from_url(url: &str) -> Option<String> {
    let re = Regex::new(r"instagram\.com/([a-zA-Z0-9._]+)").ok()?;

    re.captures(url).and_then(|caps| {
        caps.get(1).and_then(|m| {
            let handle = m.as_str().trim_end_matches('/');
            if !handle.is_empty() {
                Some(handle.to_string())
            } else {
                None
            }
        })
    })
}

fn has_valid_instagram_url(social_links: &Option<String>) -> bool {
    if let Some(links) = social_links {
        links.split(',').map(|url| url.trim()).any(|url| {
            url.contains("instagram.com") && extract_instagram_handle_from_url(url).is_some()
        })
    } else {
        false
    }
}

fn parse_name_components(name: &str) -> (String, String) {
    let parts: Vec<&str> = name.split_whitespace().collect();

    match parts.len() {
        0 => (String::new(), String::new()),
        1 => (parts[0].to_lowercase(), String::new()),
        _ => {
            let first = parts[0].to_lowercase();
            let last = parts[parts.len() - 1].to_lowercase();
            (first, last)
        }
    }
}
