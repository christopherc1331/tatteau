use base64::Engine;
use indicatif::{ProgressBar, ProgressStyle};
use sqlx::PgPool;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{collections::HashMap, env, sync::Arc};
use tokio::sync::Semaphore;
use tokio::task::JoinSet;

/// Metadata defining how the LLM should classify each style type
#[derive(Debug, Clone)]
struct StyleTypeMetadata {
    /// Human-readable description of what this type represents
    description: &'static str,
    /// How many styles from this type the LLM should typically select
    cardinality: &'static str,
    /// Recommended confidence threshold for this type
    threshold_hint: &'static str,
    /// Additional guidance for the LLM
    guidance: &'static str,
}

/// Get classification metadata for a style type
fn get_type_metadata(type_name: &str) -> StyleTypeMetadata {
    match type_name {
        "Traditional & Classic" => StyleTypeMetadata {
            description: "Cultural/regional foundation styles",
            cardinality: "ONE",
            threshold_hint: "≥ 0.95",
            guidance: "Pick ONE primary foundation (e.g., 'japanese' OR 'american traditional', not both). These define the cultural/artistic origin.",
        },
        "Technique-Based" => StyleTypeMetadata {
            description: "How the tattoo is executed",
            cardinality: "MULTIPLE (2-4 typical)",
            threshold_hint: "≥ 0.85",
            guidance: "Select ALL applicable techniques. Multiple techniques often coexist (e.g., 'dotwork' + 'geometric' + 'black & grey').",
        },
        "Subject Matter" => StyleTypeMetadata {
            description: "What is depicted in the tattoo",
            cardinality: "MULTIPLE (1-4 typical)",
            threshold_hint: "≥ 0.88",
            guidance: "Identify ALL visible subjects. Tattoos commonly combine multiple subjects (e.g., 'floral' + 'animal' + 'nature').",
        },
        "Color" => StyleTypeMetadata {
            description: "Color palette approach",
            cardinality: "ONE (occasionally TWO if specialty ink)",
            threshold_hint: "≥ 0.92",
            guidance: "Pick ONE primary color approach (e.g., 'color' OR 'monochromatic', not both). Add specialty inks like 'uv' or 'glow' only if clearly visible.",
        },
        "Aesthetic" => StyleTypeMetadata {
            description: "Artistic philosophy or movement",
            cardinality: "ONE (or ZERO if not applicable)",
            threshold_hint: "≥ 0.88",
            guidance: "Optional category. Select ONE dominant aesthetic if clearly present (e.g., 'illustrative', 'surrealism', 'abstract').",
        },
        "Thematic" => StyleTypeMetadata {
            description: "Conceptual themes",
            cardinality: "MULTIPLE (0-3 when applicable)",
            threshold_hint: "≥ 0.87",
            guidance: "Optional category. Select thematic elements if relevant (e.g., 'religious' + 'cultural', 'memorial', 'vintage').",
        },
        "Scale & Application" => StyleTypeMetadata {
            description: "Size and placement context",
            cardinality: "ONE size + optional application context",
            threshold_hint: "≥ 0.85",
            guidance: "Pick ONE scale (e.g., 'large scale' OR 'micro scale'). Optionally add placement context like 'sleeve', 'cover-up', or 'minimalist'.",
        },
        "Modern & Specialty" => StyleTypeMetadata {
            description: "Contemporary hybrid styles",
            cardinality: "FEW (0-2, these are distinctive)",
            threshold_hint: "≥ 0.90",
            guidance: "Only select if clearly distinctive hybrid styles (e.g., 'biomechanical', 'trash polka', '3d'). Avoid over-tagging.",
        },
        "Pop Culture" => StyleTypeMetadata {
            description: "Pop culture references",
            cardinality: "MULTIPLE (0-3 when applicable)",
            threshold_hint: "≥ 0.88",
            guidance: "Optional category. Tag pop culture elements if present (e.g., 'anime' + 'kawaii', 'comic', 'video games').",
        },
        "Artistic Techniques" => StyleTypeMetadata {
            description: "Specialized artistic methods",
            cardinality: "FEW (0-2, very specific)",
            threshold_hint: "≥ 0.92",
            guidance: "Only select if using a very specific artistic technique (e.g., 'stippling', 'woodcut', 'pointillism'). High confidence required.",
        },
        "Typography" => StyleTypeMetadata {
            description: "Text-based elements",
            cardinality: "ZERO or ONE (or BOTH if mixed)",
            threshold_hint: "≥ 0.93",
            guidance: "Binary category. Tag 'lettering' and/or 'script' only if text is clearly visible.",
        },
        "Specialized/Medical" => StyleTypeMetadata {
            description: "PMU/cosmetic/paramedical work",
            cardinality: "Usually ZERO (ONE if PMU/cosmetic)",
            threshold_hint: "≥ 0.98",
            guidance: "SKIP this category for traditional tattoos. Only tag if this is clearly permanent makeup, cosmetic tattooing, or paramedical work.",
        },
        _ => StyleTypeMetadata {
            description: "Unknown category",
            cardinality: "VARIABLE",
            threshold_hint: "≥ 0.90",
            guidance: "Use judgment based on style visibility.",
        },
    }
}

use async_openai::{
    types::{
        ChatCompletionRequestMessage, ChatCompletionRequestMessageContentPartImage,
        ChatCompletionRequestMessageContentPartText, ChatCompletionRequestUserMessage,
        ChatCompletionRequestUserMessageContent, ChatCompletionRequestUserMessageContentPart,
        CreateChatCompletionRequestArgs, ImageUrl,
    },
    Client,
};

use crate::repository::{
    get_all_styles, get_artists_for_style_extraction, get_style_ids, insert_artist_image,
    insert_artist_image_styles, mark_artist_styles_extracted, mark_artist_styles_extraction_failed,
    update_openai_api_costs, upsert_artist_styles, Artist,
};

use super::apify_scraper::{download_image, scrape_instagram_profile};

#[derive(Debug, Clone)]
struct ProcessablePost {
    shortcode: String,
    image_data: Vec<u8>,
    timestamp: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize)]
struct StyleResult {
    shortcode: String,
    styles: Vec<StyleConfidence>,
    timestamp: Option<i64>,
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

pub async fn extract_styles(pool: &PgPool) -> Result<(), Box<dyn std::error::Error>> {
    env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY environment variable must be set");

    let confidence_threshold: f64 = env::var("STYLE_CONFIDENCE_THRESHOLD")
        .unwrap_or_else(|_| "0.9".to_string())
        .parse()
        .expect("STYLE_CONFIDENCE_THRESHOLD must be a valid number between 0 and 1");

    let batch_size: usize = env::var("VISION_BATCH_SIZE")
        .unwrap_or_else(|_| "8".to_string())
        .parse()
        .expect("VISION_BATCH_SIZE must be a valid number between 1 and 16");

    let artist_limit: i16 = env::var("ARTIST_BATCH_LIMIT")
        .unwrap_or_else(|_| "10".to_string())
        .parse()
        .expect("ARTIST_BATCH_LIMIT must be a valid number");

    let concurrent_artists: usize = env::var("CONCURRENT_ARTISTS")
        .unwrap_or_else(|_| "6".to_string())  // Increased from 3 to compensate for slower Apify requests
        .parse()
        .expect("CONCURRENT_ARTISTS must be a valid number");

    env::var("APIFY_API_TOKEN").expect("APIFY_API_TOKEN environment variable must be set");

    let max_posts: i32 = env::var("MAX_POSTS_PER_ARTIST")
        .unwrap_or_else(|_| "10".to_string())
        .parse()
        .expect("MAX_POSTS_PER_ARTIST must be a valid number");

    let artists = get_artists_for_style_extraction(pool, artist_limit).await?;

    if artists.is_empty() {
        println!("🔍 No artists found that need style extraction.");
        return Ok(());
    }

    let available_styles = get_all_styles(pool).await?;
    if available_styles.is_empty() {
        println!("❌ No styles found in database. Please populate the styles table first.");
        return Ok(());
    }

    let total_styles: usize = available_styles.values().map(|v| v.len()).sum();

    println!("🎯 Instagram Style Extraction Started");
    println!("📊 Configuration:");
    println!("   • Artists to process: {}", artists.len());
    println!("   • Concurrent artists: {}", concurrent_artists);
    println!("   • Confidence threshold: {}", confidence_threshold);
    println!("   • Vision batch size: {}", batch_size);
    println!("   • Max posts per artist: {}", max_posts);
    println!("   • Available styles: {} across {} categories", total_styles, available_styles.len());
    println!("   • Using Apify Instagram scraper");

    let progress = Arc::new(ProgressBar::new(artists.len() as u64));
    progress.set_style(
        ProgressStyle::default_bar()
            .template("🎨 [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} artists ({eta})")
            .unwrap()
            .progress_chars("##-"),
    );

    let artists_len = artists.len();
    let mut total_posts_processed = 0;
    let mut total_styles_found = 0;
    let mut total_api_cost = 0.0;

    // Process artists in batches using JoinSet for concurrent execution
    let mut join_set = JoinSet::new();
    let semaphore = Arc::new(Semaphore::new(concurrent_artists));
    let available_styles = Arc::new(available_styles);

    for artist in artists {
        let permit = semaphore.clone().acquire_owned().await.unwrap();
        let progress_clone = progress.clone();
        let artist_name = artist.name.clone();
        let styles_clone = available_styles.clone();
        let pool_clone = pool.clone();

        join_set.spawn(async move {
            progress_clone.set_message(format!("Processing {}", artist_name));
            let result =
                process_single_artist(&pool_clone, artist, max_posts, batch_size, confidence_threshold, &styles_clone).await;
            drop(permit);
            progress_clone.inc(1);
            result
        });
    }

    // Collect results from all concurrent tasks
    while let Some(result) = join_set.join_next().await {
        match result {
            Ok(Ok((artist, posts_processed, styles_found, api_cost))) => {
                total_posts_processed += posts_processed;
                total_styles_found += styles_found;
                total_api_cost += api_cost;
                if posts_processed > 0 {
                    println!(
                        "🎨 Artist {} processing complete! Processed {} posts, found {} styles\n",
                        artist.name, posts_processed, styles_found
                    );
                }
            }
            Ok(Err(e)) => {
                println!("❌ Error processing artist: {}\n", e);
            }
            Err(e) => {
                println!("❌ Task join error: {}\n", e);
            }
        }
    }

    progress.finish_with_message("🎉 Style extraction complete!");

    println!("📈 Final Results:");
    println!("   • Artists processed: {}", artists_len);
    println!("   • Total posts analyzed: {}", total_posts_processed);
    println!("   • Unique styles found: {}", total_styles_found);
    println!("   • Total API cost: ${:.4}", total_api_cost);
    println!(
        "   • Average cost per artist: ${:.4}",
        total_api_cost / artists_len as f64
    );

    Ok(())
}

async fn process_single_artist(
    pool: &PgPool,
    artist: Artist,
    max_posts: i32,
    batch_size: usize,
    confidence_threshold: f64,
    available_styles: &HashMap<String, Vec<String>>,
) -> Result<(Artist, usize, usize, f64), Box<dyn std::error::Error + Send + Sync>> {
    let ig_username = match &artist.ig_username {
        Some(username) => username.clone(),
        None => {
            println!(
                "⚠️  No valid Instagram username found for artist: {} - skipping",
                artist.name
            );

            // Mark as failed in database
            if let Err(db_err) = mark_artist_styles_extraction_failed(pool, artist.id).await {
                println!("⚠️  Error marking artist {} as failed: {}", artist.name, db_err);
            } else {
                println!("❌ [{} - ID: {}] Marked as extraction failed (invalid username)", artist.name, artist.id);
            }

            return Ok((artist, 0, 0, 0.0));
        }
    };

    println!(
        "📸 [{} - ID: {}] Instagram username: @{}",
        artist.name, artist.id, ig_username
    );

    let apify_result = scrape_instagram_profile(&ig_username, max_posts)
        .await
        .map_err(|e| e.to_string());

    let apify_posts = match apify_result {
        Ok(posts) => posts,
        Err(error_msg) => {
            println!(
                "❌ Apify scraper failed for {} (@{}):",
                artist.name, ig_username
            );
            println!("   {}", error_msg);

            if ig_username.contains("?") || ig_username.contains("&") || ig_username.contains("=") {
                println!("   💡 Hint: Username '{}' looks malformed. Check social_links format in database.", ig_username);
            }

            // Mark as failed in database before returning error
            if let Err(db_err) = mark_artist_styles_extraction_failed(pool, artist.id).await {
                println!("⚠️  Error marking artist {} as failed: {}", artist.name, db_err);
            } else {
                println!("❌ [{} - ID: {}] Marked as extraction failed", artist.name, artist.id);
            }

            return Err(Box::from(error_msg) as Box<dyn std::error::Error + Send + Sync>);
        }
    };

    if apify_posts.is_empty() {
        println!("⚠️  No posts retrieved from Instagram for {}", artist.name);

        // Mark as failed in database
        if let Err(db_err) = mark_artist_styles_extraction_failed(pool, artist.id).await {
            println!("⚠️  Error marking artist {} as failed: {}", artist.name, db_err);
        } else {
            println!("❌ [{} - ID: {}] Marked as extraction failed (no posts)", artist.name, artist.id);
        }

        return Ok((artist, 0, 0, 0.0));
    }

    let mut processable_posts = Vec::new();
    for post in apify_posts.iter().take(max_posts as usize) {
        if let Some(display_url) = &post.display_url {
            match download_image(display_url).await {
                Ok(image_data) => {
                    let shortcode = post.shortcode.clone();
                    let timestamp = post.timestamp;
                    processable_posts.push(ProcessablePost {
                        shortcode,
                        image_data,
                        timestamp,
                    });
                }
                Err(e) => {
                    println!(
                        "   ⚠️  Failed to download image for post {}: {}",
                        post.shortcode, e
                    );
                }
            }
        }
    }

    if processable_posts.is_empty() {
        println!("⚠️  No images could be downloaded for {}", artist.name);

        // Mark as failed in database
        if let Err(db_err) = mark_artist_styles_extraction_failed(pool, artist.id).await {
            println!("⚠️  Error marking artist {} as failed: {}", artist.name, db_err);
        } else {
            println!("❌ [{} - ID: {}] Marked as extraction failed (no images)", artist.name, artist.id);
        }

        return Ok((artist, 0, 0, 0.0));
    }

    println!(
        "📥 [{} - ID: {}] Downloaded {} images from Instagram",
        artist.name,
        artist.id,
        processable_posts.len()
    );
    let posts_processed = processable_posts.len();

    println!(
        "🤖 [{} - ID: {}] Processing {} posts with OpenAI Vision API...",
        artist.name,
        artist.id,
        processable_posts.len()
    );

    match process_artist_posts(
        pool,
        &artist,
        &processable_posts,
        batch_size,
        confidence_threshold,
        available_styles,
    )
    .await
    .map_err(|e| e.to_string())
    {
        Ok((style_results, api_cost)) => {
            println!(
                "💰 [{} - ID: {}] API cost: ${:.4}",
                artist.name, artist.id, api_cost
            );

            let mut all_artist_styles = HashMap::new();

            for result in style_results {
                let artist_image_id = insert_artist_image(pool, &result.shortcode, artist.id, result.timestamp).await;

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
                            let style_ids = get_style_ids(pool, &style_names).await;

                            match style_ids {
                                Ok(style_ids) => {
                                    if !style_ids.is_empty() {
                                        if let Err(e) = insert_artist_image_styles(
                                            pool,
                                            artist_image_id,
                                            &style_ids,
                                        ).await {
                                            println!(
                                                "Error saving styles for image {}: {}",
                                                result.shortcode, e
                                            );
                                        }

                                        for (name, id) in style_names.iter().zip(style_ids.iter()) {
                                            all_artist_styles.insert(name.clone(), *id);
                                        }
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

            let styles_found = if !all_artist_styles.is_empty() {
                let artist_style_ids: Vec<i64> = all_artist_styles.values().copied().collect();
                if let Err(e) = upsert_artist_styles(pool, artist.id, &artist_style_ids).await {
                    println!(
                        "❌ Error saving artist-level styles for {}: {}",
                        artist.name, e
                    );
                    0
                } else {
                    println!(
                        "✅ [{} - ID: {}] Saved {} unique styles",
                        artist.name,
                        artist.id,
                        artist_style_ids.len()
                    );
                    artist_style_ids.len()
                }
            } else {
                println!(
                    "ℹ️  [{} - ID: {}] No high-confidence styles found",
                    artist.name, artist.id
                );
                0
            };

            if let Err(e) = mark_artist_styles_extracted(pool, artist.id).await {
                println!(
                    "⚠️  Error marking artist {} as processed: {}",
                    artist.name, e
                );
            } else {
                println!(
                    "✅ [{} - ID: {}] Marked as styles_extracted",
                    artist.name, artist.id
                );
            }

            Ok((artist, posts_processed, styles_found, api_cost))
        }
        Err(error_msg) => {
            println!("❌ Error processing posts for {}: {}", artist.name, error_msg);

            if let Err(db_err) = mark_artist_styles_extraction_failed(pool, artist.id).await {
                println!(
                    "⚠️  Error marking artist {} as failed: {}",
                    artist.name, db_err
                );
            } else {
                println!("❌ [{} - ID: {}] Marked as extraction failed (OpenAI error)", artist.name, artist.id);
            }

            Err(Box::from(error_msg) as Box<dyn std::error::Error + Send + Sync>)
        }
    }
}

async fn process_artist_posts(
    pool: &PgPool,
    artist: &Artist,
    posts: &[ProcessablePost],
    batch_size: usize,
    confidence_threshold: f64,
    available_styles: &HashMap<String, Vec<String>>,
) -> Result<(Vec<StyleResult>, f64), Box<dyn std::error::Error>> {
    let client = Client::new();
    println!(
        "   🤖 Processing {} posts in batches of {} with OpenAI Vision API",
        posts.len(),
        batch_size
    );

    let semaphore = Arc::new(Semaphore::new(3));
    let mut handles = Vec::new();

    // Build type-aware prompt sections dynamically from database styles
    let mut categorized_sections = Vec::new();
    let mut classification_rules = Vec::new();

    let mut sorted_types: Vec<&String> = available_styles.keys().collect();
    sorted_types.sort();

    for style_type in sorted_types {
        if let Some(styles) = available_styles.get(style_type) {
            let metadata = get_type_metadata(style_type);
            let lowercase_styles: Vec<String> = styles.iter().map(|s| s.to_lowercase()).collect();

            // Build vocabulary section with type metadata
            categorized_sections.push(format!(
                "**{}** ({}) - {}:\n• {}",
                style_type,
                metadata.cardinality,
                metadata.description,
                lowercase_styles.join(", ")
            ));

            // Build classification rule for this type
            classification_rules.push(format!(
                "   {}: {}\n   Confidence: {} | Cardinality: {}",
                style_type,
                metadata.guidance,
                metadata.threshold_hint,
                metadata.cardinality
            ));
        }
    }

    let categorized_styles = categorized_sections.join("\n\n");
    let type_specific_rules = classification_rules.join("\n\n");

    for (batch_idx, batch) in posts.chunks(batch_size).enumerate() {
        let client = Arc::new(client.clone());
        let semaphore = Arc::clone(&semaphore);
        let threshold = confidence_threshold;
        let batch_posts: Vec<ProcessablePost> = batch.to_vec();
        let total_batches = posts.len().div_ceil(batch_size);
        let categorized_styles_clone = categorized_styles.clone();
        let type_rules_clone = type_specific_rules.clone();

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
                            "You are an expert tattoo style classifier. Analyze these {} tattoo images and identify ALL visible artistic styles.

STYLE VOCABULARY (organized by type - use lowercase, match spelling exactly):
{}

CLASSIFICATION RULES:

1. First determine: Is this image a tattoo? (set is_tattoo: true/false)
2. If NOT a tattoo (is_tattoo: false) → set styles: [] and move to next image
3. If IS a tattoo (is_tattoo: true) → classify across type categories:

TYPE-SPECIFIC GUIDANCE:
{}

GENERAL RULES:
• Use ONLY styles from vocabulary above (no invented styles)
• Use lowercase, exact spelling (e.g., \"black & grey\" not \"Black and Gray\")
• Set confidence 0.0-1.0 based on visibility
• Apply type-specific thresholds and cardinality rules
• Be comprehensive but respect cardinality limits

RESPONSE FORMAT:
Return a JSON array with one object per image (in exact order):
[
  {{\"is_tattoo\": true, \"styles\": [
    {{\"style\": \"japanese\", \"confidence\": 0.96}},
    {{\"style\": \"dotwork\", \"confidence\": 0.91}},
    {{\"style\": \"geometric\", \"confidence\": 0.89}},
    {{\"style\": \"floral\", \"confidence\": 0.93}},
    {{\"style\": \"color\", \"confidence\": 0.94}},
    {{\"style\": \"large scale\", \"confidence\": 0.87}}
  ]}},
  {{\"is_tattoo\": false, \"styles\": []}}
]

EXAMPLES:
✓ GOOD: \"japanese\" (Traditional), \"dotwork\" + \"geometric\" (Techniques), \"floral\" (Subject), \"color\" (Color), \"large scale\" (Scale)
✗ BAD: \"japanese\" + \"polynesian\" (violates Traditional cardinality: ONE only)
✓ GOOD: \"black & grey\", \"realism\", \"portraiture\" (proper multi-label)
✗ BAD: \"black and grey realistic portrait\" (should be separate styles)

Return exactly {} objects (one per image, in order). Analyze all {} images now:",
                            batch_posts.len(),
                            categorized_styles_clone,
                            type_rules_clone,
                            batch_posts.len(),
                            batch_posts.len()
                        ),
                    }
                )
            ];

            for post in &batch_posts {
                let base64_image =
                    base64::engine::general_purpose::STANDARD.encode(&post.image_data);

                // Detect image format from the data itself
                let mime_type = if post.image_data.starts_with(&[0xFF, 0xD8, 0xFF]) {
                    "image/jpeg"
                } else if post.image_data.starts_with(&[0x89, 0x50, 0x4E, 0x47]) {
                    "image/png"
                } else if post.image_data.starts_with(b"RIFF")
                    && post.image_data.len() > 11
                    && &post.image_data[8..12] == b"WEBP"
                {
                    "image/webp"
                } else if post.image_data.starts_with(b"GIF8") {
                    "image/gif"
                } else {
                    "image/jpeg" // default
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
                .max_tokens(2000u32)
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

            match tokio::time::timeout(timeout_duration, (*client).chat().create(request)).await {
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

                                        let shortcode = batch_posts[idx].shortcode.clone();
                                        let timestamp = batch_posts[idx].timestamp;

                                        let is_tattoo = result
                                            .get("is_tattoo")
                                            .and_then(|t| t.as_bool())
                                            .unwrap_or(true);

                                        if !is_tattoo {
                                            println!(
                                                "    Shortcode {}: Not a tattoo - skipping",
                                                shortcode
                                            );
                                            continue;
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

                                        batch_results.push(StyleResult { shortcode, styles, timestamp });
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
                                    // Log the actual content to help debug
                                    let preview_len = std::cmp::min(json_content.len(), 200);
                                    println!(
                                        "  Content preview (first {} chars): '{}'",
                                        preview_len,
                                        &json_content[..preview_len]
                                    );
                                    println!(
                                        "  Content length: {} chars, is_empty: {}",
                                        json_content.len(),
                                        json_content.is_empty()
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

        // Update API costs
        for _ in 0..api_calls {
            if let Err(e) =
                update_openai_api_costs(pool, "style_extraction", "gpt-4o", avg_cost).await
            {
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
