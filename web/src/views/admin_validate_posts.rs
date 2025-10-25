use crate::components::style_tag_manager::StyleTagManager;
use crate::db::entities::Style;
use crate::server::{get_non_validated_posts, mark_post_as_validated, PostValidationData};
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::hooks::use_navigate;
use thaw::*;

#[component]
pub fn AdminValidatePosts() -> impl IntoView {
    let navigate = use_navigate();
    let current_page = RwSignal::new(0i64);
    let page_size = 10i64;
    let posts = RwSignal::new(Vec::<PostValidationData>::new());
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
            match get_non_validated_posts(current_page.get(), page_size, token).await {
                Ok(response) => {
                    posts.set(response.posts);
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

    // Handle validation
    let handle_validate = move |image_id: i64| {
        let token = match get_token() {
            Some(t) => t,
            None => {
                error_message.set(Some("Not authenticated".to_string()));
                return;
            }
        };

        spawn_local(async move {
            match mark_post_as_validated(image_id, token).await {
                Ok(_) => {
                    // Remove the validated post from the list
                    let current_posts = posts.get();
                    let updated_posts: Vec<PostValidationData> = current_posts
                        .into_iter()
                        .filter(|p| p.image_id != image_id)
                        .collect();
                    posts.set(updated_posts);

                    // Update total count
                    total_count.set(total_count.get() - 1);
                }
                Err(e) => {
                    error_message.set(Some(format!("Failed to validate post: {}", e)));
                }
            }
        });
    };

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
                    on:click=move |_| navigate("/admin/dashboard", Default::default())
                >
                    <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                        <polyline points="15 18 9 12 15 6"></polyline>
                    </svg>
                    "Back to Dashboard"
                </button>
                <h1>"Validate Post Tags"</h1>
                <p>"Review and validate style tags for non-validated posts (" {move || total_count.get()} " remaining)"</p>
            </div>

            <Show when=move || error_message.get().is_some()>
                <div class="admin-error-message">
                    {move || error_message.get().unwrap_or_default()}
                </div>
            </Show>

            <Show
                when=move || loading.get()
                fallback=move || view! {
                    <div class="admin-posts-grid">
                        <For
                            each=move || posts.get()
                            key=|post| post.image_id
                            children=move |post: PostValidationData| {
                                let image_id = post.image_id;
                                let short_code = post.short_code.clone();
                                let artist_name = post.artist_name.clone().unwrap_or_else(|| "Unknown Artist".to_string());

                                // Create a signal for the current styles that updates when styles change
                                let current_styles = RwSignal::new(post.current_styles.clone());

                                let on_styles_changed = Callback::new(move |new_styles: Vec<Style>| {
                                    current_styles.set(new_styles);
                                });

                                view! {
                                    <div class="admin-post-card">
                                        <div class="admin-post-image">
                                            <img
                                                src=format!("https://www.instagram.com/p/{}/media/?size=l", short_code)
                                                alt="Post image"
                                            />
                                        </div>
                                        <div class="admin-post-details">
                                            <h3>{artist_name}</h3>
                                            <p class="admin-post-id">"Image ID: " {image_id}</p>

                                            <div class="admin-post-tags">
                                                <For
                                                    each=move || current_styles.get()
                                                    key=|style| style.id
                                                    children=move |style| view! {
                                                        <span class="admin-tag">{style.name}</span>
                                                    }
                                                />
                                            </div>

                                            <StyleTagManager
                                                image_id=image_id
                                                current_styles=Signal::from(current_styles)
                                                on_styles_changed=on_styles_changed
                                            />

                                            <Button
                                                class="admin-validate-button"
                                                appearance=ButtonAppearance::Primary
                                                on_click=move |_| handle_validate(image_id)
                                            >
                                                "Mark as Validated"
                                            </Button>
                                        </div>
                                    </div>
                                }
                            }
                        />
                    </div>

                    <Show when=move || posts.get().is_empty() && !loading.get()>
                        <div class="admin-empty-state">
                            <p>"No posts to validate. Great job!"</p>
                        </div>
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
