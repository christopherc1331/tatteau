// Reddit Artist Discovery
// Scrapes /r/tattoo subreddit to find artists and match them to existing shops
// Also scrapes shop Instagram bios to discover additional artists

use crate::repository::{self, CityStats, CityToScrape};
use crate::services::google_places::{is_tattoo_shop, parse_places_to_locations, search_text_with_location, LocationBounds};
use futures::stream::{FuturesUnordered, StreamExt};
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::PgPool;
use std::collections::HashSet;
use std::env;
use std::sync::Arc;
use strsim::jaro_winkler;

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

#[derive(Debug, Deserialize)]
struct ApifyRunResponse {
    data: ApifyRunData,
}

#[derive(Debug, Deserialize)]
struct ApifyRunData {
    id: String,
    #[serde(rename = "defaultDatasetId")]
    default_dataset_id: String,
    status: String,
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

const NUM_SHOP_THREADS: usize = 10;

pub async fn run_reddit_scraper(pool: &PgPool) -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Starting Reddit Artist Discovery (Shop-Centric)");

    let config = load_config_from_env();
    let cities = select_cities_to_scrape(pool, &config).await?;

    println!("📍 Selected {} cities to scrape", cities.len());

    for city in cities {
        println!("\n🌆 Processing: {}, {}", city.city, city.state);

        match process_city_shop_centric(&city, pool, &config).await {
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

async fn process_city_shop_centric(
    city: &CityToScrape,
    pool: &PgPool,
    config: &Config,
) -> Result<(), Box<dyn std::error::Error>> {
    // PHASE 1: Scrape Reddit → populate pending table
    println!("📱 Phase 1: Scraping Reddit posts...");
    let posts_found = scrape_reddit_to_pending(city, pool, config).await?;
    println!("   Found {} posts", posts_found);

    // PHASE 2: Get pending artists with Instagram handles
    println!("👤 Phase 2: Identifying artists with Instagram handles...");
    let pending_artists =
        repository::get_pending_artists_with_handles(pool, &city.city, &city.state).await?;
    println!(
        "   Found {} pending artists with IG handles to process",
        pending_artists.len()
    );

    if pending_artists.is_empty() {
        println!("   No artists with handles to process, marking city as complete");
        let stats = CityStats {
            posts_found,
            artists_added: 0,
            artists_updated: 0,
            artists_pending: 0,
            artists_added_from_shop_bios: 0,
            shops_scraped: 0,
        };
        finalize_city_stats(city, &stats, pool).await?;
        return Ok(());
    }

    // PHASE 3: Process artists in parallel
    println!(
        "⚡ Phase 3: Processing artists with handles ({} threads)...",
        NUM_SHOP_THREADS
    );
    let mut artist_stats = process_artists_parallel(pending_artists, pool).await?;

    // PHASE 2.5: Get pending artists WITHOUT handles but WITH names
    println!("🔍 Phase 2.5: Identifying artists without handles (name-based search)...");
    let pending_artists_no_handles =
        repository::get_pending_artists_without_handles(pool, &city.city, &city.state).await?;
    println!(
        "   Found {} pending artists without handles to search for",
        pending_artists_no_handles.len()
    );

    // PHASE 3.5: Search Instagram and process found artists
    if !pending_artists_no_handles.is_empty() {
        println!(
            "🔎 Phase 3.5: Searching Instagram and processing artists ({} threads)...",
            NUM_SHOP_THREADS
        );
        let search_stats = process_artists_via_search(pending_artists_no_handles, pool).await?;

        // Combine stats from both phases
        artist_stats.shops_processed += search_stats.shops_processed;
        artist_stats.artists_added += search_stats.artists_added;
        artist_stats.artists_updated += search_stats.artists_updated;
        artist_stats.artists_pending += search_stats.artists_pending;
    }

    // PHASE 4: Update city stats
    let final_stats = CityStats {
        posts_found,
        artists_added: artist_stats.artists_added,
        artists_updated: artist_stats.artists_updated,
        artists_pending: artist_stats.artists_pending,
        artists_added_from_shop_bios: artist_stats.artists_added,
        shops_scraped: artist_stats.shops_processed,
    };
    finalize_city_stats(city, &final_stats, pool).await?;

    Ok(())
}

// ============================================================================
// Shop-Centric Helper Functions
// ============================================================================

struct ShopProcessingStats {
    shops_processed: i32,
    artists_added: i32,
    artists_updated: i32,
    artists_pending: i32,
}

async fn scrape_reddit_to_pending(
    city: &CityToScrape,
    pool: &PgPool,
    config: &Config,
) -> Result<i32, Box<dyn std::error::Error>> {
    // Search Reddit posts for this city
    let posts = call_apify_reddit_scraper(&city.city, config).await?;
    let posts_found = posts.len() as i32;

    // Filter for posts with images
    let posts_with_images = filter_posts_with_images(posts, config.min_images);
    println!("📸 Found {} posts with images", posts_with_images.len());

    // Process each post and store in pending table
    for post in posts_with_images {
        match extract_and_store_pending(&post, city, pool).await {
            Ok(_) => {}
            Err(e) => {
                let url = post.url.as_deref().unwrap_or("unknown");
                eprintln!("⚠️  Error processing post {}: {}", url, e);
            }
        }
    }

    Ok(posts_found)
}

async fn extract_and_store_pending(
    post: &RedditPost,
    city: &CityToScrape,
    pool: &PgPool,
) -> Result<(), Box<dyn std::error::Error>> {
    // Extract artist info with OpenAI
    let extracted_artists = extract_artist_info_with_openai(post).await?;

    let post_url = post.url.as_deref().unwrap_or("unknown");

    if extracted_artists.is_empty() {
        // No info extracted - add to pending
        let title = post.title.as_deref().unwrap_or("");
        let body = post.body.as_deref().unwrap_or("");
        let post_context = format!("Title: {}\nBody: {}", title, body);

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
        return Ok(());
    }

    // Store each extracted artist in pending
    for artist_data in extracted_artists {
        let context_parts: Vec<String> = vec![
            artist_data
                .artist_name
                .as_ref()
                .map(|n| format!("Name: {}", n)),
            artist_data
                .instagram
                .as_ref()
                .map(|ig| format!("Instagram: {}", ig)),
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
            instagram_handle: artist_data.instagram.as_ref().map(|h| normalize_instagram_handle(h)),
            shop_name_mentioned: artist_data.shop.clone(),
            city: city.city.clone(),
            state: city.state.clone(),
            post_context,
            match_type: "pending".to_string(),
        };

        repository::insert_reddit_artist_pending(pool, &pending).await?;
    }

    Ok(())
}

async fn process_shops_parallel(
    shops: Vec<String>,
    pool: &PgPool,
    city: &CityToScrape,
    _config: &Config,
) -> Result<ShopProcessingStats, Box<dyn std::error::Error>> {
    let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
    let rx = Arc::new(tokio::sync::Mutex::new(rx));

    // Send all shops to channel
    for shop in shops {
        tx.send(shop).unwrap();
    }
    drop(tx);

    let mut tasks = FuturesUnordered::new();

    for thread_id in 0..NUM_SHOP_THREADS {
        let pool = pool.clone();
        let city_owned = city.clone();
        let rx = Arc::clone(&rx);

        tasks.push(tokio::spawn(async move {
            let mut local_stats = ShopProcessingStats {
                shops_processed: 0,
                artists_added: 0,
                artists_updated: 0,
                artists_pending: 0,
            };

            loop {
                let shop = match rx.lock().await.recv().await {
                    Some(s) => s,
                    None => break,
                };

                match process_single_shop(&shop, &city_owned, &pool).await {
                    Ok(stats) => {
                        local_stats.shops_processed += 1;
                        local_stats.artists_added += stats.artists_added;
                        local_stats.artists_updated += stats.artists_updated;
                        println!(
                            "   [Thread {}] ✅ {}: {} added, {} updated",
                            thread_id, shop, stats.artists_added, stats.artists_updated
                        );
                    }
                    Err(e) => {
                        eprintln!("   [Thread {}] ❌ {}: {}", thread_id, shop, e);
                    }
                }
            }

            local_stats
        }));
    }

    // Collect results from all threads
    let mut final_stats = ShopProcessingStats {
        shops_processed: 0,
        artists_added: 0,
        artists_updated: 0,
        artists_pending: 0,
    };

    while let Some(result) = tasks.next().await {
        match result {
            Ok(local_stats) => {
                final_stats.shops_processed += local_stats.shops_processed;
                final_stats.artists_added += local_stats.artists_added;
                final_stats.artists_updated += local_stats.artists_updated;
                final_stats.artists_pending += local_stats.artists_pending;
            }
            Err(e) => {
                eprintln!("⚠️  Thread panicked: {}", e);
            }
        }
    }

    println!("✅ Parallel shop processing complete!");

    Ok(final_stats)
}

// ============================================================================
// Artist-First Processing (NEW)
// ============================================================================

async fn process_artists_parallel(
    artists: Vec<repository::PendingArtistWithHandle>,
    pool: &PgPool,
) -> Result<ShopProcessingStats, Box<dyn std::error::Error>> {
    let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
    let rx = Arc::new(tokio::sync::Mutex::new(rx));

    // Send all artists to channel
    for artist in artists {
        tx.send(artist).unwrap();
    }
    drop(tx);

    let mut tasks = FuturesUnordered::new();

    for thread_id in 0..NUM_SHOP_THREADS {
        let pool = pool.clone();
        let rx = Arc::clone(&rx);

        tasks.push(tokio::spawn(async move {
            let mut local_stats = ShopProcessingStats {
                shops_processed: 0,
                artists_added: 0,
                artists_updated: 0,
                artists_pending: 0,
            };

            loop {
                let artist = match rx.lock().await.recv().await {
                    Some(a) => a,
                    None => break,
                };

                match process_artist_from_pending(
                    &pool,
                    &artist.instagram_handle,
                    &artist.city,
                    &artist.state,
                    thread_id,
                )
                .await
                {
                    Ok(artists_processed) => {
                        local_stats.shops_processed += 1; // shops discovered
                        local_stats.artists_added += artists_processed as i32;
                        println!(
                            "   [Thread {}] ✅ @{}: {} artists from shop",
                            thread_id, artist.instagram_handle, artists_processed
                        );
                    }
                    Err(e) => {
                        eprintln!(
                            "   [Thread {}] ❌ @{}: {}",
                            thread_id, artist.instagram_handle, e
                        );
                    }
                }
            }

            local_stats
        }));
    }

    // Collect results from all threads
    let mut final_stats = ShopProcessingStats {
        shops_processed: 0,
        artists_added: 0,
        artists_updated: 0,
        artists_pending: 0,
    };

    while let Some(result) = tasks.next().await {
        match result {
            Ok(local_stats) => {
                final_stats.shops_processed += local_stats.shops_processed;
                final_stats.artists_added += local_stats.artists_added;
                final_stats.artists_updated += local_stats.artists_updated;
                final_stats.artists_pending += local_stats.artists_pending;
            }
            Err(e) => {
                eprintln!("⚠️  Thread panicked: {}", e);
            }
        }
    }

    println!("✅ Parallel artist processing complete!");

    Ok(final_stats)
}

async fn process_artists_via_search(
    artists: Vec<repository::PendingArtistWithName>,
    pool: &PgPool,
) -> Result<ShopProcessingStats, Box<dyn std::error::Error>> {
    let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
    let rx = Arc::new(tokio::sync::Mutex::new(rx));

    // Send all artists to channel
    for artist in artists {
        tx.send(artist).unwrap();
    }
    drop(tx);

    let mut tasks = FuturesUnordered::new();

    for thread_id in 0..NUM_SHOP_THREADS {
        let pool = pool.clone();
        let rx = Arc::clone(&rx);

        tasks.push(tokio::spawn(async move {
            let mut local_stats = ShopProcessingStats {
                shops_processed: 0,
                artists_added: 0,
                artists_updated: 0,
                artists_pending: 0,
            };

            loop {
                let artist = match rx.lock().await.recv().await {
                    Some(a) => a,
                    None => break,
                };

                // STEP 1: Search Instagram for artist by name
                let profile = match crate::actions::apify_scraper::search_instagram_artist(&artist.artist_name).await {
                    Ok(profile) => profile,
                    Err(e) => {
                        eprintln!(
                            "   [Thread {}] ❌ {}: {}",
                            thread_id, artist.artist_name, e
                        );
                        continue;
                    }
                };

                println!(
                    "   [Thread {}] ✅ Found @{} for '{}'",
                    thread_id, profile.username, artist.artist_name
                );

                // STEP 2: Update pending record with found handle
                if let Err(e) = repository::update_pending_artist_handle(
                    &pool,
                    &artist.artist_name,
                    &profile.username,
                    &artist.city,
                    &artist.state,
                )
                .await
                {
                    eprintln!(
                        "   [Thread {}] ⚠️  Failed to update handle for {}: {}",
                        thread_id, artist.artist_name, e
                    );
                    continue;
                }

                // STEP 3: Process artist using existing flow
                match process_artist_from_pending(
                    &pool,
                    &profile.username,
                    &artist.city,
                    &artist.state,
                    thread_id,
                )
                .await
                {
                    Ok(artists_processed) => {
                        local_stats.shops_processed += 1;
                        local_stats.artists_added += artists_processed as i32;
                        println!(
                            "   [Thread {}] ✅ @{}: {} artists from shop",
                            thread_id, profile.username, artists_processed
                        );
                    }
                    Err(e) => {
                        eprintln!(
                            "   [Thread {}] ❌ @{}: {}",
                            thread_id, profile.username, e
                        );
                    }
                }
            }

            local_stats
        }));
    }

    // Collect results from all threads
    let mut final_stats = ShopProcessingStats {
        shops_processed: 0,
        artists_added: 0,
        artists_updated: 0,
        artists_pending: 0,
    };

    while let Some(result) = tasks.next().await {
        match result {
            Ok(local_stats) => {
                final_stats.shops_processed += local_stats.shops_processed;
                final_stats.artists_added += local_stats.artists_added;
                final_stats.artists_updated += local_stats.artists_updated;
                final_stats.artists_pending += local_stats.artists_pending;
            }
            Err(e) => {
                eprintln!("⚠️  Thread panicked: {}", e);
            }
        }
    }

    println!("✅ Instagram search processing complete!");

    Ok(final_stats)
}

// ============================================================================
// Shop-Centric Processing (OLD - kept for reference)
// ============================================================================

struct SingleShopStats {
    artists_added: i32,
    artists_updated: i32,
}

async fn process_single_shop(
    shop_name: &str,
    city: &CityToScrape,
    pool: &PgPool,
) -> Result<SingleShopStats, Box<dyn std::error::Error + Send + Sync>> {
    // STEP 6: Find shop in locations table
    let location_id = match repository::find_location_by_shop_fuzzy(pool, shop_name, &city.state).await? {
        Some(id) => id,
        None => {
            repository::update_pending_by_shop(
                pool,
                shop_name,
                "failed",
                "shop_not_in_database",
                Some("Shop not found in locations table"),
            ).await?;
            return Err("Shop not in database".to_string().into());
        }
    };

    // STEP 7-8: Lookup Instagram profile using direct URL approach
    let profile_info = match crate::actions::apify_scraper::lookup_shop_instagram(shop_name, Some(&city.city)).await {
        Ok(profile) => profile,
        Err(e) => {
            {
                let error_msg = e.to_string();
                eprintln!("      ❌ Error looking up Instagram profile: {}", error_msg);
                repository::update_pending_by_shop(
                    pool,
                    shop_name,
                    "failed",
                    "shop_instagram_not_found",
                    Some(&format!("Could not find shop on Instagram: {}", error_msg)),
                )
                .await?;
            }
            return Err("Shop IG not found".to_string().into());
        }
    };

    // STEP 9: Extract handles from bio
    let bio = profile_info.bio.ok_or_else(|| "No bio found".to_string())?;
    let artist_handles = extract_handles_from_bio(&bio)
        .await
        .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { e.to_string().into() })?;

    if artist_handles.is_empty() {
        repository::update_pending_by_shop(
            pool,
            shop_name,
            "failed",
            "no_artists_in_bio",
            Some("Shop Instagram found but no artist handles in bio"),
        )
        .await?;
        return Err("No artists in bio".into());
    }

    // STEP 10-14: Process each artist
    let mut artists_added = 0;
    let mut artists_updated = 0;
    for handle in artist_handles {
        match process_artist_handle(&handle, location_id, pool).await {
            Ok(ProcessResult::Added) => artists_added += 1,
            Ok(ProcessResult::Updated) => artists_updated += 1,
            Ok(ProcessResult::Skipped) => {}
            Err(e) => eprintln!("      ⚠️  Error processing @{}: {}", handle, e),
        }
    }

    // STEP 15: Mark success only if at least 1 artist processed
    if artists_added > 0 || artists_updated > 0 {
        let notes = format!(
            "Found shop (location_id: {}), added {}, updated {}",
            location_id, artists_added, artists_updated
        );
        repository::update_pending_by_shop(pool, shop_name, "processed", "success", Some(&notes))
            .await?;
    } else {
        repository::update_pending_by_shop(
            pool,
            shop_name,
            "failed",
            "no_artists_matched",
            Some("Artists found in bio but none could be matched or created"),
        )
        .await?;
    }

    Ok(SingleShopStats {
        artists_added,
        artists_updated,
    })
}

enum ProcessResult {
    Added,
    Updated,
    Skipped,
}

async fn process_artist_handle(
    handle: &str,
    location_id: i64,
    pool: &PgPool,
) -> Result<ProcessResult, Box<dyn std::error::Error + Send + Sync>> {
    // Normalize handle to remove @ symbol
    let handle = normalize_instagram_handle(handle);

    // STEP 10-11: Check if artist exists by handle
    if let Some(_artist) =
        repository::find_artist_by_instagram_at_location(pool, &handle, location_id).await?
    {
        // Artist exists with this handle - skip
        return Ok(ProcessResult::Skipped);
    }

    // STEP 13: Get artist profile from Instagram
    let profile = crate::actions::apify_scraper::get_instagram_profile_info(&handle)
        .await
        .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { e.to_string().into() })?;

    // Normalize empty string to None (some profiles return "" instead of null)
    let artist_name = profile.full_name.clone().and_then(|name| {
        if name.trim().is_empty() {
            None
        } else {
            Some(name)
        }
    });

    // Check if artist exists by name
    if let Some(name) = &artist_name {
        if let Some(artist) =
            repository::find_artist_by_name_fuzzy(pool, name, location_id).await?
        {
            // STEP 12: Artist exists without IG - UPDATE
            repository::update_artist_instagram(pool, artist.id, &handle, artist.social_links)
                .await?;
            println!("      ✏️  Updated {} with @{}", name, handle);
            return Ok(ProcessResult::Updated);
        }
    }

    // STEP 14: Artist doesn't exist - INSERT
    let instagram_url = format!("https://instagram.com/{}", handle);
    let _artist_id = repository::insert_artist_with_instagram(
        pool,
        artist_name.as_deref(),
        location_id,
        &handle,
        &instagram_url,
    )
    .await?;
    println!("      ➕ Created artist (@{})", handle);

    Ok(ProcessResult::Added)
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

    // Process posts in parallel using 10 concurrent threads
    println!("⚡ Processing posts in 10 parallel threads...");

    let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
    let rx = Arc::new(tokio::sync::Mutex::new(rx));

    // Send all posts to the channel
    for post in posts_with_images {
        tx.send(post).unwrap();
    }
    drop(tx);

    let mut tasks = FuturesUnordered::new();
    let num_threads = 10;

    for thread_id in 0..num_threads {
        let pool = pool.clone();
        let city_owned = city.clone();
        let rx = Arc::clone(&rx);

        tasks.push(tokio::spawn(async move {
            let mut local_stats = CityStats {
                posts_found: 0,
                artists_added: 0,
                artists_updated: 0,
                artists_pending: 0,
                artists_added_from_shop_bios: 0,
                shops_scraped: 0,
            };
            let mut local_matched_shops: HashSet<i64> = HashSet::new();

            loop {
                let post = match rx.lock().await.recv().await {
                    Some(p) => p,
                    None => break,
                };

                match process_reddit_post(&post, &city_owned, &pool, &mut local_stats, &mut local_matched_shops).await {
                    Ok(_) => {}
                    Err(e) => {
                        let url = post.url.as_deref().unwrap_or("unknown");
                        eprintln!("⚠️  [Thread {}] Error processing post {}: {}", thread_id, url, e);
                    }
                }
            }

            (local_stats, local_matched_shops)
        }));
    }

    // Collect results from all threads
    while let Some(result) = tasks.next().await {
        match result {
            Ok((local_stats, local_matched_shops)) => {
                stats.artists_added += local_stats.artists_added;
                stats.artists_updated += local_stats.artists_updated;
                stats.artists_pending += local_stats.artists_pending;
                matched_shops.extend(local_matched_shops);
            }
            Err(e) => {
                eprintln!("⚠️  Thread panicked: {}", e);
            }
        }
    }

    println!("✅ Parallel processing complete!");

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

    // Use subreddit search URL to search within /r/tattoos for city name
    let search_url = format!("https://www.reddit.com/r/tattoos/search/?q={}&sort=top&t=year",
        urlencoding::encode(city));
    let input = serde_json::json!({
        "startUrls": [{"url": search_url}],
        "crawlCommentsPerPost": true,
        "maxPostsCount": config.max_posts,
        "maxCommentsPerPost": 1,
        "includeNSFW": true,
        "proxy": {
            "useApifyProxy": true,
            "apifyProxyGroups": ["RESIDENTIAL"]
        }
    });

    println!("🔍 Starting Reddit scrape for {} (async API)...", city);
    println!("📋 Apify input:");
    println!("{}", serde_json::to_string_pretty(&input).unwrap_or_default());

    let client = reqwest::Client::new();

    // Step 1: Start the run
    let start_url = format!(
        "https://api.apify.com/v2/acts/{}/runs?token={}",
        actor_id, api_token
    );

    println!("🚀 Starting Apify run...");
    let start_response = client.post(&start_url).json(&input).send().await?;

    if !start_response.status().is_success() {
        let error_text = start_response.text().await?;
        return Err(format!("Failed to start Apify run: {}", error_text).into());
    }

    let run_response: ApifyRunResponse = start_response.json().await?;
    let run_id = run_response.data.id;
    let dataset_id = run_response.data.default_dataset_id;

    println!("✅ Run started: {}", run_id);

    // Step 2: Poll for completion
    let status_url = format!(
        "https://api.apify.com/v2/acts/{}/runs/{}?token={}",
        actor_id, run_id, api_token
    );

    println!("⏳ Waiting for run to complete (polling every 10s)...");
    let mut elapsed = 0;
    loop {
        tokio::time::sleep(std::time::Duration::from_secs(10)).await;
        elapsed += 10;

        let status_response = client.get(&status_url).send().await?;
        let status_data: ApifyRunResponse = status_response.json().await?;
        let status = status_data.data.status;

        println!("   Status after {}s: {}", elapsed, status);

        match status.as_str() {
            "SUCCEEDED" => {
                println!("✅ Run completed successfully!");
                break;
            }
            "FAILED" | "ABORTED" | "TIMED-OUT" => {
                return Err(format!("Apify run failed with status: {}", status).into());
            }
            _ => {
                // Continue polling (RUNNING, READY, etc.)
            }
        }

        if elapsed > 900 {
            return Err("Apify run exceeded 15 minute timeout".into());
        }
    }

    // Step 3: Fetch results
    let dataset_url = format!(
        "https://api.apify.com/v2/datasets/{}/items?token={}",
        dataset_id, api_token
    );

    println!("📥 Fetching results from dataset...");
    let dataset_response = client.get(&dataset_url).send().await?;
    let response_text = dataset_response.text().await?;

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
        println!("  - Title: {:?}", first_post.title.as_ref().map(|s| {
            s.chars().take(50).collect::<String>()
        }));
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
// Helper Functions
// ============================================================================

async fn extract_handles_from_bio(bio: &str) -> Result<Vec<String>, Box<dyn std::error::Error + Send + Sync>> {
    let api_key = env::var("OPENAI_API_KEY")?;

    let prompt = format!(
        r#"Extract all Instagram handles mentioned in this tattoo shop's Instagram bio.
Look for patterns like @username or instagram.com/username.

Bio text:
{}

IMPORTANT: Return ONLY a JSON array (not an object), like this:
["handle1", "handle2", ...]

If no handles found, return: []"#,
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
                {"role": "system", "content": "You are a helpful assistant that extracts Instagram handles from bios. Return ONLY a JSON array, nothing else."},
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

    println!("      🤖 OpenAI response: {}", content);

    // Try to parse as direct array first
    let handles: Vec<String> = match serde_json::from_str(content) {
        Ok(h) => h,
        Err(e) => {
            // Try to parse as object with InstagramHandles field
            if let Ok(obj) = serde_json::from_str::<serde_json::Value>(content) {
                if let Some(handles_array) = obj.get("InstagramHandles").or_else(|| obj.get("handles")) {
                    serde_json::from_value(handles_array.clone()).unwrap_or_else(|_| Vec::new())
                } else {
                    println!("      ⚠️  Failed to parse handles from OpenAI response: {}", e);
                    Vec::new()
                }
            } else {
                println!("      ⚠️  Failed to parse OpenAI response as JSON: {}", e);
                Vec::new()
            }
        }
    };

    println!("      📋 Extracted {} handles: {:?}", handles.len(), handles);

    Ok(handles)
}

async fn extract_shop_info_from_artist_bio(
    bio: &str,
) -> Result<(String, String), Box<dyn std::error::Error + Send + Sync>> {
    let api_key = env::var("OPENAI_API_KEY")?;

    let prompt = format!(
        r#"Extract the tattoo shop name and Instagram handle from this artist's Instagram bio.

Artist bio text:
{}

IMPORTANT: Return ONLY a JSON object with two fields:
{{
  "shopName": "the shop name",
  "shopInstagramHandle": "the_instagram_handle_without_at_symbol"
}}

If no shop information found, return:
{{
  "shopName": "",
  "shopInstagramHandle": ""
}}"#,
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
                {"role": "system", "content": "You are a helpful assistant that extracts shop information from artist bios. Return ONLY a JSON object."},
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

    println!("      🤖 Shop extraction OpenAI response: {}", content);

    // Parse the JSON response
    let shop_info: serde_json::Value = serde_json::from_str(content)?;

    let shop_name = shop_info["shopName"]
        .as_str()
        .unwrap_or("")
        .to_string();

    let shop_handle = shop_info["shopInstagramHandle"]
        .as_str()
        .unwrap_or("")
        .trim_start_matches('@')
        .to_string();

    if shop_name.is_empty() || shop_handle.is_empty() {
        return Err("No shop information found in artist bio".into());
    }

    println!("      📋 Extracted shop: {} (@{})", shop_name, shop_handle);

    Ok((shop_name, shop_handle))
}

async fn extract_shop_names_from_fullname(
    full_name: &str,
) -> Result<Vec<String>, Box<dyn std::error::Error + Send + Sync>> {
    let api_key = env::var("OPENAI_API_KEY")?;

    let prompt = format!(
        r#"Extract all potential tattoo shop names from this Instagram profile fullName field, including FUZZY VARIATIONS that might match database records.

fullName: {}

Generate ALL possible variations that could be used to find this shop in a database:

1. Extract base shop name(s) from the fullName
2. For EACH base name, generate common variations:
   - Base name only (e.g., "Rabid Hands")
   - Base + "Tattoo" (e.g., "Rabid Hands Tattoo")
   - Base + "Tattoos" (e.g., "Rabid Hands Tattoos")
   - Base + "Tattoo Studio" (e.g., "Rabid Hands Tattoo Studio")
   - Base + "Tattoo Shop"
   - Base + "Tattoo & Body Piercing"
   - Base + "Tattoo & Piercing"
   - Abbreviation variations ("Company" → "Co", "The " prefix removal)

Examples:
- "WASABI | Seattle Tattoo Studio" → ["WASABI", "Wasabi Tattoo", "Wasabi Tattoos", "Seattle Tattoo Studio", "Seattle Tattoo"]
- "Rabid Hands" → ["Rabid Hands", "Rabid Hands Tattoo", "Rabid Hands Tattoos", "Rabid Hands Tattoo Studio"]
- "Slave to the Needle" → ["Slave to the Needle", "Slave to the Needle Tattoo", "Slave to the Needle Tattoo & Body Piercing", "Slave to the Needle Tattoo & Piercing"]

IMPORTANT: Return ONLY a JSON array of shop name strings (no explanations):
["Name1", "Name2", ...]

If no shop name can be extracted, return an empty array: []"#,
        full_name
    );

    let client = reqwest::Client::new();
    let response = client
        .post("https://api.openai.com/v1/chat/completions")
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .json(&serde_json::json!({
            "model": "gpt-4",
            "messages": [
                {"role": "system", "content": "You are a helpful assistant that extracts shop names. Return ONLY a JSON array."},
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

    println!("      🤖 Shop names extraction from fullName: {}", content);

    // Parse as array of strings
    let shop_names: Vec<String> = serde_json::from_str(content)?;

    if shop_names.is_empty() {
        return Err("No shop names extracted from fullName".into());
    }

    println!("      📋 Extracted {} potential shop names: {:?}", shop_names.len(), shop_names);

    Ok(shop_names)
}

/// Lookup shop via Google Places API and insert into database if found
/// Returns location_id if shop found and inserted, None otherwise
async fn lookup_and_create_shop_via_google(
    pool: &PgPool,
    shop_name: &str,
    state: &str,
) -> Result<Option<i64>, Box<dyn std::error::Error + Send + Sync>> {
    println!("      🔍 Shop not in database - trying Google Places lookup...");

    // STEP 1: Get state boundary from database
    let state_bounds = match get_state_boundary(pool, state).await {
        Ok(bounds) => bounds,
        Err(e) => {
            println!("      ❌ Failed to get state boundary for {}: {}", state, e);
            return Ok(None);
        }
    };

    // STEP 2: Search Google Places for shop name + "tattoo" in state
    let search_query = format!("{} tattoo", shop_name);
    println!("      📍 Searching Google Places: '{}' in {}", search_query, state);

    let result = match search_text_with_location(&search_query, &state_bounds, 5).await {
        Ok(res) => res,
        Err(e) => {
            println!("      ❌ Google Places API error: {}", e);
            return Ok(None);
        }
    };

    // STEP 3: Filter to only tattoo shops
    let places = match result.get("places").and_then(|p| p.as_array()) {
        Some(p) => p,
        None => {
            println!("      ❌ No places found in Google response");
            return Ok(None);
        }
    };

    println!("      📊 Google Places returned {} raw results", places.len());

    // Filter to only tattoo shops (body_art_service or art_studio)
    let tattoo_places: Vec<&Value> = places.iter()
        .filter(|place| is_tattoo_shop(place))
        .collect();

    println!("      🏪 {} are tattoo shops", tattoo_places.len());

    // STEP 4: Conservative approach - only proceed if exactly 1 tattoo shop
    if tattoo_places.len() != 1 {
        println!("      ⚠️  Ambiguous results ({} tattoo shops) - skipping for safety", tattoo_places.len());
        return Ok(None);
    }

    // STEP 5: Verify the single result has similar name (similarity >= 0.7)
    let matched_place = tattoo_places[0];
    let result_name = matched_place.get("displayName")
        .and_then(|d| d.get("text"))
        .and_then(|t| t.as_str())
        .unwrap_or("");

    let similarity = jaro_winkler(
        &shop_name.to_lowercase(),
        &result_name.to_lowercase()
    );

    println!("      📏 Name similarity: '{}' vs '{}' = {:.3}", shop_name, result_name, similarity);

    if similarity < 0.7 {
        println!("      ⚠️  Name similarity too low ({:.3} < 0.7) - skipping for safety", similarity);
        return Ok(None);
    }

    // Parse the single matching place to LocationInfo
    let single_place_result = serde_json::json!({
        "places": [matched_place]
    });
    let locations = parse_places_to_locations(&single_place_result);

    if locations.is_empty() {
        println!("      ❌ Failed to parse matched place");
        return Ok(None);
    }

    let location = &locations[0];
    println!("      ✅ Found exactly 1 tattoo shop with similar name: {} at {}", location.name, location.address);

    // STEP 6: Insert into locations table
    match repository::upsert_locations(pool, &locations).await {
        Ok(_) => println!("      💾 Inserted shop into database"),
        Err(e) => {
            println!("      ❌ Failed to insert location: {}", e);
            return Ok(None);
        }
    }

    // STEP 7: Query back to get the database id
    let location_id = match sqlx::query_scalar::<_, i64>(
        "SELECT id FROM locations WHERE _id = $1"
    )
    .bind(&location._id)
    .fetch_one(pool)
    .await {
        Ok(id) => {
            println!("      ✅ Created shop (location_id: {})", id);
            id
        }
        Err(e) => {
            println!("      ❌ Failed to retrieve location id: {}", e);
            return Ok(None);
        }
    };

    Ok(Some(location_id))
}

/// Get state boundary bounding box from database
async fn get_state_boundary(
    pool: &PgPool,
    state: &str,
) -> Result<LocationBounds, Box<dyn std::error::Error + Send + Sync>> {
    let row = sqlx::query!(
        "SELECT low_long, low_lat, high_long, high_lat FROM state_boundaries WHERE state_name = $1",
        state
    )
    .fetch_one(pool)
    .await?;

    Ok(LocationBounds {
        low_long: row.low_long,
        low_lat: row.low_lat,
        high_long: row.high_long,
        high_lat: row.high_lat,
    })
}

/// Process a single artist from pending list - artist-first approach
async fn process_artist_from_pending(
    pool: &PgPool,
    artist_handle: &str,
    city: &str,
    state: &str,
    _thread_id: usize,
) -> Result<usize, Box<dyn std::error::Error + Send + Sync>> {
    // Normalize handle to remove @, URLs, etc.
    let normalized_handle = normalize_instagram_handle(artist_handle);

    println!("   👤 Processing artist @{}...", normalized_handle);

    // STEP 1: Get artist IG profile/bio via Apify
    println!("      📱 Getting artist IG profile...");
    let artist_profile = match crate::actions::apify_scraper::get_instagram_profile_info(&normalized_handle).await {
        Ok(profile) => profile,
        Err(e) => {
            let error_msg = format!("Failed to get artist profile: {}", e);
            println!("      ❌ {}", error_msg);
            repository::update_pending_artist_status(pool, &normalized_handle, "failed", Some("artist_profile_not_found")).await?;
            return Err(error_msg.into());
        }
    };

    // STEP 2: Extract shop IG handle from artist bio via OpenAI
    let artist_bio = artist_profile.bio.ok_or_else(|| "Artist has no bio".to_string())?;
    println!("      🤖 Extracting shop IG handle from bio...");

    let (_, shop_ig_handle) = match extract_shop_info_from_artist_bio(&artist_bio).await {
        Ok((name, handle)) => (name, handle),
        Err(e) => {
            let error_msg = format!("No shop info in bio: {}", e);
            drop(e); // Drop error before await
            println!("      ⚠️  {}", error_msg);
            repository::update_pending_artist_status(pool, &normalized_handle, "failed", Some("no_shop_in_bio")).await?;
            return Err(error_msg.into());
        }
    };

    println!("      📋 Extracted shop IG handle: @{}", shop_ig_handle);

    // STEP 3: Get shop IG profile to get the actual shop name from fullName
    println!("      📱 Getting shop IG profile (@{})...", shop_ig_handle);
    let shop_profile = match crate::actions::apify_scraper::get_instagram_profile_info(&shop_ig_handle).await {
        Ok(profile) => profile,
        Err(e) => {
            let error_msg = format!("Failed to get shop profile: {}", e);
            println!("      ❌ {}", error_msg);
            repository::update_pending_artist_status(pool, &normalized_handle, "failed", Some("shop_instagram_not_found")).await?;
            return Err(error_msg.into());
        }
    };

    // STEP 4: Extract potential shop names from fullName using OpenAI
    let shop_full_name = shop_profile.full_name.as_deref().unwrap_or(&shop_ig_handle);
    println!("      🤖 Extracting shop names from fullName: '{}'...", shop_full_name);

    let potential_shop_names = match extract_shop_names_from_fullname(shop_full_name).await {
        Ok(names) => names,
        Err(e) => {
            let error_msg = format!("Failed to extract shop names from fullName: {}", e);
            drop(e);
            println!("      ⚠️  {}", error_msg);
            repository::update_pending_artist_status(pool, &normalized_handle, "failed", Some("failed_to_parse_fullname")).await?;
            return Err(error_msg.into());
        }
    };

    // STEP 5: Try to find shop in locations table using each potential name
    println!("      🔍 Looking up shop in database (trying {} potential names)...", potential_shop_names.len());

    let mut location_id = None;
    let mut matched_shop_name = String::new();

    for shop_name in &potential_shop_names {
        println!("         Trying: '{}' ({}, {})...", shop_name, city, state);
        if let Some((id, name)) = repository::find_shop_by_name_and_city(pool, shop_name, city, state).await? {
            location_id = Some(id);
            matched_shop_name = name;
            println!("         ✅ Match found!");
            break;
        } else {
            println!("         ❌ No match");
        }
    }

    let location_id = match location_id {
        Some(id) => {
            println!("      ✅ Found shop in database: {} (location_id: {})", matched_shop_name, id);
            id
        }
        None => {
            // Shop not in database - try Google Places lookup
            println!("      ⚠️  Shop not found in database (tried names: {:?})", potential_shop_names);

            // Try each potential shop name with Google Places
            let mut google_location_id = None;
            for shop_name in &potential_shop_names {
                if let Ok(Some(id)) = lookup_and_create_shop_via_google(pool, shop_name, state).await {
                    google_location_id = Some(id);
                    matched_shop_name = shop_name.clone();
                    break;
                }
            }

            match google_location_id {
                Some(id) => {
                    println!("      ✅ Created shop via Google Places: {} (location_id: {})", matched_shop_name, id);
                    id
                }
                None => {
                    let error_msg = format!("Shop not found in database or Google Places. Tried names: {:?}", potential_shop_names);
                    println!("      ❌ {}", error_msg);
                    repository::update_pending_artist_status(pool, &normalized_handle, "failed", Some("shop_not_found")).await?;
                    return Err(error_msg.into());
                }
            }
        }
    };

    // STEP 6: Extract artist handles from shop bio via OpenAI
    let shop_bio = shop_profile.bio.ok_or_else(|| "Shop has no bio".to_string())?;
    println!("      🤖 Extracting artist handles from shop bio...");

    let artist_handles = match extract_handles_from_bio(&shop_bio).await {
        Ok(handles) => handles,
        Err(e) => {
            let error_msg = format!("Failed to extract artist handles: {}", e);
            drop(e); // Drop error before await
            println!("      ❌ {}", error_msg);
            repository::update_pending_artist_status(pool, &normalized_handle, "failed", Some("failed_to_extract_artists")).await?;
            return Err(error_msg.into());
        }
    };

    if artist_handles.is_empty() {
        let error_msg = "No artist handles found in shop bio";
        println!("      ⚠️  {}", error_msg);
        repository::update_pending_artist_status(pool, &normalized_handle, "failed", Some("no_artists_in_shop_bio")).await?;
        return Err(error_msg.into());
    }

    println!("      📋 Found {} artists in shop bio", artist_handles.len());

    // STEP 7: Process all artists from shop bio
    let mut artists_processed = 0;

    for handle in &artist_handles {
        match process_artist_handle(handle, location_id, pool).await {
            Ok(ProcessResult::Added) => {
                println!("         ✅ Added artist {}", handle);
                artists_processed += 1;
            }
            Ok(ProcessResult::Updated) => {
                println!("         ✅ Updated artist {}", handle);
                artists_processed += 1;
            }
            Ok(ProcessResult::Skipped) => {
                println!("         ⏭️  Skipped artist {} (already up to date)", handle);
            }
            Err(e) => {
                println!("         ⚠️  Failed to process {}: {}", handle, e);
            }
        }
    }

    // STEP 8: Mark original artist as successfully processed
    repository::update_pending_artist_status(pool, &normalized_handle, "success", None).await?;

    println!("      ✅ Successfully processed {} artists from shop '{}'", artists_processed, matched_shop_name);

    Ok(artists_processed)
}

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
