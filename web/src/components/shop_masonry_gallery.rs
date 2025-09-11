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
    let (selected_style, set_selected_style) = signal::<Option<i32>>(None);


    view! {
        <div class="shop-masonry-gallery__container">
            <div class="shop-masonry-gallery__filter-container">
                <div class="shop-masonry-gallery__filter-wrapper">
                    <span class="shop-masonry-gallery__filter-label">"Filter by style:"</span>
                    
                    <button 
                        on:click=move |_| {
                            set_selected_style.set(None);
                        }
                        class:shop-masonry-gallery__filter-button--active=move || selected_style.get().is_none()
                        class:shop-masonry-gallery__filter-button--inactive=move || selected_style.get().is_some()
                        class="shop-masonry-gallery__filter-button"
                    >
                        "All"
                    </button>
                    
                    {all_styles.into_iter().map(|style| {
                        let style_id = style.id;
                        let style_name = style.name.clone();
                        view! {
                            <button 
                                on:click=move |_| {
                                    set_selected_style.set(Some(style_id));
                                }
                                class:shop-masonry-gallery__filter-button--active=move || selected_style.get() == Some(style_id)
                                class:shop-masonry-gallery__filter-button--inactive=move || selected_style.get() != Some(style_id)
                                class="shop-masonry-gallery__filter-button"
                            >
                                {style_name}
                            </button>
                        }
                    }).collect_view()}
                </div>
            </div>
            
            <div class="shop-masonry-gallery__masonry">
                {shop_posts.into_iter().map(|post| {
                    let short_code = post.image.short_code.clone();
                    let artist_name = post.artist.name.unwrap_or_else(|| "Unknown Artist".to_string());
                    let post_id = post.image.id;
                    let post_styles = post.styles.clone();
                    
                    view! {
                        <div 
                            class:shop-masonry-gallery__post--hidden=move || {
                                if let Some(style_id) = selected_style.get() {
                                    !post_styles.iter().any(|s| s.id == style_id)
                                } else {
                                    false
                                }
                            }
                            class="shop-masonry-gallery__post"
                        >
                            <div class="shop-masonry-gallery__card">
                                <div class="shop-masonry-gallery__artist-header">
                                    <a href={format!("/artist/{}", post.artist.id)} 
                                       class="shop-masonry-gallery__artist-link">
                                        <span>"👤"</span>
                                        <span>{artist_name}</span>
                                    </a>
                                </div>
                                
                                <InstagramEmbed short_code={short_code} />
                                
                                {(!post.styles.is_empty()).then(|| {
                                    view! {
                                        <div class="shop-masonry-gallery__styles-container">
                                            <div class="shop-masonry-gallery__styles-wrapper">
                                                {post.styles.into_iter().map(|style| {
                                                    view! {
                                                        <span class="shop-masonry-gallery__style-tag">
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
                }).collect_view()}
            </div>
        </div>
    }
}