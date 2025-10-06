use leptos::prelude::*;
use crate::db::entities::{ArtistImage, Style, Artist};
use crate::components::instagram_embed::InstagramEmbed;

#[derive(Clone, Debug, PartialEq)]
pub struct ShopInstagramPost {
    pub image: ArtistImage,
    pub styles: Vec<Style>,
    pub artist: Artist,
}

#[component]
pub fn ShopMasonryGallery(
    shop_posts: Vec<ShopInstagramPost>,
    all_styles: Vec<Style>,
) -> impl IntoView {
    let (selected_style, set_selected_style) = signal::<Option<i32>>(None);

    // Pagination state
    let (current_page, set_current_page) = signal(0usize);
    let posts_per_page = 10;

    // Filter posts by selected style
    let filtered_posts = Memo::new(move |_| {
        if let Some(style_id) = selected_style.get() {
            shop_posts.iter()
                .filter(|post| post.styles.iter().any(|s| s.id == style_id))
                .cloned()
                .collect::<Vec<_>>()
        } else {
            shop_posts.clone()
        }
    });

    // Calculate pagination
    let total_pages = Memo::new(move |_| {
        let total = filtered_posts.get().len();
        (total + posts_per_page - 1) / posts_per_page
    });

    // Get current page posts
    let current_page_posts = Memo::new(move |_| {
        let posts = filtered_posts.get();
        let page = current_page.get();
        let start = page * posts_per_page;
        let end = std::cmp::min(start + posts_per_page, posts.len());

        if start < posts.len() {
            posts[start..end].to_vec()
        } else {
            vec![]
        }
    });

    // Reset to page 0 when filter changes
    Effect::new(move |_| {
        let _ = selected_style.get();
        set_current_page.set(0);
    });

    let prev_btn_disabled: Signal<bool> = Signal::derive(move || current_page.get() == 0);
    let next_btn_disabled: Signal<bool> = Signal::derive(move || {
        let total = total_pages.get();
        current_page.get() >= total.saturating_sub(1)
    });

    view! {
        <div class="shop-masonry-gallery__container">
            <div class="shop-masonry-gallery__filter-container">
                <div class="shop-masonry-gallery__filter-wrapper">
                    <span class="shop-masonry-gallery__filter-label">"Filter by style:"</span>

                    <button
                        on:click=move |_| {
                            set_selected_style.set(None);
                        }
                        class:shop-masonry-gallery__filter-button--active=move || selected_style.get().is_none()
                        class:shop-masonry-gallery__filter-button--inactive=move || selected_style.get().is_some()
                        class="shop-masonry-gallery__filter-button"
                    >
                        "All"
                    </button>

                    {all_styles.into_iter().map(|style| {
                        let style_id = style.id;
                        let style_name = style.name.clone();
                        view! {
                            <button
                                on:click=move |_| {
                                    set_selected_style.set(Some(style_id));
                                }
                                class:shop-masonry-gallery__filter-button--active=move || selected_style.get() == Some(style_id)
                                class:shop-masonry-gallery__filter-button--inactive=move || selected_style.get() != Some(style_id)
                                class="shop-masonry-gallery__filter-button"
                            >
                                {style_name}
                            </button>
                        }
                    }).collect_view()}
                </div>
            </div>

            <div class="shop-masonry-gallery__masonry">
                {move || current_page_posts.get().into_iter().map(|post| {
                    let short_code = post.image.short_code.clone();
                    let artist_name = post.artist.name.unwrap_or_else(|| "Unknown Artist".to_string());
                    let post_id = post.image.id;

                    view! {
                        <div class="shop-masonry-gallery__post">
                            <div class="shop-masonry-gallery__card">
                                <div class="shop-masonry-gallery__artist-header">
                                    <a href={format!("/artist/{}", post.artist.id)}
                                       class="shop-masonry-gallery__artist-link">
                                        <span>"👤"</span>
                                        <span>{artist_name}</span>
                                    </a>
                                </div>

                                <InstagramEmbed short_code={short_code} />

                                {(!post.styles.is_empty()).then(|| {
                                    view! {
                                        <div class="shop-masonry-gallery__styles-container">
                                            <div class="shop-masonry-gallery__styles-wrapper">
                                                {post.styles.into_iter().map(|style| {
                                                    view! {
                                                        <span class="shop-masonry-gallery__style-tag">
                                                            {style.name}
                                                        </span>
                                                    }
                                                }).collect_view()}
                                            </div>
                                        </div>
                                    }
                                })}
                            </div>
                        </div>
                    }
                }).collect_view()}
            </div>

            // Pagination controls
            {move || {
                let total = total_pages.get();
                (total > 1).then(|| {
                    view! {
                        <div class="shop-masonry-gallery__pagination">
                            <button
                                class="shop-masonry-gallery__pagination-button"
                                disabled=move || prev_btn_disabled.get()
                                on:click=move |_| {
                                    if !prev_btn_disabled.get() {
                                        set_current_page.set(current_page.get() - 1);
                                        // Scroll to top smoothly
                                        if let Some(window) = web_sys::window() {
                                            let _ = window.scroll_with_scroll_to_options(
                                                web_sys::ScrollToOptions::new()
                                                    .top(0.0)
                                                    .behavior(web_sys::ScrollBehavior::Smooth)
                                            );
                                        }
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
                                disabled=move || next_btn_disabled.get()
                                on:click=move |_| {
                                    if !next_btn_disabled.get() {
                                        set_current_page.set(current_page.get() + 1);
                                        // Scroll to top smoothly
                                        if let Some(window) = web_sys::window() {
                                            let _ = window.scroll_with_scroll_to_options(
                                                web_sys::ScrollToOptions::new()
                                                    .top(0.0)
                                                    .behavior(web_sys::ScrollBehavior::Smooth)
                                            );
                                        }
                                    }
                                }
                            >
                                "Next →"
                            </button>
                        </div>
                    }
                })
            }}
        </div>
    }
}