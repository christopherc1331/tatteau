use leptos::prelude::*;
use leptos_router::hooks::use_navigate;

/// 404 Error page with professional design and helpful navigation
#[component]
pub fn NotFoundPage() -> impl IntoView {
    let navigate = use_navigate();

    view! {
        <div style="min-height: 100vh; background: linear-gradient(135deg, #f8fafc, #e2e8f0); display: flex; align-items: center; justify-content: center; padding: 1rem;">
            <div style="max-width: 600px; width: 100%; text-align: center;">
                
                // Animated 404 with tattoo machine drawing effect
                <div style="margin-bottom: 2rem; position: relative;">
                    <div style="font-size: 8rem; font-weight: 900; color: transparent; background: linear-gradient(135deg, #667eea, #764ba2); background-clip: text; -webkit-background-clip: text; margin: 0; line-height: 1;">
                        "404"
                    </div>
                    
                    // Tattoo machine drawing animation placeholder
                    <div style="position: absolute; top: 0; right: -20px; font-size: 3rem; animation: bounce 2s infinite;">
                        "🖋️"
                    </div>
                </div>
                
                // Main content card
                <div style="background: white; border-radius: 20px; padding: 3rem 2rem; box-shadow: 0 20px 40px rgba(0,0,0,0.1); margin-bottom: 2rem;">
                    <h1 style="font-size: 2.5rem; font-weight: 700; color: #2d3748; margin: 0 0 1rem 0;">
                        "Page Not Found"
                    </h1>
                    
                    <p style="font-size: 1.2rem; color: #4a5568; margin: 0 0 2rem 0; line-height: 1.6;">
                        "Looks like this page got lost in the ink! The page you're looking for doesn't exist or may have been moved."
                    </p>
                    
                    // Navigation buttons
                    <div style="display: grid; grid-template-columns: repeat(auto-fit, minmax(200px, 1fr)); gap: 1rem; margin-bottom: 2rem;">
                        <button 
                            on:click={
                                let navigate = navigate.clone();
                                move |_| {
                                    navigate("/", Default::default());
                                }
                            }
                            style="background: linear-gradient(135deg, #667eea, #764ba2); color: white; padding: 1rem 1.5rem; border-radius: 12px; border: none; font-size: 1rem; font-weight: 600; cursor: pointer; transition: all 0.2s ease; box-shadow: 0 4px 12px rgba(102, 126, 234, 0.3);">
                            "🏠 Go Home"
                        </button>
                        
                        <button 
                            on:click={
                                let navigate = navigate.clone();
                                move |_| {
                                    navigate("/explore", Default::default());
                                }
                            }
                            style="background: linear-gradient(135deg, #f59e0b, #d97706); color: white; padding: 1rem 1.5rem; border-radius: 12px; border: none; font-size: 1rem; font-weight: 600; cursor: pointer; transition: all 0.2s ease; box-shadow: 0 4px 12px rgba(245, 158, 11, 0.3);">
                            "🗺️ Explore Artists"
                        </button>
                        
                        <button 
                            on:click={
                                let navigate = navigate.clone();
                                move |_| {
                                    navigate("/match", Default::default());
                                }
                            }
                            style="background: transparent; color: #667eea; padding: 1rem 1.5rem; border: 2px solid #667eea; border-radius: 12px; font-size: 1rem; font-weight: 600; cursor: pointer; transition: all 0.2s ease;">
                            "🎯 Get Matched"
                        </button>
                        
                        <button 
                            on:click={
                                let navigate = navigate.clone();
                                move |_| {
                                    navigate("/styles", Default::default());
                                }
                            }
                            style="background: transparent; color: #f59e0b; padding: 1rem 1.5rem; border: 2px solid #f59e0b; border-radius: 12px; font-size: 1rem; font-weight: 600; cursor: pointer; transition: all 0.2s ease;">
                            "🎨 Browse Styles"
                        </button>
                    </div>
                </div>
                
                // Additional help section
                <div style="background: rgba(255,255,255,0.8); border-radius: 12px; padding: 1.5rem; backdrop-filter: blur(10px);">
                    <p style="font-size: 0.9rem; color: #6b7280; margin: 0 0 1rem 0;">
                        "Still can't find what you're looking for?"
                    </p>
                    <div style="display: flex; justify-content: center; gap: 1rem; flex-wrap: wrap;">
                        <a href="mailto:support@tatteau.com" 
                           style="color: #667eea; text-decoration: none; font-weight: 500; font-size: 0.9rem;">
                            "📧 Contact Support"
                        </a>
                        <span style="color: #d1d5db;">"|"</span>
                        <button 
                            on:click=move |_| {
                                if let Some(window) = web_sys::window() {
                                    let _ = window.history().unwrap().back();
                                }
                            }
                            style="background: none; border: none; color: #667eea; text-decoration: none; font-weight: 500; font-size: 0.9rem; cursor: pointer;">
                            "⬅️ Go Back"
                        </button>
                    </div>
                </div>
            </div>
        </div>
        
        // CSS animation for the tattoo machine
        <style>
            {r#"
            @keyframes bounce {
                0%, 20%, 50%, 80%, 100% {
                    transform: translateY(0) rotate(-15deg);
                }
                40% {
                    transform: translateY(-10px) rotate(-15deg);
                }
                60% {
                    transform: translateY(-5px) rotate(-15deg);
                }
            }
            "#}
        </style>
    }
}