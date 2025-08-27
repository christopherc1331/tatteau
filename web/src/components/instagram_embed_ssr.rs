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
                    <div class="instagram-embed-loading">
                        <div style="
                            display: flex;
                            align-items: center;
                            justify-content: center;
                            min-height: 400px;
                            background: #f8fafc;
                            border-radius: 12px;
                            border: 1px solid #e2e8f0;
                        ">
                            <div style="text-align: center;">
                                <div style="
                                    display: inline-block;
                                    width: 32px;
                                    height: 32px;
                                    border: 2px solid #e5e7eb;
                                    border-top-color: #667eea;
                                    border-radius: 50%;
                                    animation: spin 1s linear infinite;
                                    margin-bottom: 1rem;
                                ">
                                    <style>
                                        {r#"
                                        @keyframes spin {
                                            to { transform: rotate(360deg); }
                                        }
                                        "#}
                                    </style>
                                </div>
                                <div style="color: #6b7280; font-size: 0.9rem;">
                                    "Loading Instagram post..."
                                </div>
                            </div>
                        </div>
                    </div>
                }
            }>
                {move || {
                    match embed_resource.get() {
                        Some(Ok(html)) => {
                            view! {
                                <div class="instagram-embed-success">
                                    <div inner_html={html}></div>
                                </div>
                            }.into_any()
                        }
                        Some(Err(_)) => {
                            view! {
                                <div class="instagram-embed-error">
                                    <InstagramFallbackCta
                                        short_code=short_code_for_fallback.clone()
                                        message="Server-side Instagram embed failed".to_string()
                                    />
                                </div>
                            }.into_any()
                        }
                        None => {
                            view! {
                                <div class="instagram-embed-loading">
                                    <div style="
                                        display: flex;
                                        align-items: center;
                                        justify-content: center;
                                        min-height: 400px;
                                        background: #f8fafc;
                                        border-radius: 12px;
                                        border: 1px solid #e2e8f0;
                                    ">
                                        <div style="text-align: center;">
                                            <div style="
                                                display: inline-block;
                                                width: 32px;
                                                height: 32px;
                                                border: 2px solid #e5e7eb;
                                                border-top-color: #667eea;
                                                border-radius: 50%;
                                                animation: spin 1s linear infinite;
                                                margin-bottom: 1rem;
                                            "></div>
                                            <div style="color: #6b7280; font-size: 0.9rem;">
                                                "Loading Instagram post..."
                                            </div>
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