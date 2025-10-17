use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::hooks::{use_navigate, use_query_map};
use thaw::*;

#[component]
pub fn FavoriteButton(
    /// The artist image ID to favorite
    artists_images_id: i32,
    /// Whether the image is currently favorited
    is_favorited_initial: bool,
) -> impl IntoView {
    let navigate = use_navigate();
    let query_map = use_query_map();

    // Track if the image is favorited (initialize with prop value)
    let is_favorited = RwSignal::new(is_favorited_initial);
    let is_loading = RwSignal::new(false);

    // Get the auth token from localStorage
    let get_auth_token = move || -> Option<String> {
        #[cfg(feature = "hydrate")]
        {
            use wasm_bindgen::prelude::*;

            #[wasm_bindgen]
            extern "C" {
                #[wasm_bindgen(js_namespace = localStorage)]
                fn getItem(key: &str) -> Option<String>;
            }

            getItem("tatteau_auth_token")
        }
        #[cfg(not(feature = "hydrate"))]
        {
            None
        }
    };

    // Check if this image should be auto-favorited after login redirect
    Effect::new(move |_| {
        let query = query_map.get();
        if let Some(favorite_id_str) = query.get("favorite") {
            if let Ok(favorite_id) = favorite_id_str.parse::<i32>() {
                if favorite_id == artists_images_id {
                    // This is the image that should be favorited
                    if let Some(token) = get_auth_token() {
                        is_loading.set(true);

                        spawn_local(async move {
                            use crate::server_favorites::toggle_favorite;

                            match toggle_favorite(token, artists_images_id).await {
                                Ok(new_status) => {
                                    is_favorited.set(new_status);
                                    is_loading.set(false);

                                    // Remove the favorite query param from URL
                                    #[cfg(feature = "hydrate")]
                                    {
                                        if let Some(window) = web_sys::window() {
                                            if let Ok(current_path) = window.location().pathname() {
                                                let _ = window.history().and_then(|h| {
                                                    h.replace_state_with_url(&wasm_bindgen::JsValue::NULL, "", Some(&current_path))
                                                });
                                            }
                                        }
                                    }
                                }
                                Err(e) => {
                                    leptos::logging::error!("Failed to auto-favorite after login: {:?}", e);
                                    is_loading.set(false);
                                }
                            }
                        });
                    }
                }
            }
        }
    });

    let handle_click = move |_| {
        // Get current location for redirect after login
        #[cfg(feature = "hydrate")]
        {
            let current_path = web_sys::window()
                .and_then(|w| w.location().pathname().ok())
                .unwrap_or_else(|| "/".to_string());

            // Check if user is logged in
            if let Some(token) = get_auth_token() {
                // User is logged in, toggle favorite
                is_loading.set(true);

                spawn_local(async move {
                    use crate::server_favorites::toggle_favorite;

                    match toggle_favorite(token, artists_images_id).await {
                        Ok(new_status) => {
                            is_favorited.set(new_status);
                            is_loading.set(false);
                        }
                        Err(e) => {
                            leptos::logging::error!("Failed to toggle favorite: {:?}", e);
                            is_loading.set(false);
                        }
                    }
                });
            } else {
                // User not logged in, redirect to login with return path and favorite ID
                let redirect_url = format!("/login?redirect={}&favorite={}", current_path, artists_images_id);
                navigate(&redirect_url, Default::default());
            }
        }
    };

    view! {
        <button
            class="favorite-button"
            on:click=handle_click
            disabled=move || is_loading.get()
        >
            <span class=move || if is_favorited.get() {
                "favorite-icon favorited"
            } else {
                "favorite-icon"
            }>
                "❤"
            </span>
        </button>
    }
}
