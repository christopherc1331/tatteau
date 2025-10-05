use crate::server::{get_available_time_slots, TimeSlot};
use leptos::prelude::*;
use thaw::*;

#[component]
pub fn TimeSlotPicker(
    artist_id: RwSignal<Option<i32>>,
    selected_date: RwSignal<String>,
    selected_time_slot: RwSignal<Option<TimeSlot>>,
    on_slot_selected: impl Fn(TimeSlot) + 'static + Copy + Send + Sync,
) -> impl IntoView {
    let time_slots_resource = Resource::new(
        move || (artist_id.get(), selected_date.get()),
        move |(id_opt, date)| async move {
            if let Some(id) = id_opt {
                if id != 0 && !date.trim().is_empty() {
                    get_available_time_slots(id, date).await.ok().unwrap_or_default()
                } else {
                    vec![]
                }
            } else {
                vec![]
            }
        },
    );

    view! {
        <div class="time-slot-picker">
            <div class="time-slot-picker-header">
                <h4>"Available Time Slots"</h4>
                <p class="time-slot-picker-subtitle">
                    {move || {
                        let date = selected_date.get();
                        if date.trim().is_empty() {
                            "Please select a date first".to_string()
                        } else {
                            format!("Available appointments for {}", date)
                        }
                    }}
                </p>
            </div>

            <div class="time-slot-picker-content">
                <Suspense fallback=move || view! {
                    <div class="time-slot-picker-loading">
                        <div class="loading-spinner"></div>
                        <p>"Loading available time slots..."</p>
                    </div>
                }>
                    {move || {
                        let slots = time_slots_resource.get().unwrap_or_default();

                        if slots.is_empty() {
                            view! {
                                <div class="time-slot-picker-empty">
                                    <p>"No available time slots for this date."</p>
                                    <p class="time-slot-picker-suggestion">"Please try selecting a different date or contact the artist directly."</p>
                                </div>
                            }.into_any()
                        } else {
                            let available_slots: Vec<_> = slots.into_iter().filter(|slot| slot.is_available).collect();

                            if available_slots.is_empty() {
                                view! {
                                    <div class="time-slot-picker-empty">
                                        <p>"All time slots are booked for this date."</p>
                                        <p class="time-slot-picker-suggestion">"Please try selecting a different date."</p>
                                    </div>
                                }.into_any()
                            } else {
                                view! {
                                    <div class="time-slot-picker-grid">
                                        {available_slots.into_iter().map(|slot| {
                                            let slot_clone = slot.clone();
                                            let slot_clone_2 = slot.clone();

                                            view! {
                                                <Button
                                                    class="time-slot-button"
                                                    appearance=ButtonAppearance::Secondary
                                                    on_click=move |_| {
                                                        selected_time_slot.set(Some(slot_clone.clone()));
                                                        on_slot_selected(slot_clone.clone());
                                                    }
                                                >
                                                    <div class="time-slot-button-content">
                                                        <span class="time-slot-time">{format!("{} - {}", slot_clone_2.start_time, slot_clone_2.end_time)}</span>
                                                        <span class="time-slot-label">"Available"</span>
                                                    </div>
                                                </Button>
                                            }
                                        }).collect::<Vec<_>>()}
                                    </div>
                                }.into_any()
                            }
                        }
                    }}
                </Suspense>
            </div>
        </div>
    }
}