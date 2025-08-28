use leptos::prelude::*;
use leptos::task::spawn_local;
use thaw::*;
use crate::db::entities::{BookingRequest, AvailabilitySlot, AvailabilityUpdate};
use crate::server::{get_booking_requests, get_artist_availability, set_artist_availability};

#[component]
pub fn ArtistCalendar() -> impl IntoView {
    let artist_id = -1; // Frank Reynolds test artist
    
    let current_year = RwSignal::new(2024);
    let current_month = RwSignal::new(8); // August
    let show_sidebar = RwSignal::new(true);
    let selected_date = RwSignal::new(None::<(i32, u32, u32)>);
    let show_availability_modal = RwSignal::new(false);
    let availability_mode = RwSignal::new("available".to_string()); // "available" or "blocked"
    let start_time = RwSignal::new("09:00".to_string());
    let end_time = RwSignal::new("17:00".to_string());
    
    
    // Resource for availability data
    let availability_resource = Resource::new_blocking(
        move || (current_year.get(), current_month.get()),
        move |(year, month)| async move {
            let start_date = format!("{}-{:02}-01", year, month);
            let end_date = format!("{}-{:02}-31", year, month);
            get_artist_availability(artist_id, start_date, end_date).await.unwrap_or_else(|_| vec![])
        }
    );

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

    let handle_day_click = move |year: i32, month: u32, day: u32| {
        selected_date.set(Some((year, month, day)));
        show_availability_modal.set(true);
    };

    let handle_save_availability = move || {
        if let Some((year, month, day)) = selected_date.get() {
            spawn_local(async move {
                let date_str = format!("{}-{:02}-{:02}", year, month, day);
                let update = AvailabilityUpdate {
                    artist_id,
                    date: Some(date_str.clone()),
                    day_of_week: None,
                    start_time: start_time.get(),
                    end_time: end_time.get(),
                    is_available: availability_mode.get() == "available",
                    is_recurring: false,
                };
                
                if set_artist_availability(update).await.is_ok() {
                    show_availability_modal.set(false);
                    selected_date.set(None);
                    availability_resource.refetch();
                }
            });
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
                    <Button>
                        <a href="/artist/dashboard/recurring" style="text-decoration: none; color: inherit;">
                            "Manage Recurring Rules"
                        </a>
                    </Button>
                </div>
            </div>
            
            <div class="calendar-note">
                <p>"💡 Tip: Set your recurring availability patterns first, then use the calendar to override specific dates as needed."</p>
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
                                let availability_data = availability_resource.get().unwrap_or_default();
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
                                    let date_str = format!("{}-{:02}-{:02}", year, month, day);
                                    let dow = day_of_week(year, month, day) as i32;
                                    
                                    // Check if this day has availability settings
                                    let day_availability = availability_data.iter().find(|a| {
                                        (a.specific_date.as_ref() == Some(&date_str)) || 
                                        (a.day_of_week == Some(dow) && a.is_recurring)
                                    });
                                    
                                    let mut day_classes = "calendar-day".to_string();
                                    if let Some(avail) = day_availability {
                                        if avail.is_available {
                                            day_classes.push_str(" available");
                                        } else {
                                            day_classes.push_str(" blocked");
                                        }
                                    }
                                    
                                    days.push(view! {
                                        <div class=day_classes on:click=move |_| handle_day_click(year, month, day)>
                                            <span class="day-number">{day_str}</span>
                                            {if day_availability.is_some() {
                                                Some(view! {
                                                    <div class="availability-indicator"></div>
                                                })
                                            } else {
                                                None
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
            
            <Show when=move || show_availability_modal.get()>
                <div class="modal-backdrop" on:click=move |_| show_availability_modal.set(false)>
                    <div class="availability-modal" on:click=|e| e.stop_propagation()>
                        <div class="modal-header">
                            <h2>"Set Availability"</h2>
                            <Button on_click=move |_| show_availability_modal.set(false)>"×"</Button>
                        </div>
                        
                        <div class="modal-content">
                            {move || {
                                if let Some((year, month, day)) = selected_date.get() {
                                    view! {
                                        <div class="availability-form">
                                            <h3>{format!("Setting availability for {}/{}/{}", month, day, year)}</h3>
                                            
                                            <div class="form-group">
                                                <label>"Availability Type:"</label>
                                                <RadioGroup value=availability_mode>
                                                    <Radio value="available" />
                                                    <label>"Available"</label>
                                                    <Radio value="blocked" />
                                                    <label>"Blocked"</label>
                                                </RadioGroup>
                                            </div>
                                            
                                            
                                            <div class="time-inputs">
                                                <div class="form-group">
                                                    <label>"Start Time:"</label>
                                                    <Input value=start_time placeholder="09:00"/>
                                                </div>
                                                <div class="form-group">
                                                    <label>"End Time:"</label>
                                                    <Input value=end_time placeholder="17:00"/>
                                                </div>
                                            </div>
                                            
                                            <div class="modal-actions">
                                                <Button 
                                                    appearance=ButtonAppearance::Primary
                                                    on_click=move |_| handle_save_availability()
                                                >
                                                    "Save Availability"
                                                </Button>
                                                <Button on_click=move |_| show_availability_modal.set(false)>
                                                    "Cancel"
                                                </Button>
                                            </div>
                                        </div>
                                    }.into_any()
                                } else {
                                    view! {
                                        <div class="availability-form">
                                            <p>"No date selected"</p>
                                        </div>
                                    }.into_any()
                                }
                            }}
                        </div>
                    </div>
                </div>
            </Show>
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