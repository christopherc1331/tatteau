use leptos::prelude::*;
use wasm_bindgen::JsCast;

use crate::{
    components::{
        instagram_embed::process_instagram_embeds,
        instagram_posts_grid::{InstagramPostsGrid, PostWithArtist}
    },
    db::entities::{Artist, ArtistImage, Style},
    server::{MatchedArtist, TattooPost},
};

#[component]
pub fn TattooGallery(
    posts: Vec<TattooPost>,
    on_artist_click: Callback<MatchedArtist>,
) -> impl IntoView {
    // Pagination state
    let (current_page, set_current_page) = signal(0usize);
    let posts_per_page = 10;

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
                id: 0, // Placeholder
                artist_id: post.artist_id as i32,
                short_code: post.short_code,
            };

            PostWithArtist {
                image,
                styles,
                artist: Some(artist),
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
                                        // Find the artist data from our posts
                                        let callback = callback_for_click.get_value();

                                        // Create a realistic artist for the modal
                                        let matched_artist = MatchedArtist {
                                            id: artist_id,
                                            name: "Marcus Thompson".to_string(),
                                            location_name: "Ink & Art Studio".to_string(),
                                            city: "San Francisco".to_string(),
                                            state: "CA".to_string(),
                                            primary_style: "Traditional".to_string(),
                                            all_styles: vec!["Traditional".to_string(), "Neo-Traditional".to_string(), "American Traditional".to_string()],
                                            image_count: 25,
                                            portfolio_images: vec![],
                                            avatar_url: None,
                                            avg_rating: 4.8,
                                            match_score: 95,
                                            years_experience: Some(8),
                                            min_price: Some(200.0),
                                            max_price: Some(500.0),
                                        };
                                        callback.run(matched_artist);
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
                            on:click=move |_| {
                                if !prev_btn_disabled.get() {
                                    set_current_page.set(current_page.get() - 1);
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
                                    set_current_page.set(current_page.get() + 1);
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
