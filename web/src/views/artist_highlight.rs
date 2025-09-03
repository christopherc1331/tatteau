use leptos::prelude::*;
use leptos_router::hooks::{use_params_map, use_query_map};

use crate::{
    components::{loading::LoadingView, artist_masonry_gallery::{ArtistMasonryGallery, InstagramPost}, ClientBookingModal},
    server::fetch_artist_data,
};

#[component]
pub fn ArtistHighlight() -> impl IntoView {
    let params = use_params_map();
    let query = use_query_map();
    
    let artist_id = Memo::new(move |_| {
        params.read()
            .get("id")
            .and_then(|id| id.parse::<i32>().ok())
            .unwrap_or(0)
    });

    let artist_data = Resource::new(
        move || artist_id.get(),
        move |id| async move {
            if id > 0 {
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
    Effect::new(move |_| {
        let query_params = query.read();
        if query_params.get("book").is_some() || query_params.get("modal").is_some() {
            booking_artist_id.set(Some(artist_id.get()));
            show_booking_modal.set(true);
        }
    });

    view! {
        <style>
            {r#"
            .shop-link:hover {
                border-bottom: 1px solid rgba(255,255,255,0.9) !important;
                opacity: 1 !important;
            }
            "#}
        </style>
        <div style="min-height: 100vh; background: #f8fafc;">
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
                                <div style="min-height: 100vh; background: #f8fafc;">
                                    <div style="background: linear-gradient(135deg, #667eea, #764ba2); color: white; padding: 2rem 1rem;">
                                        <div style="max-width: 1200px; margin: 0 auto;">
                                            <div style="display: flex; justify-content: space-between; align-items: center; flex-wrap: wrap; gap: 1rem;">
                                                <div>
                                                    <h1 style="font-size: 2.5rem; font-weight: 700; margin: 0 0 0.5rem 0;">
                                                        {artist_name.clone()}
                                                    </h1>
                                                    <div style="font-size: 1.1rem; opacity: 0.9;">
                                                        <a href={format!("/shop/{}", artist_data.location.id)} 
                                                           style="color: rgba(255,255,255,0.9); text-decoration: none; border-bottom: 1px dashed rgba(255,255,255,0.5); padding-bottom: 2px; transition: all 0.2s ease;"
                                                           class="shop-link">
                                                            {format!("🏪 {} • {}, {}", shop_name, city, state)}
                                                        </a>
                                                    </div>
                                                </div>
                                                
                                                <div style="display: flex; gap: 1rem; flex-wrap: wrap;">
                                                    <button 
                                                       on:click=move |_| {
                                                           booking_artist_id.set(Some(artist_id.get()));
                                                           show_booking_modal.set(true);
                                                       }
                                                       style="background: #f59e0b; padding: 0.5rem 1rem; border-radius: 20px; color: white; text-decoration: none; font-weight: 600; border: none; cursor: pointer; font-size: 1rem;">
                                                        "📅 Book Appointment"
                                                    </button>
                                                    
                                                    {artist_data.artist.social_links.and_then(|links| {
                                                        (!links.is_empty()).then(|| view! {
                                                            <a href={links} target="_blank" 
                                                               style="background: rgba(255,255,255,0.2); padding: 0.5rem 1rem; border-radius: 20px; color: white; text-decoration: none;">
                                                                "📱 Instagram"
                                                            </a>
                                                        })
                                                    })}

                                                    {artist_data.artist.email.and_then(|email| {
                                                        (!email.is_empty()).then(|| view! {
                                                            <a href={format!("mailto:{}", email)} 
                                                               style="background: rgba(255,255,255,0.2); padding: 0.5rem 1rem; border-radius: 20px; color: white; text-decoration: none;">
                                                                "✉️ Email"
                                                            </a>
                                                        })
                                                    })}

                                                    {artist_data.artist.phone.and_then(|phone| {
                                                        (!phone.is_empty()).then(|| view! {
                                                            <a href={format!("tel:{}", phone)} 
                                                               style="background: rgba(255,255,255,0.2); padding: 0.5rem 1rem; border-radius: 20px; color: white; text-decoration: none;">
                                                                "📞 Call"
                                                            </a>
                                                        })
                                                    })}
                                                </div>
                                            </div>
                                        </div>
                                    </div>

                                    <div style="max-width: 1200px; margin: 0 auto; padding: 2rem 1rem;">
                                        <div style="display: grid; grid-template-columns: 1fr 2fr; gap: 2rem; margin-bottom: 2rem;">
                                            {(!artist_data.styles.is_empty()).then(|| {
                                                view! {
                                                    <div style="background: white; border-radius: 16px; padding: 1.5rem; box-shadow: 0 4px 16px rgba(0,0,0,0.08);">
                                                        <h3 style="font-size: 1.25rem; font-weight: 600; color: #2d3748; margin: 0 0 1rem 0;">"Specializes In"</h3>
                                                        <div style="display: flex; flex-wrap: wrap; gap: 0.5rem;">
                                                            {artist_data.styles.into_iter().map(|style| {
                                                                view! {
                                                                    <span style="background: #667eea; color: white; padding: 0.25rem 0.75rem; border-radius: 20px; font-size: 0.8rem;">
                                                                        {style.name}
                                                                    </span>
                                                                }
                                                            }).collect_view()}
                                                        </div>
                                                    </div>
                                                }
                                            })}

                                            <div style="display: grid; gap: 1rem;">
                                                {artist_data.artist.years_experience.and_then(|years| {
                                                    (years > 0).then(|| view! {
                                                        <div style="background: white; border-radius: 16px; padding: 1.5rem; box-shadow: 0 4px 16px rgba(0,0,0,0.08);">
                                                            <h3 style="font-size: 1.25rem; font-weight: 600; color: #2d3748; margin: 0 0 0.5rem 0;">"Experience"</h3>
                                                            <div style="font-size: 2rem; font-weight: 700; color: #667eea;">
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
                                                        <div style="background: white; border-radius: 16px; padding: 1.5rem; box-shadow: 0 4px 16px rgba(0,0,0,0.08);">
                                                            <h3 style="font-size: 1.25rem; font-weight: 600; color: #2d3748; margin: 0 0 0.5rem 0;">"📍 Shop Location"</h3>
                                                            <p style="color: #4a5568; margin: 0 0 1rem 0; font-size: 0.9rem;">
                                                                {addr.clone()}
                                                            </p>
                                                            
                                                            <div style="width: 100%; height: 200px; border-radius: 8px; overflow: hidden; border: 1px solid #e2e8f0; position: relative;">
                                                                <iframe
                                                                    src={format!("https://www.openstreetmap.org/export/embed.html?bbox={},{},{},{}&layer=mapnik&marker={},{}", 
                                                                        long - 0.01, lat - 0.01, long + 0.01, lat + 0.01, lat, long)}
                                                                    style="width: 100%; height: 100%; border: none; pointer-events: none;"
                                                                    title="Shop Location Map"
                                                                ></iframe>
                                                                <div style="position: absolute; top: 0; left: 0; width: 100%; height: 100%; background: transparent; pointer-events: none;"></div>
                                                            </div>
                                                            
                                                            <div style="margin-top: 0.5rem; text-align: center;">
                                                                <a href={format!("https://www.google.com/maps/search/?api=1&query={}", encoded_addr)} 
                                                                   target="_blank"
                                                                   style="color: #667eea; text-decoration: none; font-size: 0.8rem;">
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
                                                <div style="background: white; border-radius: 16px; padding: 1.5rem; box-shadow: 0 4px 16px rgba(0,0,0,0.08);">
                                                    <h2 style="font-size: 1.5rem; font-weight: 600; color: #2d3748; margin: 0 0 1rem 0;">"Portfolio"</h2>
                                                    <ArtistMasonryGallery instagram_posts=instagram_posts artist_styles=artist_styles_for_filter />
                                                </div>
                                            }
                                        })}
                                    </div>
                                </div>
                            }.into_any()
                        }).unwrap_or_else(|| {
                            view! {
                                <div style="min-height: 100vh; background: #f8fafc;">
                                    <div style="background: linear-gradient(135deg, #667eea, #764ba2); color: white; padding: 2rem 1rem;">
                                        <div style="max-width: 1200px; margin: 0 auto; text-align: center;">
                                            <h1 style="font-size: 2.5rem; font-weight: 700; margin: 0 0 0.5rem 0;">
                                                "🎨 Artist Not Found"
                                            </h1>
                                            <div style="font-size: 1.1rem; opacity: 0.9;">
                                                "The requested artist could not be found"
                                            </div>
                                        </div>
                                    </div>

                                    <div style="max-width: 1200px; margin: 0 auto; padding: 2rem 1rem;">
                                        <div style="background: white; border-radius: 16px; padding: 2rem; box-shadow: 0 4px 16px rgba(0,0,0,0.08); text-align: center;">
                                            <p style="color: #4a5568; margin: 0;">
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