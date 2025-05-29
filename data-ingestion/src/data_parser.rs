use once_cell::sync::Lazy;
use serde_json::Value;
use shared_types::LocationInfo;
use std::collections::HashSet;

#[derive(Debug)]
pub struct ParsedLocationData<'a> {
    pub location_info: Vec<LocationInfo>,
    pub next_token: Option<&'a str>,
    pub filtered_count: usize,
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

fn convert_val_obj_to_location_info(val: &Value) -> LocationInfo {
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
        id: extract_string(&val["id"]),
        category: extract_string(&val["primaryType"]),
        name: extract_string(&val["displayName"]["text"]),
        website_uri: extract_string(&val["websiteUri"]),
        lat: extract_f64(&val["location"]["latitude"]),
        long: extract_f64(&val["location"]["longitude"]),
    }
}

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

pub fn parse_data(value: &Value) -> Option<ParsedLocationData> {
    let places = value.get("places")?.as_array()?;

    let locations: Vec<LocationInfo> = places
        .iter()
        .map(convert_val_obj_to_location_info)
        .collect();

    let filtered_locations: Vec<_> = locations
        .into_iter()
        .filter(|l| !EXCLUDE_CATEGORIES.contains(&l.category))
        .collect();

    let filtered_count = places.len() - filtered_locations.len();

    Some(ParsedLocationData {
        location_info: filtered_locations,
        next_token: value["nextPageToken"].as_str(),
        filtered_count,
    })
}
