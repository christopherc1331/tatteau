use leptos::prelude::*;
use leptos_router::components::A;

#[component]
pub fn ArtistCTA(artist_id: i32, #[prop(optional)] class: &'static str) -> impl IntoView {
    view! {
        <div class={format!("artist-cta {}", class)}>
            <A href=format!("/artist/{}", artist_id)
               attr:class="artist-cta__button artist-cta__view-profile">
                "👤 View Profile"
            </A>
            <A href=format!("/book/artist/{}", artist_id)
               attr:class="artist-cta__button artist-cta__book-now">
                "📅 Book Now"
            </A>
        </div>
    }
}
