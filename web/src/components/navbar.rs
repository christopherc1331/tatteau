use leptos::prelude::*;
use leptos_router::components::A;
use crate::utils::auth::is_authenticated;

#[component]
pub fn Navbar() -> impl IntoView {
    // Track authentication state reactively
    let is_logged_in = RwSignal::new(false);

    // Check authentication status on mount and updates
    Effect::new(move |_| {
        #[cfg(feature = "hydrate")]
        {
            is_logged_in.set(is_authenticated());
        }
    });

    // Logout handler
    let handle_logout = move |_| {
        #[cfg(feature = "hydrate")]
        {
            use wasm_bindgen::prelude::*;

            #[wasm_bindgen]
            extern "C" {
                #[wasm_bindgen(js_namespace = localStorage)]
                fn removeItem(key: &str);
            }

            // Remove the auth token
            removeItem("tatteau_auth_token");

            // Update state
            is_logged_in.set(false);

            // Redirect to home page
            if let Some(window) = web_sys::window() {
                let _ = window.location().set_href("/");
            }
        }
    };

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

                    {move || {
                        if is_logged_in.get() {
                            view! {
                                <button
                                    class="navbar__link navbar__link--cta"
                                    on:click=handle_logout
                                    style="background: none; border: none; cursor: pointer;"
                                >
                                    "Log Out"
                                </button>
                            }.into_any()
                        } else {
                            view! {
                                <A href="/login" attr:class="navbar__link navbar__link--cta">
                                    "Login"
                                </A>
                            }.into_any()
                        }
                    }}
                </div>
            </div>
        </nav>
    }
}
