use std::env;

use reqwest::{header::HeaderMap, Client};
use serde_json::{json, Value};

use crate::geographical_location::{Coordinates, USCounties};

fn make_body(coords: Coordinates, page_size: i8, page_token: Option<&str>) -> Value {
    let Coordinates {
        low_lat,
        low_long,
        high_lat,
        high_long,
    } = coords;
    let mut body = json!({
        "pageSize": page_size,
        "textQuery": "Tattoo",
        "locationRestriction": {
            "rectangle": {
                "low": {
                    "latitude": low_lat,
                    "longitude": low_long,
                },
                "high": {
                    "latitude": high_lat,
                    "longitude": high_long,
                },
            }
        }
    });

    if let Some(token) = page_token {
        body.as_object_mut()
            .expect("Body should be mappable")
            .insert("pageToken".to_string(), json!(token));
    }

    body
}

fn make_headers() -> HeaderMap {
    let google_key: String = env::var("GOOGLE_PLACES_KEY").expect("Google key to be set");

    let mut headers: HeaderMap = HeaderMap::new();
    headers.insert(
        "Content-Type",
        "application/json".parse().expect("Header to parse"),
    );
    headers.insert(
        "X-Goog-Api-Key",
        google_key.parse().expect("Header to parse"),
    );
    headers.insert("X-Goog-FieldMask", "nextPageToken,places.location,places.photos.heightPx,places.photos.widthPx,places.photos.authorAttributions.photoUri,places.displayName,places.formattedAddress,places.addressComponents,places.primaryType,places.primaryTypeDisplayName,places.id,places.nationalPhoneNumber,places.internationalPhoneNumber,places.rating,places.websiteUri,places.businessStatus,places.websiteUri".parse().expect("Header to parse"));

    headers
}

pub async fn fetch_data(
    location: &USCounties,
    limit_results_to: i8,
    current_token: &Option<String>,
) -> Value {
    let url = "https://places.googleapis.com/v1/places:searchText";
    let client = Client::new();
    client
        .post(url)
        .json(&make_body(
            location.get_coords(),
            limit_results_to,
            current_token.as_deref(),
        ))
        .headers(make_headers())
        .send()
        .await
        .expect("Payload is valid")
        .json::<Value>()
        .await
        .expect("Result is valid JSON")
}
