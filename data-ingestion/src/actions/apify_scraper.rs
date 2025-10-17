use reqwest;
use serde::{Deserialize, Serialize};
use std::env;

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
