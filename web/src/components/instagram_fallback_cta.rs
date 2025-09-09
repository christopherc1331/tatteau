use leptos::prelude::*;

#[component]
pub fn InstagramFallbackCta(
    short_code: String,
    #[prop(default = "This tattoo image is available on Instagram".to_string())] message: String,
    #[prop(default = "200px".to_string())] min_height: String,
) -> impl IntoView {
    
    view! {
        <div 
            class="instagram-fallback-cta-container"
            style={format!("--min-height: {};", min_height)}
        >
            <div class="instagram-fallback-cta-icon-container">
                "📸"
            </div>
            <h3 class="instagram-fallback-cta-title">
                "View on Instagram"
            </h3>
            <p class="instagram-fallback-cta-message">
                {message}
            </p>
            <a 
                href={format!("https://www.instagram.com/p/{}/", short_code)}
                target="_blank"
                class="instagram-fallback-cta-button"
            >
                <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
                    <path d="M7.8,2H16.2C19.4,2 22,4.6 22,7.8V16.2A5.8,5.8 0 0,1 16.2,22H7.8C4.6,22 2,19.4 2,16.2V7.8A5.8,5.8 0 0,1 7.8,2M7.6,4A3.6,3.6 0 0,0 4,7.6V16.4C4,18.39 5.61,20 7.6,20H16.4A3.6,3.6 0 0,0 20,16.4V7.6C20,5.61 18.39,4 16.4,4H7.6M17.25,5.5A1.25,1.25 0 0,1 18.5,6.75A1.25,1.25 0 0,1 17.25,8A1.25,1.25 0 0,1 16,6.75A1.25,1.25 0 0,1 17.25,5.5M12,7A5,5 0 0,1 17,12A5,5 0 0,1 12,17A5,5 0 0,1 7,12A5,5 0 0,1 12,7M12,9A3,3 0 0,0 9,12A3,3 0 0,0 12,15A3,3 0 0,0 15,12A3,3 0 0,0 12,9Z"/>
                </svg>
                "View Post"
            </a>
        </div>
    }
}