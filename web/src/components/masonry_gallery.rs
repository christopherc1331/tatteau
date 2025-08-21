use leptos::prelude::*;
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
                short_code: "DMNyOgCOe39".to_string(),
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
                short_code: "DM3HnAKu2hB".to_string(),
                artist_id: 1,
            },
            styles: vec![
                Style {
                    id: 3,
                    name: "Japanese".to_string(),
                },
                Style {
                    id: 6,
                    name: "Watercolor".to_string(),
                },
            ],
        },
        InstagramPost {
            image: ArtistImage {
                id: 3,
                short_code: "DF8HJprp4yu".to_string(),
                artist_id: 2,
            },
            styles: vec![
                Style {
                    id: 4,
                    name: "Neo-Traditional".to_string(),
                },
                Style {
                    id: 5,
                    name: "Color Realism".to_string(),
                },
            ],
        },
        InstagramPost {
            image: ArtistImage {
                id: 4,
                short_code: "DNQodNBtvOv".to_string(),
                artist_id: 3,
            },
            styles: vec![
                Style {
                    id: 7,
                    name: "Geometric".to_string(),
                },
                Style {
                    id: 8,
                    name: "Dotwork".to_string(),
                },
                Style {
                    id: 9,
                    name: "Blackwork".to_string(),
                },
            ],
        },
        InstagramPost {
            image: ArtistImage {
                id: 5,
                short_code: "DLiEi_9OPpv".to_string(),
                artist_id: 4,
            },
            styles: vec![
                Style {
                    id: 10,
                    name: "Realism".to_string(),
                },
            ],
        },
        InstagramPost {
            image: ArtistImage {
                id: 6,
                short_code: "DC2JM-6yCV4".to_string(),
                artist_id: 5,
            },
            styles: vec![
                Style {
                    id: 11,
                    name: "Tribal".to_string(),
                },
                Style {
                    id: 12,
                    name: "Polynesian".to_string(),
                },
            ],
        },
        InstagramPost {
            image: ArtistImage {
                id: 7,
                short_code: "DNOFP6Ctf7C".to_string(),
                artist_id: 6,
            },
            styles: vec![
                Style {
                    id: 13,
                    name: "Minimalist".to_string(),
                },
                Style {
                    id: 14,
                    name: "Fine Line".to_string(),
                },
                Style {
                    id: 15,
                    name: "Single Needle".to_string(),
                },
            ],
        },
        InstagramPost {
            image: ArtistImage {
                id: 8,
                short_code: "DMQeBWNOGvR".to_string(),
                artist_id: 7,
            },
            styles: vec![
                Style {
                    id: 16,
                    name: "Old School".to_string(),
                },
                Style {
                    id: 17,
                    name: "American Traditional".to_string(),
                },
            ],
        },
        InstagramPost {
            image: ArtistImage {
                id: 9,
                short_code: "DKkJ0B3OVhg".to_string(),
                artist_id: 8,
            },
            styles: vec![
                Style {
                    id: 18,
                    name: "Illustrative".to_string(),
                },
                Style {
                    id: 19,
                    name: "Sketch".to_string(),
                },
                Style {
                    id: 20,
                    name: "Graphic".to_string(),
                },
                Style {
                    id: 21,
                    name: "Etching".to_string(),
                },
            ],
        },
        InstagramPost {
            image: ArtistImage {
                id: 10,
                short_code: "DJZc9iKu0kv".to_string(),
                artist_id: 9,
            },
            styles: vec![
                Style {
                    id: 22,
                    name: "Biomechanical".to_string(),
                },
                Style {
                    id: 23,
                    name: "Horror".to_string(),
                },
            ],
        },
        InstagramPost {
            image: ArtistImage {
                id: 11,
                short_code: "DKE4I8nOtWj".to_string(),
                artist_id: 10,
            },
            styles: vec![
                Style {
                    id: 24,
                    name: "Portrait".to_string(),
                },
                Style {
                    id: 25,
                    name: "Hyperrealism".to_string(),
                },
                Style {
                    id: 26,
                    name: "Photorealism".to_string(),
                },
            ],
        },
        InstagramPost {
            image: ArtistImage {
                id: 12,
                short_code: "DLaTX99O18N".to_string(),
                artist_id: 11,
            },
            styles: vec![
                Style {
                    id: 27,
                    name: "Chicano".to_string(),
                },
                Style {
                    id: 28,
                    name: "Lettering".to_string(),
                },
                Style {
                    id: 29,
                    name: "Script".to_string(),
                },
            ],
        },
        InstagramPost {
            image: ArtistImage {
                id: 13,
                short_code: "DKAHf9VNML6".to_string(),
                artist_id: 12,
            },
            styles: vec![
                Style {
                    id: 30,
                    name: "Ornamental".to_string(),
                },
                Style {
                    id: 31,
                    name: "Mandala".to_string(),
                },
            ],
        },
        InstagramPost {
            image: ArtistImage {
                id: 14,
                short_code: "DJrVzvSOOZy".to_string(),
                artist_id: 13,
            },
            styles: vec![
                Style {
                    id: 32,
                    name: "Surrealism".to_string(),
                },
                Style {
                    id: 33,
                    name: "Abstract".to_string(),
                },
            ],
        },
        InstagramPost {
            image: ArtistImage {
                id: 15,
                short_code: "DLIN_xquVMh".to_string(),
                artist_id: 14,
            },
            styles: vec![
                Style {
                    id: 34,
                    name: "Celtic".to_string(),
                },
                Style {
                    id: 35,
                    name: "Nordic".to_string(),
                },
                Style {
                    id: 36,
                    name: "Runes".to_string(),
                },
            ],
        },
        InstagramPost {
            image: ArtistImage {
                id: 16,
                short_code: "DKhthd1OVJ-".to_string(),
                artist_id: 15,
            },
            styles: vec![
                Style {
                    id: 37,
                    name: "Trash Polka".to_string(),
                },
            ],
        },
        InstagramPost {
            image: ArtistImage {
                id: 17,
                short_code: "DJKQa9Oy2Z3".to_string(),
                artist_id: 16,
            },
            styles: vec![
                Style {
                    id: 38,
                    name: "New School".to_string(),
                },
                Style {
                    id: 39,
                    name: "Cartoon".to_string(),
                },
                Style {
                    id: 40,
                    name: "Anime".to_string(),
                },
            ],
        },
        InstagramPost {
            image: ArtistImage {
                id: 18,
                short_code: "DKCywApuCYa".to_string(),
                artist_id: 17,
            },
            styles: vec![
                Style {
                    id: 41,
                    name: "Patchwork".to_string(),
                },
                Style {
                    id: 42,
                    name: "Stick and Poke".to_string(),
                },
            ],
        },
        InstagramPost {
            image: ArtistImage {
                id: 19,
                short_code: "DL5KRdQOSiB".to_string(),
                artist_id: 18,
            },
            styles: vec![
                Style {
                    id: 43,
                    name: "Glitch".to_string(),
                },
                Style {
                    id: 44,
                    name: "Cyber".to_string(),
                },
                Style {
                    id: 45,
                    name: "Futuristic".to_string(),
                },
            ],
        },
        InstagramPost {
            image: ArtistImage {
                id: 20,
                short_code: "DNVw2GpNviS".to_string(),
                artist_id: 19,
            },
            styles: vec![
                Style {
                    id: 46,
                    name: "Sacred Geometry".to_string(),
                },
                Style {
                    id: 47,
                    name: "Spiritual".to_string(),
                },
                Style {
                    id: 48,
                    name: "Esoteric".to_string(),
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
        if width >= 1400 {
            4
        } else if width >= 1000 {
            3
        } else if width >= 768 {
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
            <p style="font-size: 0.75rem; color: #666;">
                {move || format!("Screen width: {}px | Columns: {}", screen_width.get(), column_count.get())}
            </p>

            <style>
                {r#"
                    .masonry-grid {
                        column-count: 4;
                        column-gap: 1.5rem;
                        padding: 1.5rem;
                        max-width: 1800px;
                        margin: 0 auto;
                    }
                    
                    @media (max-width: 1400px) {
                        .masonry-grid {
                            column-count: 3;
                        }
                    }
                    
                    @media (max-width: 1000px) {
                        .masonry-grid {
                            column-count: 2;
                        }
                    }
                    
                    @media (max-width: 768px) {
                        .masonry-grid {
                            column-count: 1;
                        }
                    }
                    
                    .masonry-item {
                        width: 100%;
                        break-inside: avoid;
                        margin-bottom: 1.5rem;
                        display: inline-block;
                    }
                    
                    .instagram-card {
                        background: white;
                        border-radius: 12px;
                        box-shadow: 0 4px 12px rgba(0,0,0,0.08);
                        overflow: hidden;
                        transition: transform 0.2s ease, box-shadow 0.2s ease;
                        width: 100%;
                        min-width: 0;
                        height: fit-content;
                    }
                    
                    .instagram-card:hover {
                        transform: translateY(-2px);
                        box-shadow: 0 6px 20px rgba(0,0,0,0.12);
                    }
                    
                    .style-chips {
                        padding: 0.75rem;
                        display: flex;
                        flex-wrap: wrap;
                        gap: 0.375rem;
                        background: #fafafa;
                        border-bottom: 1px solid #eee;
                    }
                    
                    .style-chip {
                        padding: 0.25rem 0.625rem;
                        border-radius: 14px;
                        background: #333;
                        color: white;
                        font-size: 0.625rem;
                        font-weight: 600;
                        letter-spacing: 0.4px;
                        text-transform: uppercase;
                        transition: all 0.2s ease;
                        display: inline-flex;
                        align-items: center;
                        white-space: nowrap;
                        border: 2px solid transparent;
                    }
                    
                    .style-chip:hover {
                        background: #555;
                        transform: translateY(-1px);
                    }
                    
                    /* Different colors for variety - more subtle */
                    .style-chip:nth-child(2n) {
                        background: #5a67d8;
                    }
                    
                    .style-chip:nth-child(3n) {
                        background: #ed8936;
                    }
                    
                    .style-chip:nth-child(4n) {
                        background: #38a169;
                    }
                    
                    .style-chip:nth-child(5n) {
                        background: #e53e3e;
                    }
                    
                    .style-chip:nth-child(6n) {
                        background: #805ad5;
                    }
                    
                    .style-chip:nth-child(7n) {
                        background: #2d3748;
                    }
                    
                    .instagram-embed-container {
                        position: relative;
                        width: 100%;
                        min-width: 0;
                        overflow: hidden;
                        height: auto;
                    }
                    
                    .instagram-wrapper {
                        width: 100%;
                        min-width: 0;
                        height: auto;
                    }
                    
                    /* Force Instagram embeds to be responsive */
                    .instagram-media {
                        max-width: 100% !important;
                        min-width: 200px !important;
                        width: 100% !important;
                        margin: 0 auto !important;
                    }
                    
                    .instagram-media iframe {
                        max-width: 100% !important;
                        min-width: 200px !important;
                        width: 100% !important;
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
                "#}
            </style>

            <div class="masonry-grid">
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
                                                    <span class="style-chip">{style.name}</span>
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
