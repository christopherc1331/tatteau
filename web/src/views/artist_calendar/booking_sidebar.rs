use leptos::prelude::*;
use thaw::*;
use crate::db::entities::BookingRequest;

#[component]
pub fn BookingSidebar(
    booking_requests: Resource<i32, Result<Vec<BookingRequest>, leptos::prelude::ServerFnError>>,
    on_booking_select: impl Fn(BookingRequest) + 'static + Copy,
) -> impl IntoView {
    let filter_status = RwSignal::new("pending".to_string());

    let filtered_bookings = move || {
        booking_requests.get().unwrap_or_else(|| Ok(vec![])).unwrap_or_default()
            .into_iter()
            .filter(|booking| {
                if filter_status.get() == "all" {
                    true
                } else {
                    booking.status == filter_status.get()
                }
            })
            .collect::<Vec<_>>()
    };

    let get_status_badge_class = |status: &str| -> &'static str {
        match status {
            "pending" => "status-badge pending",
            "approved" => "status-badge approved", 
            "declined" => "status-badge declined",
            _ => "status-badge"
        }
    };

    view! {
        <div class="booking-sidebar">
            <div class="sidebar-header">
                <h3>"Booking Requests"</h3>
                <div class="status-filter">
                    <Select 
                        value=filter_status
                        placeholder="Filter by status"
                    >
                        <SelectOption value="all">"All Requests"</SelectOption>
                        <SelectOption value="pending">"Pending"</SelectOption>
                        <SelectOption value="approved">"Approved"</SelectOption>
                        <SelectOption value="declined">"Declined"</SelectOption>
                    </Select>
                </div>
            </div>

            <div class="booking-list">
                <Suspense fallback=move || view! { 
                    <div class="loading-bookings">
                        <Spin size=SpinSize::Medium />
                        <span>"Loading booking requests..."</span>
                    </div>
                }>
                    {move || {
                        let bookings = filtered_bookings();
                        if bookings.is_empty() {
                            view! {
                                <div class="no-bookings">
                                    <p>"No booking requests found."</p>
                                </div>
                            }.into()
                        } else {
                            bookings.into_iter().map(|booking| {
                                let booking_copy = booking.clone();
                                let booking_copy2 = booking.clone();
                                view! {
                                    <div 
                                        class="booking-card"
                                        on:click=move |_| on_booking_select(booking_copy.clone())
                                    >
                                        <div class="booking-header">
                                            <h4>{&booking_copy2.client_name}</h4>
                                            <span class=get_status_badge_class(&booking_copy2.status)>
                                                {&booking_copy2.status}
                                            </span>
                                        </div>
                                        
                                        <div class="booking-details">
                                            <div class="booking-date">
                                                <strong>{&booking_copy2.requested_date}</strong>
                                                <span>" at "</span>
                                                <strong>{&booking_copy2.requested_start_time}</strong>
                                            </div>
                                            
                                            {booking_copy2.tattoo_description.as_ref().map(|desc| {
                                                view! {
                                                    <p class="tattoo-description">{desc}</p>
                                                }
                                            })}
                                            
                                            {booking_copy2.placement.as_ref().map(|placement| {
                                                view! {
                                                    <div class="placement">
                                                        <span class="label">"Placement: "</span>
                                                        <span>{placement}</span>
                                                    </div>
                                                }
                                            })}
                                            
                                            {booking_copy2.size_inches.map(|size| {
                                                view! {
                                                    <div class="size">
                                                        <span class="label">"Size: "</span>
                                                        <span>{format!("{:.1}\"", size)}</span>
                                                    </div>
                                                }
                                            })}
                                        </div>
                                        
                                        {booking_copy2.message_from_client.as_ref().map(|message| {
                                            view! {
                                                <div class="client-message">
                                                    <p>{message}</p>
                                                </div>
                                            }
                                        })}
                                        
                                        <div class="booking-actions">
                                            <Button 
                                                appearance=ButtonAppearance::Primary
                                                size=ButtonSize::Small
                                            >
                                                "View Details"
                                            </Button>
                                        </div>
                                    </div>
                                }
                            }).collect::<Vec<_>>().into()
                        }
                    }}
                </Suspense>
            </div>
        </div>
    }
}