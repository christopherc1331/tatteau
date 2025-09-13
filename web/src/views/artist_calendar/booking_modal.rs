use leptos::prelude::*;
use thaw::*;
use crate::db::entities::{BookingRequest, BookingMessage};
use crate::server::{respond_to_booking, send_booking_message, get_booking_messages, BookingResponse, NewBookingMessage};

#[component]
pub fn BookingModal(
    show: RwSignal<bool>,
    booking: RwSignal<Option<BookingRequest>>,
    on_close: impl Fn() + 'static + Copy,
) -> impl IntoView {
    let response_status = RwSignal::new(String::new());
    let artist_response = RwSignal::new(String::new());
    let estimated_price = RwSignal::new(String::new());
    let new_message = RwSignal::new(String::new());
    let active_tab = RwSignal::new("details");

    // Load messages when booking changes
    let messages_resource = Resource::new(
        move || booking.get().as_ref().map(|b| b.id),
        |booking_id_opt| async move {
            if let Some(booking_id) = booking_id_opt {
                get_booking_messages(booking_id).await
            } else {
                Ok(vec![])
            }
        }
    );

    let submit_response = create_action(|response: &BookingResponse| {
        let response = response.clone();
        async move { respond_to_booking(response).await }
    });

    let send_message = create_action(|message_data: &NewBookingMessage| {
        let message_data = message_data.clone();
        async move { send_booking_message(message_data).await }
    });

    let handle_approve = move || {
        if let Some(booking_data) = booking.get() {
            let price = estimated_price.get().parse::<f64>().ok();
            let response = BookingResponse {
                booking_id: booking_data.id,
                status: "approved".to_string(),
                artist_response: if artist_response.get().trim().is_empty() { 
                    None 
                } else { 
                    Some(artist_response.get()) 
                },
                estimated_price: price,
            };
            submit_response.dispatch(response);
        }
    };

    let handle_decline = move || {
        if let Some(booking_data) = booking.get() {
            let response = BookingResponse {
                booking_id: booking_data.id,
                status: "declined".to_string(),
                artist_response: if artist_response.get().trim().is_empty() { 
                    None 
                } else { 
                    Some(artist_response.get()) 
                },
                estimated_price: None,
            };
            submit_response.dispatch(response);
        }
    };

    let handle_send_message = move || {
        if let Some(booking_data) = booking.get() {
            if !new_message.get().trim().is_empty() {
                let message_data = NewBookingMessage {
                    booking_request_id: booking_data.id,
                    sender_type: "artist".to_string(),
                    message: new_message.get(),
                };
                send_message.dispatch(message_data);
                new_message.set(String::new());
            }
        }
    };

    view! {
        <Modal 
            open=show 
            on_close=move || on_close()
            mask_closable=true
        >
            {move || {
                if let Some(booking_data) = booking.get() {
                    view! {
                        <div class="booking-modal-container">
                            <div class="booking-modal-header">
                                <h2>"Booking Request Details"</h2>
                                <Button 
                                    appearance=ButtonAppearance::Subtle
                                    on_click=move |_| on_close()
                                >
                                    "×"
                                </Button>
                            </div>

                            <div class="booking-modal-tabs">
                                <Button 
                                    appearance=if active_tab.get() == "details" { 
                                        ButtonAppearance::Primary 
                                    } else { 
                                        ButtonAppearance::Subtle 
                                    }
                                    on_click=move |_| active_tab.set("details")
                                >
                                    "Details"
                                </Button>
                                <Button 
                                    appearance=if active_tab.get() == "messages" { 
                                        ButtonAppearance::Primary 
                                    } else { 
                                        ButtonAppearance::Subtle 
                                    }
                                    on_click=move |_| active_tab.set("messages")
                                >
                                    "Messages"
                                </Button>
                            </div>

                            <div class="booking-modal-content">
                                {move || match active_tab.get().as_str() {
                                    "details" => view! {
                                        <div class="booking-modal-details-tab">
                                            <div class="booking-modal-client-info">
                                                <h3>"Client Information"</h3>
                                                <div class="booking-modal-info-grid">
                                                    <div class="booking-modal-info-item">
                                                        <label>"Name:"</label>
                                                        <span>{&booking_data.client_name}</span>
                                                    </div>
                                                    <div class="booking-modal-info-item">
                                                        <label>"Email:"</label>
                                                        <span>{&booking_data.client_email}</span>
                                                    </div>
                                                    {booking_data.client_phone.as_ref().map(|phone| view! {
                                                        <div class="booking-modal-info-item">
                                                            <label>"Phone:"</label>
                                                            <span>{phone}</span>
                                                        </div>
                                                    })}
                                                </div>
                                            </div>

                                            <div class="booking-modal-tattoo-info">
                                                <h3>"Tattoo Details"</h3>
                                                <div class="booking-modal-info-grid">
                                                    <div class="booking-modal-info-item">
                                                        <label>"Requested Date:"</label>
                                                        <span>{&booking_data.requested_date}</span>
                                                    </div>
                                                    <div class="booking-modal-info-item">
                                                        <label>"Time:"</label>
                                                        <span>{&booking_data.requested_start_time}</span>
                                                        {booking_data.requested_end_time.as_ref().map(|end_time| 
                                                            view! { <span>{format!(" - {}", end_time)}</span> }
                                                        )}
                                                    </div>
                                                    {booking_data.tattoo_description.as_ref().map(|desc| view! {
                                                        <div class="booking-modal-info-item">
                                                            <label>"Description:"</label>
                                                            <span>{desc}</span>
                                                        </div>
                                                    })}
                                                    {booking_data.placement.as_ref().map(|placement| view! {
                                                        <div class="booking-modal-info-item">
                                                            <label>"Placement:"</label>
                                                            <span>{placement}</span>
                                                        </div>
                                                    })}
                                                    {booking_data.size_inches.map(|size| view! {
                                                        <div class="booking-modal-info-item">
                                                            <label>"Size:"</label>
                                                            <span>{format!("{:.1} inches", size)}</span>
                                                        </div>
                                                    })}
                                                </div>
                                            </div>

                                            {booking_data.message_from_client.as_ref().map(|message| view! {
                                                <div class="booking-modal-client-message">
                                                    <h3>"Client Message"</h3>
                                                    <p>{message}</p>
                                                </div>
                                            })}

                                            {if booking_data.status == "pending" {
                                                view! {
                                                    <div class="booking-modal-response-form">
                                                        <h3>"Your Response"</h3>
                                                        <Textarea 
                                                            placeholder="Add a message to the client (optional)"
                                                            value=artist_response
                                                        />
                                                        <Input 
                                                            placeholder="Estimated price (optional)"
                                                            value=estimated_price
                                                        />
                                                        <div class="booking-modal-response-actions">
                                                            <Button 
                                                                appearance=ButtonAppearance::Primary
                                                                on_click=move |_| handle_approve()
                                                            >
                                                                "Approve Booking"
                                                            </Button>
                                                            <Button 
                                                                appearance=ButtonAppearance::Default
                                                                on_click=move |_| handle_decline()
                                                            >
                                                                "Decline Booking"
                                                            </Button>
                                                        </div>
                                                    </div>
                                                }.into()
                                            } else {
                                                view! {
                                                    <div class="booking-modal-booking-status">
                                                        <h3>"Status: " {&booking_data.status}</h3>
                                                        {booking_data.artist_response.as_ref().map(|response| view! {
                                                            <div class="booking-modal-artist-response">
                                                                <label>"Your Response:"</label>
                                                                <p>{response}</p>
                                                            </div>
                                                        })}
                                                        {booking_data.estimated_price.map(|price| view! {
                                                            <div class="booking-modal-estimated-price">
                                                                <label>"Estimated Price:"</label>
                                                                <span>{format!("${:.2}", price)}</span>
                                                            </div>
                                                        })}
                                                    </div>
                                                }.into()
                                            }}
                                        </div>
                                    }.into(),
                                    "messages" => view! {
                                        <div class="booking-modal-messages-tab">
                                            <div class="booking-modal-message-history">
                                                <Suspense fallback=move || view! { 
                                                    <Spin size=SpinSize::Small />
                                                }>
                                                    {move || {
                                                        if let Some(Ok(messages)) = messages_resource.get() {
                                                            if messages.is_empty() {
                                                                view! {
                                                                    <div class="booking-modal-no-messages">
                                                                        <p>"No messages yet. Start a conversation!"</p>
                                                                    </div>
                                                                }.into()
                                                            } else {
                                                                messages.into_iter().map(|message| {
                                                                    let is_from_artist = message.sender_type == "artist";
                                                                    view! {
                                                                        <div class=format!("booking-modal-message {}", if is_from_artist { "artist" } else { "client" })>
                                                                            <div class="booking-modal-message-header">
                                                                                <span class="sender">
                                                                                    {if is_from_artist { "You" } else { &booking_data.client_name }}
                                                                                </span>
                                                                                <span class="timestamp">
                                                                                    {message.created_at.unwrap_or_default()}
                                                                                </span>
                                                                            </div>
                                                                            <div class="booking-modal-message-content">
                                                                                {message.message}
                                                                            </div>
                                                                        </div>
                                                                    }
                                                                }).collect::<Vec<_>>().into()
                                                            }
                                                        } else {
                                                            view! {
                                                                <div class="booking-modal-loading-messages">
                                                                    <Spin size=SpinSize::Small />
                                                                    <span>"Loading messages..."</span>
                                                                </div>
                                                            }.into()
                                                        }
                                                    }}
                                                </Suspense>
                                            </div>
                                            
                                            <div class="booking-modal-message-input">
                                                <Textarea 
                                                    placeholder="Type your message..."
                                                    value=new_message
                                                />
                                                <Button 
                                                    appearance=ButtonAppearance::Primary
                                                    on_click=move |_| handle_send_message()
                                                    disabled=move || new_message.get().trim().is_empty()
                                                >
                                                    "Send Message"
                                                </Button>
                                            </div>
                                        </div>
                                    }.into(),
                                    _ => view! {}.into()
                                }}
                            </div>
                        </div>
                    }.into()
                } else {
                    view! {}.into()
                }
            }}
        </Modal>
    }
}