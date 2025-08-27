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
    fn container_style(&self) -> &'static str {
        match self {
            Self::Thumbnail => "position: relative; width: 100%; height: 260px; overflow: hidden;",
            Self::Small => "position: relative; min-height: 200px;",
            Self::Medium => "position: relative; min-height: 300px;",
            Self::Large => "position: relative; min-height: 500px;",
        }
    }
    
    fn media_style(&self) -> &'static str {
        match self {
            Self::Thumbnail => "background: transparent; transform: scale(0.35); transform-origin: top left; width: 285%; height: 1600px; margin-left: -92%; margin-top: -200px;",
            Self::Small => "background: transparent; max-width: 320px; margin: 0 auto;",
            Self::Medium => "background: transparent; max-width: 420px; margin: 0 auto;",
            Self::Large => "background: transparent; max-width: 540px; margin: 0 auto;",
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

    // Simple Instagram embed initialization with loading state
    Effect::new(move |_| {
        let js_code = format!(
            r#"
            (function() {{
                const embedId = '{}';
                
                const elem = document.getElementById(embedId);
                if (!elem) {{
                    setTimeout(function() {{
                        const retryElem = document.getElementById(embedId);
                        if (retryElem && window.instgrm && window.instgrm.Embeds) {{
                            window.instgrm.Embeds.process(retryElem);
                            hideLoadingSpinner(retryElem);
                        }}
                    }}, 100);
                    return;
                }}
                
                function hideLoadingSpinner(element) {{
                    setTimeout(() => {{
                        const loadingDiv = element.querySelector('[data-instagram-loading]');
                        if (loadingDiv) {{
                            loadingDiv.style.display = 'none';
                        }}
                    }}, 2000); // Hide loading after 2 seconds
                }}
                
                if (elem.hasAttribute('data-instagram-processed')) {{
                    return;
                }}
                
                // If Instagram already processed this, just hide the spinner and mark as processed
                if (elem.querySelector('.instagram-media[data-instgrm-permalink]')) {{
                    hideLoadingSpinner(elem);
                    elem.setAttribute('data-instagram-processed', 'true');
                    return;
                }}
                elem.setAttribute('data-instagram-processed', 'true');
                
                // Ensure Instagram script is loaded
                if (!window.instgrm) {{
                    const script = document.createElement('script');
                    script.src = 'https://www.instagram.com/embed.js';
                    script.async = true;
                    script.onload = () => {{
                        if (window.instgrm && window.instgrm.Embeds) {{
                            window.instgrm.Embeds.process(elem);
                            hideLoadingSpinner(elem);
                        }}
                    }};
                    document.body.appendChild(script);
                }} else if (window.instgrm && window.instgrm.Embeds) {{
                    window.instgrm.Embeds.process(elem);
                    hideLoadingSpinner(elem);
                }}
            }})();
        "#,
            embed_id_for_effect
        );

        let _ = web_sys::js_sys::eval(&js_code);
    });

    let container_style = size.container_style();
    let media_style = size.media_style();

    view! {
        <div id={embed_id_for_view} style={container_style}>
            <div
                inner_html={format!(
                    r#"<blockquote class="instagram-media" data-instgrm-captioned data-instgrm-permalink="https://www.instagram.com/p/{}/" data-instgrm-version="14" style="{}" data-instgrm-payload-id="instagram-media-payload-0"><a href="https://www.instagram.com/p/{}/"></a></blockquote>"#,
                    short_code_for_html, media_style, short_code_for_html
                )}
            ></div>

            <div data-instagram-loading="true" style="position: absolute; top: 50%; left: 50%; transform: translate(-50%, -50%); display: flex; align-items: center; justify-content: center; background: rgba(249, 250, 251, 0.95); border-radius: 8px; padding: 1rem; pointer-events: none;">
                <div style="text-align: center;">
                    <div style="display: inline-block; width: 32px; height: 32px; border: 2px solid #e5e7eb; border-top-color: #667eea; border-radius: 50%; animation: spin 1s linear infinite;">
                        <style>
                            {r#"
                            @keyframes spin {
                                to { transform: rotate(360deg); }
                            }
                            "#}
                        </style>
                    </div>
                    <div style="margin-top: 0.5rem; color: #6b7280; font-size: 0.75rem;">
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
