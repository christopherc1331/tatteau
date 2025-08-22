use leptos::prelude::*;
use wasm_bindgen::prelude::*;

#[component]
pub fn InstagramEmbed(
    short_code: String,
) -> impl IntoView {
    let embed_id = format!("instagram-embed-{}", short_code);
    let embed_id_for_js = embed_id.clone();
    let short_code_for_html = short_code.clone();
    
    // Set up JavaScript to handle loading detection with direct DOM manipulation
    Effect::new(move |_| {
        let js_code = format!(r#"
            (function() {{
                const embedId = '{}';
                const shortCode = '{}';
                let checkCount = 0;
                const maxChecks = 50; // 5 seconds max
                
                function hideLoader() {{
                    const elem = document.getElementById(embedId);
                    if (elem) {{
                        const loadingDiv = elem.querySelector('[data-instagram-loading]');
                        if (loadingDiv && loadingDiv.style.display !== 'none') {{
                            console.log('Hiding loader for', shortCode);
                            loadingDiv.style.display = 'none';
                        }}
                    }}
                }}
                
                function checkLoaded() {{
                    const elem = document.getElementById(embedId);
                    if (!elem) {{
                        if (checkCount < maxChecks) {{
                            checkCount++;
                            setTimeout(checkLoaded, 100);
                        }} else {{
                            hideLoader();
                        }}
                        return;
                    }}
                    
                    const blockquote = elem.querySelector('.instagram-media');
                    if (!blockquote) {{
                        if (checkCount < maxChecks) {{
                            checkCount++;
                            setTimeout(checkLoaded, 100);
                        }} else {{
                            hideLoader();
                        }}
                        return;
                    }}
                    
                    // Check if Instagram has processed this embed
                    const iframe = blockquote.querySelector('iframe');
                    const hasProcessedAttr = blockquote.hasAttribute('data-instgrm-processed');
                    const hasInstagramContent = blockquote.innerHTML.length > 200; // Instagram adds content
                    const hasVisibleContent = blockquote.offsetHeight > 50;
                    
                    if (iframe || hasProcessedAttr || (hasInstagramContent && hasVisibleContent)) {{
                        console.log('Instagram embed', shortCode, 'loaded - iframe:', !!iframe, 'processed:', hasProcessedAttr, 'content:', hasInstagramContent, 'visible:', hasVisibleContent);
                        hideLoader();
                        return;
                    }}
                    
                    // Force process Instagram embeds every few checks
                    if (checkCount % 10 === 0 && window.instgrm && window.instgrm.Embeds) {{
                        console.log('Force processing Instagram embeds (attempt', checkCount, ')');
                        window.instgrm.Embeds.process();
                    }}
                    
                    if (checkCount >= maxChecks) {{
                        console.log('Instagram embed', shortCode, 'timed out after', checkCount, 'attempts');
                        hideLoader();
                    }} else {{
                        checkCount++;
                        setTimeout(checkLoaded, 100);
                    }}
                }}
                
                // Ensure Instagram script is loaded
                if (!window.instgrm) {{
                    if (!document.querySelector('script[src*="instagram.com/embed.js"]')) {{
                        console.log('Loading Instagram script for', shortCode);
                        const script = document.createElement('script');
                        script.src = 'https://www.instagram.com/embed.js';
                        script.async = true;
                        script.onload = () => {{
                            console.log('Instagram script loaded, processing embeds');
                            setTimeout(() => {{
                                if (window.instgrm && window.instgrm.Embeds) {{
                                    window.instgrm.Embeds.process();
                                }}
                            }}, 100);
                        }};
                        document.body.appendChild(script);
                    }}
                }} else {{
                    // Script already loaded, process immediately
                    setTimeout(() => {{
                        if (window.instgrm && window.instgrm.Embeds) {{
                            window.instgrm.Embeds.process();
                        }}
                    }}, 100);
                }}
                
                // Start checking
                setTimeout(checkLoaded, 300);
            }})();
        "#, embed_id_for_js, short_code);
        
        let _ = web_sys::js_sys::eval(&js_code);
    });
    
    view! {
        <div id={embed_id.clone()} style="position: relative; min-height: 400px;">
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