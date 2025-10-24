// Apify API Service Module
// Centralized service for all Apify actor interactions

use reqwest::Client;
use serde::Deserialize;
use serde_json::json;
use std::env;

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
    let api_token = env::var("APIFY_API_TOKEN")?;
    let actor_id = "apify~instagram-profile-scraper";

    // Use run-sync for immediate results
    let url = format!(
        "https://api.apify.com/v2/acts/{}/run-sync-get-dataset-items?token={}",
        actor_id, api_token
    );

    let input = json!({
        "usernames": [username]
    });

    println!("📱 Getting Instagram profile info for @{}", username);

    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(120))
        .build()?;

    let response = client.post(&url).json(&input).send().await?;

    if !response.status().is_success() {
        let status = response.status();
        let error_text = response.text().await?;
        return Err(format!(
            "Apify instagram-profile-scraper failed with status {}: {}",
            status, error_text
        )
        .into());
    }

    let response_text = response.text().await?;

    let mut profiles: Vec<InstagramProfile> = match serde_json::from_str(&response_text) {
        Ok(profiles) => profiles,
        Err(e) => {
            let preview = if response_text.len() > 500 {
                &response_text[..500]
            } else {
                &response_text
            };
            eprintln!("Failed to parse Instagram profile response. Error: {}", e);
            eprintln!("Response preview: {}", preview);
            return Err(format!("Failed to parse Instagram profile response: {}", e).into());
        }
    };

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
    let api_token = env::var("APIFY_API_TOKEN")?;
    let actor_id = "apify~instagram-scraper";

    let url = format!(
        "https://api.apify.com/v2/acts/{}/run-sync-get-dataset-items?token={}&memory=256",
        actor_id, api_token
    );

    // Try multiple search query variations
    let search_queries = vec![
        format!("{} Artist", artist_name),
        format!("{} Tattoo Artist", artist_name),
        format!("{} Tattoo", artist_name),
    ];

    println!("🔍 Searching Instagram for artist: '{}'", artist_name);

    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(300))
        .build()?;

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

        let response = match client.post(&url).json(&input).send().await {
            Ok(resp) => resp,
            Err(e) => {
                println!("      ❌ Request failed: {}", e);
                continue;
            }
        };

        if !response.status().is_success() {
            println!("      ❌ HTTP {}", response.status());
            continue;
        }

        let response_text = match response.text().await {
            Ok(text) => text,
            Err(e) => {
                println!("      ❌ Failed to read response: {}", e);
                continue;
            }
        };

        // Note: instagram-scraper uses different field names than profile-scraper
        #[derive(Deserialize)]
        struct ScraperProfile {
            username: String,
            #[serde(rename = "fullName")]
            full_name: Option<String>,
            biography: Option<String>,
        }

        let scraper_profiles: Vec<ScraperProfile> = match serde_json::from_str(&response_text) {
            Ok(profiles) => profiles,
            Err(e) => {
                println!("      ❌ Failed to parse: {}", e);
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
