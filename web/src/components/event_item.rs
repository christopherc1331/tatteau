use leptos::prelude::*;
use crate::utils::timezone::{format_time_with_timezone, format_time_range_with_timezone};

#[derive(Clone, Debug)]
pub struct EventItemData {
    pub name: String,
    pub start_time: Option<String>,
    pub end_time: Option<String>,
    pub action: String,
    pub tattoo_description: Option<String>,
    pub booking_id: Option<i64>,
}

#[component]
pub fn EventItem(
    event: EventItemData,
    timezone_signal: ReadSignal<String>,
) -> impl IntoView {
    let time_str = if event.start_time.is_none() && event.end_time.is_none() {
        format_time_with_timezone("All Day", timezone_signal)
    } else if let (Some(start), Some(end)) = (&event.start_time, &event.end_time) {
        format_time_range_with_timezone(start, Some(end), timezone_signal)
    } else if let Some(start) = &event.start_time {
        format_time_with_timezone(&format!("{}+", start), timezone_signal)
    } else {
        format_time_with_timezone("All Day", timezone_signal)
    };
    
    let booking_id = event.booking_id;
    let event_class = match event.action.as_str() {
        "blocked" => "event-item-container event-item-container--blocked",
        "available" => "event-item-container event-item-container--available", 
        "accepted" => "event-item-container event-item-container--booking-accepted event-item-container--clickable",
        "pending" => "event-item-container event-item-container--booking-pending event-item-container--clickable",
        _ => "event-item-container"
    };

    view! {
        <div class=event_class 
             on:click=move |_| {
                 if let Some(id) = booking_id {
                     if let Some(window) = web_sys::window() {
                         let location = window.location();
                         let _ = location.set_href(&format!("/artist/dashboard/booking/{}", id));
                     }
                 }
             }>
            <div class="event-item-header">
                <div class="event-item-name">{event.name.clone()}</div>
                <div class="event-item-time">{time_str}</div>
            </div>
            {if let Some(desc) = &event.tattoo_description {
                view! {
                    <div class="event-item-tattoo">{desc.clone()}</div>
                }.into_any()
            } else {
                view! {}.into_any()
            }}
        </div>
    }
}