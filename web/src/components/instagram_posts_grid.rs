use crate::components::favorite_button::FavoriteButton;
use crate::components::instagram_embed::InstagramEmbed;
use crate::components::style_tag::StyleTag;
use crate::components::style_tag_manager::StyleTagManager;
use crate::db::entities::{Artist, ArtistImage, Style};
use leptos::prelude::*;

#[derive(Clone, Debug, PartialEq)]
pub struct PostWithArtist {
    pub image: ArtistImage,
    pub styles: Vec<Style>,
    pub artist: Option<Artist>,
    pub is_favorited: bool,
}

#[component]
pub fn InstagramPostsGrid(
    posts: Vec<PostWithArtist>,
    #[prop(optional)] filter_id: Option<String>,
) -> impl IntoView {
    let posts_signal = RwSignal::new(posts);
    let grid_id = format!(
        "posts-grid-{}",
        filter_id.clone().unwrap_or_else(|| "default".to_string())
    );

    view! {
        <div class="instagram-posts-grid-container" data-filter-id={filter_id.clone()} id={grid_id}>
            <For
                each=move || posts_signal.get().into_iter().enumerate()
                key=|(idx, post)| (post.image.id, *idx)
                children=move |(idx, post)| {
                    let short_code = post.image.short_code.clone();
                    let image_id = post.image.id;
                    let is_favorited = post.is_favorited;
                    let artist_opt = post.artist.clone();

                    // Create a derived signal for this post's styles
                    let post_styles = Signal::derive(move || {
                        posts_signal.get()
                            .get(idx)
                            .map(|p| p.styles.clone())
                            .unwrap_or_default()
                    });

                    view! {
                        <div class="instagram-posts-grid-post-container">
                            <div class="instagram-posts-grid-post-card">
                                <div class="instagram-posts-grid-header">
                                    // Favorite button - positioned at top right
                                    <div class="instagram-posts-grid-favorite">
                                        <FavoriteButton artists_images_id=image_id is_favorited_initial=is_favorited />
                                    </div>

                                    // Content area
                                    <div class="instagram-posts-grid-content">
                                        {artist_opt.as_ref().map(|artist| {
                                            let artist_name = artist.name.clone().unwrap_or_else(|| "Unknown Artist".to_string());
                                            let artist_name_for_title = artist_name.clone();
                                            let artist_id = artist.id;
                                            view! {
                                                <a href={format!("/artist/{}", artist_id)}
                                                   class="instagram-posts-grid-artist-link"
                                                   title={format!("View {}'s profile and portfolio", artist_name_for_title)}>
                                                    <span class="instagram-posts-grid-artist-icon">"👤"</span>
                                                    <span class="instagram-posts-grid-artist-name">{artist_name}</span>
                                                    <span class="instagram-posts-grid-view-profile-hint">"→ View Profile"</span>
                                                </a>
                                            }
                                        })}

                                        <div class="instagram-posts-grid-style-tags">
                                            {move || {
                                                post_styles.get().into_iter().map(|style| {
                                                    view! {
                                                        <StyleTag name={style.name} />
                                                    }
                                                }).collect_view()
                                            }}
                                        </div>
                                    </div>

                                    // Admin style tag manager
                                    <StyleTagManager
                                        image_id=image_id as i64
                                        current_styles=post_styles
                                        on_styles_changed=Callback::new(move |new_styles: Vec<Style>| {
                                            // Update the post's styles in the signal
                                            posts_signal.update(|posts| {
                                                if let Some(post) = posts.get_mut(idx) {
                                                    post.styles = new_styles;
                                                }
                                            });
                                        })
                                    />
                                </div>
                                <div class="instagram-posts-grid-embed-container">
                                    <InstagramEmbed short_code={short_code} />
                                </div>

                            </div>
                        </div>
                    }
                }
            />
        </div>


    }
}
