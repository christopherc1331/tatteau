use crate::db::entities::{BusinessHours, UpdateBusinessHours};
use crate::server::{get_business_hours, update_business_hours};
use crate::utils::auth::use_authenticated_artist_id;
use crate::utils::timezone::convert_to_12_hour_format;
use leptos::ev::*;
use leptos::prelude::*;
use leptos_router::components::A;
use thaw::*;

#[component]
pub fn ArtistSettings() -> impl IntoView {
    // Get authenticated artist ID from JWT token
    let artist_id = use_authenticated_artist_id();

    let auto_reply = RwSignal::new(true);
    let availability = RwSignal::new(true);
    let base_price = RwSignal::new("150.0".to_string());
    let hourly_rate = RwSignal::new("200.0".to_string());

    // Business hours state - initialize with default values
    let business_hours = RwSignal::new(vec![
        (
            "Monday".to_string(),
            RwSignal::new("09:00".to_string()),
            RwSignal::new("18:00".to_string()),
            RwSignal::new(false),
        ),
        (
            "Tuesday".to_string(),
            RwSignal::new("09:00".to_string()),
            RwSignal::new("18:00".to_string()),
            RwSignal::new(false),
        ),
        (
            "Wednesday".to_string(),
            RwSignal::new("09:00".to_string()),
            RwSignal::new("18:00".to_string()),
            RwSignal::new(false),
        ),
        (
            "Thursday".to_string(),
            RwSignal::new("09:00".to_string()),
            RwSignal::new("18:00".to_string()),
            RwSignal::new(false),
        ),
        (
            "Friday".to_string(),
            RwSignal::new("09:00".to_string()),
            RwSignal::new("18:00".to_string()),
            RwSignal::new(false),
        ),
        (
            "Saturday".to_string(),
            RwSignal::new("".to_string()),
            RwSignal::new("".to_string()),
            RwSignal::new(true),
        ),
        (
            "Sunday".to_string(),
            RwSignal::new("".to_string()),
            RwSignal::new("".to_string()),
            RwSignal::new(true),
        ),
    ]);

    // Load existing business hours
    let business_hours_resource = Resource::new(
        move || artist_id.get(),
        |id_opt| async move {
            match id_opt {
                Some(id) => get_business_hours(id).await,
                None => Err(ServerFnError::new(
                    "No authenticated artist found".to_string(),
                )),
            }
        },
    );

    // Update business hours state when data loads
    Effect::new(move |_| {
        if let Some(Ok(hours)) = business_hours_resource.get() {
            let hours_with_signals = business_hours.get();
            for hour in hours {
                if let Some((_, start_signal, end_signal, closed_signal)) =
                    hours_with_signals.get(hour.day_of_week as usize)
                {
                    start_signal.set(hour.start_time.unwrap_or_default());
                    end_signal.set(hour.end_time.unwrap_or_default());
                    closed_signal.set(hour.is_closed);
                }
            }
        }
    });

    // Save business hours action
    let save_hours_action = Action::new(move |_: &()| async move {
        if let Some(id) = artist_id.get() {
            let hours_to_save = business_hours
                .get()
                .iter()
                .enumerate()
                .map(|(day_index, (_, start, end, is_closed))| {
                    let day_of_week = (day_index + 1) % 7; // Convert to 0=Sunday format
                    UpdateBusinessHours {
                        artist_id: id,
                        day_of_week: day_of_week as i32,
                        start_time: if is_closed.get() {
                            None
                        } else {
                            Some(start.get())
                        },
                        end_time: if is_closed.get() {
                            None
                        } else {
                            Some(end.get())
                        },
                        is_closed: is_closed.get(),
                    }
                })
                .collect::<Vec<_>>();

            update_business_hours(hours_to_save).await
        } else {
            Err(ServerFnError::new(
                "No authenticated artist found".to_string(),
            ))
        }
    });

    view! {
        <div class="artist-dashboard-container">
            <div class="dashboard-header">
                <A href="/artist/dashboard">
                    <div class="back-button">"← Back to Dashboard"</div>
                </A>
                <h1>"Artist Settings"</h1>
                <p class="dashboard-subtitle">"Configure your preferences and pricing"</p>
            </div>

            <div class="settings-grid">
                <div class="settings-card">
                    <h2>"Availability Settings"</h2>

                    <div class="setting-group">
                        <label class="setting-label">
                            <Switch
                                checked=availability
                            />
                            <span>"Accept New Bookings"</span>
                        </label>
                        <p class="setting-description">"Turn off to stop receiving new booking requests"</p>
                    </div>

                    <div class="setting-group">
                        <label class="setting-label">
                            <Switch
                                checked=auto_reply
                            />
                            <span>"Auto-Reply to Messages"</span>
                        </label>
                        <p class="setting-description">"Automatically send a response to new client messages"</p>
                    </div>
                </div>

                <div class="settings-card">
                    <h2>"Pricing Configuration"</h2>

                    <div class="setting-group">
                        <label class="setting-label">"Base Price (Small Tattoos)"</label>
                        <div class="price-input">
                            <span class="currency">"$"</span>
                            <Input
                                value=base_price
                                placeholder="150"
                            />
                        </div>
                        <p class="setting-description">"Starting price for small tattoos (2-3 inches)"</p>
                    </div>

                    <div class="setting-group">
                        <label class="setting-label">"Hourly Rate"</label>
                        <div class="price-input">
                            <span class="currency">"$"</span>
                            <Input
                                value=hourly_rate
                                placeholder="200"
                            />
                        </div>
                        <p class="setting-description">"Rate per hour for larger, custom pieces"</p>
                    </div>

                    <div class="setting-actions">
                        <button class="btn btn-primary">"Save Pricing"</button>
                    </div>
                </div>

                <div class="settings-card">
                    <h2>"Business Hours"</h2>

                    <div class="hours-grid">
                        {business_hours.get().iter().enumerate().map(|(index, (day_name, start_time, end_time, is_closed))| {
                            let day_name = day_name.clone();
                            let start_signal = start_time.clone();
                            let end_signal = end_time.clone();
                            let closed_signal = is_closed.clone();
                            let day_index = index;

                            view! {
                                <div class="day-setting">
                                    <span class="day-label">{day_name}</span>
                                    <div class="time-inputs">
                                        <Input
                                            value=start_signal
                                            placeholder="09:00"
                                            disabled=closed_signal.get()
                                        />
                                        <span>"-"</span>
                                        <Input
                                            value=end_signal
                                            placeholder="18:00"
                                            disabled=closed_signal.get()
                                        />
                                        <label class="closed-toggle">
                                            <input
                                                type="checkbox"
                                                checked=closed_signal.get()
                                                on:change=move |ev| {
                                                    let checked = event_target_checked(&ev);
                                                    closed_signal.set(checked);
                                                    if !checked {
                                                        start_signal.set("09:00".to_string());
                                                        end_signal.set("18:00".to_string());
                                                    } else {
                                                        start_signal.set("".to_string());
                                                        end_signal.set("".to_string());
                                                    }
                                                }
                                            />
                                            <span>"Closed"</span>
                                        </label>
                                    </div>
                                </div>
                            }
                        }).collect_view()}
                    </div>

                    <div class="setting-actions">
                        <button
                            class="btn btn-primary"
                            on:click=move |_| {
                                save_hours_action.dispatch(());
                            }
                            disabled=move || save_hours_action.pending().get()
                        >
                            {move || if save_hours_action.pending().get() { "Saving..." } else { "Save Hours" }}
                        </button>
                    </div>

                    {move || {
                        if let Some(Ok(_)) = save_hours_action.value().get() {
                            view! {
                                <div class="success-message">
                                    "Business hours saved successfully!"
                                </div>
                            }.into_any()
                        } else if let Some(Err(e)) = save_hours_action.value().get() {
                            view! {
                                <div class="error-message">
                                    {format!("Error saving hours: {}", e)}
                                </div>
                            }.into_any()
                        } else {
                            view! {}.into_any()
                        }
                    }}
                </div>

                <div class="settings-card">
                    <h2>"Profile Settings"</h2>

                    <div class="coming-soon-card">
                        <div class="coming-soon-icon">"⚙️"</div>
                        <h3>"Advanced Settings Coming Soon"</h3>
                        <p>"Additional configuration options in development:"</p>
                        <ul class="feature-list">
                            <li>"Portfolio management"</li>
                            <li>"Style specialization tags"</li>
                            <li>"Notification preferences"</li>
                            <li>"Payment method setup"</li>
                            <li>"Social media integration"</li>
                        </ul>
                    </div>
                </div>
            </div>
        </div>
    }
}
