use leptos::prelude::*;
use crate::db::entities::{ArtistImage, Style};
use crate::components::instagram_posts_grid::{InstagramPostsGrid, PostWithArtist};

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
    let (show_grid, set_show_grid) = signal(true);
    
    let filtered_posts = Memo::new(move |_| {
        let posts = instagram_posts.clone();
        let current_filter = selected_style.get();
        
        let filtered = if let Some(style_id) = current_filter {
            let filtered_posts: Vec<_> = posts.into_iter()
                .filter(|post| post.styles.iter().any(|s| s.id == style_id))
                .collect();
            
            // Debug logging
            leptos::logging::log!("Filtering by style ID: {}, found {} posts", style_id, filtered_posts.len());
            filtered_posts
        } else {
            leptos::logging::log!("No filter applied, showing all {} posts", posts.len());
            posts
        };
        
        // Convert to PostWithArtist format
        filtered.into_iter().map(|post| PostWithArtist {
            image: post.image,
            styles: post.styles,
            artist: None, // Artist page doesn't need artist info
        }).collect::<Vec<_>>()
    });


    view! {
        <div>
            <div style="margin-bottom: 1.5rem;">
                <div style="display: flex; flex-wrap: wrap; gap: 0.5rem; align-items: center;">
                    <span style="font-weight: 600; color: #4a5568; margin-right: 0.5rem;">"Filter by style:"</span>
                    
                    <button 
                        on:click=move |_| {
                            leptos::logging::log!("All button clicked, clearing filter");
                            set_show_grid.set(false);
                            set_selected_style.set(None);
                            // Re-show grid after a brief delay
                            set_timeout(move || set_show_grid.set(true), std::time::Duration::from_millis(50));
                        }
                        style=move || format!(
                            "background: {}; color: {}; padding: 0.25rem 0.75rem; border: 1px solid #d1d5db; border-radius: 20px; font-size: 0.8rem; cursor: pointer;",
                            if selected_style.get().is_none() { "#667eea" } else { "white" },
                            if selected_style.get().is_none() { "white" } else { "#374151" }
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
                                    leptos::logging::log!("Button clicked for style: {} (ID: {})", style_name_for_click, style_id);
                                    set_show_grid.set(false);
                                    set_selected_style.set(Some(style_id));
                                    // Re-show grid after a brief delay
                                    set_timeout(move || set_show_grid.set(true), std::time::Duration::from_millis(50));
                                }
                                style=move || format!(
                                    "background: {}; color: {}; padding: 0.25rem 0.75rem; border: 1px solid #d1d5db; border-radius: 20px; font-size: 0.8rem; cursor: pointer;",
                                    if selected_style.get() == Some(style_id) { "#667eea" } else { "white" },
                                    if selected_style.get() == Some(style_id) { "white" } else { "#374151" }
                                )
                            >
                                {style_name_for_display}
                            </button>
                        }
                    }).collect_view()}
                </div>
            </div>
            
            {move || {
                if show_grid.get() {
                    let posts = filtered_posts.get();
                    let filter_id = selected_style.get().map(|id| format!("artist-{}", id)).unwrap_or_else(|| "artist-all".to_string());
                    
                    view! {
                        <InstagramPostsGrid 
                            posts=posts
                            filter_id=filter_id
                        />
                    }.into_any()
                } else {
                    view! {
                        <div style="height: 200px; display: flex; align-items: center; justify-content: center;">
                            <span style="color: #999;">"Loading..."</span>
                        </div>
                    }.into_any()
                }
            }}
        </div>
    }
}