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
pub fn TimeBlock(
    block: TimeBlockData,
    total_blocks: usize,
) -> impl IntoView {
    let booking_id = block.booking_id;
    let action_class = match block.action.as_str() {
        "blocked" => "time-block blocked",
        "available" => "time-block available",
        "accepted" => "time-block booking-accepted clickable",
        "pending" => "time-block booking-pending clickable",
        _ => "time-block available"
    };

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
        </div>
    }
}