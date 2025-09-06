use leptos::prelude::*;
use leptos::task::spawn_local;
use thaw::*;
use crate::server::{submit_booking_request, NewBookingRequest, fetch_artist_data, get_artist_questionnaire_form, submit_questionnaire_responses};
use crate::db::entities::{ClientQuestionnaireSubmission, QuestionnaireResponse};
use crate::components::MultiStepQuestionnaire;
use std::collections::HashMap;

#[component]
pub fn ClientBookingModal(
    show: RwSignal<bool>,
    artist_id: RwSignal<Option<i32>>,
    on_close: impl Fn() + 'static + Copy + Send + Sync,
) -> impl IntoView {
    // Appointment form state (only time/date info - no contact details for authenticated users)
    let requested_date = RwSignal::new(String::new());
    let requested_start_time = RwSignal::new(String::new());
    let requested_end_time = RwSignal::new(String::new());
    let additional_message = RwSignal::new(String::new());
    
    // Questionnaire state
    let questionnaire_responses = RwSignal::new(HashMap::<i32, String>::new());
    
    // UI state
    let current_step = RwSignal::new(1); // 1: questionnaire, 2: appointment details, 3: confirmation
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
        async move {
            submit_booking_request(request).await
        }
    });

    let handle_submit = move || {
        if let Some(id) = artist_id.get() {
            is_submitting.set(true);
            submission_error.set(None);

            // For authenticated users, use placeholder values for required fields
            // Real user data should come from authentication context
            let request = NewBookingRequest {
                artist_id: id,
                client_name: "Authenticated User".to_string(), // TODO: Get from auth context
                client_email: "user@example.com".to_string(), // TODO: Get from auth context
                client_phone: None,
                tattoo_description: None, // Collected via questionnaire
                placement: None, // Collected via questionnaire
                size_inches: None, // Collected via questionnaire
                requested_date: requested_date.get(),
                requested_start_time: requested_start_time.get(),
                requested_end_time: if requested_end_time.get().trim().is_empty() { None } else { Some(requested_end_time.get()) },
                message_from_client: if additional_message.get().trim().is_empty() { None } else { Some(additional_message.get()) },
            };

            submit_booking.dispatch(request);
        }
    };

    // Handle submission result
    Effect::new(move |_| {
        if let Some(result) = submit_booking.value().get() {
            is_submitting.set(false);
            match result {
                Ok(id) => {
                    booking_id.set(Some(id));
                    
                    // Submit questionnaire responses if we have any
                    let responses = questionnaire_responses.get();
                    if !responses.is_empty() {
                        if let Some(booking_id_val) = booking_id.get() {
                            let submission = ClientQuestionnaireSubmission {
                                booking_request_id: booking_id_val,
                                responses: responses.into_iter().map(|(question_id, response)| {
                                    QuestionnaireResponse {
                                        question_id,
                                        response_text: Some(response.clone()),
                                        response_data: None,
                                    }
                                }).collect(),
                            };
                            
                            spawn_local(async move {
                                let _ = submit_questionnaire_responses(submission).await;
                            });
                        }
                    }
                    
                    current_step.set(3); // Move to confirmation step
                }
                Err(e) => {
                    submission_error.set(Some(format!("Failed to submit booking: {}", e)));
                }
            }
        }
    });

    let is_appointment_form_valid = move || {
        !requested_date.get().trim().is_empty() &&
        !requested_start_time.get().trim().is_empty()
    };

    // Create computed signal for button disabled state
    let is_submit_disabled = Memo::new(move |_| {
        !is_appointment_form_valid() || is_submitting.get() || !questionnaire_completed.get()
    });

    let reset_form = move || {
        requested_date.set(String::new());
        requested_start_time.set(String::new());
        requested_end_time.set(String::new());
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
                        3 => "Booking Request Submitted!".to_string(),
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
                                            // No questionnaire configured, skip to appointment details
                                            questionnaire_completed.set(true);
                                            current_step.set(2);
                                            view! {
                                                <div class="no-questionnaire">
                                                    <p>"No questionnaire configured. Proceeding to appointment scheduling..."</p>
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
                                                    <div class="artist-info">
                                                        <h3>{format!("Schedule with {}", artist_name)}</h3>
                                                        <p class="artist-subtitle">"Choose your preferred appointment time"</p>
                                                    </div>
                                                }.into_any()
                                            } else {
                                                view! {
                                                    <div class="artist-info">
                                                        <h3>"Artist Not Found"</h3>
                                                        <p class="artist-subtitle">"Unable to load artist information"</p>
                                                    </div>
                                                }.into_any()
                                            }
                                        })
                                    }}
                                </Suspense>

                                <form class="appointment-form-content" on:submit=move |ev| {
                                    ev.prevent_default();
                                    if is_appointment_form_valid() && questionnaire_completed.get() {
                                        handle_submit();
                                    }
                                }>
                                    <div class="form-section">
                                        <h4>"Appointment Preference"</h4>
                                        <p class="auth-note">"Since you're logged in, we'll use your account information for this booking."</p>
                                        <div class="form-row">
                                            <div class="form-group">
                                                <label for="requested-date">"Preferred Date *"</label>
                                                <Input
                                                    id="requested-date"
                                                    input_type=InputType::Date
                                                    value=requested_date
                                                />
                                            </div>
                                            <div class="form-group">
                                                <label for="requested-start-time">"Preferred Start Time *"</label>
                                                <Input
                                                    id="requested-start-time"
                                                    input_type=InputType::Time
                                                    value=requested_start_time
                                                />
                                            </div>
                                            <div class="form-group">
                                                <label for="requested-end-time">"End Time (optional)"</label>
                                                <Input
                                                    id="requested-end-time"
                                                    input_type=InputType::Time
                                                    value=requested_end_time
                                                />
                                            </div>
                                        </div>
                                    </div>

                                    <div class="form-section">
                                        <h4>"Additional Message (Optional)"</h4>
                                        <div class="form-group">
                                            <label for="additional-message">"Message to Artist"</label>
                                            <Textarea
                                                id="additional-message"
                                                placeholder="Any additional details or questions..."
                                                value=additional_message
                                            />
                                        </div>
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

                                    <div class="form-actions">
                                        <Button 
                                            appearance=ButtonAppearance::Secondary
                                            on_click=move |_| current_step.set(1)
                                        >
                                            "Back to Questionnaire"
                                        </Button>
                                        <Button 
                                            button_type=ButtonType::Submit
                                            appearance=ButtonAppearance::Primary
                                            disabled=Signal::from(is_submit_disabled)
                                            loading=is_submitting
                                        >
                                            {move || if is_submitting.get() { "Submitting..." } else { "Submit Booking Request" }}
                                        </Button>
                                    </div>
                                </form>
                            </div>
                        }.into_any(),
                        3 => view! {
                            // Step 3: Confirmation
                            <div class="confirmation-step">
                                <div class="success-icon">
                                    "✓"
                                </div>
                                <h3>"Booking Request Submitted Successfully!"</h3>
                                <p class="confirmation-text">
                                    "Your questionnaire responses and appointment request have been sent to the artist. You will receive a confirmation email shortly."
                                </p>
                                {move || {
                                    if let Some(id) = booking_id.get() {
                                        view! {
                                            <div class="booking-details">
                                                <p class="booking-id">{format!("Reference ID: #{}", id)}</p>
                                                <p class="next-steps">"The artist will review your questionnaire and appointment request, then respond within 24-48 hours. You'll be notified via email with their response."</p>
                                            </div>
                                        }.into_any()
                                    } else {
                                        view! {}.into_any()
                                    }
                                }}
                                <div class="confirmation-actions">
                                    <Button 
                                        appearance=ButtonAppearance::Primary
                                        on_click=move |_| close_modal()
                                    >
                                        "Done"
                                    </Button>
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