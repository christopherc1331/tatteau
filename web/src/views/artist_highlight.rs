use leptos::prelude::*;
use leptos_router::hooks::{use_params_map, use_query_map, use_navigate};

use crate::{
    components::{loading::LoadingView, artist_masonry_gallery::{ArtistMasonryGallery, InstagramPost}, ClientBookingModal},
    server::fetch_artist_data,
    utils::auth::is_authenticated,
};

#[component]
pub fn ArtistHighlight() -> impl IntoView {
    let params = use_params_map();
    let query = use_query_map();
    let navigate = use_navigate();
    
    let artist_id = Memo::new(move |_| {
        params.read()
            .get("id")
            .and_then(|id| id.parse::<i32>().ok())
            .unwrap_or(0)
    });

    let artist_data = Resource::new(
        move || artist_id.get(),
        move |id| async move {
            if id != 0 {
                fetch_artist_data(id).await.ok()
            } else {
                None
            }
        },
    );

    // Modal state for booking
    let show_booking_modal = RwSignal::new(false);
    let booking_artist_id = RwSignal::new(None::<i32>);
    
    // Check if we should open modal immediately (from /book/artist redirect)
    let navigate_clone = navigate.clone();
    Effect::new(move |_| {
        let query_params = query.read();
        if query_params.get("book").is_some() || query_params.get("modal").is_some() {
            // Check authentication first
            if is_authenticated() {
                booking_artist_id.set(Some(artist_id.get()));
                show_booking_modal.set(true);
            } else {
                // Redirect to login with return URL
                let current_url = format!("/artist/{}", artist_id.get());
                let login_url = format!("/login?return_url={}", urlencoding::encode(&current_url));
                navigate_clone(&login_url, Default::default());
            }
        }
    });

    view! {
        <div class="artist-highlight-container">
            <Suspense fallback=move || view! {
                <LoadingView message=Some("Loading artist information...".to_string()) />
            }>
                {move || {
                    
                    artist_data.get().map(|data| {
                        data.map(|artist_data| {
                            let artist_styles_for_filter = artist_data.styles.clone();
                            
                            let instagram_posts: Vec<InstagramPost> = artist_data.images_with_styles
                                .into_iter()
                                .map(|(image, styles)| InstagramPost {
                                    image,
                                    styles,
                                })
                                .collect();

                            let artist_name = artist_data.artist.name.unwrap_or_else(|| "Unknown Artist".to_string());
                            let shop_name = artist_data.location.name.unwrap_or_else(|| "Unknown Shop".to_string());
                            let city = artist_data.location.city.unwrap_or_else(|| "Unknown".to_string());
                            let state = artist_data.location.state.unwrap_or_else(|| "Unknown".to_string());
                            
                            view! {
                                <div class="artist-highlight-container">
                                    <div class="artist-highlight-header">
                                        <div class="artist-highlight-header-inner">
                                            <div class="artist-highlight-header-layout">
                                                <div>
                                                    <h1 class="artist-highlight-artist-name">
                                                        {artist_name.clone()}
                                                    </h1>
                                                    <div class="artist-highlight-shop-info">
                                                        <a href={format!("/shop/{}", artist_data.location.id)} 
                                                           class="artist-highlight-shop-link">
                                                            {format!("🏪 {} • {}, {}", shop_name, city, state)}
                                                        </a>
                                                    </div>
                                                </div>
                                                
                                                <div class="artist-highlight-buttons-container">
                                                    <button 
                                                       on:click={
                                                           let navigate = navigate.clone();
                                                           move |_| {
                                                               let current_artist_id = artist_id.get();
                                                               // Check authentication first
                                                               if is_authenticated() {
                                                                   booking_artist_id.set(Some(current_artist_id));
                                                                   show_booking_modal.set(true);
                                                               } else {
                                                                   // Redirect to login with return URL
                                                                   let current_url = format!("/artist/{}", current_artist_id);
                                                                   let login_url = format!("/login?return_url={}", urlencoding::encode(&current_url));
                                                                   navigate(&login_url, Default::default());
                                                               }
                                                           }
                                                       }
                                                       class="artist-highlight-book-button">
                                                        "📅 Book Appointment"
                                                    </button>
                                                    
                                                    {artist_data.artist.social_links.and_then(|links| {
                                                        (!links.is_empty()).then(|| view! {
                                                            <a href={links} target="_blank" 
                                                               class="artist-highlight-social-button">
                                                                "📱 Instagram"
                                                            </a>
                                                        })
                                                    })}

                                                    {artist_data.artist.email.and_then(|email| {
                                                        (!email.is_empty()).then(|| view! {
                                                            <a href={format!("mailto:{}", email)} 
                                                               class="artist-highlight-social-button">
                                                                "✉️ Email"
                                                            </a>
                                                        })
                                                    })}

                                                    {artist_data.artist.phone.and_then(|phone| {
                                                        (!phone.is_empty()).then(|| view! {
                                                            <a href={format!("tel:{}", phone)} 
                                                               class="artist-highlight-social-button">
                                                                "📞 Call"
                                                            </a>
                                                        })
                                                    })}
                                                </div>
                                            </div>
                                        </div>
                                    </div>

                                    <div class="artist-highlight-content">
                                        <div class="artist-highlight-main-grid">
                                            {(!artist_data.styles.is_empty()).then(|| {
                                                view! {
                                                    <div class="artist-highlight-card">
                                                        <h3 class="artist-highlight-card-heading">"Specializes In"</h3>
                                                        <div class="artist-highlight-styles-container">
                                                            {artist_data.styles.into_iter().map(|style| {
                                                                view! {
                                                                    <span class="artist-highlight-style-tag">
                                                                        {style.name}
                                                                    </span>
                                                                }
                                                            }).collect_view()}
                                                        </div>
                                                    </div>
                                                }
                                            })}

                                            <div class="artist-highlight-info-grid">
                                                {artist_data.artist.years_experience.and_then(|years| {
                                                    (years > 0).then(|| view! {
                                                        <div class="artist-highlight-card">
                                                            <h3 class="artist-highlight-experience-heading">"Experience"</h3>
                                                            <div class="artist-highlight-experience-value">
                                                                {format!("{} years", years)}
                                                            </div>
                                                        </div>
                                                    })
                                                })}

                                                {artist_data.location.address.clone().map(|addr| {
                                                    let lat = artist_data.location.lat.unwrap_or(0.0);
                                                    let long = artist_data.location.long.unwrap_or(0.0);
                                                    let encoded_addr = urlencoding::encode(&addr);
                                                    
                                                    view! {
                                                        <div class="artist-highlight-card">
                                                            <h3 class="artist-highlight-card-heading">"📍 Shop Location"</h3>
                                                            <p class="artist-highlight-location-text">
                                                                {addr.clone()}
                                                            </p>
                                                            
                                                            <div class="artist-highlight-map-container">
                                                                <iframe
                                                                    src={format!("https://www.openstreetmap.org/export/embed.html?bbox={},{},{},{}&layer=mapnik&marker={},{}", 
                                                                        long - 0.01, lat - 0.01, long + 0.01, lat + 0.01, lat, long)}
                                                                    class="artist-highlight-map-iframe"
                                                                    title="Shop Location Map"
                                                                ></iframe>
                                                                <div class="artist-highlight-map-overlay"></div>
                                                            </div>
                                                            
                                                            <div class="artist-highlight-map-link-container">
                                                                <a href={format!("https://www.google.com/maps/search/?api=1&query={}", encoded_addr)} 
                                                                   target="_blank"
                                                                   class="artist-highlight-map-link">
                                                                    "Open in Google Maps"
                                                                </a>
                                                            </div>
                                                        </div>
                                                    }
                                                })}
                                            </div>
                                        </div>

                                        {(!instagram_posts.is_empty()).then(|| {
                                            view! {
                                                <div class="artist-highlight-portfolio-card">
                                                    <h2 class="artist-highlight-portfolio-heading">"Portfolio"</h2>
                                                    <ArtistMasonryGallery instagram_posts=instagram_posts artist_styles=artist_styles_for_filter />
                                                </div>
                                            }
                                        })}
                                    </div>
                                </div>
                            }.into_any()
                        }).unwrap_or_else(|| {
                            view! {
                                <div class="artist-highlight-not-found-container">
                                    <div class="artist-highlight-not-found-header">
                                        <div class="artist-highlight-not-found-header-inner">
                                            <h1 class="artist-highlight-not-found-heading">
                                                "🎨 Artist Not Found"
                                            </h1>
                                            <div class="artist-highlight-not-found-subtitle">
                                                "The requested artist could not be found"
                                            </div>
                                        </div>
                                    </div>

                                    <div class="artist-highlight-not-found-content">
                                        <div class="artist-highlight-not-found-card">
                                            <p class="artist-highlight-not-found-text">
                                                "Please check the artist ID and try again."
                                            </p>
                                        </div>
                                    </div>
                                </div>
                            }.into_any()
                        })
                    })
                }}
            </Suspense>
            
            // Booking Modal - overlays the artist page
            <ClientBookingModal 
                show=show_booking_modal
                artist_id=booking_artist_id
                on_close=move || {
                    show_booking_modal.set(false);
                    booking_artist_id.set(None);
                }
            />
        </div>
    }
}