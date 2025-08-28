use serde::{Deserialize, Serialize};

#[cfg(feature = "ssr")]
use chrono::{NaiveDate, NaiveTime};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct CityCoords {
    pub city: String,
    pub state: String,
    pub lat: f64,
    pub long: f64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Artist {
    pub id: i32,
    pub name: Option<String>,
    pub location_id: i32,
    pub social_links: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub years_experience: Option<i32>,
    pub styles_extracted: Option<i32>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ArtistStyle {
    pub id: i32,
    pub style_id: i32,
    pub artist_id: i32,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ArtistImage {
    pub id: i32,
    pub short_code: String,
    pub artist_id: i32,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ArtistImageStyle {
    pub id: i32,
    pub artists_images_id: i32,
    pub style_id: i32,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Style {
    pub id: i32,
    pub name: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Location {
    pub id: i32,
    pub name: Option<String>,
    pub lat: Option<f64>,
    pub long: Option<f64>,
    pub city: Option<String>,
    pub state: Option<String>,
    pub address: Option<String>,
    pub website_uri: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct AvailabilitySlot {
    pub id: i32,
    pub artist_id: i32,
    pub day_of_week: Option<i32>,
    pub specific_date: Option<String>,
    pub start_time: Option<String>,
    pub end_time: Option<String>,
    pub is_available: bool,
    pub is_recurring: bool,
    pub created_at: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct BookingRequest {
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
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct BookingMessage {
    pub id: i32,
    pub booking_request_id: i32,
    pub sender_type: String,
    pub message: String,
    pub created_at: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct BookingHistory {
    pub id: i32,
    pub artist_id: i32,
    pub booking_request_id: i32,
    pub appointment_date: String,
    pub start_time: String,
    pub end_time: Option<String>,
    pub status: String,
    pub notes: Option<String>,
    pub final_price: Option<f64>,
    pub created_at: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CalendarDay {
    pub date: String,
    pub is_available: bool,
    pub availability_slots: Vec<TimeSlot>,
    pub bookings: Vec<BookingRequest>,
    pub is_blocked: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TimeSlot {
    pub start_time: String,
    pub end_time: String,
    pub is_available: bool,
    pub booking_id: Option<i32>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AvailabilityUpdate {
    pub artist_id: i32,
    pub date: Option<String>,
    pub day_of_week: Option<i32>,
    pub start_time: String,
    pub end_time: String,
    pub is_available: bool,
    pub is_recurring: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct RecurringRule {
    pub id: i32,
    pub artist_id: i32,
    pub name: String,
    pub rule_type: String, // "weekdays", "dates", "monthly"
    pub pattern: String, // JSON string for flexible pattern storage
    pub action: String, // "available" or "blocked"
    pub start_time: Option<String>,
    pub end_time: Option<String>,
    pub active: bool,
    pub created_at: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CreateRecurringRule {
    pub artist_id: i32,
    pub name: String,
    pub rule_type: String,
    pub pattern: String,
    pub action: String,
    pub start_time: Option<String>,
    pub end_time: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct UpdateRecurringRule {
    pub id: i32,
    pub name: Option<String>,
    pub pattern: Option<String>,
    pub action: Option<String>,
    pub start_time: Option<String>,
    pub end_time: Option<String>,
    pub active: Option<bool>,
}
