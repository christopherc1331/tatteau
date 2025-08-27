use crate::db::entities::{Artist, ArtistImage, Style};
use crate::components::instagram_embed::InstagramEmbed;
use leptos::prelude::*;

#[derive(Clone, Debug, PartialEq)]
pub struct PostWithArtist {
    pub image: ArtistImage,
    pub styles: Vec<Style>,
    pub artist: Option<Artist>,
}

#[component]
pub fn InstagramPostsGrid(
    posts: Vec<PostWithArtist>,
    #[prop(optional)] filter_id: Option<String>,
) -> impl IntoView {
    
    view! {
        <style>
            {r#"
            .posts-grid {
                columns: 4;
                column-gap: 1rem;
                column-fill: balance;
                width: 100%;
            }
            @media (max-width: 1200px) {
                .posts-grid { columns: 3 !important; }
            }
            @media (max-width: 768px) {
                .posts-grid { columns: 2 !important; }
            }
            @media (max-width: 480px) {
                .posts-grid { columns: 1 !important; }
            }
            .instagram-media {
                max-width: 100% !important;
                min-width: 280px !important;
            }
            "#}
        </style>
        <div class="posts-grid" data-filter-id={filter_id.clone()} id={format!("posts-grid-{}", filter_id.clone().unwrap_or_else(|| "default".to_string()))}>
            {posts.into_iter().map(|post| {
                let short_code = post.image.short_code.clone();

                view! {
                    <div style="break-inside: avoid; margin-bottom: 1rem; position: relative;">
                        <div style="background: white; border-radius: 8px; overflow: hidden; box-shadow: 0 2px 8px rgba(0,0,0,0.1); position: relative;">
                            {(!post.styles.is_empty() || post.artist.is_some()).then(|| {
                                view! {
                                    <div style="padding: 0.5rem; background: white;">
                                        <div style="display: flex; flex-wrap: wrap; gap: 0.25rem; align-items: center;">
                                            {post.artist.as_ref().map(|artist| {
                                                let artist_name = artist.name.clone().unwrap_or_else(|| "Unknown Artist".to_string());
                                                view! {
                                                    <a href={format!("/artist/{}", artist.id)}
                                                       style="background: rgba(0,0,0,0.8); color: white; padding: 0.125rem 0.375rem; border-radius: 10px; font-size: 0.6rem; font-weight: 600; text-decoration: none; margin-right: 0.25rem;">
                                                        {format!("👤 {}", artist_name)}
                                                    </a>
                                                }
                                            })}
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
                            <div style="position: relative;">
                                <InstagramEmbed short_code={short_code} />
                            </div>

                        </div>
                    </div>
                }
            }).collect_view()}
        </div>
        

    }
}
