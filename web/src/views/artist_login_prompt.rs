use leptos::prelude::*;
use leptos_router::hooks::use_navigate;

/// Page shown when unauthorized users try to access artist-only areas
#[component]
pub fn ArtistLoginPrompt() -> impl IntoView {
    let navigate = use_navigate();

    view! {
        <div class="artist-login-prompt-main-container">
            <div class="artist-login-prompt-card-container">
                // Icon and title
                <div class="artist-login-prompt-icon-container">
                    "🎨"
                </div>
                <h1 class="artist-login-prompt-title">
                    "Artist Area"
                </h1>
                
                // Main message
                <p class="artist-login-prompt-description">
                    "This area is for tattoo artists only. Please log in with your artist account to access your dashboard, bookings, and client management tools."
                </p>
                
                // Action buttons
                <div class="artist-login-prompt-button-container">
                    <button 
                        class="artist-login-prompt-primary-button"
                        on:click={
                            let navigate = navigate.clone();
                            move |_| {
                                navigate("/login?user_type=artist", Default::default());
                            }
                        }>
                        "🔐 Artist Login"
                    </button>
                    
                    <button 
                        class="artist-login-prompt-secondary-button"
                        on:click={
                            let navigate = navigate.clone();
                            move |_| {
                                navigate("/signup?user_type=artist", Default::default());
                            }
                        }>
                        "✨ Create Artist Account"
                    </button>
                </div>
                
                // Additional info
                <div class="artist-login-prompt-footer-section">
                    <p class="artist-login-prompt-footer-text">
                        "Looking for a tattoo?"
                    </p>
                    <button 
                        class="artist-login-prompt-footer-button"
                        on:click={
                            let navigate = navigate.clone();
                            move |_| {
                                navigate("/", Default::default());
                            }
                        }>
                        "🏠 Back to Homepage"
                    </button>
                </div>
            </div>
        </div>
    }
}