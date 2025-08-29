use leptos::prelude::*;
use thaw::*;

#[component]
pub fn BookingDetails(booking_id: i32) -> impl IntoView {
    view! {
        <div class="booking-details">
            <div class="booking-details-header">
                <div class="header-content">
                    <a href="/artist/dashboard/calendar" class="back-button">
                        "← Back to Calendar"
                    </a>
                    <h1>"Booking Request Details"</h1>
                </div>
            </div>
            
            <div class="booking-details-content">
                <div class="booking-overview-card">
                    <div class="card-header">
                        <h2>"Booking Overview"</h2>
                        <div class="booking-status-badge status-pending">
                            "⏳ Pending Review"
                        </div>
                    </div>
                    
                    <div class="booking-overview-grid">
                        <div class="overview-item">
                            <label>"Booking ID"</label>
                            <span class="value">{booking_id}</span>
                        </div>
                        <div class="overview-item">
                            <label>"Client Name"</label>
                            <span class="value">"Loading..."</span>
                        </div>
                        <div class="overview-item">
                            <label>"Contact Email"</label>
                            <span class="value">"Loading..."</span>
                        </div>
                        <div class="overview-item">
                            <label>"Requested Date"</label>
                            <span class="value">"Loading..."</span>
                        </div>
                    </div>
                </div>
                
                <div class="booking-actions-card">
                    <h2>"Actions"</h2>
                    <div class="actions-grid">
                        <Button appearance=ButtonAppearance::Primary>
                            "Accept Booking"
                        </Button>
                        <Button appearance=ButtonAppearance::Secondary>
                            "Decline Booking"
                        </Button>
                    </div>
                </div>
            </div>
        </div>
    }
}