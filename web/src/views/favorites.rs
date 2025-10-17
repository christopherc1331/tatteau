use crate::components::shop_masonry_gallery::{ShopInstagramPost, ShopMasonryGallery};
use crate::server_favorites::get_user_favorites_with_details;
use leptos::prelude::*;
use leptos_router::hooks::use_navigate;

#[component]
pub fn FavoritesPage() -> impl IntoView {
    let navigate = use_navigate();

    // Get the auth token from localStorage as a signal
    let auth_token = RwSignal::new(None::<String>);

    // Load token on mount
    Effect::new(move |_| {
        #[cfg(feature = "hydrate")]
        {
            use wasm_bindgen::prelude::*;

            #[wasm_bindgen]
            extern "C" {
                #[wasm_bindgen(js_namespace = localStorage)]
                fn getItem(key: &str) -> Option<String>;
            }

            if let Some(token) = getItem("tatteau_auth_token") {
                auth_token.set(Some(token));
            } else {
                // Redirect to login if no token
                if let Some(window) = web_sys::window() {
                    let _ = window.location().set_href("/login?redirect=/favorites");
                }
            }
        }
    });

    let favorites_resource = Resource::new(
        move || auth_token.get(),
        move |token| async move {
            match token {
                Some(token) => get_user_favorites_with_details(token).await.ok(),
                None => None,
            }
        },
    );

    view! {
        <div class="favorites-page">
            <div class="favorites-container">
                <div class="favorites-header">
                    <h1>"My Favorites"</h1>
                    <p class="favorites-subtitle">"Your saved tattoo inspiration"</p>
                </div>

                <Suspense fallback=move || {
                    view! {
                        <div class="favorites-loading">
                            <div class="loading-spinner"></div>
                            <p>"Loading your favorites..."</p>
                        </div>
                    }
                }>
                    {move || {
                        let navigate = navigate.clone();
                        favorites_resource.get().map(|data| {
                            match data {
                                Some(favorites) if !favorites.is_empty() => {
                                    // Convert FavoritePostWithDetails to ShopInstagramPost
                                    // Filter out favorites without artists since ShopMasonryGallery requires them
                                    let posts: Vec<ShopInstagramPost> = favorites
                                        .into_iter()
                                        .filter_map(|fav| {
                                            fav.artist.map(|artist| ShopInstagramPost {
                                                image: fav.image,
                                                styles: fav.styles,
                                                artist,
                                                is_favorited: true,
                                            })
                                        })
                                        .collect();

                                    view! {
                                        <div class="favorites-grid-wrapper">
                                            <ShopMasonryGallery shop_posts=posts all_styles=vec![] />
                                        </div>
                                    }
                                    .into_any()
                                }
                                _ => {
                                    view! {
                                        <div class="favorites-empty">
                                            <div class="empty-icon">"❤"</div>
                                            <h2>"No favorites yet"</h2>
                                            <p>"Start exploring and save your favorite tattoo designs!"</p>
                                            <button
                                                class="explore-button"
                                                on:click=move |_| {
                                                    navigate("/explore", Default::default());
                                                }
                                            >
                                                "Explore Tattoos"
                                            </button>
                                        </div>
                                    }
                                    .into_any()
                                }
                            }
                        })
                    }}
                </Suspense>
            </div>
        </div>
    }
}
