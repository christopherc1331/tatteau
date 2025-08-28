use leptos::prelude::*;
use leptos::task::spawn_local;
use thaw::*;
use crate::db::entities::BookingRequest;
use crate::server::get_booking_request_by_id;

#[component]
pub fn BookingDetails(booking_id: i32) -> impl IntoView {
    let booking_resource = Resource::new_blocking(
        move || booking_id,
        move |id| async move {
            get_booking_request_by_id(id).await.ok()
        }
    );
    
    // TODO: Add status update functionality later
    // let update_status = Action::new(move |(booking_id, new_status): &(i32, String)| {
    //     let (id, status) = (*booking_id, new_status.clone());
    //     async move {
    //         leptos::logging::log!("Updating booking {} to status {}", id, status);
    //     }
    // });

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
            
            <Suspense fallback=move || view! { <div class="loading">"Loading booking details..."</div> }>
                {move || {
                    booking_resource.get().map(|booking_option| {
                        match booking_option {
                            Some(booking) => {
                                let status_class = if booking.status == "accepted" { "status-accepted" } else { "status-pending" };
                                let is_pending = booking.status == "pending";
                                let is_accepted = booking.status == "accepted";
                                let client_phone = booking.client_phone.clone();
                                view! {
                                    <div class="booking-details-content">
                                        <div class="booking-overview-card">
                                            <div class="card-header">
                                                <h2>"Booking Overview"</h2>
                                                <div class=format!("booking-status-badge {}", status_class)>
                                                    {if is_accepted { "✅ Accepted" } else { "⏳ Pending Review" }}
                                                </div>
                                            </div>
                                            
                                            <div class="booking-overview-grid">
                                                <div class="overview-item">
                                                    <label>"Client Name"</label>
                                                    <span class="value">{booking.client_name.clone()}</span>
                                                </div>
                                                <div class="overview-item">
                                                    <label>"Contact Email"</label>
                                                    <span class="value">{booking.client_email.clone()}</span>
                                                </div>
                                                <div class="overview-item">
                                                    <label>"Phone"</label>
                                                    <span class="value">{booking.client_phone.clone().unwrap_or_else(|| "Not provided".to_string())}</span>
                                                </div>
                                                <div class="overview-item">
                                                    <label>"Requested Date"</label>
                                                    <span class="value">{booking.requested_date.clone()}</span>
                                                </div>
                                                <div class="overview-item">
                                                    <label>"Requested Time"</label>
                                                    <span class="value">
                                                        {booking.requested_start_time.clone()}
                                                        {booking.requested_end_time.as_ref().map(|end| format!(" - {}", end)).unwrap_or_default()}
                                                    </span>
                                                </div>
                                                <div class="overview-item">
                                                    <label>"Estimated Duration"</label>
                                                    <span class="value">
                                                        {booking.requested_end_time.as_ref().map(|_| "2 hours").unwrap_or("Not specified")}
                                                    </span>
                                                </div>
                                            </div>
                                        </div>
                                        
                                        <div class="tattoo-details-card">
                                            <h2>"Tattoo Details"</h2>
                                            <div class="tattoo-details-grid">
                                                <div class="detail-item">
                                                    <label>"Description"</label>
                                                    <p class="description">{booking.tattoo_description.clone().unwrap_or_else(|| "No description provided".to_string())}</p>
                                                </div>
                                                <div class="detail-item">
                                                    <label>"Placement"</label>
                                                    <span class="value">{booking.placement.clone().unwrap_or_else(|| "Not specified".to_string())}</span>
                                                </div>
                                                <div class="detail-item">
                                                    <label>"Size (inches)"</label>
                                                    <span class="value">
                                                        {booking.size_inches.map(|s| format!("{}", s)).unwrap_or_else(|| "Not specified".to_string())}
                                                    </span>
                                                </div>
                                                <div class="detail-item">
                                                    <label>"Reference Images"</label>
                                                    <span class="value">
                                                        {if booking.reference_images.is_some() { "Provided" } else { "None provided" }}
                                                    </span>
                                                </div>
                                            </div>
                                        </div>
                                        
                                        <div class="client-message-card">
                                            <h2>"Client Message"</h2>
                                            <div class="message-content">
                                                {booking.message_from_client.clone().unwrap_or_else(|| "No additional message from client".to_string())}
                                            </div>
                                        </div>
                                        
                                        <div class="artist-response-card">
                                            <h2>"Artist Response"</h2>
                                            <div class="response-content">
                                                {booking.artist_response.clone().unwrap_or_else(|| "No response yet".to_string())}
                                                {booking.estimated_price.map(|price| view! {
                                                    <div class="estimated-price">
                                                        <strong>"Estimated Price: $"{price}</strong>
                                                    </div>
                                                })}
                                            </div>
                                        </div>
                                        
                                        <div class="booking-actions-card">
                                            <h2>"Actions"</h2>
                                            <div class="actions-grid">
                                                <Show when=move || is_pending>
                                                    <Button 
                                                        appearance=ButtonAppearance::Primary 
                                                    >
                                                        "Accept Booking (TODO)"
                                                    </Button>
                                                    <Button 
                                                        appearance=ButtonAppearance::Secondary
                                                    >
                                                        "Decline Booking (TODO)"
                                                    </Button>
                                                </Show>
                                                <Show when=move || is_accepted>
                                                    <Button appearance=ButtonAppearance::Secondary>
                                                        "Modify Booking"
                                                    </Button>
                                                    <Button appearance=ButtonAppearance::Secondary>
                                                        "Send Message"
                                                    </Button>
                                                </Show>
                                                <Button appearance=ButtonAppearance::Secondary>
                                                    <a href=format!("mailto:{}", booking.client_email) style="text-decoration: none; color: inherit;">
                                                        "Email Client"
                                                    </a>
                                                </Button>
                                                {client_phone.as_ref().map(|phone| {
                                                    let phone_number = phone.clone();
                                                    view! {
                                                    <Button appearance=ButtonAppearance::Secondary>
                                                        <a href=format!("tel:{}", phone_number) style="text-decoration: none; color: inherit;">
                                                            "Call Client"
                                                        </a>
                                                    </Button>
                                                    }
                                                })}
                                            </div>
                                        </div>
                                        
                                        <div class="booking-timeline-card">
                                            <h2>"Timeline"</h2>
                                            <div class="timeline">
                                                <div class="timeline-item">
                                                    <div class="timeline-marker"></div>
                                                    <div class="timeline-content">
                                                        <div class="timeline-title">"Booking Request Submitted"</div>
                                                        <div class="timeline-date">{booking.created_at.clone().unwrap_or_else(|| "Unknown".to_string())}</div>
                                                    </div>
                                                </div>
                                                <Show when=move || is_accepted>
                                                    <div class="timeline-item">
                                                        <div class="timeline-marker accepted"></div>
                                                        <div class="timeline-content">
                                                            <div class="timeline-title">"Booking Accepted"</div>
                                                            <div class="timeline-date">{booking.updated_at.clone().unwrap_or_else(|| "Unknown".to_string())}</div>
                                                        </div>
                                                    </div>
                                                </Show>
                                            </div>
                                        </div>
                                    </div>
                                }.into_any()
                            },
                            None => {
                                view! {
                                    <div class="booking-not-found">
                                        <h2>"Booking Not Found"</h2>
                                        <p>"The requested booking could not be found."</p>
                                        <Button>
                                            <a href="/artist/dashboard/calendar" style="text-decoration: none; color: inherit;">
                                                "Return to Calendar"
                                            </a>
                                        </Button>
                                    </div>
                                }.into_any()
                            }
                        }
                    })
                }}
            </Suspense>
        </div>
    }
}