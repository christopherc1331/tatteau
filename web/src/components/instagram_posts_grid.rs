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
        <div class="instagram-posts-grid-container" data-filter-id={filter_id.clone()} id={format!("posts-grid-{}", filter_id.clone().unwrap_or_else(|| "default".to_string()))}>
            {posts.into_iter().map(|post| {
                let short_code = post.image.short_code.clone();

                view! {
                    <div class="instagram-posts-grid-post-container">
                        <div class="instagram-posts-grid-post-card">
                            {(!post.styles.is_empty() || post.artist.is_some()).then(|| {
                                view! {
                                    <div class="instagram-posts-grid-header">
                                        <div class="instagram-posts-grid-meta-wrapper">
                                            {post.artist.as_ref().map(|artist| {
                                                let artist_name = artist.name.clone().unwrap_or_else(|| "Unknown Artist".to_string());
                                                view! {
                                                    <a href={format!("/artist/{}", artist.id)}
                                                       class="instagram-posts-grid-artist-link">
                                                        {format!("👤 {}", artist_name)}
                                                    </a>
                                                }
                                            })}
                                            {post.styles.into_iter().map(|style| {
                                                view! {
                                                    <span class="instagram-posts-grid-style-tag">
                                                        {style.name}
                                                    </span>
                                                }
                                            }).collect_view()}
                                        </div>
                                    </div>
                                }
                            })}
                            <div class="instagram-posts-grid-embed-container">
                                <InstagramEmbed short_code={short_code} />
                            </div>

                        </div>
                    </div>
                }
            }).collect_view()}
        </div>
        

    }
}
