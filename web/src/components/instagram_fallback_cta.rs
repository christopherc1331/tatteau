use leptos::prelude::*;

#[component]
pub fn InstagramFallbackCta(
    short_code: String,
    #[prop(optional)] message: Option<String>,
    #[prop(optional)] min_height: Option<String>,
) -> impl IntoView {
    let message = message.unwrap_or_else(|| "This tattoo image is available on Instagram".to_string());
    let min_height = min_height.unwrap_or_else(|| "200px".to_string());
    
    view! {
        <div style={format!(
            "display: flex; \
             flex-direction: column; \
             align-items: center; \
             justify-content: center; \
             background: linear-gradient(135deg, #667eea 0%, #764ba2 100%); \
             border-radius: 12px; \
             padding: 2rem 1rem; \
             text-align: center; \
             color: white; \
             min-height: {}; \
             position: relative; \
             overflow: hidden;",
            min_height
        )}>
            <div style="
                background: rgba(255, 255, 255, 0.1);
                border-radius: 50%;
                width: 60px;
                height: 60px;
                display: flex;
                align-items: center;
                justify-content: center;
                margin-bottom: 1rem;
                font-size: 24px;
            ">
                "📸"
            </div>
            <h3 style="
                margin: 0 0 0.5rem 0;
                font-size: 1.1rem;
                font-weight: 600;
                opacity: 0.95;
            ">
                "View on Instagram"
            </h3>
            <p style="
                margin: 0 0 1.5rem 0;
                opacity: 0.8;
                font-size: 0.9rem;
                line-height: 1.4;
            ">
                {message}
            </p>
            <a 
                href={format!("https://www.instagram.com/p/{}/", short_code)}
                target="_blank"
                style="
                    background: rgba(255, 255, 255, 0.2);
                    backdrop-filter: blur(10px);
                    border: 1px solid rgba(255, 255, 255, 0.3);
                    color: white;
                    padding: 0.75rem 1.5rem;
                    border-radius: 25px;
                    text-decoration: none;
                    font-weight: 600;
                    font-size: 0.9rem;
                    transition: all 0.2s ease;
                    display: inline-flex;
                    align-items: center;
                    gap: 0.5rem;
                "
                class="instagram-cta-button"
            >
                <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
                    <path d="M7.8,2H16.2C19.4,2 22,4.6 22,7.8V16.2A5.8,5.8 0 0,1 16.2,22H7.8C4.6,22 2,19.4 2,16.2V7.8A5.8,5.8 0 0,1 7.8,2M7.6,4A3.6,3.6 0 0,0 4,7.6V16.4C4,18.39 5.61,20 7.6,20H16.4A3.6,3.6 0 0,0 20,16.4V7.6C20,5.61 18.39,4 16.4,4H7.6M17.25,5.5A1.25,1.25 0 0,1 18.5,6.75A1.25,1.25 0 0,1 17.25,8A1.25,1.25 0 0,1 16,6.75A1.25,1.25 0 0,1 17.25,5.5M12,7A5,5 0 0,1 17,12A5,5 0 0,1 12,17A5,5 0 0,1 7,12A5,5 0 0,1 12,7M12,9A3,3 0 0,0 9,12A3,3 0 0,0 12,15A3,3 0 0,0 15,12A3,3 0 0,0 12,9Z"/>
                </svg>
                "View Post"
            </a>
        </div>
    }
}