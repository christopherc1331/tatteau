use crate::components::instagram_posts_grid::{InstagramPostsGrid, PostWithArtist};
use crate::components::instagram_embed::process_instagram_embeds;
use crate::db::entities::{ArtistImage, Style};
use leptos::prelude::*;

#[derive(Clone, Debug, PartialEq)]
pub struct InstagramPost {
    pub image: ArtistImage,
    pub styles: Vec<Style>,
    pub is_favorited: bool,
}

#[component]
pub fn ArtistMasonryGallery(
    instagram_posts: Vec<InstagramPost>,
    artist_styles: Vec<Style>,
) -> impl IntoView {
    // Trigger Instagram embed processing when component mounts
    Effect::new(move |_| {
        set_timeout(move || {
            process_instagram_embeds();
        }, std::time::Duration::from_millis(100));
    });

    // Convert to PostWithArtist format
    let posts: Vec<PostWithArtist> = instagram_posts
        .into_iter()
        .map(|post| PostWithArtist {
            image: post.image,
            styles: post.styles,
            artist: None, // Artist page doesn't need artist info
            is_favorited: post.is_favorited,
        })
        .collect();

    view! {
        <div class="artist-masonry-gallery-container">
            <InstagramPostsGrid posts=posts />
        </div>
    }
}

