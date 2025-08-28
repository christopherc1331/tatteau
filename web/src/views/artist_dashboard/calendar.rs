use leptos::prelude::*;
use leptos::task::spawn_local;
use thaw::*;
use serde_json;
use crate::db::entities::{BookingRequest, AvailabilitySlot, AvailabilityUpdate, RecurringRule};
use crate::server::{get_booking_requests, get_artist_availability, set_artist_availability, get_effective_availability, get_recurring_rules};


#[component]
pub fn ArtistCalendar() -> impl IntoView {
    let artist_id = 1; // For now, hardcode artist_id as 1 - same as recurring.rs
    
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
    
    // Resource for recurring rules
    let recurring_rules_resource = Resource::new_blocking(
        move || (),
        move |_| async move {
            get_recurring_rules(artist_id).await.unwrap_or_else(|_| vec![])
        }
    );
    
    // Resource for booking requests
    let booking_requests_resource = Resource::new_blocking(
        move || (),
        move |_| async move {
            get_booking_requests(artist_id).await.unwrap_or_else(|_| vec![])
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
                                let recurring_rules = recurring_rules_resource.get().unwrap_or_default();
                                let booking_requests = booking_requests_resource.get().unwrap_or_default();
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
                                    
                                    // Check if this day has explicit availability settings
                                    let explicit_availability = availability_data.iter().find(|a| {
                                        (a.specific_date.as_ref() == Some(&date_str)) || 
                                        (a.day_of_week == Some(dow) && a.is_recurring)
                                    });
                                    
                                    // Check if this day is blocked by FULL DAY recurring rules only
                                    let blocked_by_full_day_rule = check_full_day_blocking_rules(&recurring_rules, year, month, day, dow);
                                    
                                    // Get time blocks for this day
                                    let mut time_blocks = get_day_time_blocks(&recurring_rules, year, month, day, dow);
                                    
                                    // Add booking requests for this day
                                    let day_booking_requests = booking_requests.iter().filter(|req| {
                                        let req_date = req.requested_date.clone();
                                        req_date == format!("{}-{:02}-{:02}", year, month, day)
                                    }).collect::<Vec<_>>();
                                    
                                    // Add booking requests as time blocks
                                    for booking in &day_booking_requests {
                                        let status = booking.status.clone();
                                        let block_name = format!("{}", booking.client_name);
                                        let action = if status == "accepted" { "accepted".to_string() } else { "pending".to_string() };
                                        time_blocks.push((
                                            block_name,
                                            Some(booking.requested_start_time.clone()),
                                            booking.requested_end_time.clone(),
                                            action
                                        ));
                                    }
                                    
                                    let mut day_classes = "calendar-day".to_string();
                                    
                                    // Determine the day's availability status
                                    if explicit_availability.is_some() {
                                        // Has explicit override
                                        day_classes.push_str(" has-explicit");
                                        if explicit_availability.as_ref().unwrap().is_available {
                                            day_classes.push_str(" available");
                                        } else {
                                            day_classes.push_str(" blocked");
                                        }
                                    } else if blocked_by_full_day_rule {
                                        // Blocked by full-day recurring rule only
                                        day_classes.push_str(" blocked");
                                    } else {
                                        // Default available (even if it has time blocks)
                                        day_classes.push_str(" available");
                                    }
                                    
                                    days.push(view! {
                                        <div class=day_classes on:click=move |_| handle_day_click(year, month, day)>
                                            <span class="day-number">{day.to_string()}</span>
                                            {explicit_availability.as_ref().map(|_| {
                                                view! {
                                                    <div class="explicit-indicator" title="Explicitly set availability"></div>
                                                }
                                            })}
                                            
                                            // Time blocks display
                                            <div class="time-blocks">
                                                {time_blocks.into_iter().map(|(name, start_time, end_time, action)| {
                                                    let block_class = match action.as_str() {
                                                        "blocked" => "time-block blocked",
                                                        "available" => "time-block available", 
                                                        "accepted" => "time-block booking-accepted clickable",
                                                        "pending" => "time-block booking-pending clickable",
                                                        _ => "time-block available"
                                                    };
                                                    let time_display = if start_time.is_none() && end_time.is_none() {
                                                        "All Day".to_string()
                                                    } else if let (Some(start), Some(end)) = (&start_time, &end_time) {
                                                        format!("{}-{}", start, end)
                                                    } else if let Some(start) = &start_time {
                                                        format!("{}+", start)
                                                    } else {
                                                        "All Day".to_string()
                                                    };
                                                    
                                                    // Find booking ID for this time block if it's a booking request
                                                    let booking_id = if action == "accepted" || action == "pending" {
                                                        day_booking_requests.iter().find(|req| req.client_name == name).map(|req| req.id)
                                                    } else {
                                                        None
                                                    };
                                                    
                                                    let handle_block_click = move |e: web_sys::Event| {
                                                        e.stop_propagation();
                                                        if let Some(id) = booking_id {
                                                            // Navigate to booking details page
                                                            if let Some(window) = web_sys::window() {
                                                                let location = window.location();
                                                                let _ = location.set_href(&format!("/artist/dashboard/booking/{}", id));
                                                            }
                                                        }
                                                    };
                                                    
                                                    view! {
                                                        <div class=block_class 
                                                             title=format!("{}: {}", name, time_display)>
                                                            <div class="time-block-name">{name.clone()}</div>
                                                            <div class="time-block-time">{time_display.clone()}</div>
                                                        </div>
                                                    }
                                                }).collect_view()}
                                            </div>
                                        </div>
                                    }.into_any());
                                }
                                
                                days
                            }}
                        </div>
                        
                        <div class="calendar-legend">
                            <div class="legend-item">
                                <div class="legend-color available"></div>
                                <span>"Available (default or recurring rule)"</span>
                            </div>
                            <div class="legend-item">
                                <div class="legend-color blocked"></div>
                                <span>"Blocked (recurring rule)"</span>
                            </div>
                            <div class="legend-item">
                                <div class="legend-color has-explicit"></div>
                                <span>"Explicit override"</span>
                            </div>
                            <div class="legend-item">
                                <div class="legend-color booking-pending"></div>
                                <span>"Pending Request"</span>
                            </div>
                            <div class="legend-item">
                                <div class="legend-color booking-accepted"></div>
                                <span>"Accepted Request"</span>
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
                            <Suspense fallback=move || view! { <div>"Loading booking requests..."</div> }>
                                {move || {
                                    booking_requests_resource.get().map(|bookings| {
                                        if bookings.is_empty() {
                                            view! {
                                                <div class="no-bookings">"No booking requests currently"</div>
                                            }.into_any()
                                        } else {
                                            view! {
                                                <div class="bookings-container">
                                                    {bookings.into_iter().map(|booking| {
                                                        let status_class = if booking.status == "accepted" { "booking-accepted" } else { "booking-pending" };
                                                        let navigate_to_month = {
                                                            let req_date = booking.requested_date.clone();
                                                            move |_| {
                                                                // Parse date and navigate to that month
                                                                let parts: Vec<&str> = req_date.split('-').collect();
                                                                if parts.len() >= 2 {
                                                                    if let (Ok(year), Ok(month)) = (parts[0].parse::<i32>(), parts[1].parse::<u32>()) {
                                                                        current_year.set(year);
                                                                        current_month.set(month);
                                                                    }
                                                                }
                                                            }
                                                        };
                                                        view! {
                                                            <div class=format!("booking-item {}", status_class) on:click=navigate_to_month>
                                                                <div class="booking-client">{booking.client_name.clone()}</div>
                                                                <div class="booking-date">{booking.requested_date.clone()}</div>
                                                                <div class="booking-time">
                                                                    {booking.requested_start_time.clone()}
                                                                    {booking.requested_end_time.as_ref().map(|end| format!(" - {}", end)).unwrap_or_default()}
                                                                </div>
                                                                <div class="booking-tattoo">{booking.tattoo_description.clone().unwrap_or_else(|| "No description".to_string())}</div>
                                                                <div class=format!("booking-status {}", booking.status.clone())>
                                                                    {if booking.status == "accepted" { "✅ Accepted" } else { "⏳ Pending" }}
                                                                </div>
                                                            </div>
                                                        }
                                                    }).collect_view()}
                                                </div>
                                            }.into_any()
                                        }
                                    })
                                }}
                            </Suspense>
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
                                            
                                            <div class="availability-note">
                                                <p><strong>"Note:"</strong>" Days are available by default. You can override this for specific dates or use recurring rules to set patterns."</p>
                                            </div>
                                            
                                            <div class="form-group">
                                                <label>"Availability Type:"</label>
                                                <RadioGroup value=availability_mode>
                                                    <Radio value="available" />
                                                    <label>"Available (explicit override)"</label>
                                                    <Radio value="blocked" />
                                                    <label>"Blocked (explicit override)"</label>
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

// Helper function to check if a day is blocked by FULL DAY recurring rules only
fn check_full_day_blocking_rules(rules: &[RecurringRule], year: i32, month: u32, day: u32, dow: i32) -> bool {
    for rule in rules {
        // Only check active rules that block availability AND have no specific times (full day)
        if !rule.active || rule.action != "blocked" || rule.start_time.is_some() || rule.end_time.is_some() {
            continue;
        }
        
        match rule.rule_type.as_str() {
            "weekdays" => {
                // Check if this day of week is in the blocked pattern
                if let Ok(days) = serde_json::from_str::<Vec<i32>>(&rule.pattern) {
                    if days.contains(&dow) {
                        return true;
                    }
                }
            },
            "dates" => {
                // Check if this month/day matches any annual date
                let date_str = format!("{:02}/{:02}", month, day);
                if rule.pattern.contains(&date_str) {
                    return true;
                }
                // Also check full month name format
                let month_names = ["January", "February", "March", "April", "May", "June", 
                                  "July", "August", "September", "October", "November", "December"];
                if month > 0 && month <= 12 {
                    let month_day = format!("{} {}", month_names[(month - 1) as usize], day);
                    if rule.pattern.contains(&month_day) {
                        return true;
                    }
                }
            },
            "monthly" => {
                // Check if this day of month matches the pattern
                if rule.pattern.contains(&day.to_string()) {
                    return true;
                }
            },
            _ => {}
        }
    }
    false
}

// Helper function to check if a day is blocked by recurring rules
fn check_recurring_rules(rules: &[RecurringRule], year: i32, month: u32, day: u32, dow: i32) -> bool {
    for rule in rules {
        // Only check active rules that block availability
        if !rule.active || rule.action != "blocked" {
            continue;
        }
        
        match rule.rule_type.as_str() {
            "weekdays" => {
                // Check if this day of week is in the blocked pattern
                if let Ok(days) = serde_json::from_str::<Vec<i32>>(&rule.pattern) {
                    if days.contains(&dow) {
                        return true;
                    }
                }
            },
            "dates" => {
                // Check if this month/day matches any annual date
                let date_str = format!("{:02}/{:02}", month, day);
                if rule.pattern.contains(&date_str) {
                    return true;
                }
                // Also check full month name format
                let month_names = ["January", "February", "March", "April", "May", "June", 
                                  "July", "August", "September", "October", "November", "December"];
                if month > 0 && month <= 12 {
                    let month_day = format!("{} {}", month_names[(month - 1) as usize], day);
                    if rule.pattern.contains(&month_day) {
                        return true;
                    }
                }
            },
            "monthly" => {
                // Check if this day of month matches the pattern
                if rule.pattern.contains(&day.to_string()) {
                    return true;
                }
            },
            _ => {}
        }
    }
    false
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

// Helper function to get all applicable time blocks for a specific day
fn get_day_time_blocks(rules: &[RecurringRule], _year: i32, month: u32, day: u32, dow: i32) -> Vec<(String, Option<String>, Option<String>, String)> {
    let mut time_blocks = Vec::new();
    
    // Debug logging
    leptos::logging::log!("get_day_time_blocks called for month={}, day={}, dow={}", month, day, dow);
    leptos::logging::log!("Number of rules: {}", rules.len());
    
    for rule in rules {
        leptos::logging::log!("Checking rule: {} (active: {}, type: {})", rule.name, rule.active, rule.rule_type);
        
        // Only check active rules
        if !rule.active {
            leptos::logging::log!("Skipping inactive rule: {}", rule.name);
            continue;
        }
        
        let applies_to_day = match rule.rule_type.as_str() {
            "weekdays" => {
                leptos::logging::log!("Checking weekdays rule '{}' with pattern: {}", rule.name, rule.pattern);
                if let Ok(days) = serde_json::from_str::<Vec<i32>>(&rule.pattern) {
                    leptos::logging::log!("Parsed days: {:?}, checking if contains dow={}", days, dow);
                    let applies = days.contains(&dow);
                    leptos::logging::log!("Rule '{}' applies to day: {}", rule.name, applies);
                    applies
                } else {
                    leptos::logging::log!("Failed to parse pattern: {}", rule.pattern);
                    false
                }
            },
            "dates" => {
                // Check if this month/day matches any annual date
                let date_str = format!("{:02}/{:02}", month, day);
                rule.pattern.contains(&date_str) ||
                {
                    // Also check full month name format
                    let month_names = ["January", "February", "March", "April", "May", "June", 
                                      "July", "August", "September", "October", "November", "December"];
                    if month > 0 && month <= 12 {
                        let month_day = format!("{} {}", month_names[(month - 1) as usize], day);
                        rule.pattern.contains(&month_day)
                    } else {
                        false
                    }
                }
            },
            "monthly" => {
                // Check if this day of month matches the pattern
                rule.pattern.contains(&day.to_string())
            },
            _ => false
        };
        
        if applies_to_day {
            leptos::logging::log!("Adding time block for rule: {}", rule.name);
            // Add this rule as a time block
            time_blocks.push((
                rule.name.clone(),
                rule.start_time.clone(),
                rule.end_time.clone(),
                rule.action.clone()
            ));
        }
    }
    
    leptos::logging::log!("Returning {} time blocks", time_blocks.len());
    time_blocks
}