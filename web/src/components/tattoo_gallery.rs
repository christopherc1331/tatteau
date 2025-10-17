use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::hooks::{use_query_map, use_navigate};
use wasm_bindgen::JsCast;

use crate::{
    components::{
        instagram_embed::process_instagram_embeds,
        instagram_posts_grid::{InstagramPostsGrid, PostWithArtist}
    },
    db::entities::{Artist, ArtistImage, Style},
    server::{get_artist_styles_by_id, MatchedArtist, TattooPost},
};

#[component]
pub fn TattooGallery(
    posts: Vec<TattooPost>,
    on_artist_click: Callback<MatchedArtist>,
) -> impl IntoView {
    let navigate = use_navigate();
    let query_map = use_query_map();

    // Initialize page from query params or default to 0
    let initial_page = query_map.get_untracked()
        .get("page")
        .and_then(|p| p.parse::<usize>().ok())
        .unwrap_or(0);

    let (current_page, set_current_page) = signal(initial_page);
    let posts_per_page = 10;

    // Store posts for later use in click handler
    let posts_for_modal = StoredValue::new(posts.clone());

    // Convert TattooPost to PostWithArtist format for InstagramPostsGrid
    let all_instagram_posts: Vec<PostWithArtist> = posts
        .into_iter()
        .map(|post| {
            // Create styles from the style names
            let styles: Vec<Style> = post
                .styles
                .into_iter()
                .enumerate()
                .map(|(index, name)| {
                    Style {
                        id: index as i32, // Using index as temporary ID
                        name,
                    }
                })
                .collect();

            // Create an Artist from the post data
            let artist = Artist {
                id: post.artist_id as i32,
                name: Some(post.artist_name),
                location_id: 0, // Placeholder
                email: None,
                phone: None,
                years_experience: None,
                social_links: None,
                styles_extracted: None,
            };

            // Create ArtistImage from post data
            let image = ArtistImage {
                id: post.id as i32, // Use the actual artists_images.id
                artist_id: post.artist_id as i32,
                short_code: post.short_code,
                post_date: None,
            };

            PostWithArtist {
                image,
                styles,
                artist: Some(artist),
                is_favorited: post.is_favorited,
            }
        })
        .collect();

    // Store the callback for later use
    let callback_for_click = StoredValue::new(on_artist_click);

    // Calculate pagination
    let total_posts = all_instagram_posts.len();
    let total_pages = (total_posts + posts_per_page - 1) / posts_per_page; // Ceiling division

    // Get current page posts
    let current_page_posts = Memo::new(move |_| {
        let page = current_page.get();
        let start = page * posts_per_page;
        let end = std::cmp::min(start + posts_per_page, total_posts);

        if start < total_posts {
            all_instagram_posts[start..end].to_vec()
        } else {
            vec![]
        }
    });
    let prev_btn_disabled: Signal<bool> = Signal::derive(move || current_page.get() == 0);
    let next_btn_disabled: Signal<bool> =
        Signal::derive(move || current_page.get() >= (total_pages - 1));

    view! {
        <div class="tattoo-gallery">

            // Use a custom wrapper to handle artist clicks
            <div on:click=move |ev| {
                // Check if click was on an artist link
                if let Some(target) = ev.target() {
                    if let Ok(element) = target.dyn_into::<web_sys::HtmlElement>() {
                        if let Ok(Some(href)) = element.closest("a[href*='/artist/']") {
                            ev.prevent_default();

                            // Extract artist ID from href
                            if let Ok(anchor) = href.dyn_into::<web_sys::HtmlAnchorElement>() {
                                let href_value = anchor.href();
                                if let Some(id_str) = href_value.split("/artist/").nth(1) {
                                    if let Ok(artist_id) = id_str.parse::<i64>() {
                                        // Find the tattoo post for this artist
                                        let callback = callback_for_click.get_value();
                                        let posts = posts_for_modal.get_value();

                                        if let Some(post) = posts.iter().find(|p| p.artist_id == artist_id) {
                                            let post_clone = post.clone();

                                            // Fetch all artist styles from artists_styles table
                                            spawn_local(async move {
                                                match get_artist_styles_by_id(artist_id).await {
                                                    Ok(artist_styles) => {
                                                        let all_styles = if artist_styles.is_empty() {
                                                            // Fallback to image styles if no artist styles found
                                                            post_clone.styles.clone()
                                                        } else {
                                                            artist_styles
                                                        };

                                                        let matched_artist = MatchedArtist {
                                                            id: post_clone.artist_id,
                                                            name: post_clone.artist_name.clone(),
                                                            location_name: String::new(), // Not available in TattooPost
                                                            city: String::new(),  // Not available in TattooPost
                                                            state: String::new(), // Not available in TattooPost
                                                            primary_style: all_styles.first().cloned().unwrap_or_default(),
                                                            all_styles,
                                                            image_count: 0, // Not available
                                                            portfolio_images: vec![],
                                                            avatar_url: None,
                                                            avg_rating: 0.0,
                                                            match_score: 0,
                                                            years_experience: None,
                                                            min_price: None,
                                                            max_price: None,
                                                        };
                                                        callback.run(matched_artist);
                                                    }
                                                    Err(e) => {
                                                        leptos::logging::error!("Failed to fetch artist styles: {}", e);
                                                        // Fallback to image styles on error
                                                        let matched_artist = MatchedArtist {
                                                            id: post_clone.artist_id,
                                                            name: post_clone.artist_name.clone(),
                                                            location_name: String::new(),
                                                            city: String::new(),
                                                            state: String::new(),
                                                            primary_style: post_clone.styles.first().cloned().unwrap_or_default(),
                                                            all_styles: post_clone.styles.clone(),
                                                            image_count: 0,
                                                            portfolio_images: vec![],
                                                            avatar_url: None,
                                                            avg_rating: 0.0,
                                                            match_score: 0,
                                                            years_experience: None,
                                                            min_price: None,
                                                            max_price: None,
                                                        };
                                                        callback.run(matched_artist);
                                                    }
                                                }
                                            });
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }>
                {
                    let (force_rerender, set_force_rerender) = signal(0);
                    
                    // Watch for page changes and trigger Instagram embed processing
                    Effect::new(move |_| {
                        let _ = current_page.get();
                        set_force_rerender.update(|x| *x += 1);
                        
                        // Trigger Instagram embed processing after a short delay to ensure DOM is updated
                        set_timeout(move || {
                            process_instagram_embeds();
                        }, std::time::Duration::from_millis(100));
                    });
                    
                    move || {
                        let page = current_page.get();
                        let posts = current_page_posts.get();
                        let render_key = force_rerender.get(); // Force reactivity
                        
                        // Use unique container ID to force DOM reconstruction
                        let container_id = format!("gallery-container-{}-{}", page, render_key);
                        
                        view! { 
                            <div id=container_id>
                                <InstagramPostsGrid 
                                    posts=posts 
                                    filter_id=format!("page-{}-{}", page, render_key)
                                />
                            </div>
                        }
                    }
                }
            </div>

            // Pagination controls
            {(total_pages > 1).then(|| {
                view! {
                    <div class="tattoo-gallery-pagination">
                        <button
                            class="tattoo-gallery-pagination-button"
                            disabled=move || prev_btn_disabled.get()
                            on:click={
                                let navigate = navigate.clone();
                                move |_| {
                                    if !prev_btn_disabled.get() {
                                        let new_page = current_page.get() - 1;
                                        set_current_page.set(new_page);

                                        // Update URL with new page number
                                        let current_query = query_map.get();
                                        let mut params = vec![];

                                        // Preserve existing query params
                                        if let Some(styles) = current_query.get("styles") {
                                            params.push(format!("styles={}", urlencoding::encode(&styles)));
                                        }
                                        if let Some(states) = current_query.get("states") {
                                            params.push(format!("states={}", urlencoding::encode(&states)));
                                        }
                                        if let Some(cities) = current_query.get("cities") {
                                            params.push(format!("cities={}", urlencoding::encode(&cities)));
                                        }

                                        // Add new page param
                                        params.push(format!("page={}", new_page));

                                        let new_url = format!("?{}", params.join("&"));
                                        navigate(&new_url, Default::default());

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
                            }
                        >
                            "← Previous"
                        </button>

                        <span class="tattoo-gallery-pagination-info">
                            {move || format!("Page {} of {}", current_page.get() + 1, total_pages)}
                        </span>

                        <button
                            class="tattoo-gallery-pagination-button"
                            disabled=move || next_btn_disabled.get()
                            on:click=move |_| {
                                if !next_btn_disabled.get() {
                                    let new_page = current_page.get() + 1;
                                    set_current_page.set(new_page);

                                    // Update URL with new page number
                                    let current_query = query_map.get();
                                    let mut params = vec![];

                                    // Preserve existing query params
                                    if let Some(styles) = current_query.get("styles") {
                                        params.push(format!("styles={}", urlencoding::encode(&styles)));
                                    }
                                    if let Some(states) = current_query.get("states") {
                                        params.push(format!("states={}", urlencoding::encode(&states)));
                                    }
                                    if let Some(cities) = current_query.get("cities") {
                                        params.push(format!("cities={}", urlencoding::encode(&cities)));
                                    }

                                    // Add new page param
                                    params.push(format!("page={}", new_page));

                                    let new_url = format!("?{}", params.join("&"));
                                    navigate(&new_url, Default::default());

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
            })}
        </div>
    }
}
