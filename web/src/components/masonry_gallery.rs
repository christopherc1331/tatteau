use leptos::prelude::*;
use thaw::Tag;
use wasm_bindgen::{closure::Closure, JsCast};
use web_sys::window;

// [Keep your existing structs - Artist, ArtistStyle, ArtistImage, etc.]
#[derive(Debug, Clone, PartialEq)]
pub struct Artist {
    pub id: i32,
    pub name: Option<String>,
    pub location_id: i32,
    pub social_links: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub years_experience: Option<i32>,
    pub styles_extracted: Option<i32>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ArtistStyle {
    pub id: i32,
    pub style_id: i32,
    pub artist_id: i32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ArtistImage {
    pub id: i32,
    pub short_code: String,
    pub artist_id: i32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ArtistImageStyle {
    pub id: i32,
    pub artists_images_id: i32,
    pub style_id: i32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Style {
    pub id: i32,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct InstagramPost {
    pub image: ArtistImage,
    pub styles: Vec<Style>,
}

pub fn generate_sample_posts() -> Vec<InstagramPost> {
    vec![
        InstagramPost {
            image: ArtistImage {
                id: 1,
                short_code: "DNQodNBtvOv".to_string(),
                artist_id: 1,
            },
            styles: vec![
                Style {
                    id: 1,
                    name: "Traditional".to_string(),
                },
                Style {
                    id: 2,
                    name: "Black and Gray".to_string(),
                },
            ],
        },
        InstagramPost {
            image: ArtistImage {
                id: 2,
                short_code: "DNQodNBtvOv".to_string(),
                artist_id: 1,
            },
            styles: vec![
                Style {
                    id: 3,
                    name: "Neo-Traditional".to_string(),
                },
                Style {
                    id: 4,
                    name: "Color Realism".to_string(),
                },
            ],
        },
        InstagramPost {
            image: ArtistImage {
                id: 3,
                short_code: "DNQodNBtvOv".to_string(),
                artist_id: 2,
            },
            styles: vec![
                Style {
                    id: 5,
                    name: "Japanese".to_string(),
                },
                Style {
                    id: 6,
                    name: "Watercolor".to_string(),
                },
            ],
        },
    ]
}

/// Client-only Instagram embed component that avoids hydration issues
#[component]
fn InstagramEmbed(post: InstagramPost) -> impl IntoView {
    // Track if we're on the client
    let (is_client, set_is_client) = signal(false);
    let embed_container_ref = NodeRef::<leptos::html::Div>::new();

    // Clone short_code for use in the closure
    let short_code_for_effect = post.image.short_code.clone();

    // Only run on client side
    Effect::new(move |_| {
        // This effect only runs on the client after hydration
        set_is_client.set(true);

        // Insert the Instagram embed after hydration is complete
        if let Some(container) = embed_container_ref.get() {
            if let Some(window) = window() {
                if let Some(document) = window.document() {
                    // Clear the container first
                    container.set_inner_html("");

                    // Create the blockquote element programmatically
                    if let Ok(blockquote) = document.create_element("blockquote") {
                        blockquote.set_class_name("instagram-media");
                        blockquote.set_attribute("data-instgrm-captioned", "").ok();
                        blockquote
                            .set_attribute(
                                "data-instgrm-permalink",
                                &format!("https://www.instagram.com/p/{}/", short_code_for_effect),
                            )
                            .ok();
                        blockquote.set_attribute("data-instgrm-version", "14").ok();

                        // Set the style
                        blockquote.set_attribute(
                            "style",
                            "background:#FFF; border:0; border-radius:3px; box-shadow:0 0 1px 0 rgba(0,0,0,0.5),0 1px 10px 0 rgba(0,0,0,0.15); margin: 1px; max-width:100%; min-width:326px; padding:0; width:99.375%; width:-webkit-calc(100% - 2px); width:calc(100% - 2px);"
                        ).ok();

                        // Add a link inside
                        if let Ok(link) = document.create_element("a") {
                            link.set_attribute(
                                "href",
                                &format!("https://www.instagram.com/p/{}/", short_code_for_effect),
                            )
                            .ok();
                            link.set_attribute("target", "_blank").ok();
                            link.set_text_content(Some("View this post on Instagram"));
                            blockquote.append_child(&link).ok();
                        }

                        // Append to container
                        container.append_child(&blockquote).ok();

                        // Process Instagram embeds after a short delay
                        let closure = Closure::wrap(Box::new(move || {
                            // Use JavaScript to process Instagram embeds
                            if let Some(window) = web_sys::window() {
                                if let Some(document) = window.document() {
                                    if let Ok(script) = document.create_element("script") {
                                        script.set_text_content(Some(
                                            "if (window.instgrm && window.instgrm.Embeds) { window.instgrm.Embeds.process(); }"
                                        ));
                                        if let Some(body) = document.body() {
                                            body.append_child(&script).ok();
                                            body.remove_child(&script).ok();
                                        }
                                    }
                                }
                            }
                        }) as Box<dyn FnMut()>);

                        window
                            .set_timeout_with_callback_and_timeout_and_arguments_0(
                                closure.as_ref().unchecked_ref(),
                                100,
                            )
                            .ok();
                        closure.forget();
                    }
                }
            }
        }
    });

    view! {
        <div class="instagram-embed-container">
            <Show
                when=move || !is_client.get()
                fallback=move || view! {
                    // Client-side: Container for the Instagram embed
                    <div node_ref=embed_container_ref class="instagram-wrapper"></div>
                }
            >
                // Server-side: Render a placeholder
                <div class="instagram-placeholder">
                    <div class="placeholder-content">
                        <svg width="50" height="50" viewBox="0 0 60 60" fill="#ccc">
                            <path d="M30,0 C13.4314567,0 0,13.4314567 0,30 C0,46.5685433 13.4314567,60 30,60 C46.5685433,60 60,46.5685433 60,30 C60,13.4314567 46.5685433,0 30,0 Z M30,54 C16.745166,54 6,43.254834 6,30 C6,16.745166 16.745166,6 30,6 C43.254834,6 54,16.745166 54,30 C54,43.254834 43.254834,54 30,54 Z"></path>
                        </svg>
                        <p>"Loading Instagram post..."</p>
                        <a
                            href=format!("https://www.instagram.com/p/{}/", post.image.short_code)
                            target="_blank"
                            class="fallback-link"
                        >
                            "View on Instagram"
                        </a>
                    </div>
                </div>
            </Show>
        </div>
    }
}

#[component]
pub fn MasonryGallery(
    #[prop(optional, default = Vec::new())] instagram_posts: Vec<InstagramPost>,
) -> impl IntoView {
    let (screen_width, set_screen_width) = signal(1200u32);

    // Use provided posts or fall back to sample posts
    let gallery_posts = if instagram_posts.is_empty() {
        generate_sample_posts()
    } else {
        instagram_posts
    };

    // Calculate responsive columns
    let column_count = Memo::new(move |_| {
        let width = screen_width.get();
        if width >= 1200 {
            4
        } else if width >= 768 {
            3
        } else if width >= 480 {
            2
        } else {
            1
        }
    });

    // Handle window resize
    Effect::new(move |_| {
        if let Some(win) = window() {
            let width = win.inner_width().unwrap().as_f64().unwrap() as u32;
            set_screen_width.set(width);

            let resize_closure = Closure::wrap(Box::new(move |_: web_sys::Event| {
                if let Some(win) = window() {
                    let width = win.inner_width().unwrap().as_f64().unwrap() as u32;
                    set_screen_width.set(width);
                }
            }) as Box<dyn FnMut(_)>);

            win.add_event_listener_with_callback("resize", resize_closure.as_ref().unchecked_ref())
                .unwrap();
            resize_closure.forget();
        }
    });

    view! {
        <div class="masonry-gallery">
            <h1>"Tattoo Artist Gallery"</h1>

            <style>
                {r#"
                    .masonry-grid {
                        column-gap: 1rem;
                        padding: 1rem;
                    }
                    
                    .masonry-item {
                        break-inside: avoid;
                        margin-bottom: 1rem;
                    }
                    
                    .instagram-card {
                        background: white;
                        border-radius: 8px;
                        box-shadow: 0 2px 8px rgba(0,0,0,0.1);
                        overflow: hidden;
                    }
                    
                    .style-chips {
                        padding: 0.5rem;
                        display: flex;
                        flex-wrap: wrap;
                        gap: 0.5rem;
                        background: #f5f5f5;
                    }
                    
                    .instagram-embed-container {
                        position: relative;
                        width: 100%;
                    }
                    
                    .instagram-placeholder {
                        min-height: 400px;
                        background: #fafafa;
                        border: 1px solid #e1e8ed;
                        border-radius: 3px;
                        display: flex;
                        align-items: center;
                        justify-content: center;
                    }
                    
                    .placeholder-content {
                        text-align: center;
                        color: #8899a6;
                    }
                    
                    .fallback-link {
                        color: #3897f0;
                        text-decoration: none;
                        font-weight: 600;
                        margin-top: 10px;
                        display: inline-block;
                    }
                    
                    .fallback-link:hover {
                        text-decoration: underline;
                    }
                    
                    /* Ensure Instagram embeds are responsive */
                    .instagram-media {
                        max-width: 100% !important;
                        width: 100% !important;
                    }
                "#}
            </style>

            <div
                class="masonry-grid"
                style:column-count=move || column_count.get().to_string()
            >
                <For
                    each=move || gallery_posts.clone()
                    key=|post| post.image.id
                    children=move |post: InstagramPost| {
                        // Clone styles for the inner For loop
                        let styles = post.styles.clone();

                        view! {
                            <div class="masonry-item">
                                <div class="instagram-card">
                                    <div class="style-chips">
                                        <For
                                            each=move || styles.clone()
                                            key=|style| style.id
                                            children=move |style: Style| {
                                                view! {
                                                    <Tag>{style.name}</Tag>
                                                }
                                            }
                                        />
                                    </div>
                                    // Use the client-only component
                                    <InstagramEmbed post=post.clone()/>
                                </div>
                            </div>
                        }
                    }
                />
            </div>
        </div>
    }
}
