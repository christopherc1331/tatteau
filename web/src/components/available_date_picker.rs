use leptos::prelude::*;
use leptos::task::spawn_local;
use thaw::*;
use crate::server::get_available_dates;

#[component]
pub fn AvailableDatePicker(
    artist_id: RwSignal<Option<i32>>,
    selected_date: RwSignal<String>,
    on_date_selected: impl Fn(String) + 'static + Copy + Send + Sync,
) -> impl IntoView {
    let current_month_offset = RwSignal::new(0i32);
    let available_dates = RwSignal::new(Vec::<String>::new());
    let is_loading = RwSignal::new(false);

    let fetch_available_dates = move || {
        if let Some(id) = artist_id.get() {
            is_loading.set(true);

            spawn_local(async move {
                // Calculate date range for current month view
                let today = get_today_date();
                let month_offset = current_month_offset.get();

                let (year, month) = calculate_month_offset(&today, month_offset);
                let view_start = format!("{:04}-{:02}-01", year, month);

                // Calculate last day of month
                let next_month = if month == 12 { 1 } else { month + 1 };
                let next_year = if month == 12 { year + 1 } else { year };
                let view_end = format!("{:04}-{:02}-01", next_year, next_month);
                let view_end = get_day_before(&view_end);

                match get_available_dates(id, view_start.clone(), view_end).await {
                    Ok(dates) => {
                        available_dates.set(dates);
                    }
                    Err(e) => {
                        leptos::logging::error!("Failed to fetch available dates: {}", e);
                        available_dates.set(vec![]);
                    }
                }
                is_loading.set(false);
            });
        }
    };

    // Fetch dates when artist changes or month changes
    Effect::new(move |_| {
        fetch_available_dates();
    });

    let generate_calendar_days = move || {
        let today = get_today_date();
        let month_offset = current_month_offset.get();

        let (year, month) = calculate_month_offset(&today, month_offset);

        // Get the number of days in the month
        let days_in_month = get_days_in_month(year, month);

        // Get the weekday of the first day of the month (0 = Sunday, 6 = Saturday)
        let first_weekday = get_first_weekday(year, month);

        let mut days = Vec::new();

        // Add empty cells for days before the first of the month
        for _ in 0..first_weekday {
            days.push(None);
        }

        // Add the days of the month
        for day in 1..=days_in_month {
            days.push(Some((year, month, day)));
        }

        days
    };

    view! {
        <div class="available-date-picker">
            <div class="date-picker-header">
                <Button
                    appearance=ButtonAppearance::Secondary
                    size=ButtonSize::Small
                    on_click=move |_| {
                        current_month_offset.update(|v| *v -= 1);
                    }
                    disabled=Signal::derive(move || current_month_offset.get() <= 0)
                >
                    "←"
                </Button>

                <div class="month-label">
                    {move || {
                        let today = get_today_date();
                        let month_offset = current_month_offset.get();
                        let (year, month) = calculate_month_offset(&today, month_offset);
                        format!("{} {}", get_month_name(month), year)
                    }}
                </div>

                <Button
                    appearance=ButtonAppearance::Secondary
                    size=ButtonSize::Small
                    on_click=move |_| {
                        current_month_offset.update(|v| *v += 1);
                    }
                    disabled=Signal::derive(move || current_month_offset.get() >= 3)
                >
                    "→"
                </Button>
            </div>

            {move || {
                if is_loading.get() {
                    view! {
                        <div class="date-picker-loading">
                            <div class="loading-spinner"></div>
                            <p>"Checking availability..."</p>
                        </div>
                    }.into_any()
                } else {
                    view! {
                        <div class="date-picker-calendar">
                            <div class="weekday-headers">
                                <div class="weekday-header">"Sun"</div>
                                <div class="weekday-header">"Mon"</div>
                                <div class="weekday-header">"Tue"</div>
                                <div class="weekday-header">"Wed"</div>
                                <div class="weekday-header">"Thu"</div>
                                <div class="weekday-header">"Fri"</div>
                                <div class="weekday-header">"Sat"</div>
                            </div>

                            <div class="calendar-days">
                                {
                                    let days = generate_calendar_days();
                                    let available = available_dates.get();
                                    let today = get_today_date();
                                    let selected = selected_date.get();

                                    days.into_iter().map(|day_opt| {
                                        if let Some((year, month, day)) = day_opt {
                                            let day_str = format!("{:04}-{:02}-{:02}", year, month, day);
                                            let is_available = available.contains(&day_str);
                                            let is_past = is_date_past(&day_str, &today);
                                            let is_selected = selected == day_str;

                                            view! {
                                                <button
                                                    class="calendar-day"
                                                    class:available=is_available
                                                    class:unavailable=!is_available
                                                    class:past=is_past
                                                    class:selected=is_selected
                                                    disabled=!is_available || is_past
                                                    on:click=move |_| {
                                                        if is_available && !is_past {
                                                            selected_date.set(day_str.clone());
                                                            on_date_selected(day_str.clone());
                                                        }
                                                    }
                                                >
                                                    {day}
                                                </button>
                                            }.into_any()
                                        } else {
                                            view! {
                                                <div class="calendar-day empty"></div>
                                            }.into_any()
                                        }
                                    }).collect::<Vec<_>>()
                                }
                            </div>
                        </div>
                    }.into_any()
                }
            }}

            <div class="date-picker-footer">
                {move || {
                    if selected_date.get().is_empty() {
                        view! {
                            <p class="no-selection">"Please select an available date"</p>
                        }.into_any()
                    } else {
                        view! {
                            <p class="selected-info">
                                "Selected: " {selected_date.get()}
                            </p>
                        }.into_any()
                    }
                }}
            </div>
        </div>
    }
}

// Helper functions for date calculations without chrono
fn get_today_date() -> String {
    // For simplicity, we'll use a reasonable default date
    // In a real implementation, this could be set via props or fetched from the server
    "2025-01-20".to_string()
}

fn calculate_month_offset(today: &str, offset: i32) -> (i32, u32) {
    let parts: Vec<&str> = today.split('-').collect();
    let year = parts[0].parse::<i32>().unwrap_or(2024);
    let month = parts[1].parse::<i32>().unwrap_or(1);

    let total_months = month + offset;
    let new_year = year + (total_months - 1) / 12;
    let new_month = ((total_months - 1) % 12 + 1) as u32;

    (new_year, new_month)
}

fn get_days_in_month(year: i32, month: u32) -> u32 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            if (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0) {
                29
            } else {
                28
            }
        }
        _ => 30,
    }
}

fn get_first_weekday(year: i32, month: u32) -> usize {
    // Zeller's congruence algorithm to find day of week
    let mut year = year;
    let mut month = month as i32;

    if month < 3 {
        month += 12;
        year -= 1;
    }

    let k = year % 100;
    let j = year / 100;
    let h = (1 + 13 * (month + 1) / 5 + k + k / 4 + j / 4 - 2 * j) % 7;

    // Convert to 0=Sunday, 1=Monday, etc.
    ((h + 6) % 7) as usize
}

fn get_month_name(month: u32) -> &'static str {
    match month {
        1 => "January",
        2 => "February",
        3 => "March",
        4 => "April",
        5 => "May",
        6 => "June",
        7 => "July",
        8 => "August",
        9 => "September",
        10 => "October",
        11 => "November",
        12 => "December",
        _ => "Unknown",
    }
}

fn get_day_before(date: &str) -> String {
    let parts: Vec<&str> = date.split('-').collect();
    let year = parts[0].parse::<i32>().unwrap_or(2024);
    let month = parts[1].parse::<u32>().unwrap_or(1);
    let day = parts[2].parse::<u32>().unwrap_or(1);

    if day > 1 {
        format!("{:04}-{:02}-{:02}", year, month, day - 1)
    } else if month > 1 {
        let prev_month = month - 1;
        let last_day = get_days_in_month(year, prev_month);
        format!("{:04}-{:02}-{:02}", year, prev_month, last_day)
    } else {
        format!("{:04}-12-31", year - 1)
    }
}

fn is_date_past(date: &str, today: &str) -> bool {
    date < today
}