use leptos::prelude::*;
use thaw::*;

#[cfg(feature = "ssr")]
use rusqlite::{Connection, Result as SqliteResult};
#[cfg(feature = "ssr")]
use std::path::Path;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct BookingData {
    pub id: i32,
    pub client_name: String,
    pub client_email: String,
    pub client_phone: Option<String>,
    pub placement: String,
    pub size_inches: Option<String>,
    pub notes: Option<String>,
    pub booking_date: Option<String>,
    pub status: String,
    pub created_at: String,
}

#[cfg(feature = "ssr")]
pub fn get_booking_details(booking_id: i32) -> SqliteResult<Option<BookingData>> {
    let db_path = Path::new("tatteau.db");
    let conn = Connection::open(db_path)?;
    
    let mut stmt = conn.prepare(
        "SELECT id, client_name, client_email, client_phone, placement, size_inches, notes, 
         booking_date, status, created_at
         FROM bookings WHERE id = ?"
    )?;
    
    let booking_result = stmt.query_row([booking_id], |row| {
        Ok(BookingData {
            id: row.get(0)?,
            client_name: row.get(1)?,
            client_email: row.get(2)?,
            client_phone: row.get(3)?,
            placement: row.get(4)?,
            size_inches: row.get(5)?,
            notes: row.get(6)?,
            booking_date: row.get(7)?,
            status: row.get(8)?,
            created_at: row.get(9)?,
        })
    });
    
    match booking_result {
        Ok(booking) => Ok(Some(booking)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e),
    }
}

#[server(GetBookingDetails)]
pub async fn get_booking_details_server(booking_id: i32) -> Result<Option<BookingData>, ServerFnError> {
    get_booking_details(booking_id).map_err(|e| {
        ServerFnError::ServerError(format!("Database error: {}", e))
    })
}

#[component]
pub fn BookingDetails(booking_id: i32) -> impl IntoView {
    let booking_resource = Resource::new(
        move || booking_id,
        |id| async move { get_booking_details_server(id).await }
    );
    
    view! {
        <div class="booking-details">
            <div class="booking-details-header">
                <div class="header-content">
                    <a href="/artist/dashboard/calendar" class="back-button">
                        "← Back to Calendar"
                    </a>
                    <h1>"Booking Request Details"</h1>
                </div>
            </div>
            
            <div class="booking-details-content">
                <Suspense fallback=move || view! { <div class="loading">"Loading booking details..."</div> }>
                    {move || {
                        booking_resource.get().map(|result| {
                            match result {
                                Ok(Some(booking)) => {
                                    let status_class = format!("booking-status-badge status-{}", booking.status);
                                    let status_icon = match booking.status.as_str() {
                                        "pending" => "⏳",
                                        "accepted" => "✅",
                                        "declined" => "❌",
                                        _ => "📋"
                                    };
                                    let status_text = match booking.status.as_str() {
                                        "pending" => "Pending Review",
                                        "accepted" => "Accepted",
                                        "declined" => "Declined",
                                        _ => &booking.status
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
                                                <div class="overview-item">
                                                    <label>"Booking ID"</label>
                                                    <span class="value">{booking.id}</span>
                                                </div>
                                                <div class="overview-item">
                                                    <label>"Client Name"</label>
                                                    <span class="value">{booking.client_name.clone()}</span>
                                                </div>
                                                <div class="overview-item">
                                                    <label>"Contact Email"</label>
                                                    <span class="value">{booking.client_email.clone()}</span>
                                                </div>
                                                {booking.client_phone.as_ref().map(|phone| view! {
                                                    <div class="overview-item">
                                                        <label>"Phone Number"</label>
                                                        <span class="value">{phone.clone()}</span>
                                                    </div>
                                                })}
                                                <div class="overview-item">
                                                    <label>"Placement"</label>
                                                    <span class="value">{booking.placement.clone()}</span>
                                                </div>
                                                {booking.size_inches.as_ref().map(|size| view! {
                                                    <div class="overview-item">
                                                        <label>"Size"</label>
                                                        <span class="value">{format!("{} inches", size)}</span>
                                                    </div>
                                                })}
                                                {booking.booking_date.as_ref().map(|date| view! {
                                                    <div class="overview-item">
                                                        <label>"Requested Date"</label>
                                                        <span class="value">{date.clone()}</span>
                                                    </div>
                                                })}
                                                <div class="overview-item">
                                                    <label>"Submitted"</label>
                                                    <span class="value">{booking.created_at.clone()}</span>
                                                </div>
                                            </div>
                                        </div>
                                        
                                        {booking.notes.as_ref().map(|notes| view! {
                                            <div class="booking-notes-card">
                                                <h2>"Client Notes"</h2>
                                                <div class="notes-content">
                                                    {notes.clone()}
                                                </div>
                                            </div>
                                        })}
                                        
                                        <div class="booking-actions-card">
                                            <h2>"Actions"</h2>
                                            <div class="actions-grid">
                                                {if booking.status == "pending" {
                                                    view! {
                                                        <Button appearance=ButtonAppearance::Primary>
                                                            "Accept Booking"
                                                        </Button>
                                                        <Button appearance=ButtonAppearance::Secondary>
                                                            "Decline Booking"
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
                                                    }.into_any()
                                                }}
                                            </div>
                                        </div>
                                    }.into_any()
                                },
                                Ok(None) => view! {
                                    <div class="error-card">
                                        <h2>"Booking Not Found"</h2>
                                        <p>"No booking found with ID "{booking_id}</p>
                                        <Button appearance=ButtonAppearance::Primary>
                                            <a href="/artist/dashboard/calendar" style="text-decoration: none; color: inherit;">
                                                "Back to Calendar"
                                            </a>
                                        </Button>
                                    </div>
                                }.into_any(),
                                Err(e) => view! {
                                    <div class="error-card">
                                        <h2>"Error Loading Booking"</h2>
                                        <p>{format!("Failed to load booking: {}", e)}</p>
                                        <Button appearance=ButtonAppearance::Primary>
                                            <a href="/artist/dashboard/calendar" style="text-decoration: none; color: inherit;">
                                                "Back to Calendar"
                                            </a>
                                        </Button>
                                    </div>
                                }.into_any()
                            }
                        })
                    }}
                </Suspense>
            </div>
        </div>
    }
}