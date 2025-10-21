use leptos::prelude::*;
use leptos_router::components::A;

#[component]
pub fn HomePage() -> impl IntoView {
    view! {
        <div class="home-container">
            <div class="home-header">
                <h1 class="home-title">"Tatteau"</h1>
                <p class="home-subtitle">
                    "Discover your perfect tattoo artist"
                </p>
            </div>

            <div class="home-buttons">
                <A href="/explore">
                    <button class="btn-primary">"Explore Artists"</button>
                </A>
                <A href="/match">
                    <button class="btn-primary">"Get Matched"</button>
                </A>
                <A href="/styles">
                    <button class="btn-primary">"See Styles"</button>
                </A>
            </div>

            <div class="home-styles-section">
                <h2 class="home-styles-title">"Popular Styles"</h2>
                <div class="home-styles-grid">
                    <button class="btn-outlined">"Traditional"</button>
                    <button class="btn-outlined">"Neo-Traditional"</button>
                    <button class="btn-outlined">"Realism"</button>
                    <button class="btn-outlined">"Watercolor"</button>
                    <button class="btn-outlined">"Blackwork"</button>
                    <button class="btn-outlined">"Japanese"</button>
                    <button class="btn-outlined">"Minimalist"</button>
                    <button class="btn-outlined">"Geometric"</button>
                </div>
            </div>

            <div class="home-footer">
                <p class="home-footer-text">
                    "Find the perfect artist for your next tattoo"
                </p>
            </div>
        </div>
    }
}
