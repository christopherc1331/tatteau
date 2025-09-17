use leptos::prelude::*;
use shared_types::LocationInfo;
use thaw::{Label, LabelSize};

#[component]
pub fn MapMarkerPopup(location: LocationInfo) -> impl IntoView {
    view! {
        <div class="map-marker-popup-container">
            <Label size=LabelSize::Large>{location.name.clone()}</Label>
            <p class="map-marker-popup-address">
                {format!("Address: {}", location.address)}
            </p>

            {if !location.has_artists.unwrap_or(false) {
                view! {
                    <a href=location.website_uri target="_blank"
                       class="map-marker-popup-link">
                        "Visit Website"
                    </a>
                }.into_any()
            } else if location.artist_images_count.unwrap_or(0) == 0 {
                view! {
                    <a href={format!("/shop/{}", location.id)}
                       class="map-marker-popup-link">
                        "View Shop Info"
                    </a>
                }.into_any()
            } else {
                view! {
                    <a href={format!("/shop/{}", location.id)}
                       class="map-marker-popup-link">
                        "View Shop Portfolio"
                    </a>
                }.into_any()
            }}
        </div>
    }
}

