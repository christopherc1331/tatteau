use leptos::prelude::*;
use leptos::wasm_bindgen::JsCast;
use thaw::*;
use web_sys::HtmlInputElement;

use crate::db::entities::{BookingMessage, BookingRequest};
use crate::server::{
    get_booking_messages, get_booking_request_by_id, get_client_booking_history,
    respond_to_booking, send_booking_message, suggest_booking_time, BookingHistoryEntry,
    BookingResponse, BookingSuggestion, NewBookingMessage,
};
use crate::utils::timezone::{
    format_date_for_booking, format_datetime_for_booking, format_time_range_with_timezone,
    format_time_with_timezone, get_timezone_abbreviation,
};

#[component]
pub fn BookingDetails(booking_id: i32) -> impl IntoView {
    // Get timezone for proper time display
    let timezone = get_timezone_abbreviation();

    // Fetch booking data from database
    let booking_resource = Resource::new(
        move || booking_id,
        move |id| async move { get_booking_request_by_id(id).await },
    );

    // Fetch booking messages
    let messages_resource = Resource::new(
        move || booking_id,
        move |id| async move { get_booking_messages(id).await },
    );

    // Client booking history resource - will be fetched when booking data is available
    let history_resource = Resource::new(
        move || {
            booking_resource
                .get()
                .and_then(|result| result.ok().map(|booking| booking.client_email.clone()))
        },
        move |client_email| async move {
            match client_email {
                Some(email) => get_client_booking_history(email).await,
                None => Ok(vec![]),
            }
        },
    );

    view! {
        <div class="booking-details">
            <BookingDetailsHeader />

            <div class="booking-details-content">
                <Suspense fallback=|| view! { <div class="booking-details-loading">"Loading booking details..."</div> }>
                    {move || {
                        booking_resource.get().map(|booking_result| {
                            match booking_result {
                                Ok(booking) => view! {
                                    <BookingOverviewCard booking=booking.clone() timezone=timezone />

                                    <BookingDescriptionCard
                                        description=booking.tattoo_description.clone()
                                    />

                                    <BookingClientMessageCard
                                        message=booking.message_from_client.clone()
                                    />

                                    <Suspense fallback=|| view! { <div>"Loading history..."</div> }>
                                        {move || {
                                            history_resource.get().map(|history_result| {
                                                match history_result {
                                                    Ok(history) => view! {
                                                        <BookingHistoryCard history=history timezone=timezone />
                                                    }.into_any(),
                                                    Err(_) => view! {
                                                        <BookingHistoryCard history=vec![] timezone=timezone />
                                                    }.into_any(),
                                                }
                                            })
                                        }}
                                    </Suspense>

                                    <Suspense fallback=|| view! { <div>"Loading messages..."</div> }>
                                        {move || {
                                            messages_resource.get().map(|messages_result| {
                                                match messages_result {
                                                    Ok(messages) => view! {
                                                        <BookingMessagesCard
                                                            messages=messages
                                                            booking_id=booking.id
                                                            timezone=timezone
                                                        />
                                                    }.into_any(),
                                                    Err(_) => view! {
                                                        <BookingMessagesCard
                                                            messages=vec![]
                                                            booking_id=booking.id
                                                            timezone=timezone
                                                        />
                                                    }.into_any(),
                                                }
                                            })
                                        }}
                                    </Suspense>

                                    <BookingActionsCard
                                        booking=booking.clone()
                                    />
                                }.into_any(),
                                Err(e) => view! {
                                    <div class="booking-details-error-message">
                                        {format!("Failed to load booking: {}", e)}
                                    </div>
                                }.into_any(),
                            }
                        })
                    }}
                </Suspense>
            </div>
        </div>
    }
}

#[component]
fn BookingDetailsHeader() -> impl IntoView {
    view! {
        <div class="booking-details-header">
            <div class="booking-details-header-content">
                <a href="/artist/dashboard/calendar" class="booking-details-back-button">
                    "← Back to Calendar"
                </a>
                <h1>"Booking Request Details"</h1>
            </div>
        </div>
    }
}

#[component]
fn BookingOverviewCard(booking: BookingRequest, timezone: ReadSignal<String>) -> impl IntoView {
    let status_class = format!(
        "booking-details-status-badge booking-details-status-{}",
        booking.status
    );
    let status_icon = match booking.status.as_str() {
        "pending" => "⏳",
        "approved" => "✅",
        "declined" => "❌",
        _ => "📋",
    };
    let status_text = match booking.status.as_str() {
        "pending" => "Pending Review",
        "approved" => "Approved",
        "declined" => "Declined",
        _ => &booking.status,
    };

    view! {
        <div class="booking-details-overview-card">
            <div class="booking-details-card-header">
                <h2>"Booking Overview"</h2>
                <div class=status_class>
                    {format!("{} {}", status_icon, status_text)}
                </div>
            </div>

            <div class="booking-details-overview-grid">
                <BookingOverviewItem label="Booking ID" value=booking.id.to_string() />
                <BookingOverviewItem label="Client Name" value=booking.client_name.clone() />
                <BookingOverviewItem label="Contact Email" value=booking.client_email.clone() />

                {booking.client_phone.as_ref().map(|phone| view! {
                    <BookingOverviewItem label="Phone Number" value=phone.clone() />
                })}

                {booking.placement.as_ref().map(|placement| view! {
                    <BookingOverviewItem label="Placement" value=placement.clone() />
                })}

                {booking.size_inches.as_ref().map(|size| view! {
                    <BookingOverviewItem label="Size" value=format!("{} inches", size) />
                })}

                <BookingOverviewItem
                    label="Requested Date"
                    value=format!("{} at {}",
                        format_date_for_booking(&booking.requested_date),
                        format_time_with_timezone(&booking.requested_start_time, timezone)
                    )
                />

                {booking.requested_end_time.as_ref().map(|end_time| view! {
                    <BookingOverviewItem
                        label="Duration"
                        value=format_time_range_with_timezone(&booking.requested_start_time, Some(end_time), timezone)
                    />
                })}

                <BookingOverviewItem
                    label="Submitted"
                    value=booking.created_at.as_ref()
                        .map(|dt| format_datetime_for_booking(dt, timezone))
                        .unwrap_or_else(|| "Unknown".to_string())
                />
            </div>
        </div>
    }
}

#[component]
fn BookingOverviewItem(label: &'static str, value: String) -> impl IntoView {
    view! {
        <div class="booking-details-overview-item">
            <label>{label}</label>
            <span class="booking-details-overview-value">{value}</span>
        </div>
    }
}

#[component]
fn BookingDescriptionCard(description: Option<String>) -> impl IntoView {
    description.map(|desc| {
        view! {
            <div class="booking-details-description-card">
                <h2>"Tattoo Description"</h2>
                <div class="booking-details-description-content">
                    {desc}
                </div>
            </div>
        }
    })
}

#[component]
fn BookingClientMessageCard(message: Option<String>) -> impl IntoView {
    message.map(|msg| {
        view! {
            <div class="booking-details-notes-card">
                <h2>"Client Message"</h2>
                <div class="booking-details-notes-content">
                    {msg}
                </div>
            </div>
        }
    })
}

#[component]
fn BookingActionsCard(booking: BookingRequest) -> impl IntoView {
    let booking_id = booking.id;

    // State for decline reason modal
    let (show_decline_modal, set_show_decline_modal) = signal(false);
    let decline_reason = RwSignal::new("".to_string());

    // State for suggest date/time modal
    let (show_suggest_modal, set_show_suggest_modal) = signal(false);
    let suggested_date = RwSignal::new("".to_string());
    let suggested_start_time = RwSignal::new("".to_string());
    let suggested_end_time = RwSignal::new("".to_string());

    // Actions for status updates
    let accept_action = Action::new(move |_: &()| {
        let booking_id = booking_id;
        async move {
            let response = BookingResponse {
                booking_id,
                status: "approved".to_string(),
                artist_response: Some(
                    "Your booking has been approved! Looking forward to working with you."
                        .to_string(),
                ),
                estimated_price: None,
                decline_reason: None,
            };
            respond_to_booking(response).await
        }
    });

    let decline_action = Action::new(move |reason: &String| {
        let booking_id = booking_id;
        let reason = reason.clone();
        async move {
            let response = BookingResponse {
                booking_id,
                status: "declined".to_string(),
                artist_response: Some(
                    "Unfortunately, I'm not able to take on this booking at this time.".to_string(),
                ),
                estimated_price: None,
                decline_reason: Some(reason),
            };
            respond_to_booking(response).await
        }
    });

    let suggest_time_action = Action::new(move |suggestion: &BookingSuggestion| {
        let suggestion = suggestion.clone();
        async move { suggest_booking_time(suggestion).await }
    });

    // Event handlers
    let accept_booking = move |_| {
        accept_action.dispatch(());
    };

    let decline_booking = move |_| {
        set_show_decline_modal.set(true);
    };

    let confirm_decline = move |_| {
        let reason = decline_reason.get().trim().to_string();
        if !reason.is_empty() {
            decline_action.dispatch(reason);
            set_show_decline_modal.set(false);
            decline_reason.set("".to_string());
        }
    };

    let cancel_decline = move |_| {
        set_show_decline_modal.set(false);
        decline_reason.set("".to_string());
    };

    let suggest_date_time = move |_| {
        set_show_suggest_modal.set(true);
    };

    let confirm_suggest = move |_| {
        let date = suggested_date.get().trim().to_string();
        let start_time = suggested_start_time.get().trim().to_string();
        let end_time = suggested_end_time.get().trim().to_string();

        if !date.is_empty() && !start_time.is_empty() {
            let suggestion = BookingSuggestion {
                booking_id,
                suggested_date: date,
                suggested_start_time: start_time,
                suggested_end_time: if end_time.is_empty() {
                    None
                } else {
                    Some(end_time)
                },
            };
            suggest_time_action.dispatch(suggestion);
            set_show_suggest_modal.set(false);
            suggested_date.set("".to_string());
            suggested_start_time.set("".to_string());
            suggested_end_time.set("".to_string());
        }
    };

    let cancel_suggest = move |_| {
        set_show_suggest_modal.set(false);
        suggested_date.set("".to_string());
        suggested_start_time.set("".to_string());
        suggested_end_time.set("".to_string());
    };

    // Note: Page will show updated status after refresh or navigation

    view! {
        <div class="booking-details-actions-card">
            <h2>"Actions"</h2>
            <div class="booking-details-actions-grid">
                {move || {
                    let current_status = &booking.status;
                    if current_status == "pending" {
                        view! {
                            <Button
                                appearance=ButtonAppearance::Primary
                                on_click=accept_booking
                                disabled=accept_action.pending().get()
                            >
                                {move || if accept_action.pending().get() { "Accepting..." } else { "Accept Booking" }}
                            </Button>
                            <Button
                                appearance=ButtonAppearance::Secondary
                                on_click=decline_booking
                                disabled=decline_action.pending().get()
                            >
                                {move || if decline_action.pending().get() { "Declining..." } else { "Decline Booking" }}
                            </Button>
                            <Button
                                appearance=ButtonAppearance::Subtle
                                on_click=suggest_date_time
                                disabled=suggest_time_action.pending().get()
                            >
                                {move || if suggest_time_action.pending().get() { "Suggesting..." } else { "Suggest New Date/Time" }}
                            </Button>
                        }.into_any()
                    } else if current_status == "approved" {
                        view! {
                            <Button
                                appearance=ButtonAppearance::Secondary
                                on_click=decline_booking
                                disabled=decline_action.pending().get()
                            >
                                {move || if decline_action.pending().get() { "Declining..." } else { "Decline Booking" }}
                            </Button>
                            <Button appearance=ButtonAppearance::Subtle>
                                "Send Message"
                            </Button>
                            <Button appearance=ButtonAppearance::Subtle>
                                "Reschedule"
                            </Button>
                        }.into_any()
                    } else {
                        view! {
                            <Button appearance=ButtonAppearance::Secondary>
                                "Update Status"
                            </Button>
                            <Button appearance=ButtonAppearance::Subtle>
                                "Send Message"
                            </Button>
                            <Button appearance=ButtonAppearance::Subtle>
                                "Reschedule"
                            </Button>
                        }.into_any()
                    }
                }}
            </div>

            {move || {
                accept_action.value().get().map(|result| {
                    match result {
                        Ok(_) => view! {
                            <div class="booking-details-success-message">"Booking accepted successfully!"</div>
                        }.into_any(),
                        Err(e) => view! {
                            <div class="booking-details-error-message">{format!("Failed to accept booking: {}", e)}</div>
                        }.into_any(),
                    }
                })
            }}

            {move || {
                decline_action.value().get().map(|result| {
                    match result {
                        Ok(_) => view! {
                            <div class="booking-details-success-message">"Booking declined successfully!"</div>
                        }.into_any(),
                        Err(e) => view! {
                            <div class="booking-details-error-message">{format!("Failed to decline booking: {}", e)}</div>
                        }.into_any(),
                    }
                })
            }}

            {move || {
                suggest_time_action.value().get().map(|result| {
                    match result {
                        Ok(_) => view! {
                            <div class="booking-details-success-message">"Time suggestion sent successfully!"</div>
                        }.into_any(),
                        Err(e) => view! {
                            <div class="booking-details-error-message">{format!("Failed to send time suggestion: {}", e)}</div>
                        }.into_any(),
                    }
                })
            }}
        </div>

        // Decline Reason Modal
        {move || {
            if show_decline_modal.get() {
                view! {
                    <div class="booking-details-decline-modal-overlay">
                        <Card>
                            <h3>"Decline Booking"</h3>
                            <div class="booking-details-decline-modal-content">
                                <p>"Please provide a reason for declining this booking. This will be shared with the client."</p>
                                <textarea
                                    prop:value=move || decline_reason.get()
                                    on:input=move |ev| {
                                        let target = ev.target().unwrap();
                                        let textarea = target.unchecked_into::<web_sys::HtmlTextAreaElement>();
                                        decline_reason.set(textarea.value());
                                    }
                                    placeholder="Explain why you're declining this booking..."
                                    class="booking-details-decline-reason-input"
                                    rows="4"
                                ></textarea>
                            </div>
                            <div class="booking-details-decline-modal-footer">
                                <Button
                                    appearance=ButtonAppearance::Secondary
                                    on_click=cancel_decline
                                >
                                    "Cancel"
                                </Button>
                                <Button
                                    appearance=ButtonAppearance::Primary
                                    on_click=confirm_decline
                                >
                                    "Decline with Reason"
                                </Button>
                            </div>
                        </Card>
                    </div>
                }.into_any()
            } else {
                view! {}.into_any()
            }
        }}

        // Suggest Date/Time Modal
        {move || {
            if show_suggest_modal.get() {
                view! {
                    <div class="booking-details-suggest-modal-overlay">
                        <Card>
                            <h3>"Suggest New Date & Time"</h3>
                            <div class="booking-details-suggest-modal-content">
                                <p>"Propose an alternative date and time for this booking. The client will be notified of your suggestion."</p>
                                <div class="booking-details-form-group">
                                    <label for="suggested-date">"Suggested Date:"</label>
                                    <Input
                                        id="suggested-date"
                                        input_type=InputType::Date
                                        value=suggested_date
                                        placeholder="Select a date"
                                    />
                                </div>
                                <div class="booking-details-form-group">
                                    <label for="suggested-start-time">"Start Time:"</label>
                                    <Input
                                        id="suggested-start-time"
                                        input_type=InputType::Time
                                        value=suggested_start_time
                                        placeholder="Select start time"
                                    />
                                </div>
                                <div class="booking-details-form-group">
                                    <label for="suggested-end-time">"End Time (optional):"</label>
                                    <Input
                                        id="suggested-end-time"
                                        input_type=InputType::Time
                                        value=suggested_end_time
                                        placeholder="Select end time"
                                    />
                                </div>
                            </div>
                            <div class="booking-details-suggest-modal-footer">
                                <Button
                                    appearance=ButtonAppearance::Secondary
                                    on_click=cancel_suggest
                                >
                                    "Cancel"
                                </Button>
                                <Button
                                    appearance=ButtonAppearance::Primary
                                    on_click=confirm_suggest
                                >
                                    "Send Suggestion"
                                </Button>
                            </div>
                        </Card>
                    </div>
                }.into_any()
            } else {
                view! {}.into_any()
            }
        }}
    }
}

#[component]
pub fn BookingHistoryCard(
    history: Vec<BookingHistoryEntry>,
    timezone: ReadSignal<String>,
) -> impl IntoView {
    view! {
        <div class="booking-details-history-card">
            <div class="booking-details-card-header">
                <h2>"Client Booking History"</h2>
                <span class="booking-details-history-count">
                    {format!("{} bookings", history.len())}
                </span>
            </div>
            <div class="booking-details-history-list">
                {
                    if history.is_empty() {
                        view! {
                            <div class="booking-details-no-history">
                                "No previous bookings found for this client."
                            </div>
                        }.into_any()
                    } else {
                        history.into_iter().take(5).map(|item| {
                            view! {
                                <BookingHistoryItem item=item timezone=timezone />
                            }
                        }).collect_view().into_any()
                    }
                }
            </div>
        </div>
    }
}

#[component]
fn BookingHistoryItem(item: BookingHistoryEntry, timezone: ReadSignal<String>) -> impl IntoView {
    let status_class = format!(
        "booking-details-history-status booking-details-history-status-{}",
        item.status
    );
    let status_icon = match item.status.as_str() {
        "pending" => "⏳",
        "approved" => "✅",
        "declined" => "❌",
        "completed" => "🎨",
        _ => "📋",
    };

    let booking_id = item.id;
    let navigate_to_booking = move |_| {
        // Navigate to the booking details page
        let window = web_sys::window().unwrap();
        let location = window.location();
        let _ = location.set_href(&format!("/artist/dashboard/booking/{}", booking_id));
    };

    view! {
        <div class="booking-details-history-item" on:click=navigate_to_booking>
            <div class="booking-details-history-details">
                <div class="booking-details-history-id">
                    "Booking #"{item.id}
                </div>
                <div class="booking-details-history-date">
                    {item.booking_date.as_ref()
                        .map(|date| format_date_for_booking(date))
                        .unwrap_or_else(|| "Date TBD".to_string())}
                </div>
                <div class="booking-details-history-created">
                    "Submitted "{format_datetime_for_booking(&item.created_at, timezone)}
                </div>
            </div>
            <div class="booking-details-history-status-container">
                <div class=status_class>
                    {format!("{} {}", status_icon, item.status)}
                </div>
                <div class="booking-details-history-arrow">
                    "→"
                </div>
            </div>
        </div>
    }
}

#[component]
pub fn BookingMessagesCard(
    messages: Vec<BookingMessage>,
    booking_id: i32,
    timezone: ReadSignal<String>,
) -> impl IntoView {
    let new_message = RwSignal::new("".to_string());

    let send_action = Action::new(move |message_content: &String| {
        let message_data = NewBookingMessage {
            booking_request_id: booking_id,
            sender_type: "artist".to_string(),
            message: message_content.clone(),
        };
        async move { send_booking_message(message_data).await }
    });

    let send_message = move |_| {
        let message_content = new_message.get().trim().to_string();
        if !message_content.is_empty() {
            send_action.dispatch(message_content);
            new_message.set("".to_string());
        }
    };

    view! {
        <div class="booking-details-messages-card">
            <div class="booking-details-card-header">
                <h2>"Messages"</h2>
                <span class="booking-details-message-count">
                    {format!("{} messages", messages.len())}
                </span>
            </div>

            <div class="booking-details-messages-list">
                {
                    if messages.is_empty() {
                        view! {
                            <div class="booking-details-no-messages">
                                "No messages yet. Start the conversation!"
                            </div>
                        }.into_any()
                    } else {
                        messages.into_iter().map(|msg| {
                            view! {
                                <BookingMessageItem message=msg timezone=timezone />
                            }
                        }).collect_view().into_any()
                    }
                }
            </div>

            <div class="booking-details-message-input-container">
                <input
                    type="text"
                    prop:value=move || new_message.get()
                    on:input=move |ev| {
                        let target = ev.target().unwrap();
                        let input = target.unchecked_into::<HtmlInputElement>();
                        new_message.set(input.value());
                    }
                    placeholder="Type your message..."
                    class="booking-details-message-input"
                    disabled=send_action.pending().get()
                />
                <Button
                    appearance=ButtonAppearance::Primary
                    on_click=send_message
                    disabled=send_action.pending().get()
                >
                    {move || if send_action.pending().get() { "Sending..." } else { "Send" }}
                </Button>
            </div>

            {move || {
                send_action.value().get().map(|result| {
                    match result {
                        Ok(_) => view! {
                            <div class="booking-details-success-message">"Message sent successfully! Refresh to see it in the thread."</div>
                        }.into_any(),
                        Err(e) => view! {
                            <div class="booking-details-error-message">{format!("Failed to send message: {}", e)}</div>
                        }.into_any(),
                    }
                })
            }}
        </div>
    }
}

#[component]
fn BookingMessageItem(message: BookingMessage, timezone: ReadSignal<String>) -> impl IntoView {
    let sender_class = format!(
        "booking-details-message-item booking-details-sender-{}",
        message.sender_type
    );
    let sender_label = match message.sender_type.as_str() {
        "client" => "Client",
        "artist" => "You",
        _ => "System",
    };

    view! {
        <div class=sender_class>
            <div class="booking-details-message-header">
                <span class="booking-details-sender-name">{sender_label}</span>
                <span class="booking-details-message-time">
                    {message.created_at.as_ref()
                        .map(|dt| format_datetime_for_booking(dt, timezone))
                        .unwrap_or_else(|| "Unknown".to_string())}
                </span>
            </div>
            <div class="booking-details-message-content">
                {message.message}
            </div>
        </div>
    }
}
