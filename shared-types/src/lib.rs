use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LocationInfo {
    pub city: String,
    pub county: String,
    pub state: String,
    pub country_code: String,
    pub postal_code: String,
    pub is_open: bool,
    pub address: String,
    pub id: String,
    pub category: String,
    pub name: String,
    pub website_uri: String,
    pub lat: f64,
    pub long: f64,
} 