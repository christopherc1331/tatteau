use crate::utils::timezone::{format_time_range_with_timezone, format_time_with_timezone};
use leptos::prelude::*;

#[derive(Clone, Debug)]
pub struct TimeBlockData {
    pub name: String,
    pub start_time: Option<String>,
    pub end_time: Option<String>,
    pub action: String,
    pub tattoo_description: Option<String>,
    pub booking_id: Option<i64>,
}

#[component]
pub fn TimeBlock(block: TimeBlockData, total_blocks: usize) -> impl IntoView {
    let booking_id = block.booking_id;
    let action_class = match block.action.as_str() {
        "blocked" => "time-block blocked",
        "available" => "time-block available",
        "accepted" => "time-block booking-accepted clickable",
        "pending" => "time-block booking-pending clickable",
        _ => "time-block available",
    };

    let timezone_signal = crate::utils::timezone::get_timezone_abbreviation();

    view! {
        <div class=action_class
             on:click=move |e: web_sys::MouseEvent| {
                 e.stop_propagation();
                 if let Some(id) = booking_id {
                     if let Some(window) = web_sys::window() {
                         let location = window.location();
                         let _ = location.set_href(&format!("/artist/dashboard/booking/{}", id));
                     }
                 }
             }>
            <div class="time-block-name">{block.name}</div>
            {match (&block.start_time, &block.end_time) {
                (Some(start), Some(end)) => {
                    let formatted_time = format_time_range_with_timezone(start, Some(end), timezone_signal);
                    view! {
                        <div class="time-block-time">{formatted_time}</div>
                    }.into_any()
                },
                (Some(start), None) => {
                    let formatted_time = format_time_with_timezone(start, timezone_signal);
                    view! {
                        <div class="time-block-time">{formatted_time}</div>
                    }.into_any()
                },
                _ => view! {}.into_any()
            }}
        </div>
    }
}
