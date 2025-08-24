use leptos::prelude::*;
use leptos_router::components::A;
use thaw::*;

use crate::{
    components::loading::LoadingView,
    server::{get_matched_artists, MatchedArtist},
};

#[component]
pub fn MatchResults() -> impl IntoView {
    // Fetch matched artists from database based on user preferences
    // For now, using some sample preferences - in a real app this would come from user session/URL params
    let matched_artists = Resource::new(
        || (),
        |_| async move {
            get_matched_artists(
                vec!["Traditional".to_string(), "Neo-Traditional".to_string()],
                "Washington".to_string(),
                None,
            ).await
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
    view! {
        <div class="artist-card">
            <div class="card-content">
                <div class="card-header">
                    <h3 class="artist-name">{artist.name.clone()}</h3>
                    <div class="match-badge">
                        {format!("{}% Match", artist.match_score)}
                    </div>
                </div>
                
                <div class="artist-details">
                    <div class="detail-row">
                        <span class="detail-label">"Style:"</span>
                        <span class="detail-value">{artist.primary_style}</span>
                    </div>
                    <div class="detail-row">
                        <span class="detail-label">"Location:"</span>
                        <span class="detail-value">{format!("{}, {}", artist.city, artist.state)}</span>
                    </div>
                    <div class="detail-row">
                        <span class="detail-label">"Shop:"</span>
                        <span class="detail-value">{artist.location_name}</span>
                    </div>
                </div>

                <div class="artist-stats">
                    <div class="stat">
                        <span class="stat-number">{artist.image_count}</span>
                        <span class="stat-label">"Portfolio"</span>
                    </div>
                    <div class="stat">
                        <span class="stat-number">{format!("{:.1}", artist.avg_rating)}</span>
                        <span class="stat-label">"Rating"</span>
                    </div>
                    <div class="stat">
                        <span class="stat-number">{artist.match_score}</span>
                        <span class="stat-label">"Match"</span>
                    </div>
                </div>

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