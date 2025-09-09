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
    let embed_id = format!("instagram-embed-{}", short_code);
    let embed_id_for_effect = embed_id.clone();
    let embed_id_for_view = embed_id.clone();
    let short_code_for_html = short_code.clone();

    // Instagram embed initialization with optimized processing to prevent duplicate requests
    Effect::new(move |_| {
        let js_code = format!(
            r#"
            (function() {{
                const embedId = '{}';
                
                const elem = document.getElementById(embedId);
                if (!elem) {{
                    // Retry only once after a short delay
                    setTimeout(function() {{
                        const retryElem = document.getElementById(embedId);
                        if (retryElem && window.instgrm && window.instgrm.Embeds) {{
                            processInstagramEmbed(retryElem);
                        }}
                    }}, 100);
                    return;
                }}
                
                processInstagramEmbed(elem);
                
                function processInstagramEmbed(element) {{
                    // Enhanced check to prevent duplicate processing
                    if (element.hasAttribute('data-instagram-processed') || 
                        element.querySelector('iframe[src*="instagram.com"]') ||
                        element.querySelector('.instagram-media iframe')) {{
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
                        if (document.querySelector('script[src*="instagram.com/embed.js"]')) {{
                            // Script loading in progress, wait for it
                            const checkInterval = setInterval(() => {{
                                if (window.instgrm && window.instgrm.Embeds) {{
                                    clearInterval(checkInterval);
                                    window.instgrm.Embeds.process(element);
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
                                    window.instgrm.Embeds.process(element);
                                }}
                                finalizeProcessing();
                            }};
                            script.onerror = () => finalizeProcessing();
                            document.head.appendChild(script);
                        }}
                    }} else if (window.instgrm && window.instgrm.Embeds) {{
                        window.instgrm.Embeds.process(element);
                        finalizeProcessing();
                    }} else {{
                        finalizeProcessing();
                    }}
                }}
                
                function hideLoadingSpinner(element) {{
                    setTimeout(() => {{
                        const loadingDiv = element.querySelector('[data-instagram-loading]');
                        if (loadingDiv) {{
                            loadingDiv.style.display = 'none';
                        }}
                    }}, 1500);
                }}
            }})();
        "#,
            embed_id_for_effect
        );

        let _ = web_sys::js_sys::eval(&js_code);
    });

    let container_class = size.container_class();
    let media_class = size.media_class();

    view! {
        <div id={embed_id_for_view} class={container_class}>
            <div
                class={media_class}
                inner_html={format!(
                    r#"<blockquote class="instagram-media" data-instgrm-captioned data-instgrm-permalink="https://www.instagram.com/p/{}/" data-instgrm-version="14" data-instgrm-payload-id="instagram-media-payload-0"><a href="https://www.instagram.com/p/{}/"></a></blockquote>"#,
                    short_code_for_html, short_code_for_html
                )}
            ></div>

            <div data-instagram-loading="true" class="instagram-embed-loading-overlay">
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
