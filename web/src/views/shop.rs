use leptos::prelude::*;
use leptos_router::hooks::use_params_map;

use crate::{
    components::{
        loading::LoadingView,
        shop_masonry_gallery::{ShopInstagramPost, ShopMasonryGallery},
    },
    db::entities::{Artist, Style},
    server::fetch_shop_data,
};

#[component]
pub fn Shop() -> impl IntoView {
    let params = use_params_map();
    let shop_id = Memo::new(move |_| {
        params
            .read()
            .get("id")
            .and_then(|id| id.parse::<i32>().ok())
            .unwrap_or(0)
    });

    let shop_data = Resource::new(
        move || shop_id.get(),
        move |id| async move {
            if id > 0 {
                fetch_shop_data(id).await.ok()
            } else {
                None
            }
        },
    );

    view! {
        <div class="shop-container">
            <Suspense fallback=move || view! {
                <LoadingView message=Some("Loading shop information...".to_string()) />
            }>
                {move || {
                    shop_data.get().map(|data| {
                        data.map(|shop_data| {
                            let shop_name = shop_data.location.name.unwrap_or_else(|| "Unknown Shop".to_string());
                            let city = shop_data.location.city.unwrap_or_else(|| "Unknown".to_string());
                            let state = shop_data.location.state.unwrap_or_else(|| "Unknown".to_string());

                            let all_styles_for_filter = shop_data.all_styles.clone();

                            let shop_posts: Vec<ShopInstagramPost> = shop_data.all_images_with_styles
                                .clone()
                                .into_iter()
                                .map(|(image, styles, artist)| ShopInstagramPost {
                                    image,
                                    styles,
                                    artist,
                                })
                                .collect();

                            view! {
                                <div class="shop-container">
                                    <div class="shop-header">
                                        <div class="shop-header-content">
                                            <div class="shop-header-flex">
                                                <div>
                                                    <h1 class="shop-title">
                                                        {shop_name.clone()}
                                                    </h1>
                                                    <div class="shop-location">
                                                        {format!("{}, {}", city, state)}
                                                    </div>
                                                </div>

                                                <div class="shop-actions">
                                                    <a href={format!("/book/shop/{}", shop_id.get())}
                                                       class="shop-action-button shop-action-book">
                                                        "📅 Book Appointment"
                                                    </a>
                                                    {shop_data.location.website_uri.as_ref().filter(|uri| !uri.is_empty()).map(|website_uri| {
                                                        view! {
                                                            <a href=website_uri.clone() target="_blank"
                                                               class="shop-action-button shop-action-website">
                                                                "🌐 Visit Website"
                                                            </a>
                                                        }
                                                    })}
                                                </div>
                                            </div>
                                        </div>
                                    </div>

                                    <div class="shop-main-content">
                                        <div class="shop-info-grid">
                                            // Left side: Artists and Styles combined
                                            <div class="shop-info-card">
                                                {(!shop_data.artists.is_empty()).then(|| {
                                                    view! {
                                                        <div class="shop-artists-section">
                                                            <h3 class="shop-section-title">"Our Artists"</h3>
                                                            <div class="shop-artists-list">
                                                                {shop_data.artists.into_iter().map(|artist| {
                                                                    let artist_name = artist.name.unwrap_or_else(|| "Unknown Artist".to_string());
                                                                    view! {
                                                                        <div class="shop-artist-item">
                                                                            <div>
                                                                                <div class="shop-artist-name">
                                                                                    {artist_name}
                                                                                </div>
                                                                                {artist.years_experience.and_then(|years| {
                                                                                    (years > 0).then(|| view! {
                                                                                        <div class="shop-artist-experience">
                                                                                            {format!("{} years experience", years)}
                                                                                        </div>
                                                                                    })
                                                                                })}
                                                                            </div>
                                                                            <a href={format!("/artist/{}", artist.id)}
                                                                               class="shop-artist-profile-link">
                                                                                "View Profile"
                                                                            </a>
                                                                        </div>
                                                                    }
                                                                }).collect_view()}
                                                            </div>
                                                        </div>
                                                    }
                                                })}

                                                {(!shop_data.all_styles.is_empty()).then(|| {
                                                    view! {
                                                        <div>
                                                            <h3 class="shop-section-title">"Styles We Do"</h3>
                                                            <div class="shop-styles-list">
                                                                {shop_data.all_styles.into_iter().map(|style| {
                                                                    view! {
                                                                        <span class="shop-style-tag">
                                                                            {style.name}
                                                                        </span>
                                                                    }
                                                                }).collect_view()}
                                                            </div>
                                                        </div>
                                                    }
                                                })}
                                            </div>

                                            // Right side: Map section
                                            {shop_data.location.address.clone().map(|addr| {
                                            let lat = shop_data.location.lat.unwrap_or(0.0);
                                            let long = shop_data.location.long.unwrap_or(0.0);
                                            let encoded_addr = urlencoding::encode(&addr);

                                            view! {
                                                <div class="shop-info-card">
                                                    <h3 class="shop-section-title">"📍 Shop Location"</h3>
                                                    <div style="margin-bottom: 1.5rem;">
                                                        <p class="shop-location-text">
                                                            {addr.clone()}
                                                        </p>
                                                        <a href={format!("https://www.google.com/maps/dir/?api=1&destination={}",
                                                            shop_data.location.address.as_ref().unwrap_or(&"".to_string()))}
                                                           target="_blank"
                                                           class="shop-directions-link">
                                                            <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
                                                                <path d="M12,2A10,10 0 0,0 2,12A10,10 0 0,0 12,22A10,10 0 0,0 22,12A10,10 0 0,0 12,2M12,4A8,8 0 0,1 20,12A8,8 0 0,1 12,20A8,8 0 0,1 4,12A8,8 0 0,1 12,4M16.24,7.76L15.12,6.64L8.76,13L5.64,9.88L4.52,11L8.76,15.24L16.24,7.76Z"/>
                                                                <path d="M2.5,19H21.5V21H2.5V19M22.07,9.64C21.86,8.84 21.03,8.36 20.23,8.58L14.92,10L8,3.57L6.09,4.08L10.23,11.25L5.26,12.58L3.29,11.04L1.84,11.43L3.66,14.59L4.43,15.92L6.03,15.5L11.34,14.07L15.69,12.91L21,11.5C21.81,11.26 22.28,10.44 22.07,9.64Z"/>
                                                            </svg>
                                                            "Get Directions"
                                                        </a>
                                                    </div>

                                                    <div class="shop-map-container">
                                                        <iframe
                                                            src={format!("https://www.openstreetmap.org/export/embed.html?bbox={},{},{},{}&layer=mapnik&marker={},{}",
                                                                long - 0.01, lat - 0.01, long + 0.01, lat + 0.01, lat, long)}
                                                            class="shop-map-iframe"
                                                            title="Shop Location Map"
                                                        ></iframe>
                                                        <div class="shop-map-overlay"></div>
                                                    </div>
                                                </div>
                                            }
                                        })}
                                        </div>

                                        {(!shop_posts.is_empty()).then(|| {
                                            view! {
                                                <div class="shop-info-card">
                                                    <h2 class="shop-portfolio-title">"Shop Portfolio"</h2>
                                                    <ShopMasonryGallery shop_posts=shop_posts all_styles=all_styles_for_filter />
                                                </div>
                                            }
                                        })}
                                    </div>
                                </div>
                            }.into_any()
                        }).unwrap_or_else(|| {
                            view! {
                                <div class="shop-container">
                                    <div class="shop-error-header">
                                        <div class="shop-error-header-content">
                                            <h1 class="shop-error-title">
                                                "🏪 Shop Not Found"
                                            </h1>
                                            <div class="shop-error-subtitle">
                                                "The requested shop could not be found"
                                            </div>
                                        </div>
                                    </div>

                                    <div class="shop-error-content">
                                        <div class="shop-error-card">
                                            <p class="shop-error-text">
                                                "Please check the shop ID and try again."
                                            </p>
                                        </div>
                                    </div>
                                </div>
                            }.into_any()
                        })
                    })
                }}
            </Suspense>
        </div>
    }
}

