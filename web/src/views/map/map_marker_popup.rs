use leptos::prelude::*;
use shared_types::LocationInfo;
use thaw::{Label, LabelSize};

#[component]
pub fn MapMarkerPopup(location: LocationInfo) -> impl IntoView {
    view! {
        <div style="margin: 0.5rem 0; display: flex; flex-direction: column; gap: 0.5rem;">
            <Label size=LabelSize::Large>{location.name.clone()}</Label>
            <p style="margin: 0; color: #6b7280; font-size: 0.875rem;">
                {format!("Address: {}", location.address)}
            </p>

            {if location.has_artists.unwrap_or(false) == false {
                view! {
                    <a href=location.website_uri target="_blank"
                       style="background: #667eea; color: white; padding: 0.5rem 1rem; border-radius: 6px; text-decoration: none; text-align: center; font-weight: 600;">
                        "Visit Website"
                    </a>
                }.into_any()
            } else if location.artist_images_count.unwrap_or(0) == 0 {
                view! {
                    <a href={format!("/shop/{}", location.id)}
                       style="background: #667eea; color: white; padding: 0.5rem 1rem; border-radius: 6px; text-decoration: none; text-align: center; font-weight: 600;">
                        "View Shop Info"
                    </a>
                }.into_any()
            } else {
                view! {
                    <a href={format!("/shop/{}", location.id)}
                       style="background: #667eea; color: white; padding: 0.5rem 1rem; border-radius: 6px; text-decoration: none; text-align: center; font-weight: 600;">
                        "View Shop Portfolio"
                    </a>
                }.into_any()
            }}
        </div>
    }
}

