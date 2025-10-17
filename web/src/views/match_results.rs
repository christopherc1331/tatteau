use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::components::A;
use leptos_router::hooks::use_query_map;
use thaw::*;

use crate::{
    components::{
        artist_cta::ArtistCTA,
        instagram_embed::{InstagramEmbed, InstagramEmbedSize},
        loading::LoadingView,
        TattooGallery,
    },
    server::{get_matched_artists, get_tattoo_posts_by_style, MatchedArtist, TattooPost},
};

#[component]
pub fn MatchResults() -> impl IntoView {
    let query_map = use_query_map();

    // Modal state
    let (show_modal, set_show_modal) = signal(false);
    let (selected_artist, set_selected_artist) = signal(None::<MatchedArtist>);

    // Fetch tattoo posts filtered by style and location
    let tattoo_posts = Resource::new(
        move || (query_map.get(),),
        move |(query,)| async move {
            // Parse styles from query parameters
            let styles = query
                .get("styles")
                .map(|s| s.split(',').map(|style| style.trim().to_string()).collect())
                .unwrap_or_else(|| vec!["Traditional".to_string()]);

            // Parse states from query parameters (comma-separated)
            let states = query
                .get("states")
                .map(|s| {
                    s.split(',')
                        .map(|state| state.trim().to_string())
                        .collect::<Vec<_>>()
                })
                .filter(|v: &Vec<String>| !v.is_empty());

            // Parse cities from query parameters (comma-separated)
            let cities = query
                .get("cities")
                .map(|s| {
                    s.split(',')
                        .map(|city| city.trim().to_string())
                        .collect::<Vec<_>>()
                })
                .filter(|v: &Vec<String>| !v.is_empty());

            // Get auth token from localStorage
            #[cfg(feature = "hydrate")]
            let token = {
                use wasm_bindgen::prelude::*;

                #[wasm_bindgen]
                extern "C" {
                    #[wasm_bindgen(js_namespace = localStorage)]
                    fn getItem(key: &str) -> Option<String>;
                }

                getItem("tatteau_auth_token")
            };
            #[cfg(not(feature = "hydrate"))]
            let token = None;

            get_tattoo_posts_by_style(styles, states, cities, token).await
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
                                    <div class="match-results-empty-state">
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
                            <div class="match-results-error-state">
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

            <div class="match-results-gallery-footer">
                <A href="/match" attr:class="match-results-refine-button">
                    "Refine Your Preferences"
                </A>
            </div>

            // Artist Modal
            {move || {
                if show_modal.get() {
                    view! {
                        <div class="match-results-artist-modal-overlay" on:click=move |_| set_show_modal.set(false)>
                            <div class="match-results-artist-modal" on:click=move |e| e.stop_propagation()>
                                <button
                                    class="match-results-modal-close"
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
        <div class="match-results-modal-content">
            // Header with gradient background
            <div class="match-results-modal-header">
                <div class="match-results-modal-artist-info">
                    <div class="match-results-modal-avatar-container">
                        {if let Some(avatar_url) = &artist.avatar_url {
                            view! {
                                <img src=avatar_url.clone() alt="Avatar" />
                            }.into_any()
                        } else {
                            view! {
                                <span>{artist.name.chars().next().unwrap_or('A').to_string().to_uppercase()}</span>
                            }.into_any()
                        }}
                    </div>
                    <div class="match-results-modal-artist-details">
                        <h2>{artist.name.clone()}</h2>
                        <p>"📍 " {format!("{}, {}", artist.city, artist.state)}</p>
                    </div>
                    <div class="match-results-modal-match-score">
                        <div class="score">{format!("{}%", artist.match_score)}</div>
                        <div class="label">"Match"</div>
                    </div>
                </div>

                // Specialties
                {if !artist.all_styles.is_empty() {
                    view! {
                        <div class="match-results-modal-specialties">
                            <div class="specialty-label">"Specializes in"</div>
                            <div class="specialty-chips">
                                {artist.all_styles.iter().take(4).map(|style| {
                                    view! {
                                        <span class="specialty-chip">
                                            {style.clone()}
                                        </span>
                                    }
                                }).collect_view()}
                                {if artist.all_styles.len() > 4 {
                                    view! {
                                        <span class="specialty-more">
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
            <div class="match-results-modal-body">
                // Key stats grid
                <div class="match-results-modal-stats-grid">
                    <div class="stat-item">
                        <div class="stat-value">
    "⭐ " {format!("{:.1}", artist.avg_rating)}
                        </div>
                        <div class="stat-label">"Rating"</div>
                    </div>
                    <div class="stat-item">
                        <div class="stat-value">
    "🎨 " {artist.image_count}
                        </div>
                        <div class="stat-label">"Works"</div>
                    </div>
                    <div class="stat-item">
                        <div class="stat-value">
    "⏱️ " {artist.years_experience.map(|y| if y == 0 { "New".to_string() } else { y.to_string() }).unwrap_or_else(|| "?".to_string())}
                        </div>
                        <div class="stat-label">"Years"</div>
                    </div>
                </div>

                // Pricing information
                <div class="match-results-modal-pricing">
                    <div class="pricing-container">
                        <div class="pricing-info">
                            <div class="pricing-label">"Session Pricing"</div>
                            <div class="pricing-value">
                                {match (artist.min_price, artist.max_price) {
                                    (Some(min), Some(max)) => format!("${} - ${}", min as i32, max as i32),
                                    _ => "Contact for quote".to_string()
                                }}
                            </div>
                        </div>
                        <div class="pricing-icon">"💰"</div>
                    </div>
                </div>

                // Why this artist matches
                <div class="match-results-modal-match-reason">
                    <div class="reason-header">
                        <span class="reason-icon">"✅"</span>
                        <span class="reason-title">"Why we matched you"</span>
                    </div>
                    <div class="reason-text">
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
            <div class="match-results-modal-actions">
                <ArtistCTA artist_id={artist.id as i32} class="match-results-cta" />
            </div>
        </div>
    }
}

