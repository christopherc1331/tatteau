use leptos::prelude::*;
use thaw::*;
use crate::db::entities::BookingRequest;
use crate::server::get_booking_requests;

#[component]
pub fn ArtistCalendar() -> impl IntoView {
    let artist_id = -1; // Frank Reynolds test artist
    
    let current_year = RwSignal::new(2024);
    let current_month = RwSignal::new(8); // August
    let show_sidebar = RwSignal::new(true);

    let navigate_month = move |direction: i32| {
        if direction > 0 {
            current_month.update(|month| {
                if *month == 12 {
                    *month = 1;
                    current_year.update(|year| *year += 1);
                } else {
                    *month += 1;
                }
            });
        } else if direction < 0 {
            current_month.update(|month| {
                if *month == 1 {
                    *month = 12;
                    current_year.update(|year| *year -= 1);
                } else {
                    *month -= 1;
                }
            });
        }
    };

    let month_name = move || {
        match current_month.get() {
            1 => "January", 2 => "February", 3 => "March", 4 => "April",
            5 => "May", 6 => "June", 7 => "July", 8 => "August",
            9 => "September", 10 => "October", 11 => "November", 12 => "December",
            _ => "Unknown",
        }
    };

    view! {
        <div class="artist-calendar">
            <div class="calendar-header">
                <h1>"Calendar & Booking Management"</h1>
                <div class="header-actions">
                    <Button on_click=move |_| show_sidebar.update(|s| *s = !*s)>
                        "Toggle Sidebar"
                    </Button>
                </div>
            </div>
            
            <div class="calendar-layout">
                <div class="calendar-main">
                    <div class="calendar-grid">
                        <div class="calendar-navigation">
                            <Button on_click=move |_| navigate_month(-1)>"← Previous"</Button>
                            <h2 class="current-month">
                                {move || format!("{} {}", month_name(), current_year.get())}
                            </h2>
                            <Button on_click=move |_| navigate_month(1)>"Next →"</Button>
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
                                let year = current_year.get();
                                let month = current_month.get();
                                let days_count = days_in_month(year, month);
                                let first_day_weekday = day_of_week(year, month, 1);
                                let mut days = Vec::new();
                                
                                // Empty cells before month starts
                                for _ in 0..first_day_weekday {
                                    days.push(view! {
                                        <div class="calendar-day empty">
                                            <span class="day-number">" "</span>
                                        </div>
                                    }.into_any());
                                }
                                
                                // Days of the month
                                for day in 1..=days_count {
                                    let day_str = day.to_string();
                                    days.push(view! {
                                        <div class="calendar-day">
                                            <span class="day-number">{day_str}</span>
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
                                <span>"Has Request"</span>
                            </div>
                        </div>
                    </div>
                </div>
                
                <Show when=move || show_sidebar.get()>
                    <div class="calendar-sidebar">
                        <div class="sidebar-header">
                            <h3>"Booking Requests"</h3>
                        </div>
                        
                        <div class="booking-list">
                            <div class="no-bookings">"No booking requests currently"</div>
                        </div>
                    </div>
                </Show>
            </div>
        </div>
    }
}

// Helper functions for date calculations
fn is_leap_year(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}

fn days_in_month(year: i32, month: u32) -> u32 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => if is_leap_year(year) { 29 } else { 28 },
        _ => 30,
    }
}

fn day_of_week(year: i32, month: u32, day: u32) -> u32 {
    // Zeller's congruence algorithm
    let mut m = month as i32;
    let mut y = year;
    
    if m <= 2 {
        m += 12;
        y -= 1;
    }
    
    let k = y % 100;
    let j = y / 100;
    
    let h = (day as i32 + ((13 * (m + 1)) / 5) + k + (k / 4) + (j / 4) - 2 * j) % 7;
    
    // Convert to 0=Sunday format
    ((h + 5) % 7) as u32
}