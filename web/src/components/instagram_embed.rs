use leptos::prelude::*;
use wasm_bindgen::prelude::*;

#[component]
pub fn InstagramEmbed(
    short_code: String,
) -> impl IntoView {
    let embed_id = format!("instagram-embed-{}", short_code);
    let embed_id_for_effect = embed_id.clone();
    let embed_id_for_view = embed_id.clone();
    let short_code_for_html = short_code.clone();
    let short_code_for_effect = short_code.clone();
    
    // Set up JavaScript detection for this embed instance
    Effect::new(move |_| {
        let js_code = format!(r#"
            (function() {{
                const embedId = '{}';
                const shortCode = '{}';
                
                // Check if this specific element already has tracking set up
                const elem = document.getElementById(embedId);
                if (!elem) {{
                    setTimeout(function() {{
                        const retryElem = document.getElementById(embedId);
                        if (retryElem) {{
                            initializeEmbed();
                        }}
                    }}, 100);
                    return;
                }}
                
                if (elem.hasAttribute('data-instagram-tracking')) {{
                    return; // Already tracking this specific element
                }}
                elem.setAttribute('data-instagram-tracking', 'true');
                
                function initializeEmbed() {{
                    let checkCount = 0;
                    const maxChecks = 30;
                    
                    function hideLoader() {{
                        const elem = document.getElementById(embedId);
                        if (elem) {{
                            const loadingDiv = elem.querySelector('[data-instagram-loading]');
                            if (loadingDiv) {{
                                loadingDiv.style.display = 'none';
                                console.log('✅ Loader hidden for', shortCode);
                            }}
                        }}
                    }}
                    
                    function checkLoaded() {{
                        checkCount++;
                        
                        const elem = document.getElementById(embedId);
                        if (!elem) {{
                            if (checkCount < maxChecks) {{
                                setTimeout(checkLoaded, 100);
                            }}
                            return;
                        }}
                        
                        const blockquote = elem.querySelector('.instagram-media');
                        if (!blockquote) {{
                            if (checkCount < maxChecks) {{
                                setTimeout(checkLoaded, 100);
                            }}
                            return;
                        }}
                        
                        // Simple, reliable detection: look for any meaningful change to the blockquote
                        const iframe = blockquote.querySelector('iframe');
                        const hasAnyChildElements = blockquote.children.length > 0;
                        const hasProcessedClass = blockquote.classList.contains('instagram-media-rendered') || 
                                                blockquote.hasAttribute('data-instgrm-processed');
                        
                        if (iframe || hasAnyChildElements || hasProcessedClass) {{
                            console.log('🎉 Instagram embed', shortCode, 'loaded successfully');
                            hideLoader();
                            return;
                        }}
                        
                        if (checkCount >= maxChecks) {{
                            console.log('⏰ Instagram embed', shortCode, 'timed out');
                            hideLoader();
                        }} else {{
                            setTimeout(checkLoaded, 200);
                        }}
                    }}
                    
                    // Ensure Instagram script is loaded and process
                    if (!window.instgrm) {{
                        const script = document.createElement('script');
                        script.src = 'https://www.instagram.com/embed.js';
                        script.async = true;
                        script.onload = () => {{
                            if (window.instgrm && window.instgrm.Embeds) {{
                                window.instgrm.Embeds.process();
                                setTimeout(checkLoaded, 100);
                            }}
                        }};
                        document.body.appendChild(script);
                    }} else {{
                        if (window.instgrm && window.instgrm.Embeds) {{
                            window.instgrm.Embeds.process();
                        }}
                        setTimeout(checkLoaded, 100);
                    }}
                }}
                
                initializeEmbed();
            }})();
        "#, embed_id_for_effect, short_code_for_effect);
        
        let _ = web_sys::js_sys::eval(&js_code);
    });
    
    view! {
        <div id={embed_id_for_view} style="position: relative; min-height: 400px;">
            <div
                inner_html={format!(
                    r#"<blockquote class="instagram-media" data-instgrm-captioned data-instgrm-permalink="https://www.instagram.com/p/{}/" data-instgrm-version="14" style="background: transparent;"></blockquote>"#,
                    short_code_for_html
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
    web_sys::js_sys::eval(r#"
        if (window.instgrm && window.instgrm.Embeds) {
            console.log('Processing all Instagram embeds...');
            window.instgrm.Embeds.process();
        }
    "#).ok();
}