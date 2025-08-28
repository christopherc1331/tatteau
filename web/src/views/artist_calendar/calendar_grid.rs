use leptos::prelude::*;
use thaw::*;
use crate::db::entities::{AvailabilitySlot, BookingRequest};

#[component]
pub fn CalendarGrid(
    availability: Resource<(i32, String, String), Result<Vec<AvailabilitySlot>, leptos::prelude::ServerFnError>>,
    booking_requests: Resource<i32, Result<Vec<BookingRequest>, leptos::prelude::ServerFnError>>,
    on_date_select: impl Fn(String) + 'static + Copy,
    selected_date: RwSignal<String>,
) -> impl IntoView {
    let current_date = js_sys::Date::new_0();
    let current_year = RwSignal::new(current_date.get_full_year() as i32);
    let current_month = RwSignal::new((current_date.get_month() + 1.0) as i32);

    let month_names = [
        "January", "February", "March", "April", "May", "June",
        "July", "August", "September", "October", "November", "December"
    ];

    let get_days_in_month = move |year: i32, month: i32| -> i32 {
        let js_date = js_sys::Date::new_with_year_month_day(year as f64, month as f64, 0.0);
        js_date.get_date() as i32
    };

    let get_first_day_of_month = move |year: i32, month: i32| -> i32 {
        let js_date = js_sys::Date::new_with_year_month_day(year as f64, (month - 1) as f64, 1.0);
        js_date.get_day() as i32
    };

    let navigate_month = move |direction: i32| {
        let mut year = current_year.get();
        let mut month = current_month.get() + direction;
        
        if month > 12 {
            month = 1;
            year += 1;
        } else if month < 1 {
            month = 12;
            year -= 1;
        }
        
        current_year.set(year);
        current_month.set(month);
    };

    let is_date_available = move |day: i32| -> bool {
        if let Some(Ok(availability_slots)) = availability.get() {
            let date_str = format!("{:04}-{:02}-{:02}", current_year.get(), current_month.get(), day);
            let day_of_week = js_sys::Date::new_with_year_month_day(
                current_year.get() as f64, 
                (current_month.get() - 1) as f64, 
                day as f64
            ).get_day() as i32;

            // Check for specific date availability first
            for slot in &availability_slots {
                if let Some(ref specific_date) = slot.specific_date {
                    if specific_date == &date_str {
                        return slot.is_available;
                    }
                }
            }

            // Check recurring day availability
            for slot in &availability_slots {
                if slot.is_recurring && slot.day_of_week == Some(day_of_week) {
                    return slot.is_available;
                }
            }
        }
        false
    };

    let has_booking_request = move |day: i32| -> bool {
        if let Some(Ok(bookings)) = booking_requests.get() {
            let date_str = format!("{:04}-{:02}-{:02}", current_year.get(), current_month.get(), day);
            return bookings.iter().any(|booking| booking.requested_date == date_str);
        }
        false
    };

    let get_day_class = move |day: i32| -> String {
        let mut classes = vec!["calendar-day"];
        
        if is_date_available(day) {
            classes.push("available");
        } else {
            classes.push("blocked");
        }
        
        if has_booking_request(day) {
            classes.push("has-request");
        }
        
        let date_str = format!("{:04}-{:02}-{:02}", current_year.get(), current_month.get(), day);
        if selected_date.get() == date_str {
            classes.push("selected");
        }
        
        classes.join(" ")
    };

    view! {
        <div class="calendar-grid">
            <div class="calendar-navigation">
                <Button 
                    appearance=ButtonAppearance::Subtle
                    on_click=move |_| navigate_month(-1)
                >
                    "← Previous"
                </Button>
                
                <h2 class="current-month">
                    {move || format!("{} {}", 
                        month_names[(current_month.get() - 1) as usize], 
                        current_year.get()
                    )}
                </h2>
                
                <Button 
                    appearance=ButtonAppearance::Subtle
                    on_click=move |_| navigate_month(1)
                >
                    "Next →"
                </Button>
            </div>

            <div class="calendar-weekdays">
                <div class="weekday">"Sun"</div>
                <div class="weekday">"Mon"</div>
                <div class="weekday">"Tue"</div>
                <div class="weekday">"Wed"</div>
                <div class="weekday">"Thu"</div>
                <div class="weekday">"Fri"</div>
                <div class="weekday">"Sat"</div>
            </div>

            <div class="calendar-days">
                {move || {
                    let days_in_month = get_days_in_month(current_year.get(), current_month.get());
                    let first_day = get_first_day_of_month(current_year.get(), current_month.get());
                    
                    let mut days = Vec::new();
                    
                    // Empty cells for days before the first day of the month
                    for _ in 0..first_day {
                        days.push(view! {
                            <div class="calendar-day empty"></div>
                        }.into_any());
                    }
                    
                    // Days of the current month
                    for day in 1..=days_in_month {
                        let day_copy = day;
                        days.push(view! {
                            <div 
                                class=move || get_day_class(day_copy)
                                on:click=move |_| {
                                    let date_str = format!("{:04}-{:02}-{:02}", 
                                        current_year.get(), current_month.get(), day_copy);
                                    on_date_select(date_str);
                                }
                            >
                                <span class="day-number">{day_copy}</span>
                                {move || if has_booking_request(day_copy) {
                                    view! { <div class="request-indicator">""</div> }.into()
                                } else {
                                    view! {}.into()
                                }}
                                {move || if is_date_available(day_copy) {
                                    view! { <div class="available-indicator">""</div> }.into()
                                } else {
                                    view! {}.into()
                                }}
                            </div>
                        }.into_any());
                    }
                    
                    days
                }}
            </div>

            <div class="calendar-legend">
                <div class="legend-item">
                    <div class="legend-color available"></div>
                    <span>"Available"</span>
                </div>
                <div class="legend-item">
                    <div class="legend-color blocked"></div>
                    <span>"Blocked"</span>
                </div>
                <div class="legend-item">
                    <div class="legend-color has-request"></div>
                    <span>"Booking Request"</span>
                </div>
            </div>
        </div>
    }
}