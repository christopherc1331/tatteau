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
        <div class="booking-artist-container">
            <div class="booking-artist-content">
                <p class="booking-artist-text">"Redirecting to artist page..."</p>
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
        <div class="booking-shop-container">
            <div class="booking-shop-wrapper">
                <div class="booking-shop-card">
                    <h1 class="booking-shop-title">
                        "🏪 Book at Shop"
                    </h1>
                    
                    <p class="booking-shop-subtitle">
                        {format!("Booking system for shop ID: {}", shop_id.get())}
                    </p>
                    
                    <div class="booking-shop-notice">
                        <p class="booking-shop-notice-title">
                            "🚧 Coming Soon!"
                        </p>
                        <p class="booking-shop-notice-text">
                            "Our booking system is currently under development. Please contact the shop directly for appointments."
                        </p>
                    </div>
                    
                    <div class="booking-shop-actions">
                        <a href={format!("/shop/{}", shop_id.get())} 
                           class="booking-shop-btn-primary">
                            "← Back to Shop"
                        </a>
                        
                        <a href="/" 
                           class="booking-shop-btn-secondary">
                            "🏠 Home"
                        </a>
                    </div>
                </div>
            </div>
        </div>
    }
}