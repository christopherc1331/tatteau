use leptos::prelude::*;
use crate::server::get_instagram_embed;
use crate::components::instagram_fallback_cta::InstagramFallbackCta;

#[component]
pub fn InstagramEmbedSsr(
    short_code: String,
) -> impl IntoView {
    let short_code_for_resource = short_code.clone();
    let short_code_for_fallback = short_code.clone();
    
    // Create a resource to fetch the Instagram embed from the server
    let embed_resource = Resource::new(
        move || short_code_for_resource.clone(),
        move |short_code| async move {
            get_instagram_embed(short_code).await
        },
    );

    view! {
        <div class="instagram-embed-ssr-container">
            <Suspense fallback=move || {
                view! {
                    <div class="instagram-embed-ssr-loading">
                        <div class="instagram-embed-ssr-loading-content">
                            <div class="instagram-embed-ssr-spinner"></div>
                            <div class="instagram-embed-ssr-loading-text">
                                "Loading Instagram post..."
                            </div>
                        </div>
                    </div>
                }
            }>
                {move || {
                    match embed_resource.get() {
                        Some(Ok(html)) => {
                            view! {
                                <div class="instagram-embed-ssr-success">
                                    <div inner_html={html}></div>
                                </div>
                            }.into_any()
                        }
                        Some(Err(_)) => {
                            view! {
                                <div class="instagram-embed-ssr-error">
                                    <InstagramFallbackCta
                                        short_code=short_code_for_fallback.clone()
                                        message="Server-side Instagram embed failed".to_string()
                                    />
                                </div>
                            }.into_any()
                        }
                        None => {
                            view! {
                                <div class="instagram-embed-ssr-loading">
                                    <div class="instagram-embed-ssr-loading-content">
                                        <div class="instagram-embed-ssr-spinner"></div>
                                        <div class="instagram-embed-ssr-loading-text">
                                            "Loading Instagram post..."
                                        </div>
                                    </div>
                                </div>
                            }.into_any()
                        }
                    }
                }}
            </Suspense>
        </div>
    }
}