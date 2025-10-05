use crate::components::{AvailableDatePicker, MultiStepQuestionnaire, TimeSlotPicker};
use crate::db::entities::{ClientQuestionnaireSubmission, QuestionnaireResponse};
use crate::server::{
    fetch_artist_data, get_artist_questionnaire_form, submit_booking_request,
    submit_questionnaire_responses, NewBookingRequest, TimeSlot,
};
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::hooks::use_navigate;
use std::collections::HashMap;
use thaw::*;

#[component]
pub fn ClientBookingModal(
    show: RwSignal<bool>,
    artist_id: RwSignal<Option<i32>>,
    on_close: impl Fn() + 'static + Copy + Send + Sync,
) -> impl IntoView {
    // Appointment form state (only time/date info - no contact details for authenticated users)
    let requested_date = RwSignal::new(String::new());
    let selected_time_slot = RwSignal::new(None::<TimeSlot>);
    let additional_message = RwSignal::new(String::new());

    // Questionnaire state
    let questionnaire_responses = RwSignal::new(HashMap::<i32, String>::new());

    // UI state
    let current_step = RwSignal::new(1); // 1: questionnaire, 2: appointment details
    let questionnaire_completed = RwSignal::new(false);
    let is_submitting = RwSignal::new(false);
    let submission_error = RwSignal::new(None::<String>);
    let booking_id = RwSignal::new(None::<i32>);

    // Fetch artist data when modal opens
    let artist_resource = Resource::new(
        move || artist_id.get(),
        move |id_opt| async move {
            if let Some(id) = id_opt {
                if id != 0 {
                    fetch_artist_data(id).await.ok()
                } else {
                    None
                }
            } else {
                None
            }
        },
    );

    // Fetch artist questionnaire when modal opens
    let questionnaire_resource = Resource::new(
        move || artist_id.get(),
        move |id_opt| async move {
            if let Some(id) = id_opt {
                if id != 0 {
                    get_artist_questionnaire_form(id).await.ok()
                } else {
                    None
                }
            } else {
                None
            }
        },
    );

    let submit_booking = create_action(move |request: &NewBookingRequest| {
        let request = request.clone();
        async move { submit_booking_request(request).await }
    });

    let handle_submit = move || {
        if let Some(id) = artist_id.get() {
            is_submitting.set(true);
            submission_error.set(None);

            let slot = selected_time_slot.get();
            let (start_time, end_time) = if let Some(time_slot) = slot {
                (time_slot.start_time.clone(), Some(time_slot.end_time.clone()))
            } else {
                (String::new(), None)
            };

            // For authenticated users, use placeholder values for required fields
            // Real user data should come from authentication context
            let request = NewBookingRequest {
                artist_id: id,
                client_name: "Authenticated User".to_string(), // TODO: Get from auth context
                client_email: "user@example.com".to_string(),  // TODO: Get from auth context
                client_phone: None,
                tattoo_description: None, // Collected via questionnaire
                placement: None,          // Collected via questionnaire
                size_inches: None,        // Collected via questionnaire
                requested_date: requested_date.get(),
                requested_start_time: start_time,
                requested_end_time: end_time,
                message_from_client: if additional_message.get().trim().is_empty() {
                    None
                } else {
                    Some(additional_message.get())
                },
            };

            submit_booking.dispatch(request);
        }
    };

    let navigate = use_navigate();

    // Handle submission result
    Effect::new(move |_| {
        if let Some(result) = submit_booking.value().get() {
            match result {
                Ok(id) => {
                    booking_id.set(Some(id));

                    // Submit questionnaire responses if we have any
                    let responses = questionnaire_responses.get();
                    if !responses.is_empty() {
                        if let Some(booking_id_val) = booking_id.get() {
                            let submission = ClientQuestionnaireSubmission {
                                booking_request_id: booking_id_val,
                                responses: responses
                                    .into_iter()
                                    .map(|(question_id, response)| QuestionnaireResponse {
                                        question_id,
                                        response_text: Some(response.clone()),
                                        response_data: None,
                                    })
                                    .collect(),
                            };

                            spawn_local(async move {
                                let _ = submit_questionnaire_responses(submission).await;
                            });
                        }
                    }

                    // Get artist name from the resource for the confirmation page
                    let artist_name = artist_resource.get()
                        .and_then(|artist_opt| artist_opt)
                        .and_then(|artist_data| artist_data.artist.name)
                        .unwrap_or_else(|| "the artist".to_string());

                    // Reset form state
                    requested_date.set(String::new());
                    selected_time_slot.set(None);
                    additional_message.set(String::new());
                    questionnaire_responses.set(HashMap::new());
                    questionnaire_completed.set(false);
                    current_step.set(1);
                    submission_error.set(None);
                    booking_id.set(None);

                    // Navigate to confirmation page with booking ID and artist name
                    let confirmation_url = format!("/booking/confirmation?booking_id={}&artist_name={}", id, urlencoding::encode(&artist_name));
                    let _ = navigate(&confirmation_url, Default::default());

                    // Close modal and stop submitting after navigation
                    show.set(false);
                    is_submitting.set(false);
                }
                Err(e) => {
                    submission_error.set(Some(format!("Failed to submit booking: {}", e)));
                    is_submitting.set(false);
                }
            }
        }
    });

    let is_appointment_form_valid = move || {
        !requested_date.get().trim().is_empty() && selected_time_slot.get().is_some()
    };

    // Create computed signal for button disabled state
    let is_submit_disabled = Memo::new(move |_| {
        !is_appointment_form_valid() || is_submitting.get() || !questionnaire_completed.get()
    });

    let reset_form = move || {
        requested_date.set(String::new());
        selected_time_slot.set(None);
        additional_message.set(String::new());
        questionnaire_responses.set(HashMap::new());
        questionnaire_completed.set(false);
        current_step.set(1);
        is_submitting.set(false);
        submission_error.set(None);
        booking_id.set(None);
    };

    let close_modal = move || {
        reset_form();
        on_close();
    };

    view! {
        <div class=move || if show.get() { "booking-modal-overlay show" } else { "booking-modal-overlay" }>
            <div class="booking-modal">
                <div class="modal-header">
                    <h2>{move || match current_step.get() {
                        1 => "Artist Questionnaire".to_string(),
                        2 => "Schedule Appointment".to_string(),
                        _ => "Request Booking".to_string()
                    }}</h2>
                    <Button
                        appearance=ButtonAppearance::Subtle
                        on_click=move |_| close_modal()
                        class="close-button"
                    >
                        "×"
                    </Button>
                </div>

                <div class="modal-content">
                    {move || match current_step.get() {
                        1 => view! {
                            // Step 1: Multi-Step Questionnaire
                            <Suspense fallback=move || view! {
                                <div class="loading-questionnaire">
                                    <div class="loading-spinner"></div>
                                    <p>"Loading questionnaire..."</p>
                                </div>
                            }>
                                {move || {
                                    match questionnaire_resource.get() {
                                        Some(Some(questionnaire_form)) => {
                                            view! {
                                                <MultiStepQuestionnaire
                                                    questionnaire_form=questionnaire_form
                                                    responses=questionnaire_responses
                                                    on_completion=move || {
                                                        questionnaire_completed.set(true);
                                                        current_step.set(2);
                                                    }
                                                    on_back=move || {
                                                        close_modal();
                                                    }
                                                />
                                            }.into_any()
                                        },
                                        Some(None) => {
                                            // No questionnaire configured, show message and let user proceed manually
                                            questionnaire_completed.set(true);
                                            view! {
                                                <div class="no-questionnaire">
                                                    <div class="questionnaire-empty-state">
                                                        <h3>"No Questionnaire Required"</h3>
                                                        <p>"This artist hasn't configured a custom questionnaire. You can proceed directly to scheduling your appointment."</p>
                                                        <div class="booking-modal-form-actions">
                                                            <Button
                                                                appearance=ButtonAppearance::Secondary
                                                                on_click=move |_| {
                                                                    close_modal();
                                                                }
                                                            >
                                                                "Cancel"
                                                            </Button>
                                                            <Button
                                                                appearance=ButtonAppearance::Primary
                                                                on_click=move |_| {
                                                                    current_step.set(2);
                                                                }
                                                            >
                                                                "Continue to Scheduling"
                                                            </Button>
                                                        </div>
                                                    </div>
                                                </div>
                                            }.into_any()
                                        },
                                        None => {
                                            view! {
                                                <div class="loading-questionnaire">
                                                    <div class="loading-spinner"></div>
                                                    <p>"Loading questionnaire..."</p>
                                                </div>
                                            }.into_any()
                                        }
                                    }
                                }}
                            </Suspense>
                        }.into_any(),
                        2 => view! {
                            // Step 2: Appointment Details
                            <div class="appointment-form">
                                <Suspense fallback=move || view! {
                                    <div class="loading">"Loading artist information..."</div>
                                }>
                                    {move || {
                                        artist_resource.get().map(|artist_opt| {
                                            if let Some(artist_data) = artist_opt {
                                                let artist_name = artist_data.artist.name.clone().unwrap_or_else(|| "Artist".to_string());
                                                view! {
                                                    <div class="booking-modal-artist-info">
                                                        <h3>{format!("Schedule with {}", artist_name)}</h3>
                                                        <p class="artist-subtitle">"Choose your preferred appointment time"</p>
                                                    </div>
                                                }.into_any()
                                            } else {
                                                view! {
                                                    <div class="booking-modal-artist-info">
                                                        <h3>"Artist Not Found"</h3>
                                                        <p class="artist-subtitle">"Unable to load artist information"</p>
                                                    </div>
                                                }.into_any()
                                            }
                                        })
                                    }}
                                </Suspense>

                                <div class="appointment-form-content">
                                    <div class="form-section">
                                        <h4>"Select a Date & Time"</h4>
                                        <p class="auth-note">"Choose your preferred appointment slot from the artist's available times"</p>

                                        <AvailableDatePicker
                                            artist_id=artist_id
                                            selected_date=requested_date
                                            on_date_selected=move |date| {
                                                requested_date.set(date);
                                                // Reset time slot when date changes
                                                selected_time_slot.set(None);
                                            }
                                        />

                                        {move || {
                                            if !requested_date.get().trim().is_empty() {
                                                view! {
                                                    <TimeSlotPicker
                                                        artist_id=artist_id
                                                        selected_date=requested_date
                                                        selected_time_slot=selected_time_slot
                                                        on_slot_selected=move |slot| {
                                                            selected_time_slot.set(Some(slot));
                                                        }
                                                    />
                                                }.into_any()
                                            } else {
                                                view! {}.into_any()
                                            }
                                        }}
                                    </div>

                                    {move || {
                                        if let Some(error) = submission_error.get() {
                                            view! {
                                                <div class="error-message">
                                                    <p>{error}</p>
                                                </div>
                                            }.into_any()
                                        } else {
                                            view! {}.into_any()
                                        }
                                    }}

                                    <div class="booking-modal-form-actions">
                                        <Button
                                            appearance=ButtonAppearance::Secondary
                                            on_click=move |_| current_step.set(1)
                                        >
                                            "Back to Questionnaire"
                                        </Button>
                                        <Button
                                            appearance=ButtonAppearance::Primary
                                            disabled=Signal::from(is_submit_disabled)
                                            loading=is_submitting
                                            on_click=move |_| {
                                                if is_appointment_form_valid() && questionnaire_completed.get() {
                                                    handle_submit();
                                                }
                                            }
                                        >
                                            {move || if is_submitting.get() { "Submitting..." } else { "Book Consultation" }}
                                        </Button>
                                    </div>
                                </div>
                            </div>
                        }.into_any(),
                        _ => view! {}.into_any()
                    }}
                </div>
            </div>
        </div>
    }
}

