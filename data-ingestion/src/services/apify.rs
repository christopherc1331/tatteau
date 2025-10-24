// Apify API Service Module
// Centralized service for all Apify actor interactions

use reqwest::Client;
use serde::de::DeserializeOwned;
use serde::Deserialize;
use serde_json::json;
use std::env;

// ============================================================================
// Private Generic Apify Runners
// ============================================================================

/// Run an Apify actor synchronously (using run-sync endpoint)
/// This endpoint waits for completion and returns results immediately
async fn run_apify_sync<T: DeserializeOwned>(
    actor_id: &str,
    input: serde_json::Value,
    memory_mb: Option<u16>,
    timeout_secs: u64,
) -> Result<Vec<T>, Box<dyn std::error::Error + Send + Sync>> {
    let api_token = env::var("APIFY_API_TOKEN")?;

    let url = if let Some(mem) = memory_mb {
        format!(
            "https://api.apify.com/v2/acts/{}/run-sync-get-dataset-items?token={}&memory={}",
            actor_id, api_token, mem
        )
    } else {
        format!(
            "https://api.apify.com/v2/acts/{}/run-sync-get-dataset-items?token={}",
            actor_id, api_token
        )
    };

    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(timeout_secs))
        .build()?;

    let response = client.post(&url).json(&input).send().await?;

    if !response.status().is_success() {
        let status = response.status();
        let error_text = response.text().await?;
        return Err(format!(
            "Apify actor {} failed with status {}: {}",
            actor_id, status, error_text
        )
        .into());
    }

    let response_text = response.text().await?;

    let results: Vec<T> = serde_json::from_str(&response_text).map_err(|e| {
        let preview = if response_text.len() > 500 {
            &response_text[..500]
        } else {
            &response_text
        };
        format!("Failed to parse Apify response: {}. Preview: {}", e, preview)
    })?;

    Ok(results)
}

/// Run an Apify actor asynchronously (with polling for completion)
/// This is used for actors that take longer to complete
#[derive(Deserialize)]
struct ApifyRunResponse {
    data: ApifyRunData,
}

#[derive(Deserialize)]
struct ApifyRunData {
    id: String,
    #[serde(rename = "defaultDatasetId")]
    default_dataset_id: String,
    status: String,
}

async fn run_apify_async<T: DeserializeOwned>(
    actor_id: &str,
    input: serde_json::Value,
    timeout_secs: u64,
) -> Result<Vec<T>, Box<dyn std::error::Error + Send + Sync>> {
    let api_token = env::var("APIFY_API_TOKEN")?;
    let client = Client::new();

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
            _ => {} // Continue polling (RUNNING, READY, etc.)
        }

        if elapsed > timeout_secs {
            return Err(format!("Apify run exceeded {} second timeout", timeout_secs).into());
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

    let results: Vec<T> = serde_json::from_str(&response_text).map_err(|e| {
        format!("Failed to parse dataset response: {}", e)
    })?;

    Ok(results)
}

// ============================================================================
// Instagram Profile Scraper (using apify~instagram-profile-scraper)
// ============================================================================

#[derive(Debug, Deserialize, Clone)]
pub struct InstagramProfile {
    pub username: String,
    #[serde(rename = "fullName")]
    pub full_name: Option<String>,
    pub biography: Option<String>,
    #[serde(rename = "followersCount")]
    pub followers_count: Option<i32>,
    #[serde(rename = "followsCount")]
    pub follows_count: Option<i32>,
    #[serde(rename = "postsCount")]
    pub posts_count: Option<i32>,
    pub verified: Option<bool>,
    #[serde(rename = "isBusinessAccount")]
    pub is_business_account: Option<bool>,
    pub private: Option<bool>,
}

/// Get Instagram profile info using the dedicated instagram-profile-scraper actor
/// This is more direct and reliable than using instagram-scraper with resultsType: "details"
pub async fn get_instagram_profile(
    username: &str,
) -> Result<InstagramProfile, Box<dyn std::error::Error + Send + Sync>> {
    println!("📱 Getting Instagram profile info for @{}", username);

    let input = json!({
        "usernames": [username]
    });

    let mut profiles = run_apify_sync::<InstagramProfile>(
        "apify~instagram-profile-scraper",
        input,
        None,
        120,
    )
    .await?;

    if profiles.is_empty() {
        return Err(format!("No profile found for @{}", username).into());
    }

    Ok(profiles.remove(0))
}

// ============================================================================
// Instagram Search (using apify~instagram-scraper for search functionality)
// ============================================================================

/// Search for Instagram artist by name
/// Tries multiple search query variations and validates results
/// Returns the first valid tattoo-related profile found
pub async fn search_instagram_artist(
    artist_name: &str,
) -> Result<InstagramProfile, Box<dyn std::error::Error + Send + Sync>> {
    // Note: instagram-scraper uses different field names than profile-scraper
    #[derive(Deserialize)]
    struct ScraperProfile {
        username: String,
        #[serde(rename = "fullName")]
        full_name: Option<String>,
        biography: Option<String>,
    }

    // Try multiple search query variations
    let search_queries = vec![
        format!("{} Artist", artist_name),
        format!("{} Tattoo Artist", artist_name),
        format!("{} Tattoo", artist_name),
    ];

    println!("🔍 Searching Instagram for artist: '{}'", artist_name);

    for search_query in search_queries {
        println!("   Trying search: '{}'...", search_query);

        let input = json!({
            "addParentData": false,
            "enhanceUserSearchWithFacebookPage": false,
            "isUserReelFeedURL": false,
            "isUserTaggedFeedURL": false,
            "resultsLimit": 5,
            "resultsType": "details",
            "search": search_query,
            "searchLimit": 10,
            "searchType": "user"
        });

        let scraper_profiles = match run_apify_sync::<ScraperProfile>(
            "apify~instagram-scraper",
            input,
            Some(256),
            300,
        )
        .await
        {
            Ok(profiles) => profiles,
            Err(e) => {
                println!("      ❌ Failed: {}", e);
                continue;
            }
        };

        if scraper_profiles.is_empty() {
            println!("      ❌ No results");
            continue;
        }

        // Validate results - look for tattoo-related profiles
        for profile in scraper_profiles {
            let bio_lower = profile.biography.as_ref().map(|b| b.to_lowercase());

            // Check if bio contains tattoo/artist keywords
            let is_tattoo_related = bio_lower.as_ref().map_or(false, |bio| {
                bio.contains("tattoo") || bio.contains("artist") || bio.contains("ink")
            });

            if is_tattoo_related {
                println!("      ✅ Found valid artist: @{}", profile.username);
                return Ok(InstagramProfile {
                    username: profile.username,
                    full_name: profile.full_name,
                    biography: profile.biography,
                    followers_count: None,
                    follows_count: None,
                    posts_count: None,
                    verified: None,
                    is_business_account: None,
                    private: None,
                });
            } else {
                println!("      ⚠️  Skipping @{} - bio doesn't mention tattoo/artist", profile.username);
            }
        }

        println!("      ❌ No valid tattoo artists in results");
    }

    Err(format!("No Instagram profile found for artist '{}'", artist_name).into())
}

// ============================================================================
// Reddit Scraper (using harshmaur~reddit-scraper)
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct RedditPost {
    #[serde(rename = "postUrl")]
    pub url: Option<String>,
    #[serde(rename = "dataType")]
    pub data_type: Option<String>,
    pub title: Option<String>,
    pub body: Option<String>,
    pub images: Option<Vec<String>>,
    #[serde(rename = "contentUrl")]
    pub content_url: Option<String>,
}

/// Scrape Reddit posts from r/tattoos for a given city
/// Uses "top of all time" sorting to get the best posts
pub async fn scrape_reddit_posts(
    city: &str,
    max_posts: i32,
) -> Result<Vec<RedditPost>, Box<dyn std::error::Error + Send + Sync>> {
    // Build search URL - changed from &t=year to &t=all for "top of all time"
    let search_url = format!(
        "https://www.reddit.com/r/tattoos/search/?q={}&sort=top&t=all",
        urlencoding::encode(city)
    );

    let input = json!({
        "startUrls": [{"url": search_url}],
        "crawlCommentsPerPost": true,
        "maxPostsCount": max_posts,
        "maxCommentsPerPost": 0,
        "includeNSFW": true,
        "proxy": {
            "useApifyProxy": true,
            "apifyProxyGroups": ["RESIDENTIAL"]
        }
    });

    println!("🔍 Starting Reddit scrape for {} (top of all time)...", city);
    println!("📋 Apify input:");
    println!("{}", serde_json::to_string_pretty(&input).unwrap_or_default());

    let posts = run_apify_async::<RedditPost>(
        "harshmaur~reddit-scraper",
        input,
        900, // 15 minute timeout
    )
    .await?;

    println!("📥 Retrieved {} posts from Reddit", posts.len());

    Ok(posts)
}
