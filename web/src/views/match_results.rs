use leptos::prelude::*;
use leptos_router::components::A;
use thaw::*;

#[derive(Clone, Debug)]
pub struct MatchedArtist {
    pub id: i32,
    pub name: String,
    pub style: String,
    pub price_range: String,
    pub location: String,
    pub match_score: i32,
}

#[component]
pub fn MatchResults() -> impl IntoView {
    // TODO: Fetch matched artists from database
    let matched_artists = vec![
        MatchedArtist {
            id: 1,
            name: "Sarah Johnson".to_string(),
            style: "Neo-Traditional".to_string(),
            price_range: "$150-300/hr".to_string(),
            location: "Brooklyn, NY".to_string(),
            match_score: 95,
        },
        MatchedArtist {
            id: 2,
            name: "Mike Chen".to_string(),
            style: "Japanese".to_string(),
            price_range: "$200-400/hr".to_string(),
            location: "Manhattan, NY".to_string(),
            match_score: 88,
        },
        MatchedArtist {
            id: 3,
            name: "Emma Rodriguez".to_string(),
            style: "Watercolor".to_string(),
            price_range: "$100-250/hr".to_string(),
            location: "Queens, NY".to_string(),
            match_score: 82,
        },
    ];

    view! {
        <div style="max-width: 1200px; margin: 0 auto; padding: 2rem;">
            <h1 style="text-align: center; margin-bottom: 2rem;">"Your Perfect Matches"</h1>
            <p style="text-align: center; color: #666; margin-bottom: 3rem;">
                "Based on your preferences, here are your top artist matches"
            </p>

            <div style="display: grid; grid-template-columns: repeat(auto-fill, minmax(350px, 1fr)); gap: 2rem;">
                {matched_artists.into_iter().map(|artist| {
                    view! {
                        <div class="card">
                            <div style="padding: 1.5rem;">
                                <div style="display: flex; justify-content: space-between; align-items: start; margin-bottom: 1rem;">
                                    <h3 style="margin: 0;">{artist.name.clone()}</h3>
                                    <span style="background: #28a745; color: white; padding: 0.25rem 0.75rem; border-radius: 12px; font-size: 0.85rem;">
                                        {format!("{}% Match", artist.match_score)}
                                    </span>
                                </div>
                                
                                <div style="margin-bottom: 1rem;">
                                    <p style="margin: 0.5rem 0;">
                                        <strong>"Style: "</strong> {artist.style}
                                    </p>
                                    <p style="margin: 0.5rem 0;">
                                        <strong>"Price: "</strong> {artist.price_range}
                                    </p>
                                    <p style="margin: 0.5rem 0;">
                                        <strong>"Location: "</strong> {artist.location}
                                    </p>
                                </div>

                                <div style="display: flex; gap: 1rem; margin-top: 1.5rem;">
                                    <A href={format!("/artist/{}", artist.id)}>
                                        <button class="btn-primary" style="padding: 0.75rem 1.5rem;">
                                            "View Profile"
                                        </button>
                                    </A>
                                    <A href={format!("/book/artist/{}", artist.id)}>
                                        <button class="btn-secondary">
                                            "Book Now"
                                        </button>
                                    </A>
                                </div>
                            </div>
                        </div>
                    }
                }).collect::<Vec<_>>()}
            </div>

            <div style="text-align: center; margin-top: 3rem;">
                <A href="/match">
                    <button class="btn-outlined" style="padding: 0.75rem 1.5rem;">
                        "Refine Your Preferences"
                    </button>
                </A>
            </div>
        </div>
    }
}