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

pub async fn extract_styles() -> Result<(), Box<dyn std::error::Error>> {
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
                _ => "image/jpeg",
            };

            let data_url = format!("data:{};base64,{}", mime_type, base64_image);

            let user_message = ChatCompletionRequestMessage::User(
                ChatCompletionRequestUserMessage {
                    content: ChatCompletionRequestUserMessageContent::Array(vec![
                        ChatCompletionRequestUserMessageContentPart::Text(
                            ChatCompletionRequestMessageContentPartText {
                                text: "First, determine if this image shows a tattoo. If it is NOT a tattoo, respond with exactly 'NOT_TATTOO'. If it IS a tattoo, analyze the tattoo and identify ALL the specific artistic styles present with confidence scores. Only include styles where you have high confidence (above 0.80). Your response must ONLY contain the style:confidence pairs in this exact format: 'style1:0.95,style2:0.87,style3:0.92' with NO additional text, explanations, or commentary. If you are not confident (above 0.80) about any styles, return completely empty (no text at all).".to_string(),
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

                            println!(
                                "Raw GPT response for {:?}: '{}'",
                                image_path.file_name(),
                                content_trimmed
                            );

                            if content_trimmed == "NOT_TATTOO" {
                                println!(
                                    "Image {:?} is not a tattoo - skipping style analysis",
                                    image_path.file_name()
                                );
                                return Vec::new();
                            }

                            if content_trimmed.is_empty() {
                                println!(
                                    "Image {:?} is a tattoo but no styles identified with sufficient confidence",
                                    image_path.file_name()
                                );
                                return Vec::new();
                            }

                            let mut high_confidence_styles = Vec::new();
                            let mut all_styles_with_confidence = Vec::new();

                            for style_entry in content_trimmed.split(',') {
                                let style_entry = style_entry.trim();

                                if style_entry.is_empty() {
                                    continue;
                                }

                                if let Some((style, confidence_str)) = style_entry.split_once(':') {
                                    let style = style.trim().to_lowercase();

                                    if is_valid_style_name(&style) {
                                        if let Ok(confidence) = confidence_str.trim().parse::<f64>()
                                        {
                                            if confidence >= 0.0 && confidence <= 1.0 {
                                                all_styles_with_confidence
                                                    .push(format!("{}:{:.2}", style, confidence));

                                                if confidence > 0.80 {
                                                    high_confidence_styles.push(style);
                                                }
                                            } else {
                                                println!("Rejected invalid confidence range for '{}': {}", style, confidence);
                                            }
                                        } else {
                                            println!(
                                                "Rejected invalid confidence format for '{}': '{}'",
                                                style,
                                                confidence_str.trim()
                                            );
                                        }
                                    } else {
                                        println!("Rejected invalid style name: '{}'", style);
                                    }
                                } else {
                                    println!(
                                        "Rejected malformed entry (no colon): '{}'",
                                        style_entry
                                    );
                                }
                            }

                            println!(
                                "All identified styles for {:?}: {}",
                                image_path.file_name(),
                                all_styles_with_confidence.join(", ")
                            );

                            if high_confidence_styles.is_empty() {
                                println!(
                                    "No styles with confidence > 0.80 for {:?}",
                                    image_path.file_name()
                                );
                            } else {
                                println!(
                                    "High-confidence styles (>0.80) for {:?}: {}",
                                    image_path.file_name(),
                                    high_confidence_styles.join(", ")
                                );
                            }

                            return high_confidence_styles;
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
    let mut tattoo_with_styles_count = 0;
    let mut non_tattoo_count = 0;
    let mut error_count = 0;

    for handle in handles {
        match handle.await {
            Ok(styles) => {
                if styles.is_empty() {
                    // This could be either non-tattoo or tattoo with no confident styles
                    // We'll track this as part of the processing, but the actual categorization
                    // happens in the individual tasks based on the response content
                    non_tattoo_count += 1;
                } else {
                    tattoo_with_styles_count += 1;
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
    println!(
        "Images with high-confidence tattoo styles (>0.80): {}",
        tattoo_with_styles_count
    );
    println!(
        "Images with no confident styles or non-tattoos: {}",
        non_tattoo_count
    );
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

