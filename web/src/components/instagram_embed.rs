use leptos::html::Div;
use leptos::prelude::*;
use wasm_bindgen::prelude::*;

#[derive(Clone, Debug, PartialEq)]
pub enum InstagramEmbedSize {
    Thumbnail,
    Small,
    Medium,
    Large,
}

impl Default for InstagramEmbedSize {
    fn default() -> Self {
        Self::Medium
    }
}

impl InstagramEmbedSize {
    fn container_class(&self) -> &'static str {
        match self {
            Self::Thumbnail => "instagram-embed-container instagram-embed-container--thumbnail",
            Self::Small => "instagram-embed-container instagram-embed-container--small",
            Self::Medium => "instagram-embed-container instagram-embed-container--medium",
            Self::Large => "instagram-embed-container instagram-embed-container--large",
        }
    }

    fn media_class(&self) -> &'static str {
        match self {
            Self::Thumbnail => "instagram-embed-media instagram-embed-media--thumbnail",
            Self::Small => "instagram-embed-media instagram-embed-media--small",
            Self::Medium => "instagram-embed-media instagram-embed-media--medium",
            Self::Large => "instagram-embed-media instagram-embed-media--large",
        }
    }
}

#[component]
pub fn InstagramEmbed(
    short_code: String,
    #[prop(optional, default=InstagramEmbedSize::default())] size: InstagramEmbedSize,
) -> impl IntoView {
    // Use a unique ID for each component instance to ensure Effects run on every render
    // Use a simple counter-based approach that works in both SSR and client
    use std::sync::atomic::{AtomicU64, Ordering};
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let unique_id = COUNTER.fetch_add(1, Ordering::Relaxed);
    let embed_id = format!("instagram-embed-{}-{}", short_code, unique_id);
    let embed_id_for_view = embed_id.clone();
    let embed_id_for_effect = embed_id.clone();
    let short_code_for_html = short_code.clone();

    // Use NodeRef to access the DOM element once it's rendered
    let container_ref = NodeRef::<Div>::new();

    // Instagram embed initialization - Effect runs when container_ref is populated
    Effect::new(move |_| {
        // This will re-run until container_ref.get() returns Some
        if let Some(container) = container_ref.get() {
            let js_code = format!(
                r#"
            (function(elem) {{
                processInstagramEmbed(elem);

                function processInstagramEmbed(element) {{
                    // Remove any existing processed attribute to force re-processing
                    element.removeAttribute('data-instagram-processed');

                    // Check if iframe already exists (Instagram processed it quickly)
                    const existingIframe = element.querySelector('iframe[src*="instagram.com"]') ||
                                          element.querySelector('.instagram-media iframe');
                    if (existingIframe) {{
                        // Iframe exists, but wait for it to load before hiding spinner
                        element.setAttribute('data-instagram-processed', 'true');
                        hideLoadingSpinner(element);
                        return;
                    }}

                    // Mark as processing to prevent duplicate calls
                    element.setAttribute('data-instagram-processing', 'true');

                    function finalizeProcessing() {{
                        element.setAttribute('data-instagram-processed', 'true');
                        element.removeAttribute('data-instagram-processing');
                        hideLoadingSpinner(element);
                    }}

                    // Ensure Instagram script is loaded only once
                    if (!window.instgrm) {{
                        // Check if script is already being loaded
                        const existingScript = document.querySelector('script[src*="instagram.com/embed.js"]');
                        if (existingScript) {{
                            // Script loading in progress, wait for it
                            const checkInterval = setInterval(() => {{
                                if (window.instgrm && window.instgrm.Embeds) {{
                                    clearInterval(checkInterval);
                                    window.instgrm.Embeds.process();
                                    finalizeProcessing();
                                }}
                            }}, 200);

                            // Timeout after 5 seconds
                            setTimeout(() => {{
                                clearInterval(checkInterval);
                                finalizeProcessing();
                            }}, 5000);
                        }} else {{
                            // Load script for first time
                            const script = document.createElement('script');
                            script.src = 'https://www.instagram.com/embed.js';
                            script.async = true;
                            script.onload = () => {{
                                if (window.instgrm && window.instgrm.Embeds) {{
                                    window.instgrm.Embeds.process();
                                }}
                                finalizeProcessing();
                            }};
                            script.onerror = () => finalizeProcessing();
                            document.head.appendChild(script);
                        }}
                    }} else if (window.instgrm && window.instgrm.Embeds) {{
                        // Don't call process() here - let the gallery component handle it
                        // This prevents multiple rapid process() calls
                        hideLoadingSpinner(element);
                    }} else {{
                        hideLoadingSpinner(element);
                    }}
                }}

                function hideLoadingSpinner(element) {{
                    // Watch for iframe to appear with Instagram's rendered class
                    const checkForIframe = () => {{
                        // Look for iframe with the instagram-media-rendered class OR any iframe after timeout
                        const iframe = element.querySelector('iframe.instagram-media-rendered') ||
                                     element.querySelector('iframe');
                        if (iframe) {{
                            // Hide spinner after short delay
                            setTimeout(() => {{
                                const loadingDiv = element.querySelector('[data-instagram-loading]');
                                if (loadingDiv) {{
                                    loadingDiv.style.display = 'none';
                                }}
                            }}, 500);
                            return true;
                        }}
                        return false;
                    }};

                    // Try immediately
                    if (!checkForIframe()) {{
                        // Use MutationObserver to watch for iframe insertion and attribute changes
                        const observer = new MutationObserver((mutations) => {{
                            if (checkForIframe()) {{
                                observer.disconnect();
                            }}
                        }});

                        observer.observe(element, {{
                            childList: true,
                            subtree: true,
                            attributes: true,
                            attributeFilter: ['class']
                        }});

                        // Fallback: force hide after 3 seconds
                        setTimeout(() => {{
                            observer.disconnect();
                            const loadingDiv = element.querySelector('[data-instagram-loading]');
                            if (loadingDiv) {{
                                loadingDiv.style.display = 'none';
                            }}
                        }}, 3000);
                    }}
                }}
            }})(arguments[0]);
        "#
            );

            // Execute the JavaScript with the container element as argument
            let func = web_sys::js_sys::Function::new_with_args("elem", &js_code);
            let _ = func.call1(&wasm_bindgen::JsValue::NULL, &container);
        }
    });

    let container_class = size.container_class();
    let media_class = size.media_class();

    view! {
        <div id={embed_id_for_view} class={container_class} node_ref=container_ref>
            <div
                class={media_class}
                inner_html={format!(
                    r#"<blockquote class="instagram-media" data-instgrm-captioned data-instgrm-permalink="https://www.instagram.com/p/{}/" data-instgrm-version="14" data-instgrm-payload-id="instagram-media-payload-0"><a href="https://www.instagram.com/p/{}/"></a></blockquote>"#,
                    short_code_for_html, short_code_for_html
                )}
            ></div>

            <div data-instagram-loading="true" class="instagram-embed-loading-overlay" style="display: block;">
                <div class="instagram-embed-loading-content">
                    <div class="instagram-embed-loading-spinner"></div>
                    <div class="instagram-embed-loading-text">
                        "Loading..."
                    </div>
                </div>
            </div>
        </div>
    }
}

// Helper function to trigger Instagram processing for all embeds
#[wasm_bindgen]
pub fn process_instagram_embeds() {
    web_sys::js_sys::eval(
        r#"
        if (window.instgrm && window.instgrm.Embeds) {
            window.instgrm.Embeds.process();
        }
    "#,
    )
    .ok();
}
