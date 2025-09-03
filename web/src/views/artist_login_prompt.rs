use leptos::prelude::*;
use leptos_router::hooks::use_navigate;

/// Page shown when unauthorized users try to access artist-only areas
#[component]
pub fn ArtistLoginPrompt() -> impl IntoView {
    let navigate = use_navigate();

    view! {
        <div style="min-height: 100vh; background: linear-gradient(135deg, #667eea, #764ba2); display: flex; align-items: center; justify-content: center; padding: 1rem;">
            <div style="max-width: 500px; width: 100%; background: white; border-radius: 20px; padding: 3rem 2rem; text-align: center; box-shadow: 0 20px 40px rgba(0,0,0,0.1);">
                // Icon and title
                <div style="font-size: 4rem; margin-bottom: 1rem;">
                    "🎨"
                </div>
                <h1 style="font-size: 2rem; font-weight: 700; color: #2d3748; margin: 0 0 1rem 0;">
                    "Artist Area"
                </h1>
                
                // Main message
                <p style="font-size: 1.1rem; color: #4a5568; margin: 0 0 2rem 0; line-height: 1.6;">
                    "This area is for tattoo artists only. Please log in with your artist account to access your dashboard, bookings, and client management tools."
                </p>
                
                // Action buttons
                <div style="display: flex; flex-direction: column; gap: 1rem; margin-bottom: 2rem;">
                    <button 
                        on:click={
                            let navigate = navigate.clone();
                            move |_| {
                                navigate("/login?user_type=artist", Default::default());
                            }
                        }
                        style="background: linear-gradient(135deg, #667eea, #764ba2); color: white; padding: 1rem 2rem; border-radius: 12px; border: none; font-size: 1.1rem; font-weight: 600; cursor: pointer; transition: all 0.2s ease; box-shadow: 0 4px 12px rgba(102, 126, 234, 0.3);">
                        "🔐 Artist Login"
                    </button>
                    
                    <button 
                        on:click={
                            let navigate = navigate.clone();
                            move |_| {
                                navigate("/signup?user_type=artist", Default::default());
                            }
                        }
                        style="background: transparent; color: #667eea; padding: 1rem 2rem; border: 2px solid #667eea; border-radius: 12px; font-size: 1rem; font-weight: 600; cursor: pointer; transition: all 0.2s ease;">
                        "✨ Create Artist Account"
                    </button>
                </div>
                
                // Additional info
                <div style="border-top: 1px solid #e2e8f0; padding-top: 2rem;">
                    <p style="font-size: 0.9rem; color: #6b7280; margin: 0 0 1rem 0;">
                        "Looking for a tattoo?"
                    </p>
                    <button 
                        on:click={
                            let navigate = navigate.clone();
                            move |_| {
                                navigate("/", Default::default());
                            }
                        }
                        style="background: transparent; color: #f59e0b; padding: 0.5rem 1rem; border: 1px solid #f59e0b; border-radius: 8px; font-size: 0.9rem; cursor: pointer; transition: all 0.2s ease;">
                        "🏠 Back to Homepage"
                    </button>
                </div>
            </div>
        </div>
    }
}