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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CountyBoundary {
    pub name: String,
    pub low_lat: f64,
    pub low_long: f64,
    pub high_lat: f64,
    pub high_long: f64,
    pub date_utc_last_ingested: Option<i64>,
}

#[derive(Default, Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct LatLong {
    pub lat: f64,
    pub long: f64,
}

#[derive(Default, Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct MapBounds {
    pub north_east: LatLong,
    pub south_west: LatLong,
}
