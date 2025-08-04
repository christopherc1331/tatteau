use base64::Engine;
use std::{env, fs, path::Path, sync::Arc};
use tokio::sync::Semaphore;

use async_openai::{
    types::{
        ChatCompletionRequestMessage, ChatCompletionRequestMessageContentPartImage,
        ChatCompletionRequestMessageContentPartText, ChatCompletionRequestUserMessage,
        ChatCompletionRequestUserMessageContent, ChatCompletionRequestUserMessageContentPart,
        CreateChatCompletionRequestArgs, ImageUrl,
    },
    Client,
};
use data_fetcher::fetch_data;
use data_parser::{parse_data, ParsedLocationData};
use dotenv::dotenv;
use repository::{fetch_county_boundaries, mark_county_ingested, upsert_locations};
use rusqlite::Connection;
use serde_json::Value;
use shared_types::CountyBoundary;

pub mod data_fetcher;
pub mod data_parser;
pub mod repository;
pub mod scraper;

enum IngestAction {
    Scrape,
    GoogleApi,
    ExtractStyles,
}

impl IngestAction {
    fn new(action: &str) -> Self {
        match action {
            "SCRAPE_HTML" => IngestAction::Scrape,
            "GOOGLE_API" => IngestAction::GoogleApi,
            "EXTRACT_STYLES" => IngestAction::ExtractStyles,
            _ => panic!("Invalid action"),
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    let action: String = env::var("ACTION").expect("Action to be set");
    let db_path = Path::new("tatteau.db");
    let conn: Connection = Connection::open(db_path).expect("Database should load");

    match IngestAction::new(&action) {
        IngestAction::Scrape => scraper::scrape(conn).await,
        IngestAction::GoogleApi => ingest_google(&conn).await,
        IngestAction::ExtractStyles => extract_styles().await,
    }
}

async fn ingest_google(conn: &Connection) -> Result<(), Box<dyn std::error::Error>> {
    let limit_results_to: i8 = 20;
    let max_iter: i8 = 10;

    let county_limit: i16 = 3500;
    let days_till_refetch: i16 = 160;
    let county_boundaries: Vec<CountyBoundary> =
        fetch_county_boundaries(conn, county_limit, days_till_refetch)
            .expect("County boundaries should be fetched");

    if county_boundaries.is_empty() {
        println!("No county boundaries found, exiting.");
        return Ok(());
    }

    for county_boundary in county_boundaries {
        println!("Processing county: {}", county_boundary.name);

        if let Err(e) = process_county(conn, &county_boundary, limit_results_to, max_iter).await {
            println!("Error processing county {}: {}", county_boundary.name, e);
        }

        mark_county_ingested(conn, &county_boundary)?;
    }
    Ok(())
}
async fn process_county(
    conn: &Connection,
    county_boundary: &CountyBoundary,
    limit_results_to: i8,
    max_iter: i8,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut current_token: Option<String> = None;
    let mut curr_iter = 0;
    while curr_iter < max_iter {
        curr_iter += 1;

        let res: Value = fetch_data(county_boundary, limit_results_to, &current_token).await;
        let parsed_data_opt: Option<ParsedLocationData> = parse_data(&res);
        if let Some(parsed_data) = parsed_data_opt {
            let ParsedLocationData {
                next_token,
                location_info,
                filtered_count,
            } = parsed_data;
            println!(
                "Found {} and filtered {} results out of {}",
                location_info.len(),
                filtered_count,
                limit_results_to
            );

            current_token = next_token.map(|s| s.to_string());
            let _ = upsert_locations(conn, &location_info);
            println!("Inserted {} locations", location_info.len());
        }

        if current_token.is_none() {
            break;
        }
    }

    Ok(())
}

async fn extract_styles() -> Result<(), Box<dyn std::error::Error>> {
    let images_dir = env::var("IMAGES_DIR").expect("IMAGES_DIR environment variable must be set");
    env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY environment variable must be set");

    let client = Arc::new(Client::new());
    let images_path = Path::new(&images_dir);

    if !images_path.exists() {
        return Err(format!("Images directory does not exist: {}", images_dir).into());
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
        println!("No image files found in directory: {}", images_dir);
        return Ok(());
    }

    println!("Found {} image files", image_files.len());

    let semaphore = Arc::new(Semaphore::new(5));
    let mut handles = Vec::new();
    let total_files = image_files.len();

    for (i, image_path) in image_files.into_iter().enumerate() {
        let client = Arc::clone(&client);
        let semaphore = Arc::clone(&semaphore);

        let handle = tokio::spawn(async move {
            let _permit = semaphore.acquire().await.unwrap();

            println!(
                "Processing image {}/{}: {:?}",
                i + 1,
                total_files,
                image_path.file_name()
            );

            let image_data = match fs::read(&image_path) {
                Ok(data) => data,
                Err(e) => {
                    println!("Error reading image {:?}: {}", image_path.file_name(), e);
                    return Vec::new();
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
                _ => "image/jpeg", // default fallback
            };

            let data_url = format!("data:{};base64,{}", mime_type, base64_image);

            let user_message = ChatCompletionRequestMessage::User(
                ChatCompletionRequestUserMessage {
                    content: ChatCompletionRequestUserMessageContent::Array(vec![
                        ChatCompletionRequestUserMessageContentPart::Text(
                            ChatCompletionRequestMessageContentPartText {
                                text: "First, determine if this image shows a tattoo. If it is NOT a tattoo, respond with exactly 'NOT_TATTOO'. If it IS a tattoo, analyze the tattoo and identify ALL the specific artistic styles present. Don't limit yourself to common styles - identify any and all tattoo styles you can see, including but not limited to: traditional, neo-traditional, realism, black and grey, watercolor, geometric, minimalist, tribal, Japanese, biomechanical, dotwork, linework, portrait, abstract, surreal, fine line, sketch, blackwork, color realism, photorealism, new school, old school, Celtic, mandala, script, lettering, illustrative, ornamental, or any other style. Be comprehensive and identify all styles you observe. Return only a comma-separated list of the styles you can identify.".to_string(),
                            }
                        ),
                        ChatCompletionRequestUserMessageContentPart::ImageUrl(
                            ChatCompletionRequestMessageContentPartImage {
                                image_url: ImageUrl {
                                    url: data_url,
                                    detail: None,
                                }
                            }
                        )
                    ]),
                    name: None,
                }
            );

            let request = match CreateChatCompletionRequestArgs::default()
                .model("gpt-4o")
                .max_tokens(150u32)
                .messages([user_message])
                .build()
            {
                Ok(req) => req,
                Err(e) => {
                    println!(
                        "Error building request for {:?}: {}",
                        image_path.file_name(),
                        e
                    );
                    return Vec::new();
                }
            };

            let timeout_duration = tokio::time::Duration::from_secs(30);

            match tokio::time::timeout(timeout_duration, client.chat().create(request)).await {
                Ok(Ok(response)) => {
                    if let Some(choice) = response.choices.first() {
                        if let Some(content) = &choice.message.content {
                            let content_trimmed = content.trim();

                            if content_trimmed == "NOT_TATTOO" {
                                println!(
                                    "Image {:?} is not a tattoo - skipping style analysis",
                                    image_path.file_name()
                                );
                                return Vec::new();
                            }

                            let styles: Vec<String> = content_trimmed
                                .split(',')
                                .map(|s| s.trim().to_lowercase())
                                .filter(|s| !s.is_empty())
                                .collect();
                            println!(
                                "Identified styles for {:?}: {}",
                                image_path.file_name(),
                                content
                            );
                            return styles;
                        }
                    }
                    println!("No content in response for {:?}", image_path.file_name());
                }
                Ok(Err(e)) => {
                    println!(
                        "API error processing image {:?}: {}",
                        image_path.file_name(),
                        e
                    );
                }
                Err(_) => {
                    println!("Timeout processing image {:?}", image_path.file_name());
                }
            }

            tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
            Vec::new()
        });

        handles.push(handle);
    }

    let mut all_styles = Vec::new();
    let mut tattoo_count = 0;
    let mut non_tattoo_count = 0;
    let mut error_count = 0;

    for handle in handles {
        match handle.await {
            Ok(styles) => {
                if styles.is_empty() {
                    non_tattoo_count += 1;
                } else {
                    tattoo_count += 1;
                    all_styles.extend(styles);
                }
            }
            Err(e) => {
                println!("Task failed: {}", e);
                error_count += 1;
            }
        }
    }

    println!("\n=== PROCESSING SUMMARY ===");
    println!("Total images processed: {}", total_files);
    println!("Images identified as tattoos: {}", tattoo_count);
    println!("Images identified as non-tattoos: {}", non_tattoo_count);
    if error_count > 0 {
        println!("Images with processing errors: {}", error_count);
    }

    let mut style_counts = std::collections::HashMap::new();
    for style in all_styles {
        *style_counts.entry(style).or_insert(0) += 1;
    }

    println!("\n=== TATTOO STYLE ANALYSIS RESULTS ===");
    let mut sorted_styles: Vec<_> = style_counts.iter().collect();
    sorted_styles.sort_by(|a, b| b.1.cmp(a.1));

    for (style, count) in sorted_styles {
        println!("{}: {} occurrences", style, count);
    }

    Ok(())
}
