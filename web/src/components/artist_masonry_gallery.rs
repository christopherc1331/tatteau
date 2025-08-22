use leptos::prelude::*;
use crate::db::entities::{ArtistImage, Style};

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
    
    let filtered_posts = Memo::new(move |_| {
        let posts = instagram_posts.clone();
        if let Some(style_id) = selected_style.get() {
            posts.into_iter()
                .filter(|post| post.styles.iter().any(|s| s.id == style_id))
                .collect::<Vec<_>>()
        } else {
            posts
        }
    });

    view! {
        <div>
            <div style="margin-bottom: 1.5rem;">
                <div style="display: flex; flex-wrap: wrap; gap: 0.5rem; align-items: center;">
                    <span style="font-weight: 600; color: #4a5568; margin-right: 0.5rem;">"Filter by style:"</span>
                    
                    <button 
                        on:click=move |_| set_selected_style.set(None)
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
                        view! {
                            <button 
                                on:click=move |_| set_selected_style.set(Some(style_id))
                                style=move || format!(
                                    "background: {}; color: {}; padding: 0.25rem 0.75rem; border: 1px solid #d1d5db; border-radius: 20px; font-size: 0.8rem; cursor: pointer;",
                                    if selected_style.get() == Some(style_id) { "#667eea" } else { "white" },
                                    if selected_style.get() == Some(style_id) { "white" } else { "#374151" }
                                )
                            >
                                {style_name}
                            </button>
                        }
                    }).collect_view()}
                </div>
            </div>
            
            <style>
                {r#"
                .artist-masonry {
                    columns: 4;
                    column-gap: 1rem;
                    column-fill: balance;
                    width: 100%;
                }
                @media (max-width: 1200px) {
                    .artist-masonry { columns: 3 !important; }
                }
                @media (max-width: 768px) {
                    .artist-masonry { columns: 2 !important; }
                }
                @media (max-width: 480px) {
                    .artist-masonry { columns: 1 !important; }
                }
                .instagram-media {
                    max-width: 100% !important;
                    min-width: 280px !important;
                }
                "#}
            </style>
            <div class="artist-masonry">
                {move || {
                    filtered_posts.get().into_iter().map(|post| {
                        let short_code = post.image.short_code.clone();
                        
                        view! {
                            <div style="break-inside: avoid; margin-bottom: 1rem; position: relative;">
                                <div style="background: white; border-radius: 8px; overflow: hidden; box-shadow: 0 2px 8px rgba(0,0,0,0.1); position: relative;">
                                    <div 
                                        inner_html={format!(
                                            r#"<blockquote class="instagram-media" data-instgrm-captioned data-instgrm-permalink="https://www.instagram.com/p/{}/" data-instgrm-version="14"></blockquote>"#, 
                                            short_code
                                        )}
                                    ></div>
                                    
                                    {(!post.styles.is_empty()).then(|| {
                                        view! {
                                            <div style="position: absolute; bottom: 0.5rem; left: 0.5rem; right: 0.5rem;">
                                                <div style="display: flex; flex-wrap: wrap; gap: 0.25rem;">
                                                    {post.styles.into_iter().map(|style| {
                                                        view! {
                                                            <span style="background: rgba(102, 126, 234, 0.9); color: white; padding: 0.125rem 0.375rem; border-radius: 10px; font-size: 0.6rem; font-weight: 500;">
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
                    }).collect_view()
                }}
            </div>
        </div>
    }
}