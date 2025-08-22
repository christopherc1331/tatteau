use leptos::prelude::*;
use leptos_router::hooks::use_params_map;

use crate::{
    components::{loading::LoadingView, masonry_gallery::{MasonryGallery, InstagramPost}},
    server::fetch_artist_data,
};

#[component]
pub fn ArtistHighlight() -> impl IntoView {
    let params = use_params_map();
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

    view! {
        <div style="min-height: 100vh; background: #f8fafc;">
            <Suspense fallback=move || view! {
                <LoadingView message=Some("Loading artist information...".to_string()) />
            }>
                {move || {
                    artist_data.get().map(|data| {
                        data.map(|artist_data| {
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
                                                        {format!("{} • {}, {}", shop_name, city, state)}
                                                    </div>
                                                </div>
                                                
                                                <div style="display: flex; gap: 1rem; flex-wrap: wrap;">
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
                                                    view! {
                                                        <div style="background: white; border-radius: 16px; padding: 1.5rem; box-shadow: 0 4px 16px rgba(0,0,0,0.08);">
                                                            <h3 style="font-size: 1.25rem; font-weight: 600; color: #2d3748; margin: 0 0 0.5rem 0;">"📍 Shop Location"</h3>
                                                            <p style="color: #4a5568; margin: 0; font-size: 0.9rem;">
                                                                {addr}
                                                            </p>
                                                        </div>
                                                    }
                                                })}
                                            </div>
                                        </div>

                                        {(!instagram_posts.is_empty()).then(|| {
                                            view! {
                                                <div style="background: white; border-radius: 16px; padding: 1.5rem; box-shadow: 0 4px 16px rgba(0,0,0,0.08);">
                                                    <h2 style="font-size: 1.5rem; font-weight: 600; color: #2d3748; margin: 0 0 1rem 0;">"Portfolio"</h2>
                                                    <MasonryGallery instagram_posts=instagram_posts />
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
        </div>
    }
}