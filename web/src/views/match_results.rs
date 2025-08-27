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
        <div class="artist-card-modal">
            <div class="artist-header">
                <div class="artist-avatar">
                    {artist.name.chars().next().unwrap_or('A').to_string().to_uppercase()}
                </div>
                <div class="artist-basic-info">
                    <h3>{artist.name.clone()}</h3>
                    <p class="location">{format!("{}, {}", artist.city.clone(), artist.state.clone())}</p>
                </div>
                <div class="match-percentage">
                    {format!("{}%", artist.match_score)}
                </div>
            </div>

            <div class="artist-styles">
                {artist.all_styles.iter().map(|style| {
                    view! {
                        <span class="style-chip">{style.clone()}</span>
                    }
                }).collect_view()}
            </div>

            // Sample work would go here - for now just placeholder
            <div class="modal-portfolio">
                <h4>"Portfolio Preview"</h4>
                <div class="portfolio-grid">
                    {(0..4).map(|_| {
                        view! {
                            <div class="portfolio-placeholder">
                                "Sample Work"
                            </div>
                        }
                    }).collect_view()}
                </div>
            </div>

            <div class="pricing-info">
                <div class="pricing-range">
                    <span class="label">"Pricing Range"</span>
                    <span class="price-value">
                        {match (artist.min_price, artist.max_price) {
                            (Some(min), Some(max)) => format!("${} - ${}", min as i32, max as i32),
                            _ => "Contact for pricing".to_string()
                        }}
                    </span>
                </div>
            </div>

            <div class="artist-stats">
                <div class="stat">
                    <div class="stat-value">{artist.image_count.to_string()}</div>
                    <div class="stat-label">"Portfolio"</div>
                </div>
                <div class="stat">
                    <div class="stat-value">{artist.avg_rating.to_string()}</div>
                    <div class="stat-label">"Rating"</div>
                </div>
                <div class="stat">
                    <div class="stat-value">{artist.years_experience.map(|y| y.to_string()).unwrap_or_else(|| "N/A".to_string())}</div>
                    <div class="stat-label">"Years"</div>
                </div>
            </div>

            <div class="modal-actions">
                <A href=format!("/artist/{}", artist.id) attr:class="action-button profile-button">
                    "View Profile"
                </A>
                <A href=format!("/book/artist/{}", artist.id) attr:class="action-button book-button">
                    "Book Now"
                </A>
            </div>
        </div>
    }
}