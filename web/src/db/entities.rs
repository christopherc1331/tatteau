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
    pub decline_reason: Option<String>,
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

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct BusinessHours {
    pub id: i32,
    pub artist_id: i32,
    pub day_of_week: i32, // 0=Sunday, 1=Monday, ..., 6=Saturday
    pub start_time: Option<String>,
    pub end_time: Option<String>,
    pub is_closed: bool,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct UpdateBusinessHours {
    pub artist_id: i32,
    pub day_of_week: i32,
    pub start_time: Option<String>,
    pub end_time: Option<String>,
    pub is_closed: bool,
}

// Subscription System Entities
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct SubscriptionTier {
    pub id: i32,
    pub tier_name: String,
    pub tier_level: i32,
    pub price_monthly: f64,
    pub features_json: Option<String>,
    pub created_at: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ArtistSubscription {
    pub id: i32,
    pub artist_id: i32,
    pub tier_id: i32,
    pub status: String, // 'active', 'cancelled', 'expired', 'trial'
    pub payment_method: Option<String>,
    pub subscription_start: Option<String>,
    pub subscription_end: Option<String>,
    pub last_payment: Option<String>,
    pub next_payment: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CreateArtistSubscription {
    pub artist_id: i32,
    pub tier_id: i32,
    pub status: String,
    pub payment_method: Option<String>,
}

// Questionnaire System Entities
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct QuestionnaireQuestion {
    pub id: i32,
    pub question_type: String, // 'style', 'body_part', 'text', 'datetime', 'multiselect'
    pub question_text: String,
    pub is_default: bool,
    pub options_data: Option<String>, // JSON for select/multiselect options
    pub validation_rules: Option<String>, // JSON validation config
    pub created_at: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ArtistQuestionnaire {
    pub id: i32,
    pub artist_id: i32,
    pub question_id: i32,
    pub is_required: bool,
    pub display_order: i32,
    pub custom_options: Option<String>, // JSON override for default questions
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct BookingQuestionnaireResponse {
    pub id: i32,
    pub booking_request_id: i32,
    pub question_id: i32,
    pub response_text: Option<String>,
    pub response_data: Option<String>, // JSON for complex responses
    pub created_at: Option<String>,
}

// For client booking flow
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ClientQuestionnaireForm {
    pub artist_id: i32,
    pub questions: Vec<ClientQuestionnaireQuestion>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ClientQuestionnaireQuestion {
    pub id: i32,
    pub question_type: String,
    pub question_text: String,
    pub is_required: bool,
    pub options: Vec<String>, // Parsed from options_data/custom_options
    pub validation_rules: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ClientQuestionnaireSubmission {
    pub booking_request_id: i32,
    pub responses: Vec<QuestionnaireResponse>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct QuestionnaireResponse {
    pub question_id: i32,
    pub response_text: Option<String>,
    pub response_data: Option<String>, // JSON for arrays/complex data
}

// Error Logging System
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ErrorLog {
    pub id: i32,
    pub error_type: String, // 'client', 'server', 'database'
    pub error_level: String, // 'error', 'warn', 'fatal'
    pub error_message: String,
    pub error_stack: Option<String>,
    pub url_path: Option<String>,
    pub user_agent: Option<String>,
    pub user_id: Option<i32>, // NULL if not logged in
    pub session_id: Option<String>,
    pub timestamp: String,
    pub request_headers: Option<String>, // JSON string
    pub additional_context: Option<String>, // JSON string for any extra data
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CreateErrorLog {
    pub error_type: String,
    pub error_level: String,
    pub error_message: String,
    pub error_stack: Option<String>,
    pub url_path: Option<String>,
    pub user_agent: Option<String>,
    pub user_id: Option<i32>,
    pub session_id: Option<String>,
    pub request_headers: Option<String>,
    pub additional_context: Option<String>,
}
