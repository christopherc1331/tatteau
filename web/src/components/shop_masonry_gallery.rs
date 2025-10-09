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
                {shop_posts.into_iter().map(|post| {
                    let short_code = post.image.short_code.clone();
                    let artist_name = post.artist.name.unwrap_or_else(|| "Unknown Artist".to_string());

                    view! {
                        <div class="shop-masonry-gallery__post">
                            <div class="shop-masonry-gallery__card">
                                <div class="shop-masonry-gallery__header">
                                    <a href={format!("/artist/{}", post.artist.id)}
                                       class="shop-masonry-gallery__artist-link">
                                        {artist_name}
                                    </a>

                                    {(!post.styles.is_empty()).then(|| {
                                        view! {
                                            <div class="shop-masonry-gallery__styles-wrapper">
                                                {post.styles.into_iter().map(|style| {
                                                    view! {
                                                        <span class="shop-masonry-gallery__style-tag">
                                                            {style.name}
                                                        </span>
                                                    }
                                                }).collect_view()}
                                            </div>
                                        }
                                    })}
                                </div>

                                <InstagramEmbed short_code={short_code} />
                            </div>
                        </div>
                    }
                }).collect_view()}
            </div>
        </div>
    }
}