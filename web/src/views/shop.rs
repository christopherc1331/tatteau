use leptos::prelude::*;
use leptos_router::hooks::use_params_map;

use crate::{
    components::{loading::LoadingView, shop_masonry_gallery::{ShopMasonryGallery, ShopInstagramPost}},
    server::fetch_shop_data,
    db::entities::{Artist, Style},
};

#[component]
pub fn Shop() -> impl IntoView {
    let params = use_params_map();
    let shop_id = Memo::new(move |_| {
        params.read()
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
        <div style="min-height: 100vh; background: #f8fafc;">
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
                                <div style="min-height: 100vh; background: #f8fafc;">
                                    <div style="background: linear-gradient(135deg, #667eea, #764ba2); color: white; padding: 2rem 1rem;">
                                        <div style="max-width: 1200px; margin: 0 auto;">
                                            <div style="display: flex; justify-content: space-between; align-items: center; flex-wrap: wrap; gap: 1rem;">
                                                <div>
                                                    <h1 style="font-size: 2.5rem; font-weight: 700; margin: 0 0 0.5rem 0;">
                                                        {shop_name.clone()}
                                                    </h1>
                                                    <div style="font-size: 1.1rem; opacity: 0.9;">
                                                        {format!("{}, {}", city, state)}
                                                    </div>
                                                </div>
                                                
                                                <div style="display: flex; gap: 1rem; flex-wrap: wrap;">
                                                    <a href={format!("/book/shop/{}", shop_id.get())}
                                                       style="background: #f59e0b; padding: 0.5rem 1rem; border-radius: 20px; color: white; text-decoration: none; font-weight: 600;">
                                                        "📅 Book Appointment"
                                                    </a>
                                                </div>
                                            </div>
                                        </div>
                                    </div>

                                    <div style="max-width: 1200px; margin: 0 auto; padding: 2rem 1rem;">
                                        <div style="display: grid; grid-template-columns: 1fr 1fr; gap: 2rem; margin-bottom: 2rem;">
                                            {(!shop_data.artists.is_empty()).then(|| {
                                                view! {
                                                    <div style="background: white; border-radius: 16px; padding: 1.5rem; box-shadow: 0 4px 16px rgba(0,0,0,0.08);">
                                                        <h3 style="font-size: 1.25rem; font-weight: 600; color: #2d3748; margin: 0 0 1rem 0;">"Our Artists"</h3>
                                                        <div style="display: flex; flex-direction: column; gap: 1rem;">
                                                            {shop_data.artists.into_iter().map(|artist| {
                                                                let artist_name = artist.name.unwrap_or_else(|| "Unknown Artist".to_string());
                                                                view! {
                                                                    <div style="display: flex; justify-content: space-between; align-items: center; padding: 0.75rem; border: 1px solid #e2e8f0; border-radius: 8px;">
                                                                        <div>
                                                                            <div style="font-weight: 600; color: #2d3748;">
                                                                                {artist_name}
                                                                            </div>
                                                                            {artist.years_experience.and_then(|years| {
                                                                                (years > 0).then(|| view! {
                                                                                    <div style="font-size: 0.8rem; color: #6b7280;">
                                                                                        {format!("{} years experience", years)}
                                                                                    </div>
                                                                                })
                                                                            })}
                                                                        </div>
                                                                        <a href={format!("/artist/{}", artist.id)} 
                                                                           style="background: #667eea; color: white; padding: 0.25rem 0.75rem; border-radius: 6px; text-decoration: none; font-size: 0.8rem;">
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
                                                    <div style="background: white; border-radius: 16px; padding: 1.5rem; box-shadow: 0 4px 16px rgba(0,0,0,0.08);">
                                                        <h3 style="font-size: 1.25rem; font-weight: 600; color: #2d3748; margin: 0 0 1rem 0;">"Styles We Do"</h3>
                                                        <div style="display: flex; flex-wrap: wrap; gap: 0.5rem;">
                                                            {shop_data.all_styles.into_iter().map(|style| {
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
                                        </div>

                                        {shop_data.location.address.clone().map(|addr| {
                                            let lat = shop_data.location.lat.unwrap_or(0.0);
                                            let long = shop_data.location.long.unwrap_or(0.0);
                                            let encoded_addr = urlencoding::encode(&addr);
                                            
                                            view! {
                                                <div style="background: white; border-radius: 16px; padding: 1.5rem; box-shadow: 0 4px 16px rgba(0,0,0,0.08); margin-bottom: 2rem;">
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

                                        {(!shop_posts.is_empty()).then(|| {
                                            view! {
                                                <div style="background: white; border-radius: 16px; padding: 1.5rem; box-shadow: 0 4px 16px rgba(0,0,0,0.08);">
                                                    <h2 style="font-size: 1.5rem; font-weight: 600; color: #2d3748; margin: 0 0 1rem 0;">"Shop Portfolio"</h2>
                                                    <ShopMasonryGallery shop_posts=shop_posts all_styles=all_styles_for_filter />
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
                                                "🏪 Shop Not Found"
                                            </h1>
                                            <div style="font-size: 1.1rem; opacity: 0.9;">
                                                "The requested shop could not be found"
                                            </div>
                                        </div>
                                    </div>

                                    <div style="max-width: 1200px; margin: 0 auto; padding: 2rem 1rem;">
                                        <div style="background: white; border-radius: 16px; padding: 2rem; box-shadow: 0 4px 16px rgba(0,0,0,0.08); text-align: center;">
                                            <p style="color: #4a5568; margin: 0;">
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