use crate::components::instagram_embed::{InstagramEmbed, InstagramEmbedSize};
use crate::components::instagram_embed_ssr::InstagramEmbedSsr;
use leptos::prelude::*;

#[component]
pub fn InstagramDemo() -> impl IntoView {
    view! {
        <div class="instagram-demo-container">
            <div class="instagram-demo-content">
                // Page Header
                <div class="instagram-demo-header">
                    <h1 class="instagram-demo-header-title">
                        "🚀 Instagram Server-Side Rendering Demo"
                    </h1>
                    <p class="instagram-demo-header-subtitle">
                        "Comparing client-side vs server-side Instagram embed rendering"
                    </p>
                </div>

                // Demo Grid
                <div class="instagram-demo-grid">
                    // Server-Side Rendering
                    <div class="instagram-demo-card">
                        <div class="instagram-demo-card-header">
                            <div class="instagram-demo-badge-new">
                                "NEW"
                            </div>
                            <h2 class="instagram-demo-card-title">
                                "Server-Side Rendering"
                            </h2>
                        </div>
                        <p class="instagram-demo-card-description">
                            "Fetched via Instagram oEmbed API on the server, then sent as HTML to the client. Faster, more reliable, SEO-friendly."
                        </p>
                        <div class="instagram-demo-embed-container">
                            <InstagramEmbedSsr
                                short_code="DLbEyS3uN7k".to_string()
                            />
                        </div>

                        <div class="instagram-demo-benefits">
                            <h4 class="instagram-demo-benefits-title">
                                "✅ Benefits:"
                            </h4>
                            <ul class="instagram-demo-benefits-list">
                                <li>"No client-side JavaScript dependencies"</li>
                                <li>"Faster initial render"</li>
                                <li>"SEO-friendly content"</li>
                                <li>"Graceful error handling with fallback UI"</li>
                                <li>"Works even if Instagram scripts fail"</li>
                            </ul>
                        </div>
                    </div>

                    // Client-Side Rendering
                    <div class="instagram-demo-card">
                        <div class="instagram-demo-card-header">
                            <div class="instagram-demo-badge-legacy">
                                "LEGACY"
                            </div>
                            <h2 class="instagram-demo-card-title">
                                "Client-Side Rendering"
                            </h2>
                        </div>
                        <p class="instagram-demo-card-description">
                            "Traditional iframe approach with client-side JavaScript. Slower, less reliable, but currently used throughout the app."
                        </p>
                        <div class="instagram-demo-embed-container">
                            <InstagramEmbed
                                short_code="DLbEyS3uN7k".to_string()
                                size=InstagramEmbedSize::Medium
                            />
                        </div>

                        <div class="instagram-demo-limitations">
                            <h4 class="instagram-demo-limitations-title">
                                "⚠️ Limitations:"
                            </h4>
                            <ul class="instagram-demo-limitations-list">
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
                <div class="instagram-demo-card instagram-demo-technical-section">
                    <h2 class="instagram-demo-technical-title">
                        "🔧 Technical Implementation"
                    </h2>

                    <div class="instagram-demo-technical-grid">
                        <div>
                            <h3 class="instagram-demo-process-title">
                                "Server-Side Process:"
                            </h3>
                            <ol class="instagram-demo-process-list">
                                <li>"Leptos server function calls Instagram oEmbed API"</li>
                                <li>"Instagram returns JSON with complete HTML embed"</li>
                                <li>"Server renders HTML and sends to client"</li>
                                <li>"Client displays immediately with no additional requests"</li>
                                <li>"Errors gracefully fall back to beautiful CTA component"</li>
                            </ol>
                        </div>

                        <div>
                            <h3 class="instagram-demo-process-title">
                                "Client-Side Process:"
                            </h3>
                            <ol class="instagram-demo-process-list">
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
                <div class="instagram-demo-card">
                    <h2 class="instagram-demo-test-title">
                        "🧪 Test Cases"
                    </h2>
                    <p class="instagram-demo-test-description">
                        "The demo post (DLbEyS3uN7k) is a public Instagram post that should load successfully with both methods.
                        For broken posts, the server-side version will show the fallback CTA, while the client-side version may show loading states indefinitely."
                    </p>

                    <div class="instagram-demo-next-steps">
                        <h4 class="instagram-demo-next-steps-title">
                            "📝 Next Steps:"
                        </h4>
                        <p class="instagram-demo-next-steps-content">
                            "If server-side rendering performs well, we can gradually replace the client-side InstagramEmbed
                            components throughout the app with InstagramEmbedSsr for better performance and reliability."
                        </p>
                    </div>
                </div>
            </div>
        </div>
    }
}
