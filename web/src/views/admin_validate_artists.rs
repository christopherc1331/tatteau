use crate::server::{get_non_validated_artists, mark_artist_shop_validated, ArtistValidationData};
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::hooks::use_navigate;
use thaw::*;

#[component]
pub fn AdminValidateArtists() -> impl IntoView {
    let navigate = use_navigate();
    let artists = RwSignal::new(Vec::<ArtistValidationData>::new());
    let total_count = RwSignal::new(0i64);
    let loading = RwSignal::new(false);
    let error_message = RwSignal::new(Option::<String>::None);

    // Get auth token from localStorage
    let get_token = move || -> Option<String> {
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

    // Fetch artists
    let fetch_artists = move || {
        let token = match get_token() {
            Some(t) => t,
            None => {
                error_message.set(Some("Not authenticated. Please log in.".to_string()));
                return;
            }
        };

        loading.set(true);
        error_message.set(None);

        spawn_local(async move {
            match get_non_validated_artists(token).await {
                Ok(response) => {
                    artists.set(response.artists);
                    total_count.set(response.total_count);
                }
                Err(e) => {
                    error_message.set(Some(format!("Failed to fetch artists: {}", e)));
                }
            }
            loading.set(false);
        });
    };

    // Initial load
    Effect::new(move |_| {
        fetch_artists();
    });

    // Handle validation
    let handle_validate = move |artist_id: i64| {
        let token = match get_token() {
            Some(t) => t,
            None => {
                error_message.set(Some("Not authenticated".to_string()));
                return;
            }
        };

        spawn_local(async move {
            match mark_artist_shop_validated(artist_id, token).await {
                Ok(_) => {
                    // Remove the validated artist from the list
                    let current_artists = artists.get();
                    let updated_artists: Vec<ArtistValidationData> = current_artists
                        .into_iter()
                        .filter(|a| a.id != artist_id)
                        .collect();
                    artists.set(updated_artists);

                    // Update total count
                    total_count.set(total_count.get() - 1);
                }
                Err(e) => {
                    error_message.set(Some(format!("Failed to validate artist: {}", e)));
                }
            }
        });
    };

    view! {
        <div class="admin-validate-artists">
            <div class="admin-validate-header">
                <button
                    class="admin-back-button"
                    on:click=move |_| navigate("/admin/dashboard", Default::default())
                >
                    <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                        <polyline points="15 18 9 12 15 6"></polyline>
                    </svg>
                    "Back to Dashboard"
                </button>
                <h1>"Validate Artist Shops"</h1>
                <p>"Verify artist shop assignments (" {move || total_count.get()} " remaining)"</p>
            </div>

            <Show when=move || error_message.get().is_some()>
                <div class="admin-error-message">
                    {move || error_message.get().unwrap_or_default()}
                </div>
            </Show>

            <Show
                when=move || loading.get()
                fallback=move || view! {
                    <div class="admin-artists-list">
                        <For
                            each=move || artists.get()
                            key=|artist| artist.id
                            children=move |artist: ArtistValidationData| {
                                let artist_id = artist.id;
                                let artist_name = artist.name.clone().unwrap_or_else(|| "Unknown Artist".to_string());
                                let instagram_handle_opt = artist.instagram_handle.clone();
                                let instagram_handle_display = instagram_handle_opt.clone().unwrap_or_default();
                                let location_name = artist.location_name.clone().unwrap_or_else(|| "Unknown Location".to_string());
                                let created_at = artist.created_at.clone().unwrap_or_else(|| "Unknown".to_string());

                                view! {
                                    <div class="admin-artist-card">
                                        <div class="admin-artist-info">
                                            <h3>{artist_name}</h3>
                                            <Show when=move || instagram_handle_opt.is_some()>
                                                <p class="admin-artist-instagram">
                                                    <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                                                        <rect x="2" y="2" width="20" height="20" rx="5" ry="5"></rect>
                                                        <path d="M16 11.37A4 4 0 1 1 12.63 8 4 4 0 0 1 16 11.37z"></path>
                                                        <line x1="17.5" y1="6.5" x2="17.51" y2="6.5"></line>
                                                    </svg>
                                                    "@" {instagram_handle_display.clone()}
                                                </p>
                                            </Show>
                                            <p class="admin-artist-location">
                                                <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                                                    <path d="M21 10c0 7-9 13-9 13s-9-6-9-13a9 9 0 0 1 18 0z"></path>
                                                    <circle cx="12" cy="10" r="3"></circle>
                                                </svg>
                                                {location_name}
                                            </p>
                                            <p class="admin-artist-created">"Created: " {created_at}</p>
                                        </div>
                                        <div class="admin-artist-actions">
                                            <label class="admin-checkbox-validate">
                                                <input
                                                    type="checkbox"
                                                    on:change=move |_| handle_validate(artist_id)
                                                />
                                                <span>"Shop Validated"</span>
                                            </label>
                                        </div>
                                    </div>
                                }
                            }
                        />
                    </div>

                    <Show when=move || artists.get().is_empty() && !loading.get()>
                        <div class="admin-empty-state">
                            <p>"No artists to validate. Great job!"</p>
                        </div>
                    </Show>
                }
            >
                <div class="admin-loading">
                    <Spinner />
                    <p>"Loading artists..."</p>
                </div>
            </Show>
        </div>
    }
}
