use leptos::prelude::*;
use leptos_router::components::A;
use crate::utils::auth::is_authenticated;

#[component]
pub fn Navbar() -> impl IntoView {
    // Track authentication state reactively
    let is_logged_in = RwSignal::new(false);

    // Track menu open state
    let is_menu_open = RwSignal::new(false);

    // Check authentication status on mount and updates
    Effect::new(move |_| {
        #[cfg(feature = "hydrate")]
        {
            is_logged_in.set(is_authenticated());
        }
    });

    // Toggle menu handler
    let toggle_menu = move |_| {
        is_menu_open.update(|open| *open = !*open);
    };

    // Close menu when clicking a link
    let close_menu = move |_| {
        is_menu_open.set(false);
    };

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
            is_menu_open.set(false);

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

                <button
                    class="navbar__hamburger"
                    class:navbar__hamburger--open=move || is_menu_open.get()
                    on:click=toggle_menu
                    attr:aria-label="Toggle navigation menu"
                    attr:aria-expanded=move || is_menu_open.get().to_string()
                >
                    <span class="navbar__hamburger-line"></span>
                    <span class="navbar__hamburger-line"></span>
                    <span class="navbar__hamburger-line"></span>
                </button>

                <div
                    class="navbar__links"
                    class:navbar__links--open=move || is_menu_open.get()
                >
                    <A href="/" attr:class="navbar__link" on:click=close_menu>
                        "Home"
                    </A>
                    <A href="/explore" attr:class="navbar__link" on:click=close_menu>
                        "Explore"
                    </A>
                    <A href="/match" attr:class="navbar__link" on:click=close_menu>
                        "Find My Match"
                    </A>
                    <A href="/favorites" attr:class="navbar__link" on:click=close_menu>
                        "Favorites"
                    </A>

                    {move || {
                        if is_logged_in.get() {
                            view! {
                                <button
                                    class="navbar__link navbar__link--cta"
                                    on:click=handle_logout
                                >
                                    "Log Out"
                                </button>
                            }.into_any()
                        } else {
                            view! {
                                <>
                                    <A href="/login" attr:class="navbar__link" on:click=close_menu>
                                        "Login"
                                    </A>
                                    <A href="/signup" attr:class="navbar__link navbar__link--cta" on:click=close_menu>
                                        "Sign Up"
                                    </A>
                                </>
                            }.into_any()
                        }
                    }}
                </div>
            </div>
        </nav>
    }
}
