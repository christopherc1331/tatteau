use reqwest;
use serde::{Deserialize, Deserializer, Serialize};
use std::env;
use chrono::DateTime;

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
                    .map_err(|_| de::Error::custom(format!("invalid ISO 8601 timestamp: {}", value)))
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
