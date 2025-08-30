use leptos::prelude::*;
use leptos::wasm_bindgen::JsCast;
use web_sys::HtmlInputElement;
use thaw::*;

#[cfg(feature = "ssr")]
use rusqlite::{Connection, Result as SqliteResult};
#[cfg(feature = "ssr")]
use std::path::Path;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct BookingRequestData {
    pub id: i32,
    pub artist_id: i32,
    pub client_name: String,
    pub client_email: String,
    pub client_phone: Option<String>,
    pub requested_date: String,
    pub requested_start_time: String,
    pub requested_end_time: Option<String>,
    pub tattoo_description: Option<String>,
    pub placement: Option<String>,
    pub size_inches: Option<f64>,
    pub reference_images: Option<String>,
    pub message_from_client: Option<String>,
    pub status: String,
    pub artist_response: Option<String>,
    pub estimated_price: Option<f64>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct BookingHistoryItem {
    pub id: i32,
    pub booking_date: Option<String>,
    pub status: String,
    pub created_at: String,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct BookingMessage {
    pub id: i32,
    pub sender_type: String,
    pub message: String,
    pub created_at: String,
}

#[component]
pub fn BookingDetails(booking_id: i32) -> impl IntoView {
    // TEMPORARY: Using only mock data to avoid stack overflow issue with server functions
    // TODO: Incrementally add real data fetches once the issue is resolved
    
    // Create mock booking data directly - no server fetch for now
    let booking_data = BookingRequestData {
        id: booking_id,
        artist_id: 1,
        client_name: "Sarah Johnson".to_string(),
        client_email: "sarah.johnson@example.com".to_string(),
        client_phone: Some("(555) 123-4567".to_string()),
        requested_date: "2024-03-15".to_string(),
        requested_start_time: "2:00 PM".to_string(),
        requested_end_time: Some("4:00 PM".to_string()),
        tattoo_description: Some("Small geometric mandala on my wrist, about 3 inches in diameter with intricate line work and dot patterns.".to_string()),
        placement: Some("Right wrist".to_string()),
        size_inches: Some(3.0),
        reference_images: None,
        message_from_client: Some("Hi! I've been following your work for a while and really love your geometric style. I'm looking to get my first tattoo and would love to work with you!".to_string()),
        status: "pending".to_string(),
        artist_response: None,
        estimated_price: Some(250.0),
        created_at: "2024-03-01 10:30:00".to_string(),
        updated_at: "2024-03-01 10:30:00".to_string(),
    };
    
    // Create mock booking history
    let mock_history = vec![
        BookingHistoryItem {
            id: 45,
            booking_date: Some("2024-01-20".to_string()),
            status: "completed".to_string(),
            created_at: "2024-01-05 14:20:00".to_string(),
        },
        BookingHistoryItem {
            id: 32,
            booking_date: Some("2023-11-15".to_string()),
            status: "completed".to_string(),
            created_at: "2023-10-28 09:15:00".to_string(),
        },
    ];
    
    // Create mock messages
    let mock_messages = vec![
        BookingMessage {
            id: 1,
            sender_type: "client".to_string(),
            message: "Looking forward to our session!".to_string(),
            created_at: "2024-03-02 11:15:00".to_string(),
        },
        BookingMessage {
            id: 2,
            sender_type: "artist".to_string(),
            message: "Great! I'll prepare some design options for you.".to_string(),
            created_at: "2024-03-02 14:30:00".to_string(),
        },
    ];
    
    // Simple signal for current booking status
    let (booking_status, set_booking_status) = signal::<String>(booking_data.status.clone());
    
    view! {
        <div class="booking-details">
            <BookingDetailsHeader />
            
            <div class="booking-details-content">
                <BookingOverviewCard 
                    booking=booking_data.clone() 
                    status=booking_status 
                />
                
                <BookingDescriptionCard 
                    description=booking_data.tattoo_description.clone() 
                />
                
                <BookingClientMessageCard 
                    message=booking_data.message_from_client.clone() 
                />
                
                <BookingHistoryCard 
                    history=mock_history 
                />
                
                <BookingMessagesCard 
                    messages=mock_messages 
                    booking_id=booking_data.id 
                />
                
                <BookingActionsCard 
                    status=booking_status 
                    set_status=set_booking_status 
                />
            </div>
        </div>
    }
}

#[component]
fn BookingDetailsHeader() -> impl IntoView {
    view! {
        <div class="booking-details-header">
            <div class="header-content">
                <a href="/artist/dashboard/calendar" class="back-button">
                    "← Back to Calendar"
                </a>
                <h1>"Booking Request Details"</h1>
            </div>
        </div>
    }
}

#[component]
fn BookingOverviewCard(
    booking: BookingRequestData,
    status: ReadSignal<String>,
) -> impl IntoView {
    view! {
        {move || {
            let current_status = status.get();
            let status_class = format!("booking-status-badge status-{}", current_status);
            let status_icon = match current_status.as_str() {
                "pending" => "⏳",
                "approved" => "✅", 
                "declined" => "❌",
                _ => "📋"
            };
            let status_text = match current_status.as_str() {
                "pending" => "Pending Review",
                "approved" => "Approved",
                "declined" => "Declined", 
                _ => &current_status
            };
            
            view! {
                <div class="booking-overview-card">
                    <div class="card-header">
                        <h2>"Booking Overview"</h2>
                        <div class=status_class>
                            {format!("{} {}", status_icon, status_text)}
                        </div>
                    </div>
                    
                    <div class="booking-overview-grid">
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
                            value=format!("{} at {}", booking.requested_date, booking.requested_start_time) 
                        />
                        
                        {booking.requested_end_time.as_ref().map(|end_time| view! {
                            <BookingOverviewItem 
                                label="Duration" 
                                value=format!("{} - {}", booking.requested_start_time, end_time) 
                            />
                        })}
                        
                        <BookingOverviewItem label="Submitted" value=booking.created_at.clone() />
                    </div>
                </div>
            }
        }}
    }
}

#[component]
fn BookingOverviewItem(label: &'static str, value: String) -> impl IntoView {
    view! {
        <div class="overview-item">
            <label>{label}</label>
            <span class="value">{value}</span>
        </div>
    }
}

#[component]
fn BookingDescriptionCard(description: Option<String>) -> impl IntoView {
    description.map(|desc| view! {
        <div class="booking-description-card">
            <h2>"Tattoo Description"</h2>
            <div class="description-content">
                {desc}
            </div>
        </div>
    })
}

#[component]
fn BookingClientMessageCard(message: Option<String>) -> impl IntoView {
    message.map(|msg| view! {
        <div class="booking-notes-card">
            <h2>"Client Message"</h2>
            <div class="notes-content">
                {msg}
            </div>
        </div>
    })
}

#[component]
fn BookingActionsCard(
    status: ReadSignal<String>,
    set_status: WriteSignal<String>,
) -> impl IntoView {
    let accept_booking = move |_| {
        set_status.set("approved".to_string());
    };

    let decline_booking = move |_| {
        set_status.set("declined".to_string());
    };
    
    view! {
        <div class="booking-actions-card">
            <h2>"Actions"</h2>
            <div class="actions-grid">
                {move || {
                    let current_status = status.get();
                    if current_status == "pending" {
                        view! {
                            <Button appearance=ButtonAppearance::Primary on_click=accept_booking>
                                "Accept Booking"
                            </Button>
                            <Button appearance=ButtonAppearance::Secondary on_click=decline_booking>
                                "Decline Booking"
                            </Button>
                            <Button appearance=ButtonAppearance::Subtle>
                                "Suggest New Date/Time"
                            </Button>
                        }
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
                        }
                    }
                }}
            </div>
        </div>
    }
}

#[component]
pub fn BookingHistoryCard(history: Vec<BookingHistoryItem>) -> impl IntoView {
    view! {
        <div class="booking-history-card">
            <div class="card-header">
                <h2>"Client Booking History"</h2>
                <span class="history-count">
                    {format!("{} bookings", history.len())}
                </span>
            </div>
            <div class="history-list">
                {
                    if history.is_empty() {
                        view! {
                            <div class="no-history">
                                "No previous bookings found for this client."
                            </div>
                        }.into_any()
                    } else {
                        history.into_iter().take(5).map(|item| {
                            view! {
                                <BookingHistoryItem item=item />
                            }
                        }).collect_view().into_any()
                    }
                }
            </div>
        </div>
    }
}

#[component]
fn BookingHistoryItem(item: BookingHistoryItem) -> impl IntoView {
    let status_class = format!("history-status status-{}", item.status);
    let status_icon = match item.status.as_str() {
        "pending" => "⏳",
        "approved" => "✅", 
        "declined" => "❌",
        "completed" => "🎨",
        _ => "📋"
    };
    
    let booking_id = item.id;
    let navigate_to_booking = move |_| {
        // TODO: Add navigation to /artist/dashboard/booking/{booking_id} when router is working
        leptos::logging::log!("Would navigate to booking {}", booking_id);
    };
    
    view! {
        <div class="history-item" on:click=navigate_to_booking>
            <div class="history-details">
                <div class="history-id">
                    "Booking #"{item.id}
                </div>
                <div class="history-date">
                    {item.booking_date.unwrap_or_else(|| "Date TBD".to_string())}
                </div>
                <div class="history-created">
                    "Submitted "{item.created_at}
                </div>
            </div>
            <div class="history-status-container">
                <div class=status_class>
                    {format!("{} {}", status_icon, item.status)}
                </div>
                <div class="history-arrow">
                    "→"
                </div>
            </div>
        </div>
    }
}

#[component]
pub fn BookingMessagesCard(messages: Vec<BookingMessage>, booking_id: i32) -> impl IntoView {
    let _ = booking_id; // TODO: Use for real message sending when implemented
    let new_message = RwSignal::new("".to_string());
    
    let send_message = move |_| {
        let message_content = new_message.get().trim().to_string();
        if !message_content.is_empty() {
            new_message.set("".to_string());
        }
    };
    
    view! {
        <div class="booking-messages-card">
            <div class="card-header">
                <h2>"Messages"</h2>
                <span class="message-count">
                    {format!("{} messages", messages.len())}
                </span>
            </div>
            
            <div class="messages-list">
                {
                    if messages.is_empty() {
                        view! {
                            <div class="no-messages">
                                "No messages yet. Start the conversation!"
                            </div>
                        }.into_any()
                    } else {
                        messages.into_iter().map(|msg| {
                            view! {
                                <BookingMessageItem message=msg />
                            }
                        }).collect_view().into_any()
                    }
                }
            </div>
            
            <div class="message-input-container">
                <input 
                    type="text"
                    prop:value=move || new_message.get()
                    on:input=move |ev| {
                        let target = ev.target().unwrap();
                        let input = target.unchecked_into::<HtmlInputElement>();
                        new_message.set(input.value());
                    }
                    placeholder="Type your message..."
                    class="message-input"
                />
                <Button 
                    appearance=ButtonAppearance::Primary
                    on_click=send_message
                >
                    "Send"
                </Button>
            </div>
        </div>
    }
}

#[component]
fn BookingMessageItem(message: BookingMessage) -> impl IntoView {
    let sender_class = format!("message-item sender-{}", message.sender_type);
    let sender_label = match message.sender_type.as_str() {
        "client" => "Client",
        "artist" => "You",
        _ => "System"
    };
    
    view! {
        <div class=sender_class>
            <div class="message-header">
                <span class="sender-name">{sender_label}</span>
                <span class="message-time">{message.created_at}</span>
            </div>
            <div class="message-content">
                {message.message}
            </div>
        </div>
    }
}