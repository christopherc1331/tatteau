use chrono::DateTime;
use reqwest;
use serde::{Deserialize, Deserializer, Serialize};
use std::env;

// Custom deserializer for timestamp that handles both ISO 8601 strings and i64 Unix timestamps
mod timestamp_deserializer {
    use super::*;
    use serde::de::{self, Visitor};
    use std::fmt;

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<i64>, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct TimestampVisitor;

        impl<'de> Visitor<'de> for TimestampVisitor {
            type Value = Option<i64>;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("an ISO 8601 string, Unix timestamp, or null")
            }

            fn visit_none<E>(self) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(None)
            }

            fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
            where
                D: Deserializer<'de>,
            {
                deserializer.deserialize_any(TimestampValueVisitor)
            }

            fn visit_unit<E>(self) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(None)
            }
        }

        struct TimestampValueVisitor;

        impl<'de> Visitor<'de> for TimestampValueVisitor {
            type Value = Option<i64>;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("an ISO 8601 string or Unix timestamp")
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                // Parse ISO 8601 string
                DateTime::parse_from_rfc3339(value)
                    .map(|dt| Some(dt.timestamp()))
                    .map_err(|_| {
                        de::Error::custom(format!("invalid ISO 8601 timestamp: {}", value))
                    })
            }

            fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(Some(value))
            }

            fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(Some(value as i64))
            }

            fn visit_none<E>(self) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(None)
            }

            fn visit_unit<E>(self) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(None)
            }
        }

        deserializer.deserialize_option(TimestampVisitor)
    }
}

#[derive(Debug, Serialize)]
struct ApifyInput {
    #[serde(rename = "directUrls")]
    direct_urls: Vec<String>,
    #[serde(rename = "resultsType")]
    results_type: String,
    #[serde(rename = "resultsLimit")]
    results_limit: i32,
    #[serde(rename = "includeNestedComments")]
    include_nested_comments: bool,
    #[serde(rename = "searchType")]
    search_type: String,
    #[serde(rename = "searchLimit")]
    search_limit: i32,
}

#[derive(Debug, Deserialize)]
pub struct ApifyPost {
    #[serde(rename = "shortCode")]
    pub shortcode: String,
    #[serde(rename = "displayUrl")]
    pub display_url: Option<String>,
    #[serde(deserialize_with = "timestamp_deserializer::deserialize")]
    pub timestamp: Option<i64>,
}

pub async fn scrape_instagram_profile(
    username: &str,
    max_posts: i32,
) -> Result<Vec<ApifyPost>, Box<dyn std::error::Error>> {
    let api_token =
        env::var("APIFY_API_TOKEN").expect("APIFY_API_TOKEN environment variable must be set");

    let actor_id = "apify~instagram-scraper";
    let url = format!(
        "https://api.apify.com/v2/acts/{}/run-sync-get-dataset-items?token={}&memory=256",
        actor_id, api_token
    );

    let input = ApifyInput {
        direct_urls: vec![format!("https://www.instagram.com/{}", username)],
        results_type: "posts".to_string(),
        results_limit: max_posts,
        include_nested_comments: false,
        search_type: "user".to_string(),
        search_limit: 1,
    };

    println!(
        "📱 Calling Apify Instagram scraper for @{} (max {} posts)",
        username, max_posts
    );

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(300))
        .build()?;

    let response = client.post(&url).json(&input).send().await?;

    if !response.status().is_success() {
        let status = response.status();
        let error_text = response.text().await?;
        return Err(format!(
            "Apify API request failed with status {}: {}",
            status, error_text
        )
        .into());
    }

    // First get the response as text to debug if needed
    let response_text = response.text().await?;

    // Try to parse the JSON
    let posts: Vec<ApifyPost> = match serde_json::from_str(&response_text) {
        Ok(posts) => posts,
        Err(e) => {
            // Log the first 500 chars of the response for debugging
            let preview = if response_text.len() > 500 {
                &response_text[..500]
            } else {
                &response_text
            };
            println!("Failed to parse Apify response. Error: {}", e);
            println!("Response preview: {}", preview);
            return Err(format!("Failed to parse Apify response: {}", e).into());
        }
    };

    println!("📥 Retrieved {} posts from Apify", posts.len());

    Ok(posts)
}

pub async fn download_image(url: &str) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let response = client.get(url).send().await?;

    if !response.status().is_success() {
        return Err(format!("Failed to download image: {}", response.status()).into());
    }

    let bytes = response.bytes().await?;
    Ok(bytes.to_vec())
}

// ============================================================================
// Instagram Search & Profile Scraping (for Reddit Scraper)
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct InstagramProfileInfo {
    pub username: String,
    #[serde(rename = "fullName")]
    pub full_name: Option<String>,
    #[serde(rename = "biography")]
    pub bio: Option<String>,
}

/// Generate likely Instagram username variations from a shop name
/// Example: "Moonlight Tattoo" -> ["moonlighttattoo", "moonlight_tattoo", "moonlighttattooseattle"]
fn generate_username_variations(shop_name: &str, city: Option<&str>) -> Vec<String> {
    let base = shop_name
        .to_lowercase()
        .replace(" tattoo", "")
        .replace(" ink", "")
        .replace(" ", "")
        .replace("'", "")
        .replace("&", "and")
        .replace(".", "");

    let mut variations = vec![
        format!("{}tattoo", base),
        format!("{}_tattoo", base),
        base.clone(),
    ];

    // Add city variations if provided
    if let Some(city) = city {
        let city_clean = city.to_lowercase().replace(" ", "");
        variations.push(format!("{}tattoo{}", base, city_clean));
        variations.push(format!("{}{}", base, city_clean));
    }

    variations
}

/// Lookup Instagram profile by trying direct URL with likely usernames
/// Uses resultsType: "details" to get profile info including bio in a single request
pub async fn lookup_shop_instagram(
    shop_name: &str,
    city: Option<&str>,
) -> Result<InstagramProfileInfo, Box<dyn std::error::Error + Send + Sync>> {
    let api_token =
        env::var("APIFY_API_TOKEN").expect("APIFY_API_TOKEN environment variable must be set");

    let actor_id = "apify~instagram-scraper";
    let url = format!(
        "https://api.apify.com/v2/acts/{}/run-sync-get-dataset-items?token={}&memory=256",
        actor_id, api_token
    );

    let usernames = generate_username_variations(shop_name, city);
    println!("🔍 Trying Instagram usernames for '{}': {:?}", shop_name, usernames);

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(300))
        .build()?;

    // Try each username variation with resultsType: "details" to get bio
    for username in usernames {
        let input = ApifyInput {
            direct_urls: vec![format!("https://www.instagram.com/{}/", username)],
            results_type: "details".to_string(),
            results_limit: 1,
            include_nested_comments: false,
            search_type: "user".to_string(),
            search_limit: 1,
        };

        println!("   Trying @{}...", username);

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

        // Parse as InstagramProfileInfo
        let mut profiles: Vec<InstagramProfileInfo> = match serde_json::from_str(&response_text) {
            Ok(profiles) => profiles,
            Err(e) => {
                println!("      ❌ Failed to parse: {}", e);
                continue;
            }
        };

        if profiles.is_empty() {
            println!("      ❌ No profile found");
            continue;
        }

        let profile = profiles.remove(0);

        // Validate it's a tattoo-related account
        if let Some(bio) = &profile.bio {
            if bio.to_lowercase().contains("tattoo") || bio.to_lowercase().contains("ink") {
                println!("      ✅ Confirmed tattoo shop: @{}", profile.username);
                return Ok(profile);
            } else {
                println!("      ⚠️  Profile @{} doesn't mention tattoo/ink in bio", profile.username);
            }
        }
    }

    Err(format!("No Instagram profile found for '{}' (tried {:?})", shop_name, generate_username_variations(shop_name, city)).into())
}

/// Get Instagram profile info for an artist by username
/// Used for artist processing (not shop lookup)
pub async fn get_instagram_profile_info(
    username: &str,
) -> Result<InstagramProfileInfo, Box<dyn std::error::Error + Send + Sync>> {
    let api_token =
        env::var("APIFY_API_TOKEN").expect("APIFY_API_TOKEN environment variable must be set");

    let actor_id = "apify~instagram-scraper";
    let url = format!(
        "https://api.apify.com/v2/acts/{}/run-sync-get-dataset-items?token={}&memory=256",
        actor_id, api_token
    );

    let input = ApifyInput {
        direct_urls: vec![format!("https://www.instagram.com/{}/", username)],
        results_type: "details".to_string(),
        results_limit: 1,
        include_nested_comments: false,
        search_type: "user".to_string(),
        search_limit: 1,
    };

    println!("📱 Getting Instagram profile info for @{}", username);

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(300))
        .build()?;

    let response = client.post(&url).json(&input).send().await?;

    if !response.status().is_success() {
        let status = response.status();
        let error_text = response.text().await?;
        return Err(format!(
            "Apify Instagram Profile scraper failed with status {}: {}",
            status, error_text
        )
        .into());
    }

    let response_text = response.text().await?;

    let mut profiles: Vec<InstagramProfileInfo> = match serde_json::from_str(&response_text) {
        Ok(profiles) => profiles,
        Err(e) => {
            let preview = if response_text.len() > 500 {
                &response_text[..500]
            } else {
                &response_text
            };
            println!("Failed to parse Instagram profile response. Error: {}", e);
            println!("Response preview: {}", preview);
            return Err(format!("Failed to parse Instagram profile response: {}", e).into());
        }
    };

    if profiles.is_empty() {
        return Err(format!("No profile found for @{}", username).into());
    }

    Ok(profiles.remove(0))
}
