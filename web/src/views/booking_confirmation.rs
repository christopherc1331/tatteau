use leptos::prelude::*;
use leptos_router::hooks::{use_query_map, use_navigate};
use thaw::*;

#[component]
pub fn BookingConfirmation() -> impl IntoView {
    let query = use_query_map();
    let navigate = use_navigate();

    // Extract booking ID from URL query parameters
    let booking_id = move || {
        query.get().get("booking_id")
            .and_then(|id| id.parse::<i32>().ok())
    };

    // Extract artist name from URL query parameters (optional)
    let artist_name = move || {
        query.get().get("artist_name")
            .unwrap_or_else(|| "the artist".to_string())
    };

    view! {
        <div class="booking-confirmation-container">
            <div class="booking-confirmation-content">
                <div class="booking-confirmation-header">
                    <div class="booking-confirmation-success-icon">
                        "✓"
                    </div>
                    <h1 class="booking-confirmation-title">
                        "Booking Request Submitted!"
                    </h1>
                    <p class="booking-confirmation-subtitle">
                        "Your tattoo consultation request has been successfully sent"
                    </p>
                </div>

                <div class="booking-confirmation-details">
                    {move || {
                        if let Some(id) = booking_id() {
                            view! {
                                <div class="booking-confirmation-reference">
                                    <h2 class="booking-confirmation-reference-title">
                                        "Booking Reference"
                                    </h2>
                                    <p class="booking-confirmation-reference-number">
                                        {format!("#{}", id)}
                                    </p>
                                    <p class="booking-confirmation-reference-note">
                                        "Save this reference number for your records"
                                    </p>
                                </div>
                            }.into_any()
                        } else {
                            view! {}.into_any()
                        }
                    }}

                    <div class="booking-confirmation-next-steps">
                        <h2 class="booking-confirmation-section-title">
                            "What happens next?"
                        </h2>
                        <div class="booking-confirmation-steps">
                            <div class="booking-confirmation-step">
                                <div class="booking-confirmation-step-number">"1"</div>
                                <div class="booking-confirmation-step-content">
                                    <h3>"Artist Review"</h3>
                                    <p>{format!("{} will review your questionnaire responses and appointment request", artist_name())}</p>
                                </div>
                            </div>
                            <div class="booking-confirmation-step">
                                <div class="booking-confirmation-step-number">"2"</div>
                                <div class="booking-confirmation-step-content">
                                    <h3>"Response Timeline"</h3>
                                    <p>"You'll receive a response within 24-48 hours via email"</p>
                                </div>
                            </div>
                            <div class="booking-confirmation-step">
                                <div class="booking-confirmation-step-number">"3"</div>
                                <div class="booking-confirmation-step-content">
                                    <h3>"Appointment Confirmation"</h3>
                                    <p>"Once approved, you'll receive appointment details and preparation instructions"</p>
                                </div>
                            </div>
                        </div>
                    </div>

                    <div class="booking-confirmation-contact">
                        <h2 class="booking-confirmation-section-title">
                            "Need Help?"
                        </h2>
                        <div class="booking-confirmation-contact-info">
                            <p>"If you have questions about your booking request, you can:"</p>
                            <ul class="booking-confirmation-contact-list">
                                <li>"Contact the artist directly through their profile"</li>
                                <li>"Email us at support@tatteau.com"</li>
                                <li>"Check your booking status in your account dashboard"</li>
                            </ul>
                        </div>
                    </div>
                </div>

                <div class="booking-confirmation-actions">
                    <Button
                        appearance=ButtonAppearance::Secondary
                        on_click={
                            let navigate = navigate.clone();
                            move |_| {
                                let _ = navigate("/explore", Default::default());
                            }
                        }
                    >
                        "Browse More Artists"
                    </Button>
                    <Button
                        appearance=ButtonAppearance::Primary
                        on_click={
                            let navigate = navigate.clone();
                            move |_| {
                                let _ = navigate("/", Default::default());
                            }
                        }
                    >
                        "Return Home"
                    </Button>
                </div>
            </div>
        </div>
    }
}