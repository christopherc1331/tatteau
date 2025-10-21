use leptos::prelude::*;
use leptos_router::hooks::use_navigate;

/// 404 Error page with professional design and helpful navigation
#[component]
pub fn NotFoundPage() -> impl IntoView {
    let navigate = use_navigate();

    view! {
        <div class="not-found-container">
            <div class="not-found-inner">

                // Animated 404 with tattoo machine drawing effect
                <div class="not-found-header">
                    <div class="not-found-404-text">
                        "404"
                    </div>

                    // Tattoo machine drawing animation placeholder
                    <div class="not-found-tattoo-icon">
                        "🖋️"
                    </div>
                </div>

                // Main content card
                <div class="not-found-card">
                    <h1 class="not-found-title">
                        "Page Not Found"
                    </h1>

                    <p class="not-found-description">
                        "Looks like this page got lost in the ink! The page you're looking for doesn't exist or may have been moved."
                    </p>

                    // Navigation buttons
                    <div class="not-found-button-grid">
                        <button
                            on:click={
                                let navigate = navigate.clone();
                                move |_| {
                                    navigate("/", Default::default());
                                }
                            }
                            class="not-found-btn-home">
                            "🏠 Go Home"
                        </button>

                        <button
                            on:click={
                                let navigate = navigate.clone();
                                move |_| {
                                    navigate("/explore", Default::default());
                                }
                            }
                            class="not-found-btn-explore">
                            "🗺️ Explore Artists"
                        </button>

                        <button
                            on:click={
                                let navigate = navigate.clone();
                                move |_| {
                                    navigate("/match", Default::default());
                                }
                            }
                            class="not-found-btn-match">
                            "🎯 Get Matched"
                        </button>

                        <button
                            on:click={
                                let navigate = navigate.clone();
                                move |_| {
                                    navigate("/styles", Default::default());
                                }
                            }
                            class="not-found-btn-styles">
                            "🎨 Browse Styles"
                        </button>
                    </div>
                </div>

                // Additional help section
                <div class="not-found-help-section">
                    <p class="not-found-help-text">
                        "Still can't find what you're looking for?"
                    </p>
                    <div class="not-found-help-links">
                        <a href="mailto:support@tatteau.com"
                           class="not-found-support-link">
                            "📧 Contact Support"
                        </a>
                        <span class="not-found-separator">"|"</span>
                        <button
                            on:click=move |_| {
                                if let Some(window) = web_sys::window() {
                                    let _ = window.history().unwrap().back();
                                }
                            }
                            class="not-found-back-btn">
                            "⬅️ Go Back"
                        </button>
                    </div>
                </div>
            </div>
        </div>
    }
}
