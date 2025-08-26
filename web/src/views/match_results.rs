use leptos::prelude::*;
use leptos_router::components::A;
use leptos_router::hooks::use_query_map;
use thaw::*;

use crate::{
    components::{instagram_embed::{InstagramEmbed, InstagramEmbedSize}, loading::LoadingView},
    server::{get_matched_artists, MatchedArtist},
};

#[component]
pub fn MatchResults() -> impl IntoView {
    let query_map = use_query_map();
    
    // Fetch matched artists from database based on user preferences from URL params
    let matched_artists = Resource::new(
        move || (query_map.get(), ),
        move |(query, )| async move {
            // Parse styles from query parameters
            let styles = query.get("styles")
                .map(|s| s.split(',').map(|style| style.trim().to_string()).collect())
                .unwrap_or_else(|| vec!["Traditional".to_string(), "Neo-Traditional".to_string()]);
            
            let location = query.get("location")
                .map(|s| s.clone())
                .unwrap_or_else(|| "Washington".to_string());
            
            let price_range = {
                let min_price = query.get("min_price")
                    .and_then(|s| s.parse::<f64>().ok());
                let max_price = query.get("max_price")
                    .and_then(|s| s.parse::<f64>().ok());
                    
                match (min_price, max_price) {
                    (Some(min), Some(max)) => Some((min, max)),
                    _ => None,
                }
            };
            
            get_matched_artists(styles, location, price_range).await
        },
    );

    view! {
        <div class="match-results-container">
            <div class="page-header">
                <h1>"Your Perfect Matches"</h1>
                <p class="subtitle">
                    "Based on your preferences, here are your top artist matches"
                </p>
            </div>

            <Suspense fallback=|| view! { <LoadingView message=Some("Finding your perfect matches...".to_string()) /> }>
                {move || {
                    match matched_artists.get() {
                        Some(Ok(artists)) => {
                            if artists.is_empty() {
                                view! {
                                    <div class="no-matches">
                                        <h3>"No matches found"</h3>
                                        <p>"Try adjusting your preferences to find more artists."</p>
                                        <A href="/match">
                                            <div class="btn-outlined">
                                                "Update Preferences"
                                            </div>
                                        </A>
                                    </div>
                                }.into_any()
                            } else {
                                view! {
                                    <>
                                        <div class="artists-grid">
                                            {artists.into_iter().map(|artist| {
                                                view! {
                                                    <ArtistCard artist=artist />
                                                }
                                            }).collect_view()}
                                        </div>

                                        <div class="refine-section">
                                            <A href="/match">
                                                <div class="btn-outlined">
                                                    "Refine Your Preferences"
                                                </div>
                                            </A>
                                        </div>
                                    </>
                                }.into_any()
                            }
                        },
                        Some(Err(_)) => view! {
                            <div class="no-matches">
                                <h3>"Something went wrong"</h3>
                                <p>"We couldn't load your matches right now. Please try again."</p>
                                <A href="/match">
                                    <div class="btn-outlined">
                                        "Back to Preferences"
                                    </div>
                                </A>
                            </div>
                        }.into_any(),
                        None => view! {
                            <LoadingView message=Some("Loading matches...".to_string()) />
                        }.into_any(),
                    }
                }}
            </Suspense>
        </div>
    }
}

#[component]
pub fn ArtistCard(artist: MatchedArtist) -> impl IntoView {
    // Calculate pricing percentage for progress meter (0-100 based on max possible)
    let pricing_percentage = if let (Some(min), Some(max)) = (artist.min_price, artist.max_price) {
        let avg_price = (min + max) / 2.0;
        (avg_price / 500.0 * 100.0).min(100.0) as u32 // Normalize to 500 as max
    } else {
        50 // Default to middle if no pricing available
    };

    view! {
        <div class="artist-card">
            <div class="card-content">
                // Artist avatar and header
                <div class="card-header">
                    <div class="artist-info">
                        {if let Some(avatar_url) = &artist.avatar_url {
                            view! {
                                <img src={avatar_url.clone()} alt={format!("{} avatar", artist.name)} class="artist-avatar" />
                            }.into_any()
                        } else {
                            view! {
                                <div class="artist-avatar-placeholder">
                                    {artist.name.chars().next().unwrap_or('?').to_uppercase().to_string()}
                                </div>
                            }.into_any()
                        }}
                        <div class="artist-header-text">
                            <h3 class="artist-name">{artist.name.clone()}</h3>
                            <div class="artist-location">{format!("{}, {}", artist.city, artist.state)}</div>
                            <div class="artist-shop">{artist.location_name}</div>
                        </div>
                    </div>
                    <div class="match-badge">
                        {format!("{}%", artist.match_score)}
                    </div>
                </div>

                // Style tags using div-based chips
                <div class="styles-section">
                    <div class="style-chips">
                        {artist.all_styles.into_iter().map(|style| {
                            view! {
                                <div class="style-chip">
                                    {style}
                                </div>
                            }
                        }).collect_view()}
                    </div>
                </div>

                // Portfolio thumbnails (Instagram embeds)
                {if !artist.portfolio_images.is_empty() {
                    view! {
                        <div class="portfolio-thumbnails">
                            {artist.portfolio_images.into_iter().take(4).map(|post_url| {
                                // Extract short code from Instagram URL
                                let short_code = post_url.split("/p/").nth(1)
                                    .unwrap_or("")
                                    .split("/").next()
                                    .unwrap_or("")
                                    .to_string();

                                if !short_code.is_empty() {
                                    view! {
                                        <div class="thumbnail instagram-thumbnail">
                                            <InstagramEmbed short_code={short_code} size=InstagramEmbedSize::Thumbnail />
                                        </div>
                                    }.into_any()
                                } else {
                                    view! {
                                        <div class="thumbnail portfolio-fallback">
                                            <div class="fallback-content">
                                                <div class="fallback-icon">IG</div>
                                                <div class="fallback-text">Portfolio</div>
                                            </div>
                                        </div>
                                    }.into_any()
                                }
                            }).collect_view()}
                        </div>
                    }.into_any()
                } else {
                    view! { <div></div> }.into_any()
                }}

                // Pricing meter
                <div class="pricing-section">
                    <div class="pricing-label">"Pricing Range"</div>
                    {if let (Some(min), Some(max)) = (artist.min_price, artist.max_price) {
                        view! {
                            <div class="pricing-info">
                                <div class="price-range">{format!("${:.0} - ${:.0}", min, max)}</div>
                                <div class="pricing-progress">
                                    <div class="pricing-progress-bar" style={format!("width: {}%", pricing_percentage)}></div>
                                </div>
                            </div>
                        }.into_any()
                    } else {
                        view! {
                            <div class="pricing-info">
                                <div class="price-range">"Price on consultation"</div>
                                <div class="pricing-progress">
                                    <div class="pricing-progress-bar" style="width: 50%"></div>
                                </div>
                            </div>
                        }.into_any()
                    }}
                </div>

                // Stats row
                <div class="artist-stats">
                    <div class="stat">
                        <span class="stat-number">{artist.image_count}</span>
                        <span class="stat-label">"Portfolio"</span>
                    </div>
                    <div class="stat">
                        <span class="stat-number">{format!("{:.1}", artist.avg_rating)}</span>
                        <span class="stat-label">"Rating"</span>
                    </div>
                    {if let Some(years) = artist.years_experience {
                        view! {
                            <div class="stat">
                                <span class="stat-number">{years}</span>
                                <span class="stat-label">"Years"</span>
                            </div>
                        }.into_any()
                    } else {
                        view! { <div></div> }.into_any()
                    }}
                </div>

                // Action buttons
                <div class="card-actions">
                    <A href={format!("/artist/{}", artist.id)}>
                        <div class="btn btn-primary">
                            "View Profile"
                        </div>
                    </A>
                    <A href={format!("/book/artist/{}", artist.id)}>
                        <div class="btn btn-secondary">
                            "Book Now"
                        </div>
                    </A>
                </div>
            </div>
        </div>
    }
}

