// Common Google Places API service module
// Provides reusable functions for interacting with Google Places API

use once_cell::sync::Lazy;
use reqwest::{header::HeaderMap, Client};
use serde_json::{json, Value};
use shared_types::LocationInfo;
use std::collections::HashSet;
use std::env;

// Bounding box for geographic restriction
#[derive(Debug, Clone)]
pub struct LocationBounds {
    pub low_lat: f32,
    pub low_long: f32,
    pub high_lat: f32,
    pub high_long: f32,
}

// Categories to exclude from results (from existing parser.rs)
static EXCLUDE_CATEGORIES: Lazy<HashSet<String>> = Lazy::new(|| {
    [
        "grocery_store",
        "beauty_salon",
        "bakery",
        "",
        "barber_shop",
        "restaurant",
        "sporting_goods_store",
        "wholesaler",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect()
});

/// Search Google Places by text query with location restriction
/// Returns raw JSON response from Google Places API
pub async fn search_text_with_location(
    query: &str,
    bounds: &LocationBounds,
    page_size: i8,
) -> Result<Value, Box<dyn std::error::Error>> {
    let api_key = env::var("GOOGLE_PLACES_API_KEY")?;
    let url = "https://places.googleapis.com/v1/places:searchText";

    let body = json!({
        "textQuery": query,
        "pageSize": page_size,
        "locationRestriction": {
            "rectangle": {
                "low": {
                    "latitude": bounds.low_lat,
                    "longitude": bounds.low_long
                },
                "high": {
                    "latitude": bounds.high_lat,
                    "longitude": bounds.high_long
                }
            }
        }
    });

    let mut headers = HeaderMap::new();
    headers.insert("Content-Type", "application/json".parse()?);
    headers.insert("X-Goog-Api-Key", api_key.parse()?);
    headers.insert(
        "X-Goog-FieldMask",
        "places.location,places.photos.heightPx,places.photos.widthPx,places.photos.authorAttributions.photoUri,places.displayName,places.formattedAddress,places.addressComponents,places.primaryType,places.primaryTypeDisplayName,places.id,places.nationalPhoneNumber,places.internationalPhoneNumber,places.rating,places.websiteUri,places.businessStatus,places.types"
            .parse()?,
    );

    let client = Client::new();
    let response = client.post(url).headers(headers).json(&body).send().await?;

    if !response.status().is_success() {
        let status = response.status();
        let error_text = response.text().await?;
        return Err(format!("API error {}: {}", status, error_text).into());
    }

    let result: Value = response.json().await?;
    Ok(result)
}

/// Search Google Places by text query within a rectangular region (from existing fetcher.rs)
/// Used for county-based searching
pub async fn search_text_in_rectangle(
    text_query: &str,
    bounds: &LocationBounds,
    page_size: i8,
    page_token: Option<&str>,
) -> Result<Value, Box<dyn std::error::Error>> {
    let api_key = env::var("GOOGLE_PLACES_API_KEY")?;
    let url = "https://places.googleapis.com/v1/places:searchText";

    let mut body = json!({
        "pageSize": page_size,
        "textQuery": text_query,
        "locationRestriction": {
            "rectangle": {
                "low": {
                    "latitude": bounds.low_lat,
                    "longitude": bounds.low_long
                },
                "high": {
                    "latitude": bounds.high_lat,
                    "longitude": bounds.high_long
                }
            }
        }
    });

    if let Some(token) = page_token {
        body.as_object_mut()
            .expect("Body should be mappable")
            .insert("pageToken".to_string(), json!(token));
    }

    let mut headers = HeaderMap::new();
    headers.insert("Content-Type", "application/json".parse()?);
    headers.insert("X-Goog-Api-Key", api_key.parse()?);
    headers.insert("X-Goog-FieldMask", "nextPageToken,places.location,places.photos.heightPx,places.photos.widthPx,places.photos.authorAttributions.photoUri,places.displayName,places.formattedAddress,places.addressComponents,places.primaryType,places.primaryTypeDisplayName,places.id,places.nationalPhoneNumber,places.internationalPhoneNumber,places.rating,places.websiteUri,places.businessStatus,places.websiteUri".parse()?);

    let client = Client::new();
    let response = client.post(url).headers(headers).json(&body).send().await?;

    if !response.status().is_success() {
        let status = response.status();
        let error_text = response.text().await?;
        return Err(format!("API error {}: {}", status, error_text).into());
    }

    let result: Value = response.json().await?;
    Ok(result)
}

/// Parse Google Places API response into LocationInfo structs
/// Filters out excluded categories (grocery stores, beauty salons, etc.)
pub fn parse_places_to_locations(value: &Value) -> Vec<LocationInfo> {
    let places = match value.get("places").and_then(|p| p.as_array()) {
        Some(p) => p,
        None => return Vec::new(),
    };

    places
        .iter()
        .map(|place| convert_place_to_location_info(place))
        .filter(|l| !EXCLUDE_CATEGORIES.contains(&l.category))
        .collect()
}

/// Convert a single Google Place JSON object to LocationInfo struct
/// (from existing parser.rs logic)
fn convert_place_to_location_info(val: &Value) -> LocationInfo {
    let address_components = val
        .get("addressComponents")
        .and_then(Value::as_array)
        .map(|v| v.as_slice())
        .unwrap_or(&[]);

    let fallback = |i: usize, key: &str| {
        val.get("addressComponents")
            .and_then(|v| v.get(i))
            .and_then(|v| v.get(key))
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string()
    };

    LocationInfo {
        city: extract_component(address_components, "locality")
            .unwrap_or_else(|| fallback(3, "longText")),
        county: extract_component(address_components, "administrative_area_level_2")
            .unwrap_or_else(|| fallback(4, "longText")),
        state: extract_component(address_components, "administrative_area_level_1")
            .unwrap_or_else(|| fallback(5, "longText")),
        country_code: extract_component(address_components, "country")
            .unwrap_or_else(|| fallback(6, "longText")),
        postal_code: extract_component(address_components, "postal_code").unwrap_or_else(|| {
            val.get("addressComponents")
                .and_then(|v| v.get(7))
                .and_then(|v| v.get("shortText").or_else(|| v.get("longText")))
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_string()
        }),
        is_open: extract_string(&val["businessStatus"]) == "OPERATIONAL",
        address: extract_string(&val["formattedAddress"]),
        _id: extract_string(&val["id"]),
        category: extract_string(&val["primaryType"]),
        name: extract_string(&val["displayName"]["text"]),
        website_uri: extract_string(&val["websiteUri"]),
        lat: extract_f64(&val["location"]["latitude"]),
        long: extract_f64(&val["location"]["longitude"]),
        id: -1,
        ..Default::default()
    }
}

fn extract_string(val: &Value) -> String {
    val.as_str().unwrap_or_default().to_string()
}

fn extract_f64(val: &Value) -> f64 {
    val.as_f64().unwrap_or(0.0)
}

fn extract_component(components: &[Value], type_name: &str) -> Option<String> {
    components.iter().find_map(|ac| {
        let types = ac.get("types")?.as_array()?;
        if types.iter().any(|t| t == type_name) {
            ac.get("longText")
                .and_then(|t| t.as_str())
                .map(|s| s.to_string())
        } else {
            None
        }
    })
}

/// Check if a place has tattoo-related types (body_art_service, etc.)
pub fn is_tattoo_shop(place: &Value) -> bool {
    place
        .get("types")
        .and_then(|t| t.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str())
                .any(|t| t == "body_art_service" || t == "art_studio")
        })
        .unwrap_or(false)
}
