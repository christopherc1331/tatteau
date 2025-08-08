use base64::Engine;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{collections::HashMap, env, fs, path::Path, process::Command, sync::Arc};
use tokio::sync::Semaphore;

use async_openai::{
    config::OpenAIConfig,
    types::{
        ChatCompletionRequestMessage, ChatCompletionRequestMessageContentPartImage,
        ChatCompletionRequestMessageContentPartText, ChatCompletionRequestUserMessage,
        ChatCompletionRequestUserMessageContent, ChatCompletionRequestUserMessageContentPart,
        CreateChatCompletionRequestArgs, ImageUrl,
    },
    Client,
};

use crate::repository::{
    get_artists_for_style_extraction, get_or_create_style_ids, insert_artist_image,
    insert_artist_image_styles, mark_artist_styles_extracted, update_openai_api_costs,
    upsert_artist_styles, Artist,
};

#[derive(Debug, Deserialize, Clone)]
struct InstaPost {
    shortcode: String,
    filepath: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct StyleResult {
    shortcode: String,
    styles: Vec<StyleConfidence>,
}

#[derive(Debug)]
struct BatchResult {
    style_results: Vec<StyleResult>,
    api_cost: f64,
}

#[derive(Debug, Serialize, Deserialize)]
struct StyleConfidence {
    style: String,
    confidence: f64,
}

fn is_valid_style_name(style: &str) -> bool {
    if style.is_empty() || style.len() > 50 {
        return false;
    }

    let invalid_phrases = [
        "this image",
        "shows",
        "tattoo",
        "image",
        "appears",
        "seems",
        "contains",
        "displays",
        "depicts",
        "features",
        "has",
        "is",
        "the style",
        "the tattoo",
        "analysis",
        "identified",
        "present",
    ];

    for phrase in &invalid_phrases {
        if style.contains(phrase) {
            return false;
        }
    }

    if style.contains('.') || style.contains('!') || style.contains('?') {
        return false;
    }

    let valid_chars = style.chars().all(|c| {
        c.is_alphabetic() || c.is_whitespace() || c == '-' || c == '&' || c.is_ascii_digit()
    });

    valid_chars
}

fn calculate_gpt4o_cost(prompt_tokens: u32, completion_tokens: u32) -> f64 {
    // GPT-4o pricing (as of latest known rates)
    let input_cost_per_1k = 0.0025;
    let output_cost_per_1k = 0.01;

    let input_cost = (prompt_tokens as f64 / 1000.0) * input_cost_per_1k;
    let output_cost = (completion_tokens as f64 / 1000.0) * output_cost_per_1k;

    input_cost + output_cost
}

pub async fn extract_styles(conn: &Connection) -> Result<(), Box<dyn std::error::Error>> {
    env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY environment variable must be set");

    let confidence_threshold: f64 = env::var("STYLE_CONFIDENCE_THRESHOLD")
        .unwrap_or_else(|_| "0.9".to_string())
        .parse()
        .expect("STYLE_CONFIDENCE_THRESHOLD must be a valid number between 0 and 1");

    let batch_size: usize = env::var("VISION_BATCH_SIZE")
        .unwrap_or_else(|_| "16".to_string())
        .parse()
        .expect("VISION_BATCH_SIZE must be a valid number between 1 and 16");

    let artist_limit: i16 = env::var("ARTIST_BATCH_LIMIT")
        .unwrap_or_else(|_| "10".to_string())
        .parse()
        .expect("ARTIST_BATCH_LIMIT must be a valid number");

    let cookie_file = env::var("INSTALOADER_COOKIE_FILE")
        .expect("INSTALOADER_COOKIE_FILE environment variable must be set");

    let max_posts: String = env::var("INSTALOADER_MAX_POSTS").unwrap_or_else(|_| "10".to_string());

    let artists = get_artists_for_style_extraction(conn, artist_limit)?;

    if artists.is_empty() {
        println!("No artists found that need style extraction.");
        return Ok(());
    }

    println!(
        "Found {} artists to process for style extraction",
        artists.len()
    );

    let client = Arc::new(Client::new());

    for artist in artists {
        println!(
            "\n=== Processing Artist: {} (ID: {}) ===",
            artist.name, artist.id
        );

        let ig_username = match &artist.ig_username {
            Some(username) => username.clone(),
            None => {
                println!(
                    "No valid Instagram username found for artist: {} - skipping",
                    artist.name
                );
                continue;
            }
        };
        println!("Instagram username: {}", ig_username);

        let insta_posts = match call_instaloader(&ig_username, &cookie_file, &max_posts) {
            Ok(posts) => posts,
            Err(e) => {
                println!("Error calling instaloader for {}: {}", artist.name, e);
                let _ = cleanup_instaloader_files(&ig_username);
                if let Err(e) = mark_artist_styles_extracted(conn, artist.id) {
                    println!("Error marking artist {} as processed: {}", artist.name, e);
                }
                continue;
            }
        };

        if insta_posts.is_empty() {
            println!("No posts retrieved from Instagram for {}", artist.name);
            let _ = cleanup_instaloader_files(&ig_username);
            if let Err(e) = mark_artist_styles_extracted(conn, artist.id) {
                println!("Error marking artist {} as processed: {}", artist.name, e);
            }
            continue;
        }

        println!(
            "Retrieved {} posts from Instagram for {}",
            insta_posts.len(),
            artist.name
        );

        match process_artist_posts(
            conn,
            &client,
            &artist,
            &insta_posts,
            batch_size,
            confidence_threshold,
        )
        .await
        {
            Ok(style_results) => {
                let mut all_artist_styles = HashMap::new();

                for result in style_results {
                    match insert_artist_image(conn, &result.shortcode, artist.id) {
                        Ok(artist_image_id) => {
                            let style_names: Vec<String> = result
                                .styles
                                .iter()
                                .filter(|s| s.confidence >= confidence_threshold)
                                .map(|s| s.style.clone())
                                .filter(|s| is_valid_style_name(s))
                                .collect();

                            if !style_names.is_empty() {
                                match get_or_create_style_ids(conn, &style_names) {
                                    Ok(style_ids) => {
                                        if let Err(e) = insert_artist_image_styles(
                                            conn,
                                            artist_image_id,
                                            &style_ids,
                                        ) {
                                            println!(
                                                "Error saving styles for image {}: {}",
                                                result.shortcode, e
                                            );
                                        } else {
                                            println!(
                                                "Saved {} styles for shortcode {}",
                                                style_ids.len(),
                                                result.shortcode
                                            );
                                        }

                                        for (name, id) in style_names.iter().zip(style_ids.iter()) {
                                            all_artist_styles.insert(name.clone(), *id);
                                        }
                                    }
                                    Err(e) => {
                                        println!(
                                            "Error mapping styles to IDs for image {}: {}",
                                            result.shortcode, e
                                        );
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            println!(
                                "Error inserting artist_image for {}: {}",
                                result.shortcode, e
                            );
                        }
                    }
                }

                if !all_artist_styles.is_empty() {
                    let artist_style_ids: Vec<i64> = all_artist_styles.values().copied().collect();
                    if let Err(e) = upsert_artist_styles(conn, artist.id, &artist_style_ids) {
                        println!(
                            "Error saving artist-level styles for {}: {}",
                            artist.name, e
                        );
                    } else {
                        println!(
                            "Successfully saved {} unique styles for artist: {}",
                            artist_style_ids.len(),
                            artist.name
                        );
                    }
                }
            }
            Err(e) => {
                println!("Error processing styles for artist {}: {}", artist.name, e);
            }
        }

        if let Err(e) = mark_artist_styles_extracted(conn, artist.id) {
            println!("Error marking artist {} as processed: {}", artist.name, e);
        } else {
            println!("Marked artist {} as styles_extracted", artist.name);
        }

        if let Err(e) = cleanup_instaloader_files(&ig_username) {
            println!(
                "Warning: Error cleaning up files for {}: {}",
                ig_username, e
            );
        }
    }

    println!("\n=== Style Extraction Complete ===");
    Ok(())
}

fn call_instaloader(
    username: &str,
    cookie_file: &str,
    max_posts: &str,
) -> Result<Vec<InstaPost>, Box<dyn std::error::Error>> {
    println!("Calling instaloader for username: {}", username);

    let output = Command::new("python")
        .arg("load_ig_profiles.py")
        .arg(username)
        .arg("--cookiefile")
        .arg(cookie_file)
        .arg("--max-posts")
        .arg(max_posts)
        .current_dir("data-ingestion")
        .output()?;

    if !output.status.success() {
        let error = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Instaloader failed: {}", error).into());
    }

    let json_file = format!("data-ingestion/{}_posts.json", username);
    let json_content = fs::read_to_string(&json_file)?;
    let posts: Vec<InstaPost> = serde_json::from_str(&json_content)?;

    Ok(posts)
}

fn cleanup_instaloader_files(username: &str) -> Result<(), Box<dyn std::error::Error>> {
    let json_file = format!("data-ingestion/{}_posts.json", username);
    if Path::new(&json_file).exists() {
        fs::remove_file(&json_file)?;
        println!("Deleted JSON file: {}", json_file);
    }

    let image_dir = format!("data-ingestion/{}", username);
    let image_path = Path::new(&image_dir);
    if image_path.exists() && image_path.is_dir() {
        fs::remove_dir_all(image_path)?;
        println!("Deleted image directory: {}", image_dir);
    }

    Ok(())
}

async fn process_artist_posts(
    conn: &Connection,
    client: &Arc<Client<OpenAIConfig>>,
    artist: &Artist,
    posts: &[InstaPost],
    batch_size: usize,
    confidence_threshold: f64,
) -> Result<Vec<StyleResult>, Box<dyn std::error::Error>> {
    println!(
        "Processing {} posts for artist: {}, in batches of {}",
        posts.len(),
        artist.name,
        batch_size
    );

    let semaphore = Arc::new(Semaphore::new(3));
    let mut handles = Vec::new();

    for (batch_idx, batch) in posts.chunks(batch_size).enumerate() {
        let client = Arc::clone(client);
        let semaphore = Arc::clone(&semaphore);
        let threshold = confidence_threshold;
        let batch_posts: Vec<InstaPost> = batch.to_vec();
        let artist_name = artist.name.clone();

        let handle = tokio::spawn(async move {
            let _permit = semaphore.acquire().await.unwrap();

            println!(
                "  Processing batch {} of {} posts for artist: {}",
                batch_idx + 1,
                batch_posts.len(),
                artist_name
            );

            let mut content_parts = vec![
                ChatCompletionRequestUserMessageContentPart::Text(
                    ChatCompletionRequestMessageContentPartText {
                        text: format!(
                            "Analyze these {} tattoo images. For each image, identify the artistic styles and provide confidence scores (0.0 to 1.0).

                            Format your response as a JSON array with one object per image (in order):
                            [
                              {{\"shortcode\": \"{}\", \"styles\": [{{\"style\": \"blackwork\", \"confidence\": 0.95}}, {{\"style\": \"geometric\", \"confidence\": 0.87}}]}},
                              {{\"shortcode\": \"{}\", \"styles\": [{{\"style\": \"traditional\", \"confidence\": 0.92}}]}},
                              {{\"shortcode\": \"{}\", \"styles\": []}}
                            ]

                            Include all styles you can identify, even with lower confidence. Use standard tattoo style names (e.g., blackwork, traditional, realism, watercolor, geometric, etc.).",
                            batch_posts.len(),
                            batch_posts.get(0).map(|p| &p.shortcode).unwrap_or(&"shortcode1".to_string()),
                            batch_posts.get(1).map(|p| &p.shortcode).unwrap_or(&"shortcode2".to_string()),
                            batch_posts.get(2).map(|p| &p.shortcode).unwrap_or(&"shortcode3".to_string())
                        ),
                    }
                )
            ];

            for post in &batch_posts {
                let image_path_str = format!("data-ingestion/{}", post.filepath);
                let image_path = Path::new(&image_path_str);

                let image_data = match fs::read(image_path) {
                    Ok(data) => data,
                    Err(e) => {
                        println!("  Error reading image {}: {}", post.filepath, e);
                        continue;
                    }
                };

                let base64_image = base64::engine::general_purpose::STANDARD.encode(&image_data);
                let mime_type = match image_path
                    .extension()
                    .and_then(|ext| ext.to_str())
                    .map(|ext| ext.to_lowercase())
                {
                    Some(ext) if ext == "jpg" || ext == "jpeg" => "image/jpeg",
                    Some(ext) if ext == "png" => "image/png",
                    Some(ext) if ext == "webp" => "image/webp",
                    Some(ext) if ext == "gif" => "image/gif",
                    _ => "image/jpeg",
                };

                let data_url = format!("data:{};base64,{}", mime_type, base64_image);
                content_parts.push(ChatCompletionRequestUserMessageContentPart::ImageUrl(
                    ChatCompletionRequestMessageContentPartImage {
                        image_url: ImageUrl {
                            url: data_url,
                            detail: None,
                        },
                    },
                ));
            }

            let user_message =
                ChatCompletionRequestMessage::User(ChatCompletionRequestUserMessage {
                    content: ChatCompletionRequestUserMessageContent::Array(content_parts),
                    name: None,
                });

            let request = match CreateChatCompletionRequestArgs::default()
                .model("gpt-4o")
                .max_tokens(1000u32)
                .messages([user_message])
                .build()
            {
                Ok(req) => req,
                Err(e) => {
                    println!(
                        "  Error building request for batch {}: {}",
                        batch_idx + 1,
                        e
                    );
                    return BatchResult {
                        style_results: Vec::new(),
                        api_cost: 0.0,
                    };
                }
            };

            let timeout_duration = tokio::time::Duration::from_secs(90);

            match tokio::time::timeout(timeout_duration, client.chat().create(request)).await {
                Ok(Ok(response)) => {
                    let api_cost = if let Some(usage) = &response.usage {
                        let cost =
                            calculate_gpt4o_cost(usage.prompt_tokens, usage.completion_tokens);
                        println!(
                            "  Batch {} API cost: ${:.4} (tokens: {} prompt + {} completion)",
                            batch_idx + 1,
                            cost,
                            usage.prompt_tokens,
                            usage.completion_tokens
                        );
                        cost
                    } else {
                        0.0
                    };

                    if let Some(choice) = response.choices.first() {
                        if let Some(content) = &choice.message.content {
                            let content_trimmed = content.trim();

                            println!(
                                "  Raw GPT response for batch {}: '{}'",
                                batch_idx + 1,
                                content_trimmed
                            );

                            let json_content = if content_trimmed.starts_with("```json") {
                                content_trimmed
                                    .strip_prefix("```json")
                                    .and_then(|s| s.strip_suffix("```"))
                                    .unwrap_or(content_trimmed)
                                    .trim()
                                    .to_string()
                            } else if content_trimmed.starts_with("```") {
                                let lines: Vec<&str> = content_trimmed.lines().collect();
                                if lines.len() > 2 {
                                    lines[1..lines.len() - 1].join("\n")
                                } else {
                                    content_trimmed.to_string()
                                }
                            } else {
                                content_trimmed.to_string()
                            };

                            match serde_json::from_str::<Vec<Value>>(&json_content) {
                                Ok(results) => {
                                    let mut batch_results = Vec::new();

                                    for (idx, result) in results.iter().enumerate() {
                                        if idx >= batch_posts.len() {
                                            break;
                                        }

                                        let shortcode = result
                                            .get("shortcode")
                                            .and_then(|s| s.as_str())
                                            .unwrap_or(&batch_posts[idx].shortcode)
                                            .to_string();

                                        let mut styles = Vec::new();

                                        if let Some(styles_array) =
                                            result.get("styles").and_then(|s| s.as_array())
                                        {
                                            for style_obj in styles_array {
                                                if let (Some(style_name), Some(confidence)) = (
                                                    style_obj.get("style").and_then(|s| s.as_str()),
                                                    style_obj
                                                        .get("confidence")
                                                        .and_then(|c| c.as_f64()),
                                                ) {
                                                    let style_name = style_name
                                                        .trim()
                                                        .replace('_', " ")
                                                        .to_lowercase();

                                                    if is_valid_style_name(&style_name)
                                                        && confidence >= threshold
                                                    {
                                                        styles.push(StyleConfidence {
                                                            style: style_name,
                                                            confidence,
                                                        });
                                                    }
                                                }
                                            }
                                        }

                                        println!(
                                            "    Shortcode {}: {} high-confidence styles",
                                            shortcode,
                                            styles.len()
                                        );

                                        batch_results.push(StyleResult { shortcode, styles });
                                    }

                                    return BatchResult {
                                        style_results: batch_results,
                                        api_cost,
                                    };
                                }
                                Err(e) => {
                                    println!(
                                        "  Failed to parse JSON response for batch {}: {}",
                                        batch_idx + 1,
                                        e
                                    );
                                }
                            }
                        }
                    }
                    println!("  No content in response for batch {}", batch_idx + 1);
                }
                Ok(Err(e)) => {
                    println!("  API error processing batch {}: {}", batch_idx + 1, e);
                }
                Err(_) => {
                    println!("  Timeout processing batch {}", batch_idx + 1);
                }
            }

            tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
            BatchResult {
                style_results: Vec::new(),
                api_cost: 0.0,
            }
        });

        handles.push(handle);
    }

    let mut all_results = Vec::new();
    let mut total_cost = 0.0;
    let mut api_calls = 0;

    for handle in handles {
        match handle.await {
            Ok(batch_result) => {
                all_results.extend(batch_result.style_results);
                total_cost += batch_result.api_cost;
                if batch_result.api_cost > 0.0 {
                    api_calls += 1;
                }
            }
            Err(e) => {
                println!("  Task failed: {}", e);
            }
        }
    }

    if api_calls > 0 {
        let avg_cost = total_cost / api_calls as f64;
        for _ in 0..api_calls {
            if let Err(e) = update_openai_api_costs(conn, "style_extraction", "gpt-4o", avg_cost) {
                println!("  Warning: Failed to update API cost tracking: {}", e);
            }
        }
        println!(
            "  Total API cost for {}: ${:.4} across {} calls (avg: ${:.4})",
            artist.name, total_cost, api_calls, avg_cost
        );
    }

    Ok(all_results)
}
