use crate::views::map::map_marker_popup::MapMarkerPopup;
use leptos::prelude::*;
use leptos_leaflet::prelude::*;
use shared_types::LocationInfo;

#[component]
pub fn MapMarker(location: LocationInfo) -> impl IntoView {
    let fill_color = if location.has_artists.unwrap_or(false) == false {
        "%236b7280"
    } else if location.artist_images_count.unwrap_or(0) == 0 {
        "%23f97316"
    } else {
        "%235b21b6"
    };

    let icon_svg = format!(
        "data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='28' height='42' viewBox='0 0 28 42'%3E%3Cdefs%3E%3Cfilter id='shadow' x='-50%25' y='-50%25' width='200%25' height='200%25'%3E%3CfeDropShadow dx='0' dy='1' stdDeviation='1.5' flood-color='%23000' flood-opacity='0.25'/%3E%3C/filter%3E%3C/defs%3E%3Cpath fill='{}' stroke='%23ffffff' stroke-width='1.5' filter='url(%23shadow)' d='M14 2C8.5 2 4 6.5 4 12c0 8.5 10 26 10 26s10-17.5 10-26c0-5.5-4.5-10-10-10zm0 13.5c-1.9 0-3.5-1.6-3.5-3.5s1.6-3.5 3.5-3.5 3.5 1.6 3.5 3.5-1.6 3.5-3.5 3.5z'/%3E%3C/svg%3E",
        fill_color
    );

    view! {
        <Marker
            position=Position::new(location.lat, location.long)
            draggable=false
            icon_url=Some(icon_svg)
            icon_size=Some((28.0, 42.0))
            icon_anchor=Some((14.0, 42.0))
        >
            <Popup>
                <MapMarkerPopup location=location />
            </Popup>
        </Marker>
    }
}
