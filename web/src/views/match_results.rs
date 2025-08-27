use leptos::prelude::*;
use leptos_router::components::A;
use leptos_router::hooks::use_query_map;
use leptos::task::spawn_local;
use thaw::*;

use crate::{
    components::{instagram_embed::{InstagramEmbed, InstagramEmbedSize}, loading::LoadingView, TattooGallery},
    server::{get_tattoo_posts_by_style, get_matched_artists, MatchedArtist, TattooPost},
};

#[component]
pub fn MatchResults() -> impl IntoView {
    let query_map = use_query_map();
    
    // Modal state
    let (show_modal, set_show_modal) = signal(false);
    let (selected_artist, set_selected_artist) = signal(None::<MatchedArtist>);
    
    // Fetch tattoo posts filtered by style
    let tattoo_posts = Resource::new(
        move || (query_map.get(), ),
        move |(query, )| async move {
            // Parse styles from query parameters
            let styles = query.get("styles")
                .map(|s| s.split(',').map(|style| style.trim().to_string()).collect())
                .unwrap_or_else(|| vec!["Traditional".to_string()]);
            
            get_tattoo_posts_by_style(styles, Some(50)).await
        },
    );

    // Fetch full artist data when artist is clicked
    let load_artist_details = move |artist_id: i64| {
        spawn_local(async move {
            // For now, create a minimal artist - in a real app you'd fetch full details
            let matched_artist = MatchedArtist {
                id: artist_id,
                name: "Loading...".to_string(),
                location_name: "Loading...".to_string(),
                city: "Loading...".to_string(),
                state: "Loading...".to_string(),
                primary_style: "Traditional".to_string(),
                all_styles: vec![],
                image_count: 25,
                portfolio_images: vec![],
                avatar_url: None,
                avg_rating: 4.2,
                match_score: 95,
                years_experience: Some(5),
                min_price: Some(150.0),
                max_price: Some(400.0),
            };
            set_selected_artist.set(Some(matched_artist));
            set_show_modal.set(true);
        });
    };

    let on_artist_click = Callback::new(move |artist: MatchedArtist| {
        set_selected_artist.set(Some(artist));
        set_show_modal.set(true);
    });

    view! {
        <div class="match-results-container">
            <div class="page-header">
                <h1>"Tattoo Gallery"</h1>
                <p class="subtitle">
                    "Discover amazing tattoo work in your selected style"
                </p>
            </div>

            <Suspense fallback=move || view! { 
                <LoadingView message=Some("Loading tattoo gallery...".to_string()) /> 
            }>
                {move || {
                    match tattoo_posts.get() {
                        Some(Ok(posts)) => {
                            if posts.is_empty() {
                                view! {
                                    <div class="empty-state">
                                        <h3>"No tattoos found"</h3>
                                        <p>"Try selecting different styles or check back later."</p>
                                        <A href="/match" attr:class="btn-primary">
                                            "Refine Your Search"
                                        </A>
                                    </div>
                                }.into_any()
                            } else {
                                view! {
                                    <TattooGallery 
                                        posts=posts
                                        on_artist_click=on_artist_click
                                    />
                                }.into_any()
                            }
                        },
                        Some(Err(_)) => view! {
                            <div class="error-state">
                                <h3>"Unable to load tattoo gallery"</h3>
                                <p>"Please try refreshing the page or contact support if the problem persists."</p>
                            </div>
                        }.into_any(),
                        None => view! {
                            <LoadingView message=Some("Loading tattoo gallery...".to_string()) />
                        }.into_any(),
                    }
                }}
            </Suspense>

            <div class="gallery-footer">
                <A href="/match" attr:class="refine-button">
                    "Refine Your Preferences"
                </A>
            </div>

            // Artist Modal
            {move || {
                if show_modal.get() {
                    view! {
                        <div class="artist-modal-overlay" on:click=move |_| set_show_modal.set(false)>
                            <div class="artist-modal" on:click=move |e| e.stop_propagation()>
                                <button 
                                    class="modal-close"
                                    on:click=move |_| set_show_modal.set(false)
                                >
                                    "×"
                                </button>
                                {move || {
                                    if let Some(artist) = selected_artist.get() {
                                        view! {
                                            <ArtistModalContent artist=artist />
                                        }.into_any()
                                    } else {
                                        view! { <div>"Loading artist details..."</div> }.into_any()
                                    }
                                }}
                            </div>
                        </div>
                    }.into_any()
                } else {
                    view! { <div></div> }.into_any()
                }
            }}
        </div>
    }
}

#[component]
fn ArtistModalContent(artist: MatchedArtist) -> impl IntoView {
    view! {
        <div style="padding: 0;">
            // Header with gradient background
            <div style="background: linear-gradient(135deg, #667eea, #764ba2); color: white; padding: 2rem; position: relative;">
                <div style="display: flex; align-items: center; gap: 1rem; margin-bottom: 1rem;">
                    <div style="width: 60px; height: 60px; border-radius: 50%; background: rgba(255,255,255,0.2); display: flex; align-items: center; justify-content: center; font-size: 1.5rem; font-weight: bold; backdrop-filter: blur(10px);">
                        {if let Some(avatar_url) = &artist.avatar_url {
                            view! {
                                <img src=avatar_url.clone() alt="Avatar" style="width: 100%; height: 100%; border-radius: 50%; object-fit: cover;" />
                            }.into_any()
                        } else {
                            view! {
                                <span>{artist.name.chars().next().unwrap_or('A').to_string().to_uppercase()}</span>
                            }.into_any()
                        }}
                    </div>
                    <div style="flex: 1;">
                        <h2 style="margin: 0 0 0.25rem 0; font-size: 1.5rem; font-weight: 700;">{artist.name.clone()}</h2>
                        <p style="margin: 0; opacity: 0.9; font-size: 0.95rem;">"📍 " {format!("{}, {}", artist.city, artist.state)}</p>
                    </div>
                    <div style="text-align: center; background: rgba(255,255,255,0.2); padding: 0.5rem 1rem; border-radius: 20px; backdrop-filter: blur(10px);">
                        <div style="font-size: 1.25rem; font-weight: bold; margin: 0;">{format!("{}%", artist.match_score)}</div>
                        <div style="font-size: 0.7rem; opacity: 0.8; margin: 0;">"Match"</div>
                    </div>
                </div>
                
                // Specialties
                {if !artist.all_styles.is_empty() {
                    view! {
                        <div>
                            <div style="font-size: 0.8rem; opacity: 0.8; margin-bottom: 0.5rem;">"Specializes in"</div>
                            <div style="display: flex; flex-wrap: wrap; gap: 0.5rem;">
                                {artist.all_styles.iter().take(4).map(|style| {
                                    view! {
                                        <span style="background: rgba(255,255,255,0.2); padding: 0.25rem 0.75rem; border-radius: 15px; font-size: 0.8rem; backdrop-filter: blur(10px);">
                                            {style.clone()}
                                        </span>
                                    }
                                }).collect_view()}
                                {if artist.all_styles.len() > 4 {
                                    view! {
                                        <span style="background: rgba(255,255,255,0.1); padding: 0.25rem 0.75rem; border-radius: 15px; font-size: 0.8rem; backdrop-filter: blur(10px);">
                                            {format!("+{} more", artist.all_styles.len() - 4)}
                                        </span>
                                    }.into_any()
                                } else {
                                    view! { <span></span> }.into_any()
                                }}
                            </div>
                        </div>
                    }.into_any()
                } else {
                    view! { <div></div> }.into_any()
                }}
            </div>

            // Content body
            <div style="padding: 1.5rem;">
                // Key stats grid
                <div style="display: grid; grid-template-columns: repeat(3, 1fr); gap: 1rem; margin-bottom: 1.5rem;">
                    <div style="text-align: center; padding: 1rem; background: #f8fafc; border-radius: 12px;">
                        <div style="font-size: 1.5rem; font-weight: bold; color: #667eea; margin-bottom: 0.25rem;">
"⭐ " {format!("{:.1}", artist.avg_rating)}
                        </div>
                        <div style="font-size: 0.8rem; color: #6b7280;">"Rating"</div>
                    </div>
                    <div style="text-align: center; padding: 1rem; background: #f8fafc; border-radius: 12px;">
                        <div style="font-size: 1.5rem; font-weight: bold; color: #667eea; margin-bottom: 0.25rem;">
"🎨 " {artist.image_count}
                        </div>
                        <div style="font-size: 0.8rem; color: #6b7280;">"Works"</div>
                    </div>
                    <div style="text-align: center; padding: 1rem; background: #f8fafc; border-radius: 12px;">
                        <div style="font-size: 1.5rem; font-weight: bold; color: #667eea; margin-bottom: 0.25rem;">
"⏱️ " {artist.years_experience.map(|y| if y == 0 { "New".to_string() } else { y.to_string() }).unwrap_or_else(|| "?".to_string())}
                        </div>
                        <div style="font-size: 0.8rem; color: #6b7280;">"Years"</div>
                    </div>
                </div>

                // Pricing information
                <div style="background: linear-gradient(135deg, #f59e0b, #f97316); color: white; padding: 1.25rem; border-radius: 12px; margin-bottom: 1.5rem;">
                    <div style="display: flex; align-items: center; justify-content: space-between;">
                        <div>
                            <div style="font-size: 0.8rem; opacity: 0.9; margin-bottom: 0.25rem;">"Session Pricing"</div>
                            <div style="font-size: 1.25rem; font-weight: bold;">
                                {match (artist.min_price, artist.max_price) {
                                    (Some(min), Some(max)) => format!("${} - ${}", min as i32, max as i32),
                                    _ => "Contact for quote".to_string()
                                }}
                            </div>
                        </div>
                        <div style="font-size: 2rem; opacity: 0.7;">"💰"</div>
                    </div>
                </div>

                // Why this artist matches
                <div style="background: #f0fdf4; border: 1px solid #bbf7d0; border-radius: 12px; padding: 1.25rem; margin-bottom: 1.5rem;">
                    <div style="display: flex; align-items: center; gap: 0.5rem; margin-bottom: 0.75rem;">
                        <span style="font-size: 1.25rem;">"✅"</span>
                        <span style="font-weight: 600; color: #065f46;">"Why we matched you"</span>
                    </div>
                    <div style="color: #047857; font-size: 0.9rem; line-height: 1.4;">
                        "This artist specializes in " <strong>{artist.primary_style.clone()}</strong> 
                        " and " {
                            match artist.years_experience {
                                Some(0) => "is new to the platform".to_string(),
                                Some(years) => format!("has {} years of experience", years),
                                None => "has experience in this style".to_string()
                            }
                        } " with a " <strong>{format!("{:.1}⭐", artist.avg_rating)}</strong> " rating."
                    </div>
                </div>
            </div>

            // Action buttons
            <div style="padding: 0 1.5rem 1.5rem 1.5rem;">
                <div style="display: grid; grid-template-columns: 1fr 1fr; gap: 0.75rem;">
                    <A href=format!("/artist/{}", artist.id) 
                       attr:style="background: #667eea; color: white; padding: 0.75rem 1.5rem; border-radius: 10px; text-decoration: none; text-align: center; font-weight: 600; transition: all 0.2s; display: block;"
                       attr:class="view-profile-btn">
                        "👤 View Profile"
                    </A>
                    <A href=format!("/book/artist/{}", artist.id) 
                       attr:style="background: #f59e0b; color: white; padding: 0.75rem 1.5rem; border-radius: 10px; text-decoration: none; text-align: center; font-weight: 600; transition: all 0.2s; display: block;"
                       attr:class="book-now-btn">
                        "📅 Book Now"
                    </A>
                </div>
            </div>
        </div>
    }
}