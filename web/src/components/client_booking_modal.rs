use leptos::prelude::*;
use thaw::*;
use crate::server::{submit_booking_request, NewBookingRequest, fetch_artist_data};

#[component]
pub fn ClientBookingModal(
    show: RwSignal<bool>,
    artist_id: RwSignal<Option<i32>>,
    on_close: impl Fn() + 'static + Copy + Send + Sync,
) -> impl IntoView {
    // Form state
    let client_name = RwSignal::new(String::new());
    let client_email = RwSignal::new(String::new());
    let client_phone = RwSignal::new(String::new());
    let tattoo_description = RwSignal::new(String::new());
    let placement = RwSignal::new(String::new());
    let size_inches = RwSignal::new(String::new());
    let requested_date = RwSignal::new(String::new());
    let requested_start_time = RwSignal::new(String::new());
    let requested_end_time = RwSignal::new(String::new());
    let message_from_client = RwSignal::new(String::new());
    
    // UI state
    let current_step = RwSignal::new(1); // 1: booking form, 2: confirmation
    let is_submitting = RwSignal::new(false);
    let submission_error = RwSignal::new(None::<String>);
    let booking_id = RwSignal::new(None::<i32>);

    // Fetch artist data when modal opens
    let artist_resource = Resource::new(
        move || artist_id.get(),
        move |id_opt| async move {
            if let Some(id) = id_opt {
                if id > 0 {
                    fetch_artist_data(id).await.ok()
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

            let request = NewBookingRequest {
                artist_id: id,
                client_name: client_name.get(),
                client_email: client_email.get(),
                client_phone: if client_phone.get().trim().is_empty() { None } else { Some(client_phone.get()) },
                tattoo_description: if tattoo_description.get().trim().is_empty() { None } else { Some(tattoo_description.get()) },
                placement: if placement.get().trim().is_empty() { None } else { Some(placement.get()) },
                size_inches: size_inches.get().trim().parse::<f32>().ok(),
                requested_date: requested_date.get(),
                requested_start_time: requested_start_time.get(),
                requested_end_time: if requested_end_time.get().trim().is_empty() { None } else { Some(requested_end_time.get()) },
                message_from_client: if message_from_client.get().trim().is_empty() { None } else { Some(message_from_client.get()) },
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
                    current_step.set(2); // Move to confirmation step
                }
                Err(e) => {
                    submission_error.set(Some(format!("Failed to submit booking: {}", e)));
                }
            }
        }
    });

    let is_form_valid = move || {
        !client_name.get().trim().is_empty() &&
        !client_email.get().trim().is_empty() &&
        !requested_date.get().trim().is_empty() &&
        !requested_start_time.get().trim().is_empty()
    };

    // Create computed signal for button disabled state
    let is_button_disabled = Memo::new(move |_| {
        !is_form_valid() || is_submitting.get()
    });

    let reset_form = move || {
        client_name.set(String::new());
        client_email.set(String::new());
        client_phone.set(String::new());
        tattoo_description.set(String::new());
        placement.set(String::new());
        size_inches.set(String::new());
        requested_date.set(String::new());
        requested_start_time.set(String::new());
        requested_end_time.set(String::new());
        message_from_client.set(String::new());
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
                        1 => "Request a Booking".to_string(),
                        2 => "Booking Request Submitted!".to_string(),
                        _ => "Booking".to_string()
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
                            <div class="booking-form">
                                <Suspense fallback=move || view! { 
                                    <div class="loading">"Loading artist information..."</div>
                                }>
                                    {move || {
                                        artist_resource.get().map(|artist_opt| {
                                            if let Some(artist_data) = artist_opt {
                                                let artist_name = artist_data.artist.name.clone().unwrap_or_else(|| "Artist".to_string());
                                                view! {
                                                    <div class="artist-info">
                                                        <h3>{format!("Book with {}", artist_name)}</h3>
                                                        <p class="artist-subtitle">"Complete the form below to request an appointment"</p>
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

                                <form class="booking-form-content" on:submit=move |ev| {
                                    ev.prevent_default();
                                    if is_form_valid() {
                                        handle_submit();
                                    }
                                }>
                                    <div class="form-section">
                                        <h4>"Contact Information"</h4>
                                        <div class="form-row">
                                            <div class="form-group">
                                                <label for="client-name">"Full Name *"</label>
                                                <Input
                                                    id="client-name"
                                                    placeholder="Your full name"
                                                    value=client_name
                                                />
                                            </div>
                                            <div class="form-group">
                                                <label for="client-email">"Email Address *"</label>
                                                <Input
                                                    id="client-email"
                                                    input_type=InputType::Email
                                                    placeholder="your@email.com"
                                                    value=client_email
                                                />
                                            </div>
                                        </div>
                                        <div class="form-group">
                                            <label for="client-phone">"Phone Number"</label>
                                            <Input
                                                id="client-phone"
                                                input_type=InputType::Tel
                                                placeholder="(555) 123-4567"
                                                value=client_phone
                                            />
                                        </div>
                                    </div>

                                    <div class="form-section">
                                        <h4>"Tattoo Details"</h4>
                                        <div class="form-group">
                                            <label for="tattoo-description">"Tattoo Description"</label>
                                            <Textarea
                                                id="tattoo-description"
                                                placeholder="Describe your tattoo idea..."
                                                value=tattoo_description
                                            />
                                        </div>
                                        <div class="form-row">
                                            <div class="form-group">
                                                <label for="placement">"Placement"</label>
                                                <Input
                                                    id="placement"
                                                    placeholder="e.g., forearm, shoulder"
                                                    value=placement
                                                />
                                            </div>
                                            <div class="form-group">
                                                <label for="size-inches">"Size (inches)"</label>
                                                <Input
                                                    id="size-inches"
                                                    placeholder="e.g., 4.5"
                                                    value=size_inches
                                                />
                                            </div>
                                        </div>
                                    </div>

                                    <div class="form-section">
                                        <h4>"Appointment Preference"</h4>
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
                                        <h4>"Additional Message"</h4>
                                        <div class="form-group">
                                            <label for="message-from-client">"Message to Artist"</label>
                                            <Textarea
                                                id="message-from-client"
                                                placeholder="Any additional details or questions..."
                                                value=message_from_client
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
                                            on_click=move |_| close_modal()
                                        >
                                            "Cancel"
                                        </Button>
                                        <Button 
                                            button_type=ButtonType::Submit
                                            appearance=ButtonAppearance::Primary
                                            disabled=Signal::from(is_button_disabled)
                                            loading=is_submitting
                                        >
                                            {move || if is_submitting.get() { "Submitting..." } else { "Submit Booking Request" }}
                                        </Button>
                                    </div>
                                </form>
                            </div>
                        }.into_any(),
                        2 => view! {
                            <div class="confirmation-step">
                                <div class="success-icon">
                                    "✓"
                                </div>
                                <h3>"Booking Request Submitted Successfully!"</h3>
                                <p class="confirmation-text">
                                    "Your booking request has been sent to the artist. You will receive a confirmation email shortly."
                                </p>
                                {move || {
                                    if let Some(id) = booking_id.get() {
                                        view! {
                                            <div class="booking-details">
                                                <p class="booking-id">{format!("Reference ID: #{}", id)}</p>
                                                <p class="next-steps">"The artist will review your request and respond within 24-48 hours. You'll be notified via email with their response."</p>
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