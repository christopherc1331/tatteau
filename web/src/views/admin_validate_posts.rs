use crate::components::instagram_embed::process_instagram_embeds;
use crate::components::instagram_posts_grid::{InstagramPostsGrid, PostWithArtist};
use crate::server::get_unvalidated_posts_for_gallery;
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::hooks::use_navigate;
use thaw::*;

#[component]
pub fn AdminValidatePosts() -> impl IntoView {
    let navigate = use_navigate();
    let current_page = RwSignal::new(0i64);
    let page_size = 20i64;
    let posts = RwSignal::new(Vec::<PostWithArtist>::new());
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

    // Fetch posts
    let fetch_posts = move || {
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
            match get_unvalidated_posts_for_gallery(current_page.get(), page_size, token).await {
                Ok(response) => {
                    // Convert to PostWithArtist format
                    let converted_posts: Vec<PostWithArtist> = response
                        .posts
                        .into_iter()
                        .map(|(image, styles, artist, is_favorited)| PostWithArtist {
                            image,
                            styles,
                            artist,
                            is_favorited,
                        })
                        .collect();

                    posts.set(converted_posts);
                    total_count.set(response.total_count);
                }
                Err(e) => {
                    error_message.set(Some(format!("Failed to fetch posts: {}", e)));
                }
            }
            loading.set(false);
        });
    };

    // Initial load
    Effect::new(move |_| {
        fetch_posts();
    });

    // Trigger Instagram embed processing when posts change
    Effect::new(move |_| {
        let _ = posts.get();
        set_timeout(
            move || {
                process_instagram_embeds();
            },
            std::time::Duration::from_millis(100),
        );
    });

    let total_pages = Memo::new(move |_| {
        let count = total_count.get();
        ((count + page_size - 1) / page_size).max(1)
    });

    let go_to_prev_page = move |_| {
        if current_page.get() > 0 {
            current_page.set(current_page.get() - 1);
            fetch_posts();
        }
    };

    let go_to_next_page = move |_| {
        if current_page.get() < total_pages.get() - 1 {
            current_page.set(current_page.get() + 1);
            fetch_posts();
        }
    };

    view! {
        <div class="admin-validate-posts">
            <div class="admin-validate-header">
                <button
                    class="admin-back-button"
                    on:click={
                        let navigate = navigate.clone();
                        move |_| navigate("/admin/dashboard", Default::default())
                    }
                >
                    <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                        <polyline points="15 18 9 12 15 6"></polyline>
                    </svg>
                    "Back to Dashboard"
                </button>
                <h1>"Validate Post Tags"</h1>
                <p>"Review and validate style tags for non-validated posts (" {move || total_count.get()} " remaining)"</p>
                <p class="admin-help-text">"Tags are automatically marked as validated when you update them"</p>
            </div>

            <Show when=move || error_message.get().is_some()>
                <div class="admin-error-message">
                    {move || error_message.get().unwrap_or_default()}
                </div>
            </Show>

            <Show
                when=move || loading.get()
                fallback=move || view! {
                    <Show when=move || posts.get().is_empty() && !loading.get()>
                        <div class="admin-empty-state">
                            <p>"No posts to validate. Great job!"</p>
                        </div>
                    </Show>

                    <Show when=move || !posts.get().is_empty()>
                        <InstagramPostsGrid posts=posts.get() />
                    </Show>

                    <div class="admin-pagination">
                        <Button
                            on_click=go_to_prev_page
                            disabled=Signal::derive(move || current_page.get() == 0)
                        >
                            "Previous"
                        </Button>
                        <span class="admin-page-info">
                            "Page " {move || current_page.get() + 1} " of " {move || total_pages.get()}
                        </span>
                        <Button
                            on_click=go_to_next_page
                            disabled=Signal::derive(move || current_page.get() >= total_pages.get() - 1)
                        >
                            "Next"
                        </Button>
                    </div>
                }
            >
                <div class="admin-loading">
                    <Spinner />
                    <p>"Loading posts..."</p>
                </div>
            </Show>
        </div>
    }
}
