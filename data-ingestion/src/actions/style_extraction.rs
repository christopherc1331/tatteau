use base64::Engine;
use rusqlite::Connection;
use serde_json::Value;
use std::{collections::HashMap, env, fs, path::Path, sync::Arc};
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
    get_artists_for_style_extraction, get_or_create_style_ids, mark_artist_styles_extracted,
    upsert_artist_styles, Artist,
};

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

        match process_artist_styles(&client, &artist, batch_size, confidence_threshold).await {
            Ok(extracted_styles) => {
                if extracted_styles.is_empty() {
                    println!("No styles extracted for artist: {}", artist.name);
                } else {
                    println!(
                        "Extracted {} unique styles for artist: {}",
                        extracted_styles.len(),
                        artist.name
                    );

                    match get_or_create_style_ids(conn, &extracted_styles) {
                        Ok(style_ids) => {
                            if let Err(e) = upsert_artist_styles(conn, artist.id, &style_ids) {
                                println!("Error saving styles for artist {}: {}", artist.name, e);
                            } else {
                                println!(
                                    "Successfully saved {} styles for artist: {}",
                                    style_ids.len(),
                                    artist.name
                                );
                            }
                        }
                        Err(e) => {
                            println!(
                                "Error mapping styles to IDs for artist {}: {}",
                                artist.name, e
                            );
                        }
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
    }

    println!("\n=== Style Extraction Complete ===");
    Ok(())
}

async fn process_artist_styles(
    client: &Arc<Client<OpenAIConfig>>,
    artist: &Artist,
    batch_size: usize,
    confidence_threshold: f64,
) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let images_path = Path::new(&artist.images_dir);

    if !images_path.exists() {
        println!(
            "Images directory does not exist for artist {}: {}",
            artist.name, artist.images_dir
        );
        return Ok(Vec::new());
    }

    let entries = fs::read_dir(images_path)?;
    let mut image_files = Vec::new();

    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        if let Some(ext) = path.extension() {
            let ext = ext.to_string_lossy().to_lowercase();
            if matches!(ext.as_str(), "jpg" | "jpeg" | "png" | "webp" | "gif") {
                image_files.push(path);
            }
        }
    }

    if image_files.is_empty() {
        println!("No image files found for artist: {}", artist.name);
        return Ok(Vec::new());
    }

    println!(
        "Found {} image files for artist: {}, processing in batches of {}",
        image_files.len(),
        artist.name,
        batch_size
    );

    let semaphore = Arc::new(Semaphore::new(3));
    let mut handles = Vec::new();

    for (batch_idx, batch) in image_files.chunks(batch_size).enumerate() {
        let client = Arc::clone(client);
        let semaphore = Arc::clone(&semaphore);
        let threshold = confidence_threshold;
        let batch_paths: Vec<_> = batch.to_vec();
        let artist_name = artist.name.clone();

        let handle = tokio::spawn(async move {
            let _permit = semaphore.acquire().await.unwrap();

            println!(
                "  Processing batch {} of {} images for artist: {}",
                batch_idx + 1,
                batch_paths.len(),
                artist_name
            );

            let mut content_parts = vec![
                ChatCompletionRequestUserMessageContentPart::Text(
                    ChatCompletionRequestMessageContentPartText {
                        text: format!(
                            "Analyze these {} images. For each image, determine if it shows a tattoo. If it is NOT a tattoo, respond with 'NOT_TATTOO'. If it IS a tattoo, identify the artistic styles with confidence scores above {}.

                              Format your response as a JSON array with one object per image (in order):
                              [
                                {{\"image\": 1, \"result\": \"NOT_TATTOO\"}},
                                {{\"image\": 2, \"result\": \"style1:0.95,style2:0.97\"}},
                                {{\"image\": 3, \"result\": \"\"}}
                              ]

                              Only include styles with confidence > {}. If no confident styles, use empty string for result.",
                            batch_paths.len(), threshold, threshold
                        ),
                    }
                )
            ];

            for image_path in &batch_paths {
                let image_data = match fs::read(image_path) {
                    Ok(data) => data,
                    Err(e) => {
                        println!("  Error reading image {:?}: {}", image_path.file_name(), e);
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
                .max_tokens(500u32)
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
                    return Vec::new();
                }
            };

            let timeout_duration = tokio::time::Duration::from_secs(90);

            match tokio::time::timeout(timeout_duration, client.chat().create(request)).await {
                Ok(Ok(response)) => {
                    if let Some(choice) = response.choices.first() {
                        if let Some(content) = &choice.message.content {
                            let content_trimmed = content.trim();

                            println!(
                                "  Raw GPT response for batch {}: '{}'",
                                batch_idx + 1,
                                content_trimmed
                            );

                            let mut batch_styles = Vec::new();

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
                                    for (img_idx, result) in results.iter().enumerate() {
                                        if img_idx >= batch_paths.len() {
                                            break;
                                        }

                                        let image_path = &batch_paths[img_idx];
                                        if let Some(result_str) =
                                            result.get("result").and_then(|r| r.as_str())
                                        {
                                            if result_str == "NOT_TATTOO" {
                                                println!(
                                                    "    Image {:?} is not a tattoo",
                                                    image_path.file_name()
                                                );
                                            } else if result_str.is_empty() {
                                                println!("    Image {:?} is a tattoo but no confident styles", image_path.file_name());
                                            } else {
                                                let mut image_styles = Vec::new();
                                                for style_entry in result_str.split(',') {
                                                    let style_entry = style_entry.trim();
                                                    if let Some((style, confidence_str)) =
                                                        style_entry.split_once(':')
                                                    {
                                                        let style = style
                                                            .trim()
                                                            .replace('_', " ")
                                                            .to_lowercase();
                                                        if is_valid_style_name(&style) {
                                                            if let Ok(confidence) =
                                                                confidence_str.trim().parse::<f64>()
                                                            {
                                                                if confidence > threshold {
                                                                    image_styles.push(style);
                                                                }
                                                            }
                                                        }
                                                    }
                                                }

                                                if !image_styles.is_empty() {
                                                    println!(
                                                        "    High-confidence styles for {:?}: {}",
                                                        image_path.file_name(),
                                                        image_styles.join(", ")
                                                    );
                                                    batch_styles.extend(image_styles);
                                                }
                                            }
                                        }
                                    }
                                }
                                Err(e) => {
                                    println!(
                                        "  Failed to parse JSON response for batch {}: {}",
                                        batch_idx + 1,
                                        e
                                    );
                                }
                            }

                            return batch_styles;
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
            Vec::new()
        });

        handles.push(handle);
    }

    let mut all_styles = Vec::new();
    for handle in handles {
        match handle.await {
            Ok(styles) => {
                all_styles.extend(styles);
            }
            Err(e) => {
                println!("  Task failed: {}", e);
            }
        }
    }

    let mut unique_styles = HashMap::new();
    for style in all_styles {
        *unique_styles.entry(style).or_insert(0) += 1;
    }

    let style_names: Vec<String> = unique_styles.keys().cloned().collect();

    if !style_names.is_empty() {
        println!(
            "  Unique styles found for {}: {}",
            artist.name,
            style_names.join(", ")
        );
    }

    Ok(style_names)
}

