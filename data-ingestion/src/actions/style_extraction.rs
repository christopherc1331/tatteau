use base64::Engine;
use indicatif::{ProgressBar, ProgressStyle};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{collections::HashMap, env, fs, path::Path, process::Command, sync::{Arc, Mutex}};
use tokio::sync::Semaphore;
use futures::stream::{FuturesUnordered, StreamExt};

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

pub async fn extract_styles(conn: Connection) -> Result<(), Box<dyn std::error::Error>> {
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

    let concurrent_artists: usize = env::var("CONCURRENT_ARTISTS")
        .unwrap_or_else(|_| "3".to_string())
        .parse()
        .expect("CONCURRENT_ARTISTS must be a valid number");

    let artists = get_artists_for_style_extraction(&conn, artist_limit)?;

    if artists.is_empty() {
        println!("🔍 No artists found that need style extraction.");
        return Ok(());
    }

    println!("🎯 Instagram Style Extraction Started");
    println!("📊 Configuration:");
    println!("   • Artists to process: {}", artists.len());
    println!("   • Concurrent artists: {}", concurrent_artists);
    println!("   • Confidence threshold: {}", confidence_threshold);
    println!("   • Vision batch size: {}", batch_size);
    println!("   • Max posts per artist: {}", max_posts);
    println!("   • Cookie file: {}", cookie_file);

    let progress = ProgressBar::new(artists.len() as u64);
    progress.set_style(
        ProgressStyle::default_bar()
            .template("🎨 [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} artists ({eta})")
            .unwrap()
            .progress_chars("##-"),
    );

    let client = Arc::new(Client::new());
    let conn = Arc::new(Mutex::new(conn));
    let instaloader_semaphore = Arc::new(Semaphore::new(concurrent_artists));
    let progress = Arc::new(progress);
    
    let total_posts_processed = Arc::new(Mutex::new(0));
    let total_styles_found = Arc::new(Mutex::new(0));
    let total_api_cost = Arc::new(Mutex::new(0.0));

    let mut tasks = FuturesUnordered::new();
    let artists_len = artists.len();

    for (artist_idx, artist) in artists.into_iter().enumerate() {
        let client = Arc::clone(&client);
        let conn = Arc::clone(&conn);
        let instaloader_semaphore = Arc::clone(&instaloader_semaphore);
        let progress = Arc::clone(&progress);
        let total_posts_processed = Arc::clone(&total_posts_processed);
        let total_styles_found = Arc::clone(&total_styles_found);
        let total_api_cost = Arc::clone(&total_api_cost);
        let cookie_file = cookie_file.clone();
        let max_posts = max_posts.clone();

        tasks.push(tokio::spawn(async move {
            let _permit = instaloader_semaphore.acquire().await.unwrap();
            
            progress.set_message(format!("Processing {}", artist.name));
            println!(
                "\n🎨 [{}/{}] Processing Artist: {} (ID: {})",
                artist_idx + 1,
                artists_len,
                artist.name,
                artist.id
            );

            let ig_username = match &artist.ig_username {
                Some(username) => username.clone(),
                None => {
                    println!(
                        "⚠️  No valid Instagram username found for artist: {} - skipping",
                        artist.name
                    );
                    progress.inc(1);
                    return;
                }
            };
            println!("📸 Instagram username: @{}", ig_username);

        let insta_posts = match call_instaloader(&ig_username, &cookie_file, &max_posts) {
            Ok(posts) => posts,
            Err(e) => {
                println!(
                    "❌ Instaloader failed for {} (@{}):",
                    artist.name, ig_username
                );
                println!("   {}", e);

                if ig_username.contains("?")
                    || ig_username.contains("&")
                    || ig_username.contains("=")
                {
                    println!("   💡 Hint: Username '{}' looks malformed. Check social_links format in database.", ig_username);
                }

                let _ = cleanup_instaloader_files(&ig_username);
                {
                    let conn_guard = conn.lock().unwrap();
                    if let Err(e) = mark_artist_styles_extracted(&conn_guard, artist.id) {
                        println!(
                            "⚠️  Error marking artist {} as processed: {}",
                            artist.name, e
                        );
                    }
                }
                progress.inc(1);
                return;
            }
        };

        if insta_posts.is_empty() {
            println!("⚠️  No posts retrieved from Instagram for {}", artist.name);
            let _ = cleanup_instaloader_files(&ig_username);
            {
                let conn_guard = conn.lock().unwrap();
                if let Err(e) = mark_artist_styles_extracted(&conn_guard, artist.id) {
                    println!(
                        "⚠️  Error marking artist {} as processed: {}",
                        artist.name, e
                    );
                }
            }
            progress.inc(1);
            return;
        }

        println!("📥 Retrieved {} posts from Instagram", insta_posts.len());
        {
            let mut counter = total_posts_processed.lock().unwrap();
            *counter += insta_posts.len();
        }

        println!(
            "🤖 Processing {} posts with OpenAI Vision API...",
            insta_posts.len()
        );
        match process_artist_posts(
            &conn,
            &client,
            &artist,
            &insta_posts,
            batch_size,
            confidence_threshold,
        )
        .await
        {
            Ok((style_results, api_cost)) => {
                println!("💰 API cost for {}: ${:.4}", artist.name, api_cost);
                {
                    let mut cost_counter = total_api_cost.lock().unwrap();
                    *cost_counter += api_cost;
                }

                let mut all_artist_styles = HashMap::new();

                for result in style_results {
                    let artist_image_id = {
                        let conn_guard = conn.lock().unwrap();
                        insert_artist_image(&conn_guard, &result.shortcode, artist.id)
                    };
                    
                    match artist_image_id {
                        Ok(artist_image_id) => {
                            let style_names: Vec<String> = result
                                .styles
                                .iter()
                                .filter(|s| s.confidence >= confidence_threshold)
                                .map(|s| s.style.clone())
                                .filter(|s| is_valid_style_name(s))
                                .collect();

                            if !style_names.is_empty() {
                                let style_ids = {
                                    let conn_guard = conn.lock().unwrap();
                                    get_or_create_style_ids(&conn_guard, &style_names)
                                };
                                
                                match style_ids {
                                    Ok(style_ids) => {
                                        {
                                            let conn_guard = conn.lock().unwrap();
                                            if let Err(e) = insert_artist_image_styles(
                                                &conn_guard,
                                                artist_image_id,
                                                &style_ids,
                                            ) {
                                                println!(
                                                    "Error saving styles for image {}: {}",
                                                    result.shortcode, e
                                                );
                                            }
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
                    {
                        let conn_guard = conn.lock().unwrap();
                        if let Err(e) = upsert_artist_styles(&conn_guard, artist.id, &artist_style_ids) {
                            println!(
                                "❌ Error saving artist-level styles for {}: {}",
                                artist.name, e
                            );
                        } else {
                            println!(
                                "✅ Saved {} unique styles for artist",
                                artist_style_ids.len()
                            );
                            let mut styles_counter = total_styles_found.lock().unwrap();
                            *styles_counter += artist_style_ids.len();
                        }
                    }
                } else {
                    println!("ℹ️  No high-confidence styles found for artist");
                }
            }
            Err(e) => {
                println!(
                    "❌ Error processing styles for artist {}: {}",
                    artist.name, e
                );
            }
        }

        {
            let conn_guard = conn.lock().unwrap();
            if let Err(e) = mark_artist_styles_extracted(&conn_guard, artist.id) {
                println!(
                    "⚠️  Error marking artist {} as processed: {}",
                    artist.name, e
                );
            } else {
                println!("✅ Marked artist as styles_extracted");
            }
        }

        if let Err(e) = cleanup_instaloader_files(&ig_username) {
            println!(
                "⚠️  Warning: Error cleaning up files for @{}: {}",
                ig_username, e
            );
        } else {
            println!("🗑️  Cleaned up temporary files");
        }

        progress.inc(1);
        println!("🎨 Artist {} processing complete!\n", artist.name);
        }));
    }

    // Wait for all tasks to complete
    while let Some(_) = tasks.next().await {}

    progress.finish_with_message("🎉 Style extraction complete!");

    // Extract final values from Arc<Mutex<>> wrappers
    let final_posts_processed = *total_posts_processed.lock().unwrap();
    let final_styles_found = *total_styles_found.lock().unwrap();
    let final_api_cost = *total_api_cost.lock().unwrap();

    println!("📈 Final Results:");
    println!("   • Artists processed: {}", artists_len);
    println!("   • Total posts analyzed: {}", final_posts_processed);
    println!("   • Unique styles found: {}", final_styles_found);
    println!("   • Total API cost: ${:.4}", final_api_cost);
    println!(
        "   • Average cost per artist: ${:.4}",
        final_api_cost / artists_len as f64
    );

    Ok(())
}

fn call_instaloader(
    username: &str,
    cookie_file: &str,
    max_posts: &str,
) -> Result<Vec<InstaPost>, Box<dyn std::error::Error>> {
    println!(
        "📱 Calling instaloader for @{} (max {} posts)",
        username, max_posts
    );

    let output = Command::new("python3")
        .arg("load_ig_profiles.py")
        .arg(username)
        .arg("--cookiefile")
        .arg(cookie_file)
        .arg("--max-posts")
        .arg(max_posts)
        .current_dir("data-ingestion")
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);

        println!("   ❌ Instaloader stderr: {}", stderr);
        if !stdout.is_empty() {
            println!("   📋 Instaloader stdout: {}", stdout);
        }

        // Check for common error patterns and provide helpful suggestions
        if stderr.contains("404 Not Found") {
            if stderr.contains("web_profile_info") {
                return Err(format!("Instagram profile @{} not found or account is private. This could mean:\n  • Username doesn't exist\n  • Account is private/restricted\n  • Instagram API changes\n  • Rate limiting in effect", username).into());
            }
        } else if stderr.contains("login") || stderr.contains("authentication") {
            return Err(format!("Instagram authentication failed for @{}. Check if:\n  • Cookie file is valid and not expired\n  • Cookies are from the correct Instagram account\n  • Instagram session is still active", username).into());
        } else if stderr.contains("rate") || stderr.contains("limit") {
            return Err(format!("Instagram rate limiting detected for @{}. Consider:\n  • Waiting before retrying\n  • Using different cookies\n  • Reducing max posts per request", username).into());
        }

        return Err(format!("Instaloader failed for @{}: {}", username, stderr).into());
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
        println!("   🗑️  Deleted JSON file: {}", json_file);
    }

    let image_dir = format!("data-ingestion/{}", username);
    let image_path = Path::new(&image_dir);
    if image_path.exists() && image_path.is_dir() {
        let file_count = fs::read_dir(image_path)?.count();
        fs::remove_dir_all(image_path)?;
        println!(
            "   🗑️  Deleted image directory with {} files: {}",
            file_count, image_dir
        );
    }

    Ok(())
}

async fn process_artist_posts(
    conn: &Arc<Mutex<Connection>>,
    client: &Arc<Client<OpenAIConfig>>,
    artist: &Artist,
    posts: &[InstaPost],
    batch_size: usize,
    confidence_threshold: f64,
) -> Result<(Vec<StyleResult>, f64), Box<dyn std::error::Error>> {
    println!(
        "   🤖 Processing {} posts in batches of {} with OpenAI Vision API",
        posts.len(),
        batch_size
    );

    let semaphore = Arc::new(Semaphore::new(3));
    let mut handles = Vec::new();

    for (batch_idx, batch) in posts.chunks(batch_size).enumerate() {
        let client = Arc::clone(client);
        let semaphore = Arc::clone(&semaphore);
        let threshold = confidence_threshold;
        let batch_posts: Vec<InstaPost> = batch.to_vec();
        let total_batches = (posts.len() + batch_size - 1) / batch_size;

        let handle = tokio::spawn(async move {
            let _permit = semaphore.acquire().await.unwrap();

            println!(
                "     📸 Processing batch {}/{} ({} posts)",
                batch_idx + 1,
                total_batches,
                batch_posts.len()
            );
            let mut content_parts = vec![
                ChatCompletionRequestUserMessageContentPart::Text(
                    ChatCompletionRequestMessageContentPartText {
                        text: format!(
                            "Analyze these {} images. For each image, first determine if it shows a tattoo, then identify ALL artistic styles present if it is a tattoo.

                            Format your response as a JSON array with one object per image (in the EXACT order provided):
                            [
                              {{\"is_tattoo\": true, \"styles\": [{{\"style\": \"blackwork\", \"confidence\": 0.95}}, {{\"style\": \"geometric\", \"confidence\": 0.87}}]}},
                              {{\"is_tattoo\": true, \"styles\": [{{\"style\": \"traditional\", \"confidence\": 0.92}}]}},
                              {{\"is_tattoo\": false, \"styles\": []}},
                              {{\"is_tattoo\": true, \"styles\": []}}
                            ]

                            CRITICAL INSTRUCTIONS:
                            • FIRST: Determine if the image shows a tattoo (set \"is_tattoo\": true/false)
                            • If \"is_tattoo\": false, set \"styles\": [] and move to next image
                            • If \"is_tattoo\": true, identify ANY and ALL tattoo styles you can see - do NOT limit yourself to common examples
                            • Include rare, niche, or unique styles (e.g., trash polka, chicano, biomechanical, dotwork, fine line, etc.)
                            • Use specific style names when possible (e.g., 'neo traditional' not just 'traditional')
                            • Include cultural styles (e.g., japanese, polynesian, celtic, etc.)
                            • Include technique-based styles (e.g., stippling, linework, shading styles)
                            • The examples (blackwork, traditional, realism, watercolor, geometric) are just EXAMPLES - identify whatever styles you actually see
                            • Include all styles you can identify, even with lower confidence scores
                            • Be comprehensive and thorough in your style identification for tattoo images
                            
                            IMPORTANT: Return exactly {} objects in the array, one for each image in order.",
                            batch_posts.len(),
                            batch_posts.len()
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

                                        // Always use the real shortcode from instaloader, not GPT's response
                                        let shortcode = batch_posts[idx].shortcode.clone();

                                        // Check if this image is a tattoo
                                        let is_tattoo = result
                                            .get("is_tattoo")
                                            .and_then(|t| t.as_bool())
                                            .unwrap_or(true); // Default to true for backward compatibility

                                        if !is_tattoo {
                                            println!(
                                                "    Shortcode {}: Not a tattoo - skipping",
                                                shortcode
                                            );
                                            continue; // Skip non-tattoo images entirely
                                        }

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
            let conn_guard = conn.lock().unwrap();
            if let Err(e) = update_openai_api_costs(&conn_guard, "style_extraction", "gpt-4o", avg_cost) {
                println!("  Warning: Failed to update API cost tracking: {}", e);
            }
        }
        println!(
            "  Total API cost for {}: ${:.4} across {} calls (avg: ${:.4})",
            artist.name, total_cost, api_calls, avg_cost
        );
    }

    Ok((all_results, total_cost))
}
