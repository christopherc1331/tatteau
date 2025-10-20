use leptos::prelude::*;
use crate::db::entities::{ArtistImage, Style, Artist};
use crate::components::instagram_embed::InstagramEmbed;
use crate::components::favorite_button::FavoriteButton;
use crate::components::style_tag_manager::StyleTagManager;

#[derive(Clone, Debug, PartialEq)]
pub struct ShopInstagramPost {
    pub image: ArtistImage,
    pub styles: Vec<Style>,
    pub artist: Artist,
    pub is_favorited: bool,
}

#[component]
pub fn ShopMasonryGallery(
    shop_posts: Vec<ShopInstagramPost>,
    all_styles: Vec<Style>,
) -> impl IntoView {
    // Store posts in a signal so they can be updated when styles change
    let posts_signal = RwSignal::new(shop_posts);
    // Process all Instagram embeds after component mounts
    Effect::new(move |_| {
        // Wait for all embeds to render, then process them all at once
        set_timeout(
            move || {
                let _ = web_sys::js_sys::eval(
                    r#"
                    if (window.instgrm && window.instgrm.Embeds) {
                        window.instgrm.Embeds.process();
                    }
                    "#
                );
            },
            std::time::Duration::from_millis(500),
        );
    });

    view! {
        <div class="shop-masonry-gallery__container">
            <div class="shop-masonry-gallery__masonry">
                <For
                    each=move || posts_signal.get().into_iter().enumerate()
                    key=|(idx, post)| (post.image.id, *idx)
                    children=move |(idx, post)| {
                        let short_code = post.image.short_code.clone();
                        let artist_name = post.artist.name.clone().unwrap_or_else(|| "Unknown Artist".to_string());
                        let image_id = post.image.id;
                        let is_favorited = post.is_favorited;
                        let artist_id = post.artist.id;

                        // Create a derived signal for this post's styles
                        let post_styles = Signal::derive(move || {
                            posts_signal.get()
                                .get(idx)
                                .map(|p| p.styles.clone())
                                .unwrap_or_default()
                        });

                        view! {
                            <div class="shop-masonry-gallery__post">
                                <div class="shop-masonry-gallery__card">
                                    <div class="shop-masonry-gallery__header">
                                        // Favorite button - positioned at top right
                                        <div class="shop-masonry-gallery__favorite">
                                            <FavoriteButton artists_images_id=image_id is_favorited_initial=is_favorited />
                                        </div>

                                        // Content area
                                        <div class="shop-masonry-gallery__content">
                                            <a href={format!("/artist/{}", artist_id)}
                                               class="shop-masonry-gallery__artist-link">
                                                {artist_name}
                                            </a>

                                            <div class="shop-masonry-gallery__style-tags">
                                                {move || {
                                                    post_styles.get().into_iter().map(|style| {
                                                        view! {
                                                            <span class="shop-masonry-gallery__style-tag">
                                                                {style.name}
                                                            </span>
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

                                    <InstagramEmbed short_code={short_code} />
                                </div>
                            </div>
                        }
                    }
                />
            </div>
        </div>
    }
}