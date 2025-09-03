use leptos::prelude::*;
use leptos_router::hooks::{use_params_map, use_navigate};

#[component]
pub fn ArtistBooking() -> impl IntoView {
    let params = use_params_map();
    let navigate = use_navigate();
    
    let artist_id = Memo::new(move |_| {
        params.read()
            .get("id")
            .and_then(|id| id.parse::<i32>().ok())
    });

    // Redirect to artist page with modal open
    // The artist page will handle the modal display
    Effect::new(move |_| {
        if let Some(id) = artist_id.get() {
            // Navigate to the artist highlight page with book query parameter
            navigate(&format!("/artist/{}?book=true", id), Default::default());
        }
    });

    view! {
        <div style="min-height: 100vh; background: #f8fafc; display: flex; align-items: center; justify-content: center;">
            <div style="text-align: center;">
                <p style="color: #4a5568;">"Redirecting to artist page..."</p>
            </div>
        </div>
    }
}

#[component] 
pub fn ShopBooking() -> impl IntoView {
    let params = use_params_map();
    let shop_id = Memo::new(move |_| {
        params.read()
            .get("id")
            .and_then(|id| id.parse::<i32>().ok())
            .unwrap_or(0)
    });

    view! {
        <div style="min-height: 100vh; background: #f8fafc; padding: 2rem 1rem;">
            <div style="max-width: 800px; margin: 0 auto;">
                <div style="background: white; border-radius: 16px; padding: 2rem; box-shadow: 0 4px 16px rgba(0,0,0,0.08); text-align: center;">
                    <h1 style="font-size: 2.5rem; font-weight: 700; margin: 0 0 1rem 0; color: #2d3748;">
                        "🏪 Book at Shop"
                    </h1>
                    
                    <p style="font-size: 1.1rem; color: #4a5568; margin: 0 0 2rem 0;">
                        {format!("Booking system for shop ID: {}", shop_id.get())}
                    </p>
                    
                    <div style="background: #fef3c7; border: 1px solid #fcd34d; border-radius: 8px; padding: 1rem; margin: 2rem 0;">
                        <p style="color: #92400e; margin: 0; font-weight: 600;">
                            "🚧 Coming Soon!"
                        </p>
                        <p style="color: #92400e; margin: 0.5rem 0 0 0; font-size: 0.9rem;">
                            "Our booking system is currently under development. Please contact the shop directly for appointments."
                        </p>
                    </div>
                    
                    <div style="display: flex; gap: 1rem; justify-content: center; flex-wrap: wrap; margin-top: 2rem;">
                        <a href={format!("/shop/{}", shop_id.get())} 
                           style="background: #667eea; color: white; padding: 0.75rem 1.5rem; border-radius: 8px; text-decoration: none; font-weight: 600;">
                            "← Back to Shop"
                        </a>
                        
                        <a href="/" 
                           style="background: #e2e8f0; color: #4a5568; padding: 0.75rem 1.5rem; border-radius: 8px; text-decoration: none; font-weight: 600;">
                            "🏠 Home"
                        </a>
                    </div>
                </div>
            </div>
        </div>
    }
}