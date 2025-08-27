use leptos::prelude::*;
use thaw::*;

use crate::{
    components::instagram_embed::{InstagramEmbed, InstagramEmbedSize},
    server::{MatchedArtist, TattooPost},
};

#[component]
pub fn TattooGallery(
    posts: Vec<TattooPost>,
    on_artist_click: Callback<MatchedArtist>,
) -> impl IntoView {
    view! {
        <div class="tattoo-gallery">
            <div class="tattoo-grid">
                {posts.into_iter().map(|post| {
                    view! {
                        <TattooGalleryItem post=post on_artist_click=on_artist_click />
                    }
                }).collect_view()}
            </div>
        </div>
    }
}

#[component]
fn TattooGalleryItem(post: TattooPost, on_artist_click: Callback<MatchedArtist>) -> impl IntoView {
    let artist_name_for_click = post.artist_name.clone();
    let artist_id_for_click = post.artist_id;

    view! {
        <div class="tattoo-gallery-item">
            <div class="artist-header">
                <button
                    class="artist-name-button"
                    on:click=move |_| {
                        // Create a minimal MatchedArtist for the modal
                        let matched_artist = MatchedArtist {
                            id: artist_id_for_click,
                            name: artist_name_for_click.clone(),
                            location_name: "Loading...".to_string(), // Will be loaded in modal
                            city: "Loading...".to_string(),
                            state: "Loading...".to_string(),
                            primary_style: "Traditional".to_string(),
                            all_styles: vec![],
                            image_count: 0,
                            portfolio_images: vec![],
                            avatar_url: None,
                            avg_rating: 4.2,
                            match_score: 100,
                            years_experience: Some(0),
                            min_price: None,
                            max_price: None,
                        };
                        on_artist_click.run(matched_artist);
                    }
                >
                    {post.artist_name.clone()}
                </button>

                <div class="style-tags">
                    {post.styles.iter().take(2).map(|style| {
                        view! {
                            <span class="style-tag">{style.clone()}</span>
                        }
                    }).collect_view()}
                </div>
            </div>

            <div class="instagram-post">
                <InstagramEmbed
                    short_code=post.short_code
                    size=InstagramEmbedSize::Medium
                />
            </div>
        </div>
    }
}

