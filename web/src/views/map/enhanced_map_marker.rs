use crate::server::EnhancedLocationInfo;
use leptos::prelude::*;
use leptos_leaflet::prelude::*;

#[component]
pub fn EnhancedMapMarker(
    location: EnhancedLocationInfo,
    #[prop(default = None)] selected_styles: Option<Vec<i32>>,
) -> impl IntoView {
    let fill_color = if location.location.has_artists.unwrap_or(false) == false {
        "%236b7280"
    } else if location.image_count == 0 {
        "%23f97316"
    } else {
        "%235b21b6"
    };

    // Create SVG with badge if there are artists
    let icon_svg = if location.artist_count > 0 {
        // Include the count badge in the SVG
        let badge_y = 5;
        let count_text = if location.artist_count > 99 {
            "99+".to_string()
        } else {
            location.artist_count.to_string()
        };
        
        format!(
            "data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='40' height='56' viewBox='0 0 40 56'%3E%3Cdefs%3E%3Cfilter id='shadow' x='-50%25' y='-50%25' width='200%25' height='200%25'%3E%3CfeDropShadow dx='0' dy='2' stdDeviation='2' flood-color='%23000' flood-opacity='0.3'/%3E%3C/filter%3E%3C/defs%3E%3Cpath fill='{}' stroke='%23ffffff' stroke-width='2' filter='url(%23shadow)' d='M20 10C13.5 10 8 15.5 8 22c0 10.5 12 30 12 30s12-19.5 12-30c0-6.5-5.5-12-12-12zm0 16c-2.2 0-4-1.8-4-4s1.8-4 4-4 4 1.8 4 4-1.8 4-4 4z'/%3E%3Ccircle cx='30' cy='{}' r='10' fill='%23ffffff' stroke='%236b7280' stroke-width='1'/%3E%3Ctext x='30' y='{}' text-anchor='middle' font-family='Arial, sans-serif' font-size='11' font-weight='bold' fill='%23111827'%3E{}%3C/text%3E%3C/svg%3E",
            fill_color,
            badge_y + 5,  // Circle y position
            badge_y + 9,  // Text y position (slightly lower for vertical centering)
            count_text
        )
    } else {
        // No badge for locations without artists
        format!(
            "data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='32' height='48' viewBox='0 0 32 48'%3E%3Cdefs%3E%3Cfilter id='shadow' x='-50%25' y='-50%25' width='200%25' height='200%25'%3E%3CfeDropShadow dx='0' dy='2' stdDeviation='2' flood-color='%23000' flood-opacity='0.3'/%3E%3C/filter%3E%3C/defs%3E%3Cpath fill='{}' stroke='%23ffffff' stroke-width='2' filter='url(%23shadow)' d='M16 2C9.5 2 4 7.5 4 14c0 10.5 12 30 12 30s12-19.5 12-30c0-6.5-5.5-12-12-12zm0 16c-2.2 0-4-1.8-4-4s1.8-4 4-4 4 1.8 4 4-1.8 4-4 4z'/%3E%3C/svg%3E",
            fill_color
        )
    };

    let icon_size = if location.artist_count > 0 {
        (40.0, 56.0)
    } else {
        (32.0, 48.0)
    };
    
    let icon_anchor = if location.artist_count > 0 {
        (20.0, 52.0)
    } else {
        (16.0, 48.0)
    };

    view! {
        <Marker
            position=Position::new(location.location.lat, location.location.long)
            draggable=false
            icon_url=Some(icon_svg)
            icon_size=Some(icon_size)
            icon_anchor=Some(icon_anchor)
        >
            <Popup>
                <EnhancedMapPopup location=location selected_styles=selected_styles />
            </Popup>
        </Marker>
    }
}

#[component]
pub fn EnhancedMapPopup(
    location: EnhancedLocationInfo,
    #[prop(default = None)] selected_styles: Option<Vec<i32>>,
) -> impl IntoView {
    // Artist details are now included in the location prop, no need to fetch

    view! {
        <div class="location-popup">
            <div class="popup-header">
                <h3>{location.location.name.clone()}</h3>
                <p class="popup-address">{location.location.address}</p>
            </div>
            
            <div class="popup-stats">
                <div class="stat">
                    <span class="stat-value">{location.artist_count}</span>
                    <span>" artists"</span>
                </div>
                {if location.image_count > 0 {
                    view! {
                        <div class="stat">
                            <span class="stat-value">{location.image_count}</span>
                            <span>" images"</span>
                        </div>
                    }.into_any()
                } else {
                    view! {}.into_any()
                }}

                // Show price range if available
                {if location.min_price.is_some() || location.max_price.is_some() {
                    view! {
                        <div class="stat price-stat">
                            <span class="stat-label">"Price: "</span>
                            <span class="stat-value">
                                {match (location.min_price, location.max_price) {
                                    (Some(min), Some(max)) => format!("${}-${}", min as i32, max as i32),
                                    (Some(min), None) => format!("${min}+", min = min as i32),
                                    (None, Some(max)) => format!("Up to ${max}", max = max as i32),
                                    (None, None) => "Contact for pricing".to_string(),
                                }}
                            </span>
                        </div>
                    }.into_any()
                } else {
                    view! {}.into_any()
                }}
            </div>
            
            // Artist thumbnails section
            {if !location.artists.is_empty() {
                view! {
                    <div class="popup-artists">
                        <h4>"Featured Artists"</h4>
                        <div class="artist-list">
                            {location.artists.into_iter().take(4).map(|artist| {
                                view! {
                                    <div class="artist-item">
                                        <div class="artist-initial">
                                            <span>{artist.artist_name.chars().next().unwrap_or('A')}</span>
                                        </div>
                                        <div class="artist-info">
                                            <span class="artist-name">{artist.artist_name}</span>
                                            {if let Some(style) = artist.primary_style {
                                                view! { <span class="artist-style">{style}</span> }.into_any()
                                            } else {
                                                view! { <span class="artist-style">"Tattoo Artist"</span> }.into_any()
                                            }}
                                        </div>
                                    </div>
                                }
                            }).collect_view()}
                        </div>
                    </div>
                }.into_any()
            } else {
                view! {}.into_any()
            }}
            
            {if !location.styles.is_empty() {
                view! {
                    <div class="popup-styles">
                        {location.styles.into_iter().take(5).map(|style| {
                            view! {
                                <span class="style-tag">{style}</span>
                            }
                        }).collect_view()}
                    </div>
                }.into_any()
            } else {
                view! {}.into_any()
            }}

            <a
                class="popup-cta"
                href={
                    let base_url = format!("/shop/{}", location.location.id);
                    if let Some(styles) = selected_styles.as_ref() {
                        if !styles.is_empty() {
                            let styles_str = styles.iter()
                                .map(|id| id.to_string())
                                .collect::<Vec<_>>()
                                .join(",");
                            format!("{}?styles={}", base_url, styles_str)
                        } else {
                            base_url
                        }
                    } else {
                        base_url
                    }
                }
            >
                {if location.image_count > 0 {
                    "View Portfolio & Artists"
                } else if location.artist_count > 0 {
                    "View Shop Details"
                } else {
                    "Visit Website"
                }}
            </a>
        </div>
    }
}