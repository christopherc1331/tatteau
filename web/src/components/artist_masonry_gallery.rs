use crate::components::instagram_posts_grid::{InstagramPostsGrid, PostWithArtist};
use crate::components::instagram_embed::process_instagram_embeds;
use crate::db::entities::{ArtistImage, Style};
use leptos::prelude::*;

#[derive(Clone, Debug, PartialEq)]
pub struct InstagramPost {
    pub image: ArtistImage,
    pub styles: Vec<Style>,
}

#[component]
pub fn ArtistMasonryGallery(
    instagram_posts: Vec<InstagramPost>,
    artist_styles: Vec<Style>,
) -> impl IntoView {
    let (selected_style, set_selected_style) = signal::<Option<i32>>(None);
    
    // Trigger Instagram embed processing when filter changes
    Effect::new(move |_| {
        let _ = selected_style.get();
        // Small delay to ensure DOM is updated
        set_timeout(move || {
            process_instagram_embeds();
        }, std::time::Duration::from_millis(100));
    });

    let filtered_posts = Memo::new(move |_| {
        let posts = instagram_posts.clone();
        let current_filter = selected_style.get();

        let filtered = if let Some(style_id) = current_filter {
            let filtered_posts: Vec<_> = posts
                .into_iter()
                .filter(|post| post.styles.iter().any(|s| s.id == style_id))
                .collect();

            filtered_posts
        } else {
            posts
        };

        // Convert to PostWithArtist format
        filtered
            .into_iter()
            .map(|post| PostWithArtist {
                image: post.image,
                styles: post.styles,
                artist: None, // Artist page doesn't need artist info
            })
            .collect::<Vec<_>>()
    });

    view! {
        <div class="artist-masonry-gallery-container">
            <div class="artist-masonry-gallery-filter-container">
                <div class="artist-masonry-gallery-filter-wrapper">
                    <span class="artist-masonry-gallery-filter-label">"Filter by style:"</span>

                    <button
                        on:click=move |_| {
                            set_selected_style.set(None);
                        }
                        class=move || format!(
                            "artist-masonry-gallery-filter-button {}",
                            if selected_style.get().is_none() { "artist-masonry-gallery-filter-button--active" } else { "artist-masonry-gallery-filter-button--inactive" }
                        )
                    >
                        "All"
                    </button>

                    {artist_styles.into_iter().map(|style| {
                        let style_id = style.id;
                        let style_name = style.name.clone();
                        let style_name_for_display = style_name.clone();
                        let style_name_for_click = style_name.clone();
                        view! {
                            <button
                                on:click=move |_| {
                                    set_selected_style.set(Some(style_id));
                                }
                                class=move || format!(
                                    "artist-masonry-gallery-filter-button {}",
                                    if selected_style.get() == Some(style_id) { "artist-masonry-gallery-filter-button--active" } else { "artist-masonry-gallery-filter-button--inactive" }
                                )
                            >
                                {style_name_for_display}
                            </button>
                        }
                    }).collect_view()}
                </div>
            </div>

            {move || {
                let posts = filtered_posts.get();
                let filter_id = selected_style.get().map(|id| format!("artist-{}", id)).unwrap_or_else(|| "artist-all".to_string());
                
                view! {
                    <InstagramPostsGrid
                        posts=posts
                        filter_id=filter_id.clone()
                    />
                }
            }}
        </div>
    }
}

