use leptos::prelude::*;
use leptos_router::hooks::{use_params_map, use_query_map};

use crate::{
    components::{
        loading::LoadingView,
        shop_masonry_gallery::{ShopInstagramPost, ShopMasonryGallery},
    },
    db::entities::{Artist, Style},
    server::{fetch_shop_data, fetch_shop_images_paginated},
};

#[component]
pub fn Shop() -> impl IntoView {
    let params = use_params_map();
    let query = use_query_map();
    let navigate = leptos_router::hooks::use_navigate();

    let shop_id = Memo::new(move |_| {
        params
            .read()
            .get("id")
            .and_then(|id| id.parse::<i32>().ok())
            .unwrap_or(0)
    });

    // Get auth token from localStorage
    let get_auth_token = move || -> Option<String> {
        #[cfg(feature = "hydrate")]
        {
            use wasm_bindgen::prelude::*;

            #[wasm_bindgen]
            extern "C" {
                #[wasm_bindgen(js_namespace = localStorage)]
                fn getItem(key: &str) -> Option<String>;
            }

            getItem("tatteau_auth_token")
        }
        #[cfg(not(feature = "hydrate"))]
        {
            None
        }
    };

    // Parse style IDs and page from query params
    let selected_styles = RwSignal::new(Vec::<i32>::new());
    let current_page = RwSignal::new(0);
    let per_page = 10;

    // Update selected styles and page from query params
    Effect::new(move |_| {
        let query_map = query.read();

        // Parse styles
        if let Some(styles_str) = query_map.get("styles") {
            let style_ids: Vec<i32> = styles_str
                .split(',')
                .filter_map(|s| s.trim().parse::<i32>().ok())
                .collect();
            selected_styles.set(style_ids);
        } else {
            selected_styles.set(Vec::new());
        }

        // Parse page
        if let Some(page_str) = query_map.get("page") {
            if let Ok(page) = page_str.parse::<i32>() {
                current_page.set(page.max(0));
            } else {
                current_page.set(0);
            }
        } else {
            current_page.set(0);
        }
    });

    let shop_data = Resource::new(
        move || shop_id.get(),
        move |id| async move {
            if id > 0 {
                let token = get_auth_token();
                fetch_shop_data(id, token).await.ok()
            } else {
                None
            }
        },
    );

    // Paginated images resource
    let paginated_images = Resource::new(
        move || (shop_id.get(), selected_styles.get(), current_page.get()),
        move |(id, styles, page)| async move {
            if id > 0 {
                let style_filter = if styles.is_empty() {
                    None
                } else {
                    Some(styles)
                };
                let token = get_auth_token();
                fetch_shop_images_paginated(id, style_filter, page, per_page, token)
                    .await
                    .ok()
            } else {
                None
            }
        },
    );

    view! {
        <div class="shop-container">
            <Suspense fallback=move || view! {
                <LoadingView message=Some("Loading shop information...".to_string()) />
            }>
                {
                    let navigate_clone = navigate.clone();
                    move || {
                        let navigate = navigate_clone.clone();
                        shop_data.get().map(|data| {
                        data.map(|shop_data| {
                            let shop_name = shop_data.location.name.unwrap_or_else(|| "Unknown Shop".to_string());
                            let city = shop_data.location.city.unwrap_or_else(|| "Unknown".to_string());
                            let state = shop_data.location.state.unwrap_or_else(|| "Unknown".to_string());

                            let all_styles_for_filter = shop_data.all_styles.clone();

                            let shop_posts: Vec<ShopInstagramPost> = shop_data.all_images_with_styles
                                .clone()
                                .into_iter()
                                .map(|(image, styles, artist, is_favorited)| ShopInstagramPost {
                                    image,
                                    styles,
                                    artist,
                                    is_favorited,
                                })
                                .collect();

                            let address = shop_data.location.address.clone().unwrap_or_else(|| String::new());
                            let artists_clone = shop_data.artists.clone();

                            // Create Google Maps directions URL
                            let directions_url = if !address.is_empty() {
                                format!("https://www.google.com/maps/dir/?api=1&destination={}",
                                    urlencoding::encode(&address))
                            } else if !city.is_empty() && !state.is_empty() {
                                format!("https://www.google.com/maps/dir/?api=1&destination={},{}",
                                    urlencoding::encode(&city),
                                    urlencoding::encode(&state))
                            } else {
                                String::new()
                            };

                            view! {
                                <div class="shop-container">
                                    <div class="shop-header">
                                        <div class="shop-header-content">
                                            <div class="shop-header-main">
                                                <div class="shop-header-info">
                                                    <h1 class="shop-title">
                                                        {shop_name.clone()}
                                                    </h1>
                                                    <div class="shop-location-header">
                                                        {format!("{}, {}", city, state)}
                                                    </div>
                                                </div>

                                                <div class="shop-actions">
                                                    {(!directions_url.is_empty()).then(|| {
                                                        view! {
                                                            <a href=directions_url.clone() target="_blank"
                                                               class="shop-action-button shop-action-directions">
                                                                "📍 Get Directions"
                                                            </a>
                                                        }
                                                    })}
                                                    {shop_data.location.website_uri.as_ref().filter(|uri| !uri.is_empty()).map(|website_uri| {
                                                        view! {
                                                            <a href=website_uri.clone() target="_blank"
                                                               class="shop-action-button shop-action-website">
                                                                "🌐 Visit Website"
                                                            </a>
                                                        }
                                                    })}
                                                </div>
                                            </div>
                                        </div>
                                    </div>

                                    <div class="shop-main-content">
                                        {(!artists_clone.is_empty()).then(|| {
                                            view! {
                                                <div class="shop-artists-compact">
                                                    <h3 class="shop-section-subtitle">"Artists"</h3>
                                                    <div class="shop-artists-chips">
                                                        {artists_clone.into_iter().map(|artist| {
                                                            let artist_name = artist.name.unwrap_or_else(|| "Unknown".to_string());
                                                            view! {
                                                                <a href={format!("/artist/{}", artist.id)}
                                                                   class="shop-artist-chip">
                                                                    {artist_name}
                                                                </a>
                                                            }
                                                        }).collect_view()}
                                                    </div>
                                                </div>
                                            }
                                        })}

                                        // Portfolio section with server-side filtering
                                        <div class="shop-info-card">
                                            <h2 class="shop-portfolio-title">"Shop Portfolio"</h2>

                                            // Style filter chips
                                            <div class="shop-masonry-gallery__filter-container">
                                                <div class="shop-masonry-gallery__filter-wrapper">
                                                    <span class="shop-masonry-gallery__filter-label">"Filter by style:"</span>

                                                    {
                                                        let navigate_all = navigate.clone();
                                                        view! {
                                                            <button
                                                                on:click=move |_| {
                                                                    let nav_options = leptos_router::NavigateOptions {
                                                                        scroll: false,
                                                                        ..Default::default()
                                                                    };
                                                                    navigate_all(&format!("/shop/{}", shop_id.get()), nav_options);
                                                                }
                                                                class="shop-masonry-gallery__filter-button"
                                                                class:shop-masonry-gallery__filter-button--active=move || selected_styles.get().is_empty()
                                                                class:shop-masonry-gallery__filter-button--inactive=move || !selected_styles.get().is_empty()
                                                            >
                                                                "All"
                                                            </button>
                                                        }
                                                    }

                                                    {all_styles_for_filter.clone().into_iter().map(|style| {
                                                        let style_id = style.id;
                                                        let style_name = style.name.clone();
                                                        let navigate_style = navigate.clone();
                                                        view! {
                                                            <button
                                                                on:click=move |_| {
                                                                    // Calculate new styles based on current URL state (single-select)
                                                                    let current_styles = selected_styles.get();
                                                                    let new_styles: Vec<i32> = if current_styles.contains(&style_id) {
                                                                        Vec::new()
                                                                    } else {
                                                                        vec![style_id]
                                                                    };

                                                                    let nav_options = leptos_router::NavigateOptions {
                                                                        scroll: false,
                                                                        ..Default::default()
                                                                    };

                                                                    if new_styles.is_empty() {
                                                                        navigate_style(&format!("/shop/{}", shop_id.get()), nav_options);
                                                                    } else {
                                                                        let styles_str = new_styles
                                                                            .iter()
                                                                            .map(|id| id.to_string())
                                                                            .collect::<Vec<_>>()
                                                                            .join(",");
                                                                        navigate_style(&format!("/shop/{}?styles={}", shop_id.get(), styles_str), nav_options);
                                                                    }
                                                                }
                                                                class="shop-masonry-gallery__filter-button"
                                                                class:shop-masonry-gallery__filter-button--active=move || selected_styles.get().contains(&style_id)
                                                                class:shop-masonry-gallery__filter-button--inactive=move || !selected_styles.get().contains(&style_id)
                                                            >
                                                                {style_name}
                                                            </button>
                                                        }
                                                    }).collect_view()}
                                                </div>
                                            </div>

                                            // Paginated gallery
                                            <Suspense fallback=|| view! { <div>"Loading images..."</div> }>
                                                {move || {
                                                    paginated_images.get().map(|result| {
                                                        result.map(|(images, total_count)| {
                                                            let total_pages = (total_count as f32 / per_page as f32).ceil() as i32;

                                                            let shop_posts: Vec<ShopInstagramPost> = images
                                                                .into_iter()
                                                                .map(|(image, styles, artist, is_favorited)| ShopInstagramPost {
                                                                    image,
                                                                    styles,
                                                                    artist,
                                                                    is_favorited,
                                                                })
                                                                .collect();

                                                            view! {
                                                                <>
                                                                    {(!shop_posts.is_empty()).then(|| {
                                                                        view! {
                                                                            <ShopMasonryGallery
                                                                                shop_posts=shop_posts
                                                                                all_styles=vec![]
                                                                            />
                                                                        }
                                                                    })}

                                                                    {(total_pages > 1).then(|| {
                                                                        let total = total_pages;
                                                                        let navigate_prev = navigate.clone();
                                                                        let navigate_next = navigate.clone();
                                                                        view! {
                                                                            <div class="shop-masonry-gallery__pagination">
                                                                                <button
                                                                                    class="shop-masonry-gallery__pagination-button"
                                                                                    disabled={move || current_page.get() == 0}
                                                                                    on:click=move |_| {
                                                                                        if current_page.get() > 0 {
                                                                                            let new_page = current_page.get() - 1;
                                                                                            let styles_str = selected_styles.get()
                                                                                                .iter()
                                                                                                .map(|id| id.to_string())
                                                                                                .collect::<Vec<_>>()
                                                                                                .join(",");
                                                                                            let mut url = format!("/shop/{}", shop_id.get());
                                                                                            let mut params = vec![];
                                                                                            if !styles_str.is_empty() {
                                                                                                params.push(format!("styles={}", styles_str));
                                                                                            }
                                                                                            if new_page > 0 {
                                                                                                params.push(format!("page={}", new_page));
                                                                                            }
                                                                                            if !params.is_empty() {
                                                                                                url = format!("{}?{}", url, params.join("&"));
                                                                                            }
                                                                                            let nav_options = leptos_router::NavigateOptions {
                                                                                                scroll: false,
                                                                                                ..Default::default()
                                                                                            };
                                                                                            navigate_prev(&url, nav_options);
                                                                                        }
                                                                                    }
                                                                                >
                                                                                    "← Previous"
                                                                                </button>

                                                                                <span class="shop-masonry-gallery__pagination-info">
                                                                                    {move || format!("Page {} of {}", current_page.get() + 1, total)}
                                                                                </span>

                                                                                <button
                                                                                    class="shop-masonry-gallery__pagination-button"
                                                                                    disabled={move || current_page.get() >= total - 1}
                                                                                    on:click=move |_| {
                                                                                        if current_page.get() < total - 1 {
                                                                                            let new_page = current_page.get() + 1;
                                                                                            let styles_str = selected_styles.get()
                                                                                                .iter()
                                                                                                .map(|id| id.to_string())
                                                                                                .collect::<Vec<_>>()
                                                                                                .join(",");
                                                                                            let mut url = format!("/shop/{}", shop_id.get());
                                                                                            let mut params = vec![];
                                                                                            if !styles_str.is_empty() {
                                                                                                params.push(format!("styles={}", styles_str));
                                                                                            }
                                                                                            params.push(format!("page={}", new_page));
                                                                                            if !params.is_empty() {
                                                                                                url = format!("{}?{}", url, params.join("&"));
                                                                                            }
                                                                                            let nav_options = leptos_router::NavigateOptions {
                                                                                                scroll: false,
                                                                                                ..Default::default()
                                                                                            };
                                                                                            navigate_next(&url, nav_options);
                                                                                        }
                                                                                    }
                                                                                >
                                                                                    "Next →"
                                                                                </button>
                                                                            </div>
                                                                        }
                                                                    })}
                                                                </>
                                                            }.into_any()
                                                        }).unwrap_or_else(|| {
                                                            view! {
                                                                <div>"No images found."</div>
                                                            }.into_any()
                                                        })
                                                    })
                                                }}
                                            </Suspense>
                                        </div>
                                    </div>
                                </div>
                            }.into_any()
                        }).unwrap_or_else(|| {
                            view! {
                                <div class="shop-container">
                                    <div class="shop-error-header">
                                        <div class="shop-error-header-content">
                                            <h1 class="shop-error-title">
                                                "🏪 Shop Not Found"
                                            </h1>
                                            <div class="shop-error-subtitle">
                                                "The requested shop could not be found"
                                            </div>
                                        </div>
                                    </div>

                                    <div class="shop-error-content">
                                        <div class="shop-error-card">
                                            <p class="shop-error-text">
                                                "Please check the shop ID and try again."
                                            </p>
                                        </div>
                                    </div>
                                </div>
                            }.into_any()
                        })
                    })
                    }
                }
            </Suspense>
        </div>
    }
}

