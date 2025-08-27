use leptos::prelude::*;
use crate::components::instagram_embed_ssr::InstagramEmbedSsr;
use crate::components::instagram_embed::{InstagramEmbed, InstagramEmbedSize};

#[component]
pub fn InstagramDemo() -> impl IntoView {
    view! {
        <div style="min-height: 100vh; background: #f8fafc; padding: 2rem 0;">
            <div style="max-width: 1200px; margin: 0 auto; padding: 0 1rem;">
                // Page Header
                <div style="background: linear-gradient(135deg, #667eea, #764ba2); color: white; padding: 3rem 2rem; border-radius: 16px; margin-bottom: 3rem; text-align: center;">
                    <h1 style="font-size: 2.5rem; font-weight: 700; margin: 0 0 1rem 0;">
                        "🚀 Instagram Server-Side Rendering Demo"
                    </h1>
                    <p style="font-size: 1.1rem; opacity: 0.9; margin: 0; line-height: 1.5;">
                        "Comparing client-side vs server-side Instagram embed rendering"
                    </p>
                </div>

                // Demo Grid
                <div style="display: grid; grid-template-columns: 1fr 1fr; gap: 2rem; margin-bottom: 3rem;">
                    // Server-Side Rendering
                    <div style="background: white; border-radius: 16px; padding: 2rem; box-shadow: 0 4px 16px rgba(0,0,0,0.08);">
                        <div style="display: flex; align-items: center; gap: 0.5rem; margin-bottom: 1.5rem;">
                            <div style="
                                background: #10b981;
                                color: white;
                                padding: 0.25rem 0.75rem;
                                border-radius: 20px;
                                font-size: 0.8rem;
                                font-weight: 600;
                            ">
                                "NEW"
                            </div>
                            <h2 style="font-size: 1.5rem; font-weight: 700; color: #1a202c; margin: 0;">
                                "Server-Side Rendering"
                            </h2>
                        </div>
                        <p style="color: #4a5568; margin: 0 0 2rem 0; line-height: 1.6;">
                            "Fetched via Instagram oEmbed API on the server, then sent as HTML to the client. Faster, more reliable, SEO-friendly."
                        </p>
                        <div style="border: 2px solid #e2e8f0; border-radius: 12px; padding: 1rem;">
                            <InstagramEmbedSsr 
                                short_code="DLbEyS3uN7k".to_string() 
                            />
                        </div>
                        
                        <div style="margin-top: 1.5rem; padding: 1rem; background: #f0fdf4; border: 1px solid #bbf7d0; border-radius: 8px;">
                            <h4 style="color: #166534; margin: 0 0 0.5rem 0; font-size: 0.9rem; font-weight: 600;">
                                "✅ Benefits:"
                            </h4>
                            <ul style="color: #166534; margin: 0; padding-left: 1rem; font-size: 0.8rem; line-height: 1.4;">
                                <li>"No client-side JavaScript dependencies"</li>
                                <li>"Faster initial render"</li>
                                <li>"SEO-friendly content"</li>
                                <li>"Graceful error handling with fallback UI"</li>
                                <li>"Works even if Instagram scripts fail"</li>
                            </ul>
                        </div>
                    </div>

                    // Client-Side Rendering
                    <div style="background: white; border-radius: 16px; padding: 2rem; box-shadow: 0 4px 16px rgba(0,0,0,0.08);">
                        <div style="display: flex; align-items: center; gap: 0.5rem; margin-bottom: 1.5rem;">
                            <div style="
                                background: #f59e0b;
                                color: white;
                                padding: 0.25rem 0.75rem;
                                border-radius: 20px;
                                font-size: 0.8rem;
                                font-weight: 600;
                            ">
                                "LEGACY"
                            </div>
                            <h2 style="font-size: 1.5rem; font-weight: 700; color: #1a202c; margin: 0;">
                                "Client-Side Rendering"
                            </h2>
                        </div>
                        <p style="color: #4a5568; margin: 0 0 2rem 0; line-height: 1.6;">
                            "Traditional iframe approach with client-side JavaScript. Slower, less reliable, but currently used throughout the app."
                        </p>
                        <div style="border: 2px solid #e2e8f0; border-radius: 12px; padding: 1rem;">
                            <InstagramEmbed 
                                short_code="DLbEyS3uN7k".to_string() 
                                size=InstagramEmbedSize::Medium
                            />
                        </div>
                        
                        <div style="margin-top: 1.5rem; padding: 1rem; background: #fef3c7; border: 1px solid #fde68a; border-radius: 8px;">
                            <h4 style="color: #92400e; margin: 0 0 0.5rem 0; font-size: 0.9rem; font-weight: 600;">
                                "⚠️ Limitations:"
                            </h4>
                            <ul style="color: #92400e; margin: 0; padding-left: 1rem; font-size: 0.8rem; line-height: 1.4;">
                                <li>"Requires Instagram's embed.js to load"</li>
                                <li>"Slower initial render"</li>
                                <li>"CORS limitations for error detection"</li>
                                <li>"Complex client-side error handling"</li>
                                <li>"Not SEO-friendly"</li>
                            </ul>
                        </div>
                    </div>
                </div>

                // Technical Details
                <div style="background: white; border-radius: 16px; padding: 2rem; box-shadow: 0 4px 16px rgba(0,0,0,0.08); margin-bottom: 3rem;">
                    <h2 style="font-size: 1.75rem; font-weight: 700; color: #1a202c; margin: 0 0 1.5rem 0;">
                        "🔧 Technical Implementation"
                    </h2>
                    
                    <div style="display: grid; grid-template-columns: 1fr 1fr; gap: 2rem;">
                        <div>
                            <h3 style="font-size: 1.2rem; font-weight: 600; color: #374151; margin: 0 0 1rem 0;">
                                "Server-Side Process:"
                            </h3>
                            <ol style="color: #4a5568; margin: 0; padding-left: 1.5rem; line-height: 1.6;">
                                <li>"Leptos server function calls Instagram oEmbed API"</li>
                                <li>"Instagram returns JSON with complete HTML embed"</li>
                                <li>"Server renders HTML and sends to client"</li>
                                <li>"Client displays immediately with no additional requests"</li>
                                <li>"Errors gracefully fall back to beautiful CTA component"</li>
                            </ol>
                        </div>
                        
                        <div>
                            <h3 style="font-size: 1.2rem; font-weight: 600; color: #374151; margin: 0 0 1rem 0;">
                                "Client-Side Process:"
                            </h3>
                            <ol style="color: #4a5568; margin: 0; padding-left: 1.5rem; line-height: 1.6;">
                                <li>"Client renders placeholder iframe"</li>
                                <li>"JavaScript loads Instagram embed.js script"</li>
                                <li>"Instagram script processes all embeds on page"</li>
                                <li>"Multiple network requests and JavaScript execution"</li>
                                <li>"Complex error detection with CORS limitations"</li>
                            </ol>
                        </div>
                    </div>
                </div>

                // Test Cases
                <div style="background: white; border-radius: 16px; padding: 2rem; box-shadow: 0 4px 16px rgba(0,0,0,0.08);">
                    <h2 style="font-size: 1.75rem; font-weight: 700; color: #1a202c; margin: 0 0 1.5rem 0;">
                        "🧪 Test Cases"
                    </h2>
                    <p style="color: #4a5568; margin: 0 0 2rem 0; line-height: 1.6;">
                        "The demo post (DLbEyS3uN7k) is a public Instagram post that should load successfully with both methods. 
                        For broken posts, the server-side version will show the fallback CTA, while the client-side version may show loading states indefinitely."
                    </p>
                    
                    <div style="background: #f1f5f9; border-radius: 8px; padding: 1.5rem; border-left: 4px solid #3b82f6;">
                        <h4 style="color: #1e40af; margin: 0 0 0.5rem 0; font-size: 1rem; font-weight: 600;">
                            "📝 Next Steps:"
                        </h4>
                        <p style="color: #1e40af; margin: 0; line-height: 1.5; font-size: 0.9rem;">
                            "If server-side rendering performs well, we can gradually replace the client-side InstagramEmbed 
                            components throughout the app with InstagramEmbedSsr for better performance and reliability."
                        </p>
                    </div>
                </div>
            </div>
        </div>
    }
}