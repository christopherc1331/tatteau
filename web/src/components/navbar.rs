use leptos::prelude::*;
use leptos_router::components::A;

#[component]
pub fn Navbar() -> impl IntoView {
    view! {
        <nav class="navbar">
            <div class="navbar__container">
                <div class="navbar__brand">
                    <A href="/" attr:class="navbar__logo">
                        "Tatteau"
                    </A>
                </div>

                <div class="navbar__links">
                    <A href="/explore" attr:class="navbar__link">
                        "Explore"
                    </A>
                    <A href="/match" attr:class="navbar__link">
                        "Find My Match"
                    </A>
                    <A href="/login" attr:class="navbar__link navbar__link--cta">
                        "Artist Login"
                    </A>
                </div>
            </div>
        </nav>
    }
}
