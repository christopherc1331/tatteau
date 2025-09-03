use leptos::prelude::*;
use leptos_router::components::A;

#[component]
pub fn HomePage() -> impl IntoView {
    view! {
        <div class="homepage-container" style="padding: 2rem; max-width: 1200px; margin: 0 auto;">
            <div style="text-align: center; margin-bottom: 3rem;">
                <h1 style="font-size: 3rem; margin-bottom: 1rem;">"Tatteau"</h1>
                <p style="font-size: 1.2rem; color: #666; margin-bottom: 2rem;">
                    "Discover your perfect tattoo artist"
                </p>
            </div>

            <div style="display: flex; gap: 2rem; justify-content: center; margin-bottom: 3rem;">
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

            <div style="margin-top: 3rem;">
                <h2 style="text-align: center; margin-bottom: 2rem;">"Popular Styles"</h2>
                <div style="display: flex; flex-wrap: wrap; gap: 1rem; justify-content: center;">
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

            <div style="text-align: center; margin-top: 4rem;">
                <p style="color: #888;">
                    "Find the perfect artist for your next tattoo"
                </p>
            </div>
        </div>
    }
}