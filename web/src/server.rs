use leptos::prelude::*;
use leptos::server;
use shared_types::LocationInfo;
use shared_types::MapBounds;

#[cfg(feature = "ssr")]
use tracing::instrument;

use crate::db::entities::{
    Artist, ArtistImage, ArtistQuestionnaire, ArtistSubscription, AvailabilitySlot,
    AvailabilityUpdate, BookingMessage, BookingQuestionnaireResponse, BookingRequest, CityCoords,
    ClientQuestionnaireForm, ClientQuestionnaireSubmission, CreateErrorLog, ErrorLog, Location,
    QuestionnaireQuestion, RecurringRule, Style, SubscriptionTier,
};
use crate::db::search_repository::SearchResult;
use serde::{Deserialize, Serialize};

#[cfg(feature = "ssr")]
use chrono::{Datelike, NaiveDateTime, Utc};

#[derive(Deserialize)]
struct InstagramOEmbedResponse {
    html: String,
    width: Option<u32>,
    height: Option<u32>,
    #[serde(rename = "type")]
    embed_type: String,
    version: String,
    provider_name: String,
}

#[cfg(feature = "ssr")]
use crate::db::repository::{
    check_artist_availability, delete_artist_question, get_all_default_questions,
    get_all_images_with_styles_by_location, get_all_styles_by_location, get_artist_by_id,
    get_artist_id_from_user_id, get_artist_images_with_styles, get_artist_location,
    get_artist_questionnaire, get_artist_questionnaire_config, get_artist_styles,
    get_artists_by_location, get_booking_questionnaire_responses, get_cities_and_coords,
    get_city_coordinates, get_errors_by_type, get_location_by_id, get_recent_errors, get_states,
    get_styles_by_location, log_error, query_locations, save_questionnaire_responses,
    update_artist_questionnaire_config,
};

// Helper function to extract user_id from JWT token
#[cfg(feature = "ssr")]
fn extract_user_id_from_token(token: &str) -> Option<i64> {
    use jsonwebtoken::{decode, DecodingKey, Validation};

    #[derive(Debug, Serialize, Deserialize)]
    struct Claims {
        sub: String,
        exp: usize,
        user_type: String,
        user_id: i64,
    }

    let secret = "tatteau-jwt-secret-key-change-in-production";
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_ref()),
        &Validation::default(),
    )
    .ok()?;

    Some(token_data.claims.user_id)
}

#[cfg_attr(feature = "ssr", instrument(skip(bounds), err, level = "info"))]
#[server]
#[cfg_attr(feature = "ssr", instrument(skip(bounds), err, level = "info"))]
pub async fn fetch_locations(
    state: String,
    city: String,
    bounds: MapBounds,
) -> Result<Vec<LocationInfo>, ServerFnError> {
    match query_locations(state, city, bounds).await {
        Ok(locations) => Ok(locations),
        Err(e) => Err(ServerFnError::new(format!("Database error: {}", e))),
    }
}

#[cfg_attr(feature = "ssr", instrument(err, level = "info"))]
#[server]
#[cfg_attr(feature = "ssr", instrument(err, level = "info"))]
pub async fn get_cities(state: String) -> Result<Vec<CityCoords>, ServerFnError> {
    match get_cities_and_coords(state).await {
        Ok(cities) => Ok(cities),
        Err(e) => Err(ServerFnError::new(format!("Database error: {}", e))),
    }
}

#[cfg_attr(feature = "ssr", instrument(err, level = "info"))]
#[server]
#[cfg_attr(feature = "ssr", instrument(err, level = "info"))]
pub async fn get_states_list() -> Result<Vec<String>, ServerFnError> {
    match get_states().await {
        Ok(states) => Ok(states.into_iter().map(|s| s.state).collect()),
        Err(e) => Err(ServerFnError::new(format!("Database error: {}", e))),
    }
    // Ok(vec![
    //     "Texas".to_string(),
    //     "California".to_string(),
    //     "New York".to_string(),
    // ])
}

#[cfg_attr(feature = "ssr", instrument(skip(cities), err, level = "info"))]
#[server]
#[cfg_attr(feature = "ssr", instrument(skip(cities), err, level = "info"))]
pub async fn get_center_coordinates_for_cities(
    cities: Vec<CityCoords>,
) -> Result<CityCoords, ServerFnError> {
    let city_name = &cities
        .first()
        .expect("At least one city should be passed")
        .city;

    match get_city_coordinates(city_name.to_string()).await {
        Ok(coords) => Ok(coords),
        Err(_) => get_geographic_center(&cities)
            .ok_or_else(|| ServerFnError::new("No coordinates found".to_string())),
    }
}

fn get_geographic_center(cities: &[CityCoords]) -> Option<CityCoords> {
    if cities.is_empty() {
        return None;
    }

    let (mut x_total, mut y_total, mut z_total) = (0.0, 0.0, 0.0);
    cities.iter().for_each(|city| {
        let lat_rad = city.lat.to_radians();
        let long_rad = city.long.to_radians();

        x_total += lat_rad.cos() * long_rad.cos();
        y_total += lat_rad.cos() * long_rad.sin();
        z_total += lat_rad.sin();
    });

    let count = cities.len() as f64;
    let x_avg = x_total / count;
    let y_avg = y_total / count;
    let z_avg = z_total / count;

    let long = y_avg.atan2(x_avg).to_degrees();
    let hyp = (x_avg.powi(2) + y_avg.powi(2)).sqrt();
    let lat = z_avg.atan2(hyp).to_degrees();

    cities.first().map(|first_city| CityCoords {
        city: first_city.clone().city,
        state: first_city.clone().state,
        lat,
        long,
    })
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct ArtistData {
    pub artist: Artist,
    pub location: Location,
    pub styles: Vec<Style>,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct ShopData {
    pub location: Location,
    pub artists: Vec<Artist>,
    pub all_styles: Vec<Style>,
}

#[cfg_attr(feature = "ssr", instrument(err, level = "info"))]
#[server]
#[cfg_attr(feature = "ssr", instrument(err, level = "info"))]
pub async fn fetch_artist_data(artist_id: i32) -> Result<ArtistData, ServerFnError> {
    let artist = get_artist_by_id(artist_id)
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to fetch artist: {}", e)))?;

    let location = get_artist_location(artist.location_id)
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to fetch location: {}", e)))?;

    let styles = get_artist_styles(artist_id)
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to fetch styles: {}", e)))?;

    Ok(ArtistData {
        artist,
        location,
        styles,
    })
}

#[cfg_attr(feature = "ssr", instrument(skip(token), err, level = "info"))]
#[server]
#[cfg_attr(feature = "ssr", instrument(skip(token), err, level = "info"))]
pub async fn fetch_shop_data(
    location_id: i32,
    token: Option<String>,
) -> Result<ShopData, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        let user_id = token.as_deref().and_then(extract_user_id_from_token);

        let location = get_location_by_id(location_id)
            .await
            .map_err(|e| ServerFnError::new(format!("Failed to fetch location: {}", e)))?;

        let artists = get_artists_by_location(location_id)
            .await
            .map_err(|e| ServerFnError::new(format!("Failed to fetch artists: {}", e)))?;

        let all_styles = get_all_styles_by_location(location_id)
            .await
            .map_err(|e| ServerFnError::new(format!("Failed to fetch styles: {}", e)))?;

        Ok(ShopData {
            location,
            artists,
            all_styles,
        })
    }
    #[cfg(not(feature = "ssr"))]
    {
        Ok(ShopData {
            location: Location::default(),
            artists: vec![],
            all_styles: vec![],
        })
    }
}

#[cfg_attr(
    feature = "ssr",
    instrument(skip(token, style_ids), err, level = "info")
)]
#[server]
#[cfg_attr(
    feature = "ssr",
    instrument(skip(style_ids, token), err, level = "info")
)]
pub async fn fetch_shop_images_paginated(
    location_id: i32,
    style_ids: Option<Vec<i32>>,
    page: i32,
    per_page: i32,
    token: Option<String>,
) -> Result<(Vec<(ArtistImage, Vec<Style>, Artist, bool)>, i32), ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use crate::db::repository::get_shop_images_paginated;
        let user_id = token.as_deref().and_then(extract_user_id_from_token);
        match get_shop_images_paginated(location_id, style_ids, page, per_page, user_id).await {
            Ok(result) => Ok(result),
            Err(e) => Err(ServerFnError::new(format!(
                "Failed to fetch paginated shop images: {}",
                e
            ))),
        }
    }
    #[cfg(not(feature = "ssr"))]
    {
        Ok((Vec::new(), 0))
    }
}

#[cfg_attr(
    feature = "ssr",
    instrument(skip(token, style_ids), err, level = "info")
)]
#[server]
#[cfg_attr(
    feature = "ssr",
    instrument(skip(style_ids, token), err, level = "info")
)]
pub async fn fetch_artist_images_paginated(
    artist_id: i32,
    style_ids: Option<Vec<i32>>,
    page: i32,
    per_page: i32,
    token: Option<String>,
) -> Result<(Vec<(ArtistImage, Vec<Style>, bool)>, i32), ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use crate::db::repository::get_artist_images_paginated;
        let user_id = token.as_deref().and_then(extract_user_id_from_token);
        match get_artist_images_paginated(artist_id, style_ids, page, per_page, user_id).await {
            Ok(result) => Ok(result),
            Err(e) => Err(ServerFnError::new(format!(
                "Failed to fetch paginated artist images: {}",
                e
            ))),
        }
    }
    #[cfg(not(feature = "ssr"))]
    {
        let _ = (artist_id, style_ids, page, per_page, token);
        Ok((Vec::new(), 0))
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct LocationStats {
    pub shop_count: i32,
    pub artist_count: i32,
    pub styles_available: i32,
}

#[cfg_attr(feature = "ssr", instrument(err, level = "info"))]
#[server]
#[cfg_attr(feature = "ssr", instrument(err, level = "info"))]
pub async fn get_location_stats(
    city: String,
    state: String,
) -> Result<LocationStats, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use crate::db::repository::get_location_stats_for_city;
        match get_location_stats_for_city(city, state).await {
            Ok(stats) => Ok(stats),
            Err(e) => Err(ServerFnError::new(format!(
                "Failed to fetch location stats: {}",
                e
            ))),
        }
    }
    #[cfg(not(feature = "ssr"))]
    {
        Ok(LocationStats {
            shop_count: 0,
            artist_count: 0,
            styles_available: 0,
        })
    }
}

#[cfg_attr(feature = "ssr", instrument(err, level = "info"))]
#[server]
#[cfg_attr(feature = "ssr", instrument(err, level = "info"))]
pub async fn get_available_styles() -> Result<Vec<StyleWithCount>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use crate::db::repository::get_all_styles_with_counts;
        match get_all_styles_with_counts().await {
            Ok(styles) => Ok(styles),
            Err(e) => Err(ServerFnError::new(format!("Failed to fetch styles: {}", e))),
        }
    }
    #[cfg(not(feature = "ssr"))]
    {
        Ok(vec![])
    }
}

#[cfg_attr(feature = "ssr", instrument(err, level = "info"))]
#[server]
#[cfg_attr(feature = "ssr", instrument(skip(bounds), err, level = "info"))]
pub async fn get_styles_in_bounds(bounds: MapBounds) -> Result<Vec<StyleWithCount>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use crate::db::repository::get_styles_with_counts_in_bounds;
        match get_styles_with_counts_in_bounds(bounds).await {
            Ok(styles) => Ok(styles),
            Err(e) => Err(ServerFnError::new(format!(
                "Failed to fetch styles in bounds: {}",
                e
            ))),
        }
    }
    #[cfg(not(feature = "ssr"))]
    {
        Ok(vec![])
    }
}

#[cfg_attr(feature = "ssr", instrument(skip(cities, states), err, level = "info"))]
#[server]
#[cfg_attr(feature = "ssr", instrument(skip(states, cities), err, level = "info"))]
pub async fn get_styles_by_location_filter(
    states: Option<Vec<String>>,
    cities: Option<Vec<String>>,
) -> Result<Vec<StyleWithCount>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        match get_styles_by_location(states, cities).await {
            Ok(styles) => Ok(styles),
            Err(e) => Err(ServerFnError::new(format!(
                "Failed to fetch styles by location: {}",
                e
            ))),
        }
    }
    #[cfg(not(feature = "ssr"))]
    {
        Ok(vec![])
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct EnhancedLocationInfo {
    pub location: LocationInfo,
    pub artist_count: i32,
    pub image_count: i32,
    pub styles: Vec<String>,
    pub artists: Vec<ArtistThumbnail>,
    pub min_price: Option<f64>,
    pub max_price: Option<f64>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ArtistThumbnail {
    pub artist_id: i64,
    pub artist_name: String,
    pub image_url: Option<String>,
    pub primary_style: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct LocationDetailInfo {
    pub location: LocationInfo,
    pub artist_count: i32,
    pub image_count: i32,
    pub styles: Vec<String>,
    pub artists: Vec<ArtistThumbnail>,
    pub min_price: Option<f64>,
    pub max_price: Option<f64>,
    pub average_rating: Option<f64>,
}

#[cfg_attr(
    feature = "ssr",
    instrument(skip(bounds, style_filter), err, level = "info")
)]
#[server]
#[cfg_attr(
    feature = "ssr",
    instrument(skip(bounds, style_filter), err, level = "info")
)]
pub async fn get_locations_with_details(
    state: String,
    city: String,
    bounds: MapBounds,
    style_filter: Option<Vec<i32>>,
) -> Result<Vec<EnhancedLocationInfo>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use crate::db::repository::query_locations_with_details;
        match query_locations_with_details(state, city, bounds, style_filter).await {
            Ok(locations) => Ok(locations),
            Err(e) => {
                println!("{}", e.to_string());
                Err(ServerFnError::new(format!(
                    "Failed to fetch locations: {}",
                    e
                )))
            }
        }
    }
    #[cfg(not(feature = "ssr"))]
    {
        Ok(vec![])
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MatchedArtist {
    pub id: i64,
    pub name: String,
    pub location_name: String,
    pub city: String,
    pub state: String,
    pub primary_style: String,
    pub all_styles: Vec<String>,
    pub image_count: i32,
    pub portfolio_images: Vec<String>, // First 4 portfolio images
    pub avatar_url: Option<String>,    // First portfolio image as avatar
    pub avg_rating: f64,
    pub match_score: i32,
    pub years_experience: Option<i32>,
    pub min_price: Option<f64>,
    pub max_price: Option<f64>,
}

#[cfg_attr(
    feature = "ssr",
    instrument(skip(style_preferences), err, level = "info")
)]
#[server]
#[cfg_attr(
    feature = "ssr",
    instrument(skip(style_preferences), err, level = "info")
)]
pub async fn get_matched_artists(
    style_preferences: Vec<String>,
    location: String,
    price_range: Option<(f64, f64)>,
) -> Result<Vec<MatchedArtist>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use crate::db::repository::query_matched_artists;
        match query_matched_artists(style_preferences, location, price_range).await {
            Ok(artists) => Ok(artists),
            Err(e) => Err(ServerFnError::new(format!(
                "Failed to fetch matched artists: {}",
                e
            ))),
        }
    }
    #[cfg(not(feature = "ssr"))]
    {
        Ok(vec![])
    }
}

#[cfg_attr(feature = "ssr", instrument(err, level = "info"))]
#[server]
#[cfg_attr(feature = "ssr", instrument(err, level = "info"))]
pub async fn get_location_details(location_id: i32) -> Result<LocationDetailInfo, ServerFnError> {
    use crate::db::repository::get_location_with_artist_details;
    match get_location_with_artist_details(location_id).await {
        Ok(details) => Ok(details),
        Err(e) => {
            println!("Error fetching location details: {}", e);
            Err(ServerFnError::new(format!(
                "Failed to get location details: {}",
                e
            )))
        }
    }
}

#[cfg_attr(feature = "ssr", instrument(err, level = "info"))]
#[server]
#[cfg_attr(feature = "ssr", instrument(err, level = "info"))]
pub async fn search_by_postal_code(postal_code: String) -> Result<CityCoords, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use crate::db::repository::get_coords_by_postal_code;
        match get_coords_by_postal_code(postal_code).await {
            Ok(coords) => Ok(coords),
            Err(e) => Err(ServerFnError::new(format!(
                "Failed to find postal code: {}",
                e
            ))),
        }
    }
    #[cfg(not(feature = "ssr"))]
    {
        Ok(CityCoords {
            city: String::new(),
            state: String::new(),
            lat: 0.0,
            long: 0.0,
        })
    }
}

#[cfg_attr(feature = "ssr", instrument(err, level = "info"))]
#[server]
#[cfg_attr(feature = "ssr", instrument(err, level = "info"))]
pub async fn universal_search(query: String) -> Result<Vec<SearchResult>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use crate::db::search_repository::universal_location_search;
        match universal_location_search(query).await {
            Ok(results) => Ok(results),
            Err(e) => Err(ServerFnError::new(format!("Search failed: {}", e))),
        }
    }
    #[cfg(not(feature = "ssr"))]
    {
        Ok(vec![])
    }
}

#[cfg_attr(feature = "ssr", instrument(err, level = "info"))]
#[server]
#[cfg_attr(feature = "ssr", instrument(err, level = "debug"))]
pub async fn get_search_suggestions(query: String) -> Result<Vec<String>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use crate::db::search_repository::get_search_suggestions as get_suggestions;
        match get_suggestions(query, 10).await {
            Ok(suggestions) => Ok(suggestions),
            Err(e) => Err(ServerFnError::new(format!(
                "Failed to get suggestions: {}",
                e
            ))),
        }
    }
    #[cfg(not(feature = "ssr"))]
    {
        Ok(vec![])
    }
}

#[cfg_attr(feature = "ssr", instrument(err, level = "info"))]
#[server]
#[cfg_attr(feature = "ssr", instrument(err, level = "info"))]
pub async fn get_instagram_embed(short_code: String) -> Result<String, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        let url = format!(
            "https://www.instagram.com/p/{}/oembed/?url=https://www.instagram.com/p/{}/",
            short_code, short_code
        );

        let client = reqwest::Client::new();

        match client.get(&url).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    match response.json::<InstagramOEmbedResponse>().await {
                        Ok(oembed_data) => {
                            // Return the HTML embed code
                            Ok(oembed_data.html)
                        }
                        Err(e) => {
                            leptos::logging::log!("Failed to parse Instagram oEmbed JSON: {}", e);
                            Err(ServerFnError::new(format!(
                                "Failed to parse Instagram embed data for post: {}",
                                short_code
                            )))
                        }
                    }
                } else {
                    leptos::logging::log!(
                        "Instagram oEmbed API returned error status: {} for post: {}",
                        response.status(),
                        short_code
                    );
                    Err(ServerFnError::new(format!(
                        "Instagram post not found or not accessible: {}",
                        short_code
                    )))
                }
            }
            Err(e) => {
                leptos::logging::log!("HTTP request to Instagram oEmbed failed: {}", e);
                Err(ServerFnError::new(format!(
                    "Failed to fetch Instagram embed for post: {}",
                    short_code
                )))
            }
        }
    }
    #[cfg(not(feature = "ssr"))]
    {
        Err(ServerFnError::new(
            "Server-side rendering not available".to_string(),
        ))
    }
}

// Artist Dashboard Data Structures
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtistDashboardData {
    pub todays_bookings: i32,
    pub pending_sketches: i32,
    pub unread_messages: i32,
    pub monthly_revenue: f64,
    pub recent_bookings: Vec<RecentBooking>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecentBooking {
    pub id: i32,
    pub client_name: Option<String>,
    pub placement: Option<String>,
    pub created_at: String,
}

#[cfg_attr(feature = "ssr", instrument(err, level = "info"))]
#[server]
pub async fn get_artist_dashboard_data(
    artist_id: i32,
) -> Result<ArtistDashboardData, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use sqlx::Row;

        async fn query_dashboard_data(artist_id: i32) -> Result<ArtistDashboardData, sqlx::Error> {
            let pool = crate::db::pool::get_pool();

            // Get today's bookings count
            let today = Utc::now().naive_utc().date();
            let today_str = today.to_string();
            let row = sqlx::query(
                "SELECT COUNT(*) FROM bookings
                 WHERE artist_id = $1 AND DATE(created_at) = $2",
            )
            .bind(artist_id)
            .bind(&today_str)
            .fetch_one(pool)
            .await?;
            let todays_bookings: i64 = row.get(0);

            // Get pending sketch requests count (placeholder - would need sketch_requests table)
            let pending_sketches = 3; // Placeholder

            // Get unread messages count (placeholder - would need messages table)
            let unread_messages = 7; // Placeholder

            // Get monthly revenue (placeholder calculation)
            let monthly_revenue = 1250.0; // Placeholder

            // Get recent bookings
            let rows = sqlx::query(
                "SELECT b.id, b.client_name, b.placement, b.created_at
                 FROM bookings b
                 WHERE b.artist_id = $1
                 ORDER BY b.created_at DESC
                 LIMIT 5",
            )
            .bind(artist_id)
            .fetch_all(pool)
            .await?;

            let recent_bookings: Vec<RecentBooking> = rows
                .iter()
                .map(|row| {
                    let created_at_str: String = row.get("created_at");
                    // Try to parse the date string and format it, fallback to original if parsing fails
                    let formatted_date = if let Ok(naive_date) =
                        NaiveDateTime::parse_from_str(&created_at_str, "%Y-%m-%d %H:%M:%S")
                    {
                        naive_date.format("%B %d, %Y").to_string()
                    } else {
                        created_at_str.clone()
                    };

                    RecentBooking {
                        id: row.get("id"),
                        client_name: row.get("client_name"),
                        placement: row.get("placement"),
                        created_at: formatted_date,
                    }
                })
                .collect();

            Ok(ArtistDashboardData {
                todays_bookings: todays_bookings as i32,
                pending_sketches,
                unread_messages,
                monthly_revenue,
                recent_bookings,
            })
        }

        match query_dashboard_data(artist_id).await {
            Ok(data) => Ok(data),
            Err(e) => Err(ServerFnError::new(format!(
                "Failed to fetch artist dashboard data: {}",
                e
            ))),
        }
    }
    #[cfg(not(feature = "ssr"))]
    {
        // Return placeholder data for client-side
        Ok(ArtistDashboardData {
            todays_bookings: 0,
            pending_sketches: 0,
            unread_messages: 0,
            monthly_revenue: 0.0,
            recent_bookings: Vec::new(),
        })
    }
}

#[cfg_attr(feature = "ssr", instrument(err, level = "info"))]
#[server]
pub async fn log_match_impression(
    session_id: Option<i32>,
    artist_id: i64,
    impression_type: String, // "view" or "click"
) -> Result<(), ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use std::time::{SystemTime, UNIX_EPOCH};

        // Use session ID if available, otherwise create a temp session ID based on timestamp
        let session_id = session_id.unwrap_or_else(|| {
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs() as i32
        });

        let pool = crate::db::pool::get_pool();

        sqlx::query(
            "INSERT INTO client_match_impressions (session_id, artist_id, impression_type)
             VALUES ($1, $2, $3)",
        )
        .bind(session_id)
        .bind(artist_id)
        .bind(impression_type)
        .execute(pool)
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to log impression: {}", e)))?;
    }

    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StyleWithCount {
    pub id: i32,
    pub name: String,
    pub description: Option<String>,
    pub artist_count: i32,
    pub sample_images: Option<Vec<String>>,
}

#[cfg_attr(feature = "ssr", instrument(err, level = "info"))]
#[server]
pub async fn get_all_styles_with_counts() -> Result<Vec<StyleWithCount>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use sqlx::Row;

        let pool = crate::db::pool::get_pool();

        let rows = sqlx::query(
            "
            SELECT
                s.id,
                s.name,
                COUNT(DISTINCT ast.artist_id) as artist_count,
                STRING_AGG(DISTINCT ai.instagram_url, '|') as sample_images
            FROM styles s
            LEFT JOIN artists_styles ast ON s.id = ast.style_id
            LEFT JOIN artists a ON ast.artist_id = a.id
            LEFT JOIN artists_images ai ON a.id = ai.artist_id
            GROUP BY s.id, s.name
            HAVING COUNT(DISTINCT ast.artist_id) > 0
            ORDER BY artist_count DESC, s.name ASC
        ",
        )
        .fetch_all(pool)
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to query styles: {}", e)))?;

        let result: Vec<StyleWithCount> = rows
            .iter()
            .map(|row| {
                let sample_images_str: Option<String> = row.get("sample_images");
                let sample_images = sample_images_str
                    .and_then(|s| if s.trim().is_empty() { None } else { Some(s) })
                    .map(|s| {
                        s.split('|')
                            .map(|url| url.trim().to_string())
                            .filter(|url| !url.is_empty())
                            .collect::<Vec<_>>()
                    });

                StyleWithCount {
                    id: row.get("id"),
                    name: row.get("name"),
                    description: None, // No description column in the table
                    artist_count: row.get::<i64, _>("artist_count") as i32,
                    sample_images,
                }
            })
            .collect();

        Ok(result)
    }

    #[cfg(not(feature = "ssr"))]
    {
        // Return placeholder data for client-side
        Ok(vec![
            StyleWithCount {
                id: 1,
                name: "Traditional".to_string(),
                description: Some(
                    "Classic American tattoo style with bold lines and bright colors".to_string(),
                ),
                artist_count: 12,
                sample_images: None,
            },
            StyleWithCount {
                id: 2,
                name: "Neo-Traditional".to_string(),
                description: Some(
                    "Modern take on traditional tattoos with enhanced detail and color".to_string(),
                ),
                artist_count: 8,
                sample_images: None,
            },
        ])
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TattooPost {
    pub id: i64,
    pub short_code: String,
    pub artist_id: i64,
    pub artist_name: String,
    pub artist_instagram: Option<String>,
    pub styles: Vec<String>,
    pub is_favorited: bool,
}

#[cfg_attr(
    feature = "ssr",
    instrument(skip(token, cities, states), err, level = "info")
)]
#[server]
pub async fn get_tattoo_posts_by_style(
    style_names: Vec<String>,
    states: Option<Vec<String>>,
    cities: Option<Vec<String>>,
    token: Option<String>,
) -> Result<Vec<TattooPost>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use sqlx::Row;

        let pool = crate::db::pool::get_pool();
        let user_id = token.as_deref().and_then(extract_user_id_from_token);

        // Build WHERE clause based on filters
        let mut where_clauses = Vec::new();
        let mut bind_index = 1;

        // Add style filter - support multiple styles with IN clause
        let style_placeholders = if !style_names.is_empty() {
            let placeholders: Vec<String> = (0..style_names.len())
                .map(|i| format!("${}", bind_index + i))
                .collect();
            bind_index += style_names.len();
            where_clauses.push(format!("LOWER(s.name) IN ({})", placeholders.join(",")));
            placeholders
        } else {
            vec![]
        };

        // Add state filter if provided
        let state_placeholders = if let Some(state_list) = &states {
            if !state_list.is_empty() {
                let placeholders: Vec<String> = (0..state_list.len())
                    .map(|i| format!("${}", bind_index + i))
                    .collect();
                bind_index += state_list.len();
                where_clauses.push(format!("l.state IN ({})", placeholders.join(",")));
                placeholders
            } else {
                vec![]
            }
        } else {
            vec![]
        };

        // Add city filter if provided
        let city_placeholders = if let Some(city_list) = &cities {
            if !city_list.is_empty() {
                let placeholders: Vec<String> = (0..city_list.len())
                    .map(|i| format!("${}", bind_index + i))
                    .collect();
                where_clauses.push(format!("l.city IN ({})", placeholders.join(",")));
                placeholders
            } else {
                vec![]
            }
        } else {
            vec![]
        };

        let where_clause = if !where_clauses.is_empty() {
            format!("WHERE {}", where_clauses.join(" AND "))
        } else {
            String::new()
        };

        let (favorites_join, favorites_select) = if let Some(uid) = user_id {
            (
                format!("LEFT JOIN user_favorites uf ON ai.id = uf.artists_images_id AND uf.user_id = {}", uid),
                ", CASE WHEN uf.id IS NOT NULL THEN TRUE ELSE FALSE END as is_favorited"
            )
        } else {
            (String::new(), ", FALSE as is_favorited")
        };

        let query = format!(
            "
            SELECT DISTINCT
                ai.id,
                ai.short_code,
                ai.artist_id,
                a.name as artist_name,
                a.instagram_handle as artist_instagram
                {}
            FROM artists_images ai
            JOIN artists a ON ai.artist_id = a.id
            JOIN locations l ON a.location_id = l.id
            JOIN artists_images_styles ais ON ai.id = ais.artists_images_id
            JOIN styles s ON ais.style_id = s.id
            {}
            {}
            ORDER BY ai.id DESC
        ",
            favorites_select, favorites_join, where_clause
        );

        let mut query_builder = sqlx::query(&query);

        // Bind style parameters
        for style in &style_names {
            query_builder = query_builder.bind(style.to_lowercase());
        }

        // Bind state parameters
        if let Some(state_list) = &states {
            for state in state_list {
                query_builder = query_builder.bind(state);
            }
        }

        // Bind city parameters
        if let Some(city_list) = &cities {
            for city in city_list {
                query_builder = query_builder.bind(city);
            }
        }

        let rows = query_builder
            .fetch_all(pool)
            .await
            .map_err(|e| ServerFnError::new(format!("Failed to query posts: {}", e)))?;

        let mut posts = Vec::new();
        for row in rows {
            let image_id: i64 = row.get("id");
            let short_code: String = row.get("short_code");
            let artist_id: i64 = row.get("artist_id");
            let artist_name: String = row.get("artist_name");
            let artist_instagram: Option<String> = row.get("artist_instagram");
            let is_favorited: bool = row.get("is_favorited");

            // Get styles for this specific image
            let style_rows = sqlx::query(
                "
                SELECT s.name
                FROM styles s
                JOIN artists_images_styles ais ON s.id = ais.style_id
                WHERE ais.artists_images_id = $1
            ",
            )
            .bind(image_id)
            .fetch_all(pool)
            .await
            .map_err(|e| ServerFnError::new(format!("Failed to query styles: {}", e)))?;

            let styles: Vec<String> = style_rows.iter().map(|row| row.get("name")).collect();

            posts.push(TattooPost {
                id: image_id,
                short_code,
                artist_id,
                artist_name,
                artist_instagram,
                styles,
                is_favorited,
            });
        }

        Ok(posts)
    }

    #[cfg(not(feature = "ssr"))]
    {
        // Return placeholder data for client-side
        Ok(vec![TattooPost {
            id: 1,
            short_code: "ABC123".to_string(),
            artist_id: 1,
            artist_name: "Sample Artist".to_string(),
            artist_instagram: Some("sample_artist".to_string()),
            styles: vec!["Japanese".to_string(), "Traditional".to_string()],
        }])
    }
}

#[cfg_attr(feature = "ssr", instrument(err, level = "info"))]
#[server]
pub async fn get_artist_availability(
    artist_id: i32,
    start_date: String,
    end_date: String,
) -> Result<Vec<AvailabilitySlot>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use sqlx::Row;

        async fn query_availability(
            artist_id: i32,
            start_date: String,
            end_date: String,
        ) -> Result<Vec<AvailabilitySlot>, sqlx::Error> {
            let pool = crate::db::pool::get_pool();

            let rows = sqlx::query(
                "
                SELECT id, artist_id, day_of_week, specific_date, start_time, end_time,
                       is_available, is_recurring, created_at
                FROM artist_availability
                WHERE artist_id = $1
                AND (specific_date IS NULL OR (specific_date >= $2 AND specific_date <= $3))
                ORDER BY day_of_week, start_time
            ",
            )
            .bind(artist_id)
            .bind(start_date)
            .bind(end_date)
            .fetch_all(pool)
            .await?;

            let slots: Vec<AvailabilitySlot> = rows
                .iter()
                .map(|row| AvailabilitySlot {
                    id: row.get("id"),
                    artist_id: row.get("artist_id"),
                    day_of_week: row.get("day_of_week"),
                    specific_date: row.get("specific_date"),
                    start_time: row.get("start_time"),
                    end_time: row.get("end_time"),
                    is_available: row.get("is_available"),
                    is_recurring: row.get("is_recurring"),
                    created_at: row.get("created_at"),
                })
                .collect();

            Ok(slots)
        }

        match query_availability(artist_id, start_date, end_date).await {
            Ok(availability) => Ok(availability),
            Err(e) => Err(ServerFnError::new(format!(
                "Failed to get availability: {}",
                e
            ))),
        }
    }
    #[cfg(not(feature = "ssr"))]
    {
        Ok(vec![])
    }
}

#[cfg_attr(feature = "ssr", instrument(err, level = "info"))]
#[server]
pub async fn set_artist_availability(
    availability: AvailabilityUpdate,
) -> Result<(), ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        async fn update_availability(availability: AvailabilityUpdate) -> Result<(), sqlx::Error> {
            let pool = crate::db::pool::get_pool();

            sqlx::query(
                "
                INSERT INTO artist_availability
                (artist_id, day_of_week, specific_date, start_time, end_time, is_available, is_recurring)
                VALUES ($1, $2, $3, $4, $5, $6, $7)
            ",
            )
            .bind(availability.artist_id)
            .bind(availability.day_of_week)
            .bind(availability.date)
            .bind(availability.start_time)
            .bind(availability.end_time)
            .bind(availability.is_available)
            .bind(availability.is_recurring)
            .execute(pool)
            .await?;

            Ok(())
        }

        match update_availability(availability).await {
            Ok(_) => Ok(()),
            Err(e) => Err(ServerFnError::new(format!(
                "Failed to set availability: {}",
                e
            ))),
        }
    }
    #[cfg(not(feature = "ssr"))]
    {
        Ok(())
    }
}

#[cfg_attr(feature = "ssr", instrument(err, level = "info"))]
#[server]
pub async fn get_booking_requests(artist_id: i32) -> Result<Vec<BookingRequest>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use sqlx::Row;

        async fn query_bookings(artist_id: i32) -> Result<Vec<BookingRequest>, sqlx::Error> {
            let pool = crate::db::pool::get_pool();

            let rows = sqlx::query(
                "
                SELECT id, artist_id, client_name, client_email, client_phone,
                       requested_date, requested_start_time, requested_end_time,
                       tattoo_description, placement, size_inches, reference_images,
                       message_from_client, status, artist_response, estimated_price,
                       created_at, updated_at, decline_reason
                FROM booking_requests
                WHERE artist_id = $1
                ORDER BY created_at DESC
            ",
            )
            .bind(artist_id)
            .fetch_all(pool)
            .await?;

            let bookings: Vec<BookingRequest> = rows
                .iter()
                .map(|row| BookingRequest {
                    id: row.get("id"),
                    artist_id: row.get("artist_id"),
                    client_name: row.get("client_name"),
                    client_email: row.get("client_email"),
                    client_phone: row.get("client_phone"),
                    requested_date: row.get("requested_date"),
                    requested_start_time: row.get("requested_start_time"),
                    requested_end_time: row.get("requested_end_time"),
                    tattoo_description: row.get("tattoo_description"),
                    placement: row.get("placement"),
                    size_inches: row.get("size_inches"),
                    reference_images: row.get("reference_images"),
                    message_from_client: row.get("message_from_client"),
                    status: row.get("status"),
                    artist_response: row.get("artist_response"),
                    estimated_price: row.get("estimated_price"),
                    created_at: row.get("created_at"),
                    updated_at: row.get("updated_at"),
                    decline_reason: row.get("decline_reason"),
                })
                .collect();

            Ok(bookings)
        }

        match query_bookings(artist_id).await {
            Ok(bookings) => Ok(bookings),
            Err(e) => Err(ServerFnError::new(format!(
                "Failed to get booking requests: {}",
                e
            ))),
        }
    }
    #[cfg(not(feature = "ssr"))]
    {
        Ok(vec![])
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct BookingResponse {
    pub booking_id: i32,
    pub status: String,
    pub artist_response: Option<String>,
    pub estimated_price: Option<f64>,
    pub decline_reason: Option<String>,
}

#[cfg_attr(feature = "ssr", instrument(err, level = "info"))]
#[server]
pub async fn respond_to_booking(response: BookingResponse) -> Result<(), ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        async fn update_booking(response: BookingResponse) -> Result<(), sqlx::Error> {
            let pool = crate::db::pool::get_pool();

            sqlx::query(
                "
                UPDATE booking_requests
                SET status = $1, artist_response = $2, estimated_price = $3, decline_reason = $4, updated_at = CURRENT_TIMESTAMP
                WHERE id = $5
            ",
            )
            .bind(response.status)
            .bind(response.artist_response)
            .bind(response.estimated_price)
            .bind(response.decline_reason)
            .bind(response.booking_id)
            .execute(pool)
            .await?;

            Ok(())
        }

        match update_booking(response).await {
            Ok(_) => Ok(()),
            Err(e) => Err(ServerFnError::new(format!(
                "Failed to respond to booking: {}",
                e
            ))),
        }
    }
    #[cfg(not(feature = "ssr"))]
    {
        Ok(())
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct NewBookingMessage {
    pub booking_request_id: i32,
    pub sender_type: String,
    pub message: String,
}

#[cfg_attr(feature = "ssr", instrument(err, level = "info"))]
#[server]
pub async fn send_booking_message(message_data: NewBookingMessage) -> Result<(), ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        async fn insert_message(message_data: NewBookingMessage) -> Result<(), sqlx::Error> {
            let pool = crate::db::pool::get_pool();

            sqlx::query(
                "
                INSERT INTO booking_messages (booking_request_id, sender_type, message)
                VALUES ($1, $2, $3)
            ",
            )
            .bind(message_data.booking_request_id)
            .bind(message_data.sender_type)
            .bind(message_data.message)
            .execute(pool)
            .await?;

            Ok(())
        }

        match insert_message(message_data).await {
            Ok(_) => Ok(()),
            Err(e) => Err(ServerFnError::new(format!("Failed to send message: {}", e))),
        }
    }
    #[cfg(not(feature = "ssr"))]
    {
        Ok(())
    }
}

#[cfg_attr(feature = "ssr", instrument(err, level = "info"))]
#[server]
pub async fn get_booking_messages(
    booking_request_id: i32,
) -> Result<Vec<BookingMessage>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use sqlx::Row;

        async fn query_messages(
            booking_request_id: i32,
        ) -> Result<Vec<BookingMessage>, sqlx::Error> {
            let pool = crate::db::pool::get_pool();

            let rows = sqlx::query(
                "
                SELECT id, booking_request_id, sender_type, message, created_at
                FROM booking_messages
                WHERE booking_request_id = $1
                ORDER BY created_at ASC
            ",
            )
            .bind(booking_request_id)
            .fetch_all(pool)
            .await?;

            let messages: Vec<BookingMessage> = rows
                .iter()
                .map(|row| BookingMessage {
                    id: row.get("id"),
                    booking_request_id: row.get("booking_request_id"),
                    sender_type: row.get("sender_type"),
                    message: row.get("message"),
                    created_at: row.get("created_at"),
                })
                .collect();

            Ok(messages)
        }

        match query_messages(booking_request_id).await {
            Ok(messages) => Ok(messages),
            Err(e) => Err(ServerFnError::new(format!("Failed to get messages: {}", e))),
        }
    }
    #[cfg(not(feature = "ssr"))]
    {
        Ok(vec![])
    }
}

// Recurring Rule Server Functions

#[cfg_attr(feature = "ssr", instrument(err, level = "info"))]
#[server]
pub async fn get_recurring_rules(artist_id: i32) -> Result<Vec<RecurringRule>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use sqlx::Row;

        async fn query_recurring_rules(artist_id: i32) -> Result<Vec<RecurringRule>, sqlx::Error> {
            let pool = crate::db::pool::get_pool();

            // Query artist_availability table for recurring rules
            let rows = sqlx::query(
                "
                SELECT id, artist_id, day_of_week, start_time, end_time, is_available, created_at
                FROM artist_availability
                WHERE artist_id = $1 AND is_recurring = true
                ORDER BY day_of_week, start_time
            ",
            )
            .bind(artist_id)
            .fetch_all(pool)
            .await?;

            let rules: Vec<RecurringRule> = rows
                .iter()
                .map(|row| {
                    let day_of_week: Option<i32> = row.get("day_of_week");
                    let start_time: Option<String> = row.get("start_time");
                    let end_time: Option<String> = row.get("end_time");
                    let is_available: bool = row.get("is_available");

                    // Map day_of_week to day name
                    let day_name = match day_of_week {
                        Some(0) => "Sunday",
                        Some(1) => "Monday",
                        Some(2) => "Tuesday",
                        Some(3) => "Wednesday",
                        Some(4) => "Thursday",
                        Some(5) => "Friday",
                        Some(6) => "Saturday",
                        _ => "Unknown Day",
                    };

                    // Create descriptive name
                    let name = if is_available {
                        format!(
                            "{} Available: {} - {}",
                            day_name,
                            start_time.as_deref().unwrap_or("00:00"),
                            end_time.as_deref().unwrap_or("00:00")
                        )
                    } else {
                        format!(
                            "{} Blocked: {} - {}",
                            day_name,
                            start_time.as_deref().unwrap_or("00:00"),
                            end_time.as_deref().unwrap_or("00:00")
                        )
                    };

                    // Create pattern from day_of_week
                    let pattern = match day_of_week {
                        Some(day) => format!("[{}]", day),
                        None => "[]".to_string(),
                    };

                    RecurringRule {
                        id: row.get("id"),
                        artist_id: row.get("artist_id"),
                        name,
                        rule_type: "weekdays".to_string(),
                        pattern,
                        action: if is_available {
                            "available".to_string()
                        } else {
                            "blocked".to_string()
                        },
                        start_time,
                        end_time,
                        active: true, // All recurring rules are considered active
                        created_at: row.get("created_at"),
                    }
                })
                .collect();

            Ok(rules)
        }

        match query_recurring_rules(artist_id).await {
            Ok(rules) => Ok(rules),
            Err(e) => Err(ServerFnError::new(format!(
                "Failed to get recurring rules: {}",
                e
            ))),
        }
    }
    #[cfg(not(feature = "ssr"))]
    {
        Ok(vec![])
    }
}

#[cfg_attr(feature = "ssr", instrument(err, level = "info"))]
#[server]
pub async fn create_recurring_rule(
    artist_id: i32,
    name: String,
    rule_type: String,
    pattern: String,
    action: String,
    start_time: Option<String>,
    end_time: Option<String>,
) -> Result<i32, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use sqlx::Row;

        async fn insert_recurring_rule(
            artist_id: i32,
            name: String,
            rule_type: String,
            pattern: String,
            action: String,
            start_time: Option<String>,
            end_time: Option<String>,
        ) -> Result<i32, sqlx::Error> {
            let pool = crate::db::pool::get_pool();

            // Create recurring_rules table if it doesn't exist (PostgreSQL syntax)
            sqlx::query(
                "
                CREATE TABLE IF NOT EXISTS recurring_rules (
                    id SERIAL PRIMARY KEY,
                    artist_id INTEGER NOT NULL,
                    name TEXT NOT NULL,
                    rule_type TEXT NOT NULL,
                    pattern TEXT NOT NULL,
                    action TEXT NOT NULL,
                    start_time TEXT,
                    end_time TEXT,
                    active BOOLEAN DEFAULT true,
                    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
                )
            ",
            )
            .execute(pool)
            .await?;

            let row = sqlx::query(
                "
                INSERT INTO recurring_rules (artist_id, name, rule_type, pattern, action, start_time, end_time)
                VALUES ($1, $2, $3, $4, $5, $6, $7)
                RETURNING id
            ",
            )
            .bind(artist_id)
            .bind(name)
            .bind(rule_type)
            .bind(pattern)
            .bind(action)
            .bind(start_time.unwrap_or_default())
            .bind(end_time.unwrap_or_default())
            .fetch_one(pool)
            .await?;

            Ok(row.get("id"))
        }

        match insert_recurring_rule(
            artist_id,
            name,
            rule_type,
            pattern.clone(),
            action,
            start_time,
            end_time,
        )
        .await
        {
            Ok(id) => Ok(id),
            Err(e) => Err(ServerFnError::new(format!(
                "Failed to create recurring rule - Error: {} - Pattern: {} - Artist ID: {}",
                e, pattern, artist_id
            ))),
        }
    }
    #[cfg(not(feature = "ssr"))]
    {
        Ok(0)
    }
}

#[cfg_attr(feature = "ssr", instrument(err, level = "info"))]
#[server]
pub async fn update_recurring_rule(
    id: i32,
    name: Option<String>,
    pattern: Option<String>,
    action: Option<String>,
    start_time: Option<String>,
    end_time: Option<String>,
    active: Option<bool>,
) -> Result<(), ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use sqlx::Row;

        async fn update_rule(
            id: i32,
            name: Option<String>,
            pattern: Option<String>,
            action: Option<String>,
            start_time: Option<String>,
            end_time: Option<String>,
            active: Option<bool>,
        ) -> Result<(), sqlx::Error> {
            let pool = crate::db::pool::get_pool();

            let mut updates = Vec::new();
            let mut param_count = 1;

            if name.is_some() {
                updates.push(format!("name = ${}", param_count));
                param_count += 1;
            }
            if pattern.is_some() {
                updates.push(format!("pattern = ${}", param_count));
                param_count += 1;
            }
            if action.is_some() {
                updates.push(format!("action = ${}", param_count));
                param_count += 1;
            }
            if start_time.is_some() {
                updates.push(format!("start_time = ${}", param_count));
                param_count += 1;
            }
            if end_time.is_some() {
                updates.push(format!("end_time = ${}", param_count));
                param_count += 1;
            }
            if active.is_some() {
                updates.push(format!("active = ${}", param_count));
                param_count += 1;
            }

            if updates.is_empty() {
                return Ok(());
            }

            let sql = format!(
                "UPDATE recurring_rules SET {} WHERE id = ${}",
                updates.join(", "),
                param_count
            );

            let mut query = sqlx::query(&sql);

            if let Some(n) = name {
                query = query.bind(n);
            }
            if let Some(p) = pattern {
                query = query.bind(p);
            }
            if let Some(a) = action {
                query = query.bind(a);
            }
            if let Some(st) = start_time {
                query = query.bind(st);
            }
            if let Some(et) = end_time {
                query = query.bind(et);
            }
            if let Some(act) = active {
                query = query.bind(act);
            }
            query = query.bind(id);

            query.execute(pool).await?;
            Ok(())
        }

        match update_rule(id, name, pattern, action, start_time, end_time, active).await {
            Ok(_) => Ok(()),
            Err(e) => Err(ServerFnError::new(format!(
                "Failed to update recurring rule: {}",
                e
            ))),
        }
    }
    #[cfg(not(feature = "ssr"))]
    {
        Ok(())
    }
}

#[cfg_attr(feature = "ssr", instrument(err, level = "info"))]
#[server]
pub async fn get_effective_availability(
    artist_id: i32,
    date: String,
) -> Result<bool, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use chrono::{Datelike, NaiveDate, Weekday};
        use sqlx::Row;

        async fn check_effective_availability(
            artist_id: i32,
            date: String,
        ) -> Result<bool, sqlx::Error> {
            let pool = crate::db::pool::get_pool();

            // First check if there's an explicit availability record for this date
            let explicit_availability: Option<bool> = sqlx::query(
                "SELECT is_available FROM availability_slots
                WHERE artist_id = $1 AND specific_date = $2",
            )
            .bind(artist_id)
            .bind(&date)
            .fetch_optional(pool)
            .await?
            .map(|row| row.get("is_available"));

            if let Some(is_available) = explicit_availability {
                return Ok(is_available);
            }

            // No explicit record, check recurring rules
            let parsed_date = match NaiveDate::parse_from_str(&date, "%Y-%m-%d") {
                Ok(d) => d,
                Err(_) => return Ok(true), // Default to available if date parsing fails
            };

            // Get all active recurring rules for this artist
            let rules = sqlx::query(
                "SELECT rule_type, pattern, action FROM recurring_rules
                WHERE artist_id = $1 AND active = true",
            )
            .bind(artist_id)
            .fetch_all(pool)
            .await?;

            for row in rules {
                let rule_type: String = row.get("rule_type");
                let pattern: String = row.get("pattern");
                let action: String = row.get("action");

                let matches = match rule_type.as_str() {
                    "weekdays" => {
                        let weekday_num = match parsed_date.weekday() {
                            Weekday::Sun => 0,
                            Weekday::Mon => 1,
                            Weekday::Tue => 2,
                            Weekday::Wed => 3,
                            Weekday::Thu => 4,
                            Weekday::Fri => 5,
                            Weekday::Sat => 6,
                        };
                        let weekday_name = match weekday_num {
                            0 => "Sunday",
                            1 => "Monday",
                            2 => "Tuesday",
                            3 => "Wednesday",
                            4 => "Thursday",
                            5 => "Friday",
                            6 => "Saturday",
                            _ => "",
                        };
                        pattern.contains(weekday_name)
                    }
                    "dates" => {
                        let month_day = parsed_date.format("%B %e").to_string().trim().to_string();
                        pattern.contains(&month_day)
                    }
                    "monthly" => {
                        // Simple pattern matching for common monthly patterns
                        if pattern.contains("1st") && parsed_date.day() <= 7 {
                            let first_weekday_of_month = parsed_date.with_day(1).unwrap().weekday();
                            let target_day = parsed_date.weekday();
                            (parsed_date.day() - 1) / 7 == 0 && first_weekday_of_month == target_day
                        } else {
                            false // More complex patterns would need more parsing
                        }
                    }
                    _ => false,
                };

                if matches {
                    return Ok(action == "available");
                }
            }

            // Default to available if no rules match
            Ok(true)
        }

        match check_effective_availability(artist_id, date).await {
            Ok(is_available) => Ok(is_available),
            Err(e) => Err(ServerFnError::new(format!(
                "Failed to check effective availability: {}",
                e
            ))),
        }
    }
    #[cfg(not(feature = "ssr"))]
    {
        Ok(true)
    }
}

#[cfg_attr(feature = "ssr", instrument(err, level = "info"))]
#[server]
pub async fn delete_recurring_rule(rule_id: i32) -> Result<(), ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use sqlx::Row;

        async fn delete_rule(rule_id: i32) -> Result<(), sqlx::Error> {
            let pool = crate::db::pool::get_pool();

            sqlx::query("DELETE FROM recurring_rules WHERE id = $1")
                .bind(rule_id)
                .execute(pool)
                .await?;
            Ok(())
        }

        match delete_rule(rule_id).await {
            Ok(_) => Ok(()),
            Err(e) => Err(ServerFnError::new(format!(
                "Failed to delete recurring rule: {}",
                e
            ))),
        }
    }
    #[cfg(not(feature = "ssr"))]
    {
        Ok(())
    }
}

#[cfg_attr(feature = "ssr", instrument(err, level = "info"))]
#[server]
pub async fn get_booking_request_by_id(booking_id: i32) -> Result<BookingRequest, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use sqlx::Row;

        async fn query_booking_by_id(booking_id: i32) -> Result<BookingRequest, sqlx::Error> {
            let pool = crate::db::pool::get_pool();

            let row = sqlx::query(
                "SELECT id, artist_id, client_name, client_email, client_phone,
                       requested_date, requested_start_time, requested_end_time,
                       tattoo_description, placement, size_inches, reference_images,
                       message_from_client, status, artist_response, estimated_price,
                       created_at, updated_at, decline_reason
                FROM booking_requests
                WHERE id = $1",
            )
            .bind(booking_id)
            .fetch_one(pool)
            .await?;

            Ok(BookingRequest {
                id: row.get("id"),
                artist_id: row.get("artist_id"),
                client_name: row.get("client_name"),
                client_email: row.get("client_email"),
                client_phone: row.get("client_phone"),
                requested_date: row.get("requested_date"),
                requested_start_time: row.get("requested_start_time"),
                requested_end_time: row.get("requested_end_time"),
                tattoo_description: row.get("tattoo_description"),
                placement: row.get("placement"),
                size_inches: row.get("size_inches"),
                reference_images: row.get("reference_images"),
                message_from_client: row.get("message_from_client"),
                status: row.get("status"),
                artist_response: row.get("artist_response"),
                estimated_price: row.get("estimated_price"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
                decline_reason: row.get("decline_reason"),
            })
        }

        match query_booking_by_id(booking_id).await {
            Ok(booking) => Ok(booking),
            Err(e) => Err(ServerFnError::new(format!("Failed to get booking: {}", e))),
        }
    }
    #[cfg(not(feature = "ssr"))]
    {
        Err(ServerFnError::new("Not available on client".to_string()))
    }
}

#[cfg_attr(feature = "ssr", instrument(err, level = "info"))]
#[server]
pub async fn get_business_hours(
    artist_id: i32,
) -> Result<Vec<crate::db::entities::BusinessHours>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use crate::db::entities::BusinessHours;
        use sqlx::Row;

        let pool = crate::db::pool::get_pool();

        let rows = sqlx::query(
            "SELECT id, artist_id, day_of_week, start_time, end_time, is_closed, created_at, updated_at
            FROM business_hours
            WHERE artist_id = $1
            ORDER BY day_of_week"
        )
        .bind(artist_id)
        .fetch_all(pool)
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to query business hours: {}", e)))?;

        let result = rows
            .into_iter()
            .map(|row| BusinessHours {
                id: row.get("id"),
                artist_id: row.get("artist_id"),
                day_of_week: row.get("day_of_week"),
                start_time: row.get("start_time"),
                end_time: row.get("end_time"),
                is_closed: row.get("is_closed"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            })
            .collect();

        Ok(result)
    }
    #[cfg(not(feature = "ssr"))]
    {
        Err(ServerFnError::new("Not available on client".to_string()))
    }
}

#[cfg_attr(feature = "ssr", instrument(err, level = "info"))]
#[server]
pub async fn update_business_hours(
    hours: Vec<crate::db::entities::UpdateBusinessHours>,
) -> Result<(), ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use sqlx::Row;

        let pool = crate::db::pool::get_pool();

        for hour in hours {
            sqlx::query(
                "INSERT INTO business_hours (artist_id, day_of_week, start_time, end_time, is_closed, updated_at)
                VALUES ($1, $2, $3, $4, $5, CURRENT_TIMESTAMP)
                ON CONFLICT (artist_id, day_of_week)
                DO UPDATE SET start_time = $3, end_time = $4, is_closed = $5, updated_at = CURRENT_TIMESTAMP"
            )
            .bind(hour.artist_id)
            .bind(hour.day_of_week)
            .bind(hour.start_time)
            .bind(hour.end_time)
            .bind(hour.is_closed)
            .execute(pool)
            .await
            .map_err(|e| ServerFnError::new(format!("Failed to update business hours: {}", e)))?;
        }

        Ok(())
    }
    #[cfg(not(feature = "ssr"))]
    {
        Err(ServerFnError::new("Not available on client".to_string()))
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct BookingHistoryEntry {
    pub id: i32,
    pub booking_date: Option<String>,
    pub status: String,
    pub created_at: String,
}

#[cfg_attr(feature = "ssr", instrument(err, level = "info"))]
#[server]
pub async fn get_client_booking_history(
    client_email: String,
) -> Result<Vec<BookingHistoryEntry>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use sqlx::Row;

        async fn query_client_history(
            client_email: String,
        ) -> Result<Vec<BookingHistoryEntry>, sqlx::Error> {
            let pool = crate::db::pool::get_pool();

            let rows = sqlx::query(
                "SELECT id, requested_date, status, created_at
                FROM booking_requests
                WHERE client_email = $1
                ORDER BY created_at DESC
                LIMIT 10",
            )
            .bind(&client_email)
            .fetch_all(pool)
            .await?;

            let history = rows
                .into_iter()
                .map(|row| BookingHistoryEntry {
                    id: row.get("id"),
                    booking_date: row.get("requested_date"),
                    status: row.get("status"),
                    created_at: row
                        .try_get("created_at")
                        .unwrap_or_else(|_| "Unknown".to_string()),
                })
                .collect();

            Ok(history)
        }

        match query_client_history(client_email).await {
            Ok(history) => Ok(history),
            Err(e) => Err(ServerFnError::new(format!(
                "Failed to get client history: {}",
                e
            ))),
        }
    }
    #[cfg(not(feature = "ssr"))]
    {
        Ok(vec![])
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct BookingSuggestion {
    pub booking_id: i32,
    pub suggested_date: String,
    pub suggested_start_time: String,
    pub suggested_end_time: Option<String>,
}

#[cfg_attr(feature = "ssr", instrument(err, level = "debug"))]
#[server]
pub async fn suggest_booking_time(suggestion: BookingSuggestion) -> Result<(), ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use sqlx::Row;

        async fn update_suggested_time(suggestion: BookingSuggestion) -> Result<(), sqlx::Error> {
            let pool = crate::db::pool::get_pool();

            sqlx::query(
                "UPDATE booking_requests
                SET suggested_date = $1, suggested_start_time = $2, suggested_end_time = $3, updated_at = CURRENT_TIMESTAMP
                WHERE id = $4"
            )
            .bind(suggestion.suggested_date)
            .bind(suggestion.suggested_start_time)
            .bind(suggestion.suggested_end_time)
            .bind(suggestion.booking_id)
            .execute(pool)
            .await?;

            Ok(())
        }

        match update_suggested_time(suggestion).await {
            Ok(_) => Ok(()),
            Err(e) => Err(ServerFnError::new(format!(
                "Failed to suggest booking time: {}",
                e
            ))),
        }
    }
    #[cfg(not(feature = "ssr"))]
    {
        Ok(())
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct NewBookingRequest {
    pub artist_id: i32,
    pub client_name: String,
    pub client_email: String,
    pub client_phone: Option<String>,
    pub tattoo_description: Option<String>,
    pub placement: Option<String>,
    pub size_inches: Option<f32>,
    pub requested_date: String,
    pub requested_start_time: String,
    pub requested_end_time: Option<String>,
    pub message_from_client: Option<String>,
}

#[cfg_attr(feature = "ssr", instrument(err, level = "info"))]
#[server]
pub async fn submit_booking_request(request: NewBookingRequest) -> Result<i32, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use sqlx::Row;

        async fn insert_booking_request(request: NewBookingRequest) -> Result<i32, sqlx::Error> {
            let pool = crate::db::pool::get_pool();

            let row = sqlx::query(
                "INSERT INTO booking_requests (
                    artist_id, client_name, client_email, client_phone,
                    tattoo_description, placement, size_inches,
                    requested_date, requested_start_time, requested_end_time,
                    message_from_client, status, created_at
                ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, 'pending', CURRENT_TIMESTAMP)
                RETURNING id"
            )
            .bind(request.artist_id)
            .bind(request.client_name)
            .bind(request.client_email)
            .bind(request.client_phone.unwrap_or_else(|| "".to_string()))
            .bind(request.tattoo_description.unwrap_or_else(|| "".to_string()))
            .bind(request.placement.unwrap_or_else(|| "".to_string()))
            .bind(request.size_inches)
            .bind(request.requested_date)
            .bind(request.requested_start_time)
            .bind(request.requested_end_time.unwrap_or_else(|| "".to_string()))
            .bind(request.message_from_client.unwrap_or_else(|| "".to_string()))
            .fetch_one(pool)
            .await?;

            Ok(row.get("id"))
        }

        match insert_booking_request(request).await {
            Ok(booking_id) => Ok(booking_id),
            Err(e) => Err(ServerFnError::new(format!(
                "Failed to submit booking request: {}",
                e
            ))),
        }
    }
    #[cfg(not(feature = "ssr"))]
    {
        Ok(0)
    }
}

#[cfg_attr(feature = "ssr", instrument(err, level = "info"))]
#[server]
pub async fn check_availability(
    artist_id: i32,
    requested_date: String,
    requested_time: String,
) -> Result<bool, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        match check_artist_availability(artist_id, &requested_date, &requested_time).await {
            Ok(is_available) => Ok(is_available),
            Err(e) => Err(ServerFnError::new(format!(
                "Failed to check availability: {}",
                e
            ))),
        }
    }
    #[cfg(not(feature = "ssr"))]
    {
        Ok(true) // Default to available on client-side
    }
}

// ================================
// Authentication Server Functions
// ================================

use crate::views::auth::{LoginData, SignupData};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthResponse {
    pub success: bool,
    pub token: Option<String>,
    pub user_type: Option<String>,
    pub user_id: Option<i64>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserInfo {
    pub id: i64,
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub user_type: String,
}

#[cfg_attr(feature = "ssr", instrument(err, level = "info"))]
#[server]
pub async fn login_user(login_data: LoginData) -> Result<AuthResponse, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use bcrypt::verify;
        use jsonwebtoken::{encode, EncodingKey, Header};
        use sqlx::Row;

        #[derive(Debug, Serialize, Deserialize)]
        struct Claims {
            sub: String,       // User ID
            exp: usize,        // Expiration time
            user_type: String, // "client", "artist", or "admin" (for backwards compatibility)
            user_id: i64,
        }

        let pool = crate::db::pool::get_pool();

        // Query unified users table
        let result = sqlx::query(
            "SELECT id, password_hash, first_name, last_name, role::text as role
             FROM users
             WHERE email = $1 AND is_active = true",
        )
        .bind(&login_data.email)
        .fetch_optional(pool)
        .await
        .map_err(|e| ServerFnError::new(format!("Database error: {}", e)))?;

        let (user_id, stored_password_hash, _first_name, _last_name, role) = match result {
            Some(row) => (
                row.get::<i64, _>("id"),
                row.get::<String, _>("password_hash"),
                row.get::<String, _>("first_name"),
                row.get::<String, _>("last_name"),
                row.get::<String, _>("role"),
            ),
            None => {
                return Ok(AuthResponse {
                    success: false,
                    token: None,
                    user_type: None,
                    user_id: None,
                    error: Some("Invalid email or password".to_string()),
                })
            }
        };

        // Verify the user_type matches the role in database
        // Allow admins to log in regardless of which option they select
        if role != "admin" && role != login_data.user_type {
            return Ok(AuthResponse {
                success: false,
                token: None,
                user_type: None,
                user_id: None,
                error: Some("Invalid email or password".to_string()),
            });
        }

        // Verify password
        let password_valid = verify(&login_data.password, &stored_password_hash)
            .map_err(|e| ServerFnError::new(format!("Password verification error: {}", e)))?;

        if !password_valid {
            return Ok(AuthResponse {
                success: false,
                token: None,
                user_type: None,
                user_id: None,
                error: Some("Invalid email or password".to_string()),
            });
        }

        // Update last_login
        let _ = sqlx::query("UPDATE users SET last_login = CURRENT_TIMESTAMP WHERE id = $1")
            .bind(user_id)
            .execute(pool)
            .await;

        // Create JWT token
        let expiration = chrono::Utc::now()
            .checked_add_signed(chrono::Duration::days(7))
            .expect("valid timestamp")
            .timestamp() as usize;

        let claims = Claims {
            sub: user_id.to_string(),
            exp: expiration,
            user_type: role.clone(),
            user_id,
        };

        // Use a simple secret for now - in production this should be from environment
        let secret = "tatteau-jwt-secret-key-change-in-production";
        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(secret.as_ref()),
        )
        .map_err(|e| ServerFnError::new(format!("Token generation error: {}", e)))?;

        Ok(AuthResponse {
            success: true,
            token: Some(token),
            user_type: Some(role),
            user_id: Some(user_id),
            error: None,
        })
    }
    #[cfg(not(feature = "ssr"))]
    {
        Ok(AuthResponse {
            success: false,
            token: None,
            user_type: None,
            user_id: None,
            error: Some("Server-side only".to_string()),
        })
    }
}

#[cfg_attr(feature = "ssr", instrument(err, level = "info"))]
#[server]
pub async fn signup_user(signup_data: SignupData) -> Result<AuthResponse, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use bcrypt::{hash, DEFAULT_COST};
        use jsonwebtoken::{encode, EncodingKey, Header};
        use sqlx::Row;

        #[derive(Debug, Serialize, Deserialize)]
        struct Claims {
            sub: String,
            exp: usize,
            user_type: String,
            user_id: i64,
        }

        let pool = crate::db::pool::get_pool();

        // Check if email already exists
        let user_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users WHERE email = $1")
            .bind(&signup_data.email)
            .fetch_one(pool)
            .await
            .map_err(|e| ServerFnError::new(format!("Database query error: {}", e)))?;

        if user_count > 0 {
            return Ok(AuthResponse {
                success: false,
                token: None,
                user_type: None,
                user_id: None,
                error: Some("Email already exists".to_string()),
            });
        }

        // Hash password
        let password_hash = hash(&signup_data.password, DEFAULT_COST)
            .map_err(|e| ServerFnError::new(format!("Password hashing error: {}", e)))?;

        let user_id = if signup_data.user_type == "client" {
            // Insert into users table with client role
            let row = sqlx::query(
                "INSERT INTO users (first_name, last_name, email, phone, password_hash, role)
                 VALUES ($1, $2, $3, $4, $5, 'client')
                 RETURNING id",
            )
            .bind(&signup_data.first_name)
            .bind(&signup_data.last_name)
            .bind(&signup_data.email)
            .bind(&signup_data.phone)
            .bind(&password_hash)
            .fetch_one(pool)
            .await
            .map_err(|e| ServerFnError::new(format!("Failed to create user account: {}", e)))?;

            row.get::<i64, _>("id")
        } else {
            // For artists, create artist record first, then user record with artist role

            // First create artist record - using placeholder location_id for now
            let artist_row = sqlx::query(
                "INSERT INTO artists (name, location_id, email, availability_status)
                 VALUES ($1, $2, $3, $4)
                 RETURNING id",
            )
            .bind(format!(
                "{} {}",
                signup_data.first_name, signup_data.last_name
            ))
            .bind(1) // Placeholder location - will be updated during onboarding
            .bind(&signup_data.email)
            .bind("pending_onboarding")
            .fetch_one(pool)
            .await
            .map_err(|e| ServerFnError::new(format!("Failed to create artist record: {}", e)))?;

            let artist_id: i64 = artist_row.get("id");

            // Then create user record with artist role
            let user_row = sqlx::query(
                "INSERT INTO users (first_name, last_name, email, phone, password_hash, role, artist_id)
                 VALUES ($1, $2, $3, $4, $5, 'artist', $6)
                 RETURNING id"
            )
            .bind(&signup_data.first_name)
            .bind(&signup_data.last_name)
            .bind(&signup_data.email)
            .bind(&signup_data.phone)
            .bind(&password_hash)
            .bind(artist_id)
            .fetch_one(pool)
            .await
            .map_err(|e| ServerFnError::new(format!("Failed to create artist user account: {}", e)))?;

            user_row.get::<i64, _>("id")
        };

        // Create JWT token
        let expiration = chrono::Utc::now()
            .checked_add_signed(chrono::Duration::days(7))
            .expect("valid timestamp")
            .timestamp() as usize;

        let claims = Claims {
            sub: user_id.to_string(),
            exp: expiration,
            user_type: signup_data.user_type.clone(),
            user_id,
        };

        let secret = "tatteau-jwt-secret-key-change-in-production";
        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(secret.as_ref()),
        )
        .map_err(|e| ServerFnError::new(format!("Token generation error: {}", e)))?;

        Ok(AuthResponse {
            success: true,
            token: Some(token),
            user_type: Some(signup_data.user_type),
            user_id: Some(user_id),
            error: None,
        })
    }
    #[cfg(not(feature = "ssr"))]
    {
        Ok(AuthResponse {
            success: false,
            token: None,
            user_type: None,
            user_id: None,
            error: Some("Server-side only".to_string()),
        })
    }
}

#[cfg_attr(feature = "ssr", instrument(skip(token), err, level = "info"))]
#[server]
pub async fn verify_token(token: String) -> Result<Option<UserInfo>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use jsonwebtoken::{decode, DecodingKey, Validation};
        use sqlx::Row;

        #[derive(Debug, Serialize, Deserialize)]
        struct Claims {
            sub: String,
            exp: usize,
            user_type: String,
            user_id: i64,
        }

        let secret = "tatteau-jwt-secret-key-change-in-production";
        let token_data = decode::<Claims>(
            &token,
            &DecodingKey::from_secret(secret.as_ref()),
            &Validation::default(),
        );

        match token_data {
            Ok(data) => {
                let claims = data.claims;
                let pool = crate::db::pool::get_pool();

                let user_info = sqlx::query(
                    "SELECT id, first_name, last_name, email, role::text as role
                     FROM users
                     WHERE id = $1 AND is_active = true",
                )
                .bind(claims.user_id)
                .fetch_optional(pool)
                .await
                .ok()
                .flatten()
                .map(|row| UserInfo {
                    id: row.get("id"),
                    first_name: row.get("first_name"),
                    last_name: row.get("last_name"),
                    email: row.get("email"),
                    user_type: row.get("role"),
                });

                Ok(user_info)
            }
            Err(_) => Ok(None),
        }
    }
    #[cfg(not(feature = "ssr"))]
    {
        Ok(None)
    }
}

// Subscription System Server Functions

#[cfg_attr(feature = "ssr", instrument(err, level = "info"))]
#[server]
pub async fn get_subscription_tiers() -> Result<Vec<SubscriptionTier>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use sqlx::Row;

        let pool = crate::db::pool::get_pool();

        let rows = sqlx::query(
            "SELECT id, tier_name, tier_level, price_monthly, features_json, created_at
            FROM subscription_tiers
            ORDER BY tier_level ASC",
        )
        .fetch_all(pool)
        .await
        .map_err(|e| ServerFnError::new(format!("Query execution error: {}", e)))?;

        let tiers = rows
            .into_iter()
            .map(|row| SubscriptionTier {
                id: row.get("id"),
                tier_name: row.get("tier_name"),
                tier_level: row.get("tier_level"),
                price_monthly: row.get("price_monthly"),
                features_json: row.get("features_json"),
                created_at: row.get("created_at"),
            })
            .collect();

        Ok(tiers)
    }
    #[cfg(not(feature = "ssr"))]
    {
        Ok(vec![])
    }
}

#[cfg_attr(feature = "ssr", instrument(err, level = "info"))]
#[server]
pub async fn create_artist_subscription(
    artist_id: i32,
    tier_id: i32,
    status: String,
    payment_method: Option<String>,
) -> Result<i32, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use sqlx::Row;

        let pool = crate::db::pool::get_pool();

        let row = sqlx::query(
            "INSERT INTO artist_subscriptions (artist_id, tier_id, status, payment_method, subscription_start, next_payment)
             VALUES ($1, $2, $3, $4, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP + INTERVAL '1 month')
             RETURNING id"
        )
        .bind(artist_id)
        .bind(tier_id)
        .bind(&status)
        .bind(&payment_method)
        .fetch_one(pool)
        .await
        .map_err(|e| ServerFnError::new(format!("Subscription creation error: {}", e)))?;

        Ok(row.get("id"))
    }
    #[cfg(not(feature = "ssr"))]
    {
        Ok(0)
    }
}

#[cfg_attr(feature = "ssr", instrument(err, level = "info"))]
#[server]
pub async fn get_artist_subscription(
    artist_id: i32,
) -> Result<Option<ArtistSubscription>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use sqlx::Row;

        let pool = crate::db::pool::get_pool();

        match sqlx::query(
            "SELECT id, artist_id, tier_id, status, payment_method, subscription_start, subscription_end, last_payment, next_payment
             FROM artist_subscriptions WHERE artist_id = $1"
        )
        .bind(artist_id)
        .fetch_optional(pool)
        .await
        {
            Ok(Some(row)) => Ok(Some(ArtistSubscription {
                id: row.get("id"),
                artist_id: row.get("artist_id"),
                tier_id: row.get("tier_id"),
                status: row.get("status"),
                payment_method: row.get("payment_method"),
                subscription_start: row.get("subscription_start"),
                subscription_end: row.get("subscription_end"),
                last_payment: row.get("last_payment"),
                next_payment: row.get("next_payment"),
            })),
            Ok(None) => Ok(None),
            Err(e) => Err(ServerFnError::new(format!("Query error: {}", e))),
        }
    }
    #[cfg(not(feature = "ssr"))]
    {
        Ok(None)
    }
}

#[cfg_attr(feature = "ssr", instrument(err, level = "info"))]
#[server]
pub async fn check_artist_has_active_subscription(artist_id: i32) -> Result<bool, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use sqlx::Row;

        let pool = crate::db::pool::get_pool();

        let exists = sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS(SELECT 1 FROM artist_subscriptions WHERE artist_id = $1 AND status = 'active')"
        )
        .bind(artist_id)
        .fetch_one(pool)
        .await
        .map_err(|e| ServerFnError::new(format!("Query error: {}", e)))?;

        Ok(exists)
    }
    #[cfg(not(feature = "ssr"))]
    {
        Ok(false)
    }
}

// ================================
// Questionnaire System Server Functions
// ================================

#[cfg_attr(feature = "ssr", instrument(err, level = "info"))]
#[server]
pub async fn get_artist_questionnaire_form(
    artist_id: i32,
) -> Result<ClientQuestionnaireForm, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        match get_artist_questionnaire(artist_id).await {
            Ok(form) => Ok(form),
            Err(e) => Err(ServerFnError::new(format!("Database error: {}", e))),
        }
    }
    #[cfg(not(feature = "ssr"))]
    {
        Err(ServerFnError::new("Not available on client".to_string()))
    }
}

#[cfg_attr(feature = "ssr", instrument(err, level = "info"))]
#[server]
pub async fn get_default_questions() -> Result<Vec<QuestionnaireQuestion>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        match get_all_default_questions().await {
            Ok(questions) => Ok(questions),
            Err(e) => Err(ServerFnError::new(format!("Database error: {}", e))),
        }
    }
    #[cfg(not(feature = "ssr"))]
    {
        Err(ServerFnError::new("Not available on client".to_string()))
    }
}

#[cfg_attr(feature = "ssr", instrument(err, level = "info"))]
#[server]
pub async fn get_artist_questionnaire_configuration(
    artist_id: i32,
) -> Result<Vec<ArtistQuestionnaire>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        match get_artist_questionnaire_config(artist_id).await {
            Ok(config) => Ok(config),
            Err(e) => Err(ServerFnError::new(format!("Database error: {}", e))),
        }
    }
    #[cfg(not(feature = "ssr"))]
    {
        Err(ServerFnError::new("Not available on client".to_string()))
    }
}

#[cfg_attr(feature = "ssr", instrument(err, level = "info"))]
#[server]
pub async fn get_artist_id_from_jwt_user_id(
    jwt_user_id: i64,
) -> Result<Option<i32>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        match get_artist_id_from_user_id(jwt_user_id).await {
            Ok(artist_id) => Ok(artist_id),
            Err(e) => Err(ServerFnError::new(format!("Database error: {}", e))),
        }
    }
    #[cfg(not(feature = "ssr"))]
    {
        Ok(None)
    }
}

#[cfg_attr(feature = "ssr", instrument(err, level = "info"))]
#[server]
pub async fn update_artist_questionnaire_configuration(
    artist_id: i32,
    config: Vec<ArtistQuestionnaire>,
) -> Result<(), ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        match update_artist_questionnaire_config(artist_id, config).await {
            Ok(()) => Ok(()),
            Err(e) => Err(ServerFnError::new(format!("Database error: {}", e))),
        }
    }
    #[cfg(not(feature = "ssr"))]
    {
        Ok(())
    }
}

#[cfg_attr(feature = "ssr", instrument(err, level = "info"))]
#[server]
pub async fn delete_artist_questionnaire_question(
    artist_id: i32,
    question_id: i32,
) -> Result<(), ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        match delete_artist_question(artist_id, question_id).await {
            Ok(()) => Ok(()),
            Err(e) => Err(ServerFnError::new(format!("Database error: {}", e))),
        }
    }
    #[cfg(not(feature = "ssr"))]
    {
        Ok(())
    }
}

#[cfg_attr(feature = "ssr", instrument(err, level = "info"))]
#[server]
pub async fn submit_questionnaire_responses(
    submission: ClientQuestionnaireSubmission,
) -> Result<(), ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        // Convert client responses to database format
        let responses: Vec<BookingQuestionnaireResponse> = submission
            .responses
            .into_iter()
            .map(|r| BookingQuestionnaireResponse {
                id: 0, // Will be auto-generated
                booking_request_id: submission.booking_request_id,
                question_id: r.question_id,
                response_text: r.response_text,
                response_data: r.response_data,
                created_at: None, // Will be auto-generated
            })
            .collect();

        match save_questionnaire_responses(submission.booking_request_id, responses).await {
            Ok(()) => Ok(()),
            Err(e) => Err(ServerFnError::new(format!("Database error: {}", e))),
        }
    }
    #[cfg(not(feature = "ssr"))]
    {
        Ok(())
    }
}

#[cfg_attr(feature = "ssr", instrument(err, level = "info"))]
#[server]
pub async fn get_booking_responses(
    booking_request_id: i32,
) -> Result<Vec<BookingQuestionnaireResponse>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        match get_booking_questionnaire_responses(booking_request_id).await {
            Ok(responses) => Ok(responses),
            Err(e) => Err(ServerFnError::new(format!("Database error: {}", e))),
        }
    }
    #[cfg(not(feature = "ssr"))]
    {
        Err(ServerFnError::new("Not available on client".to_string()))
    }
}

// Error Logging Server Functions
#[cfg_attr(feature = "ssr", instrument(err, level = "info"))]
#[server]
pub async fn log_client_error(
    error_type: String,
    error_level: String,
    error_message: String,
    error_stack: Option<String>,
    url_path: Option<String>,
    user_agent: Option<String>,
    session_id: Option<String>,
    additional_context: Option<String>,
) -> Result<i64, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        let error_data = CreateErrorLog {
            error_type,
            error_level,
            error_message,
            error_stack,
            url_path,
            user_agent,
            user_id: None, // TODO: Extract from JWT token when available
            session_id,
            request_headers: None, // TODO: Extract from request context
            additional_context,
        };

        match log_error(error_data).await {
            Ok(id) => Ok(id),
            Err(e) => Err(ServerFnError::new(format!("Failed to log error: {}", e))),
        }
    }
    #[cfg(not(feature = "ssr"))]
    {
        Err(ServerFnError::new("Not available on client".to_string()))
    }
}

#[cfg_attr(feature = "ssr", instrument(err, level = "info"))]
#[server]
pub async fn get_artist_id_from_user(user_id: i32) -> Result<i32, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use sqlx::Row;

        let pool = crate::db::pool::get_pool();

        let artist_id: i32 =
            sqlx::query_scalar("SELECT artist_id FROM users WHERE id = $1 AND role = 'artist'")
                .bind(user_id)
                .fetch_one(pool)
                .await
                .map_err(|e| ServerFnError::new(format!("Failed to get artist_id: {}", e)))?;

        Ok(artist_id)
    }
    #[cfg(not(feature = "ssr"))]
    {
        Err(ServerFnError::new("Not available on client".to_string()))
    }
}

#[cfg_attr(feature = "ssr", instrument(err, level = "info"))]
#[server]
pub async fn get_artist_styles_by_id(artist_id: i64) -> Result<Vec<String>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use sqlx::Row;

        let pool = crate::db::pool::get_pool();

        let styles: Vec<String> = sqlx::query_scalar(
            "SELECT s.name
            FROM styles s
            JOIN artists_styles ast ON s.id = ast.style_id
            WHERE ast.artist_id = $1
            ORDER BY s.name ASC",
        )
        .bind(artist_id)
        .fetch_all(pool)
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to query styles: {}", e)))?;

        Ok(styles)
    }
    #[cfg(not(feature = "ssr"))]
    {
        Ok(vec![])
    }
}

#[cfg_attr(feature = "ssr", instrument(err, level = "info"))]
#[server]
pub async fn log_server_error(
    error_message: String,
    error_stack: Option<String>,
    url_path: Option<String>,
    additional_context: Option<String>,
) -> Result<i64, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        let error_data = CreateErrorLog {
            error_type: "server".to_string(),
            error_level: "error".to_string(),
            error_message,
            error_stack,
            url_path,
            user_agent: None,
            user_id: None, // TODO: Extract from JWT token when available
            session_id: None,
            request_headers: None, // TODO: Extract from request context
            additional_context,
        };

        match log_error(error_data).await {
            Ok(id) => Ok(id),
            Err(e) => Err(ServerFnError::new(format!(
                "Failed to log server error: {}",
                e
            ))),
        }
    }
    #[cfg(not(feature = "ssr"))]
    {
        Err(ServerFnError::new("Not available on client".to_string()))
    }
}

#[cfg_attr(feature = "ssr", instrument(err, level = "info"))]
#[server]
pub async fn get_error_logs(
    limit: Option<i32>,
    error_type: Option<String>,
) -> Result<Vec<ErrorLog>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        let result = if let Some(err_type) = error_type {
            get_errors_by_type(err_type, limit.unwrap_or(100)).await
        } else {
            get_recent_errors(limit.unwrap_or(100)).await
        };

        match result {
            Ok(errors) => Ok(errors),
            Err(e) => Err(ServerFnError::new(format!(
                "Failed to fetch error logs: {}",
                e
            ))),
        }
    }
    #[cfg(not(feature = "ssr"))]
    {
        Err(ServerFnError::new("Not available on client".to_string()))
    }
}

// Booking Availability Server Functions

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct TimeSlot {
    pub start_time: String,
    pub end_time: String,
    pub is_available: bool,
}

#[cfg_attr(feature = "ssr", instrument(err, level = "info"))]
#[server]
pub async fn get_available_dates(
    artist_id: i32,
    start_date: String,
    end_date: String,
) -> Result<Vec<String>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use sqlx::Row;

        async fn query_available_dates(
            artist_id: i32,
            start_date: String,
            end_date: String,
        ) -> Result<Vec<String>, sqlx::Error> {
            let pool = crate::db::pool::get_pool();

            // Get business hours for this artist
            let business_hours_rows = sqlx::query(
                "SELECT day_of_week, start_time, end_time, is_closed
                 FROM business_hours
                 WHERE artist_id = $1
                 ORDER BY day_of_week",
            )
            .bind(artist_id)
            .fetch_all(pool)
            .await?;

            let mut business_hours = std::collections::HashMap::new();
            for row in business_hours_rows {
                let day: i32 = row.get("day_of_week");
                let start: Option<String> = row.get("start_time");
                let end: Option<String> = row.get("end_time");
                let closed: bool = row.get("is_closed");
                business_hours.insert(day, (start, end, closed));
            }

            // Get specific availability overrides
            let availability_rows = sqlx::query(
                "SELECT specific_date, is_available
                 FROM artist_availability
                 WHERE artist_id = $1
                   AND specific_date IS NOT NULL
                   AND specific_date BETWEEN $2 AND $3",
            )
            .bind(artist_id)
            .bind(&start_date)
            .bind(&end_date)
            .fetch_all(pool)
            .await?;

            let mut availability_overrides = std::collections::HashMap::new();
            for row in availability_rows {
                let date: String = row.get("specific_date");
                let available: bool = row.get("is_available");
                availability_overrides.insert(date, available);
            }

            // Get existing bookings in date range
            let booking_rows = sqlx::query(
                "SELECT requested_date
                 FROM booking_requests
                 WHERE artist_id = $1
                   AND requested_date BETWEEN $2 AND $3
                   AND status IN ('pending', 'approved')",
            )
            .bind(artist_id)
            .bind(&start_date)
            .bind(&end_date)
            .fetch_all(pool)
            .await?;

            let mut booked_dates = std::collections::HashSet::new();
            for row in booking_rows {
                let date: String = row.get("requested_date");
                booked_dates.insert(date);
            }

            // Generate available dates
            let mut available_dates = Vec::new();
            let start = chrono::NaiveDate::parse_from_str(&start_date, "%Y-%m-%d")
                .map_err(|e| sqlx::Error::Decode(Box::new(e)))?;
            let end = chrono::NaiveDate::parse_from_str(&end_date, "%Y-%m-%d")
                .map_err(|e| sqlx::Error::Decode(Box::new(e)))?;
            let today = Utc::now().naive_utc().date();

            let mut current_date = start;
            while current_date <= end {
                let date_str = current_date.format("%Y-%m-%d").to_string();

                // Skip past dates
                if current_date < today {
                    current_date += chrono::Duration::days(1);
                    continue;
                }

                // Check for specific availability override
                if let Some(&is_available) = availability_overrides.get(&date_str) {
                    if is_available && !booked_dates.contains(&date_str) {
                        available_dates.push(date_str);
                    }
                } else {
                    // Check business hours for this day of week
                    let day_of_week = current_date.weekday().num_days_from_sunday() as i32;

                    if let Some((start_time, end_time, is_closed)) =
                        business_hours.get(&day_of_week)
                    {
                        if !is_closed
                            && start_time.is_some()
                            && end_time.is_some()
                            && !booked_dates.contains(&date_str)
                        {
                            available_dates.push(date_str);
                        }
                    }
                }

                current_date += chrono::Duration::days(1);
            }

            Ok(available_dates)
        }

        match query_available_dates(artist_id, start_date, end_date).await {
            Ok(dates) => Ok(dates),
            Err(e) => Err(ServerFnError::new(format!(
                "Failed to get available dates: {}",
                e
            ))),
        }
    }
    #[cfg(not(feature = "ssr"))]
    {
        Ok(vec![])
    }
}

#[cfg_attr(feature = "ssr", instrument(err, level = "info"))]
#[server]
pub async fn get_available_time_slots(
    artist_id: i32,
    date: String,
) -> Result<Vec<TimeSlot>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use sqlx::Row;

        async fn query_time_slots(
            artist_id: i32,
            date: String,
        ) -> Result<Vec<TimeSlot>, sqlx::Error> {
            let pool = crate::db::pool::get_pool();

            // Parse the date to get day of week
            let parsed_date = chrono::NaiveDate::parse_from_str(&date, "%Y-%m-%d")
                .map_err(|e| sqlx::Error::Decode(Box::new(e)))?;
            let day_of_week = parsed_date.weekday().num_days_from_sunday() as i32;

            // Get business hours for this day
            let business_hours: Option<(String, String)> = sqlx::query(
                "SELECT start_time, end_time
                 FROM business_hours
                 WHERE artist_id = $1 AND day_of_week = $2 AND is_closed = false",
            )
            .bind(artist_id)
            .bind(day_of_week)
            .fetch_optional(pool)
            .await?
            .map(|row| (row.get("start_time"), row.get("end_time")));

            if business_hours.is_none() {
                return Ok(vec![]);
            }

            let (start_time_str, end_time_str) = business_hours.unwrap();

            // Check for existing bookings on this date
            let booking_rows = sqlx::query(
                "SELECT requested_start_time, requested_end_time
                 FROM booking_requests
                 WHERE artist_id = $1
                   AND requested_date = $2
                   AND status IN ('pending', 'approved')",
            )
            .bind(artist_id)
            .bind(&date)
            .fetch_all(pool)
            .await?;

            let mut booked_slots = Vec::new();
            for row in booking_rows {
                let start: String = row.get("requested_start_time");
                let end: Option<String> = row.get("requested_end_time");
                booked_slots.push((start, end));
            }

            // Generate time slots (hourly slots from business hours)
            let mut time_slots = Vec::new();

            // Parse start and end times
            let start_hour = start_time_str
                .split(':')
                .next()
                .unwrap_or("9")
                .parse::<u32>()
                .unwrap_or(9);
            let end_hour = end_time_str
                .split(':')
                .next()
                .unwrap_or("17")
                .parse::<u32>()
                .unwrap_or(17);

            for hour in start_hour..end_hour {
                let slot_start = format!("{:02}:00", hour);
                let slot_end = format!("{:02}:00", hour + 1);

                // Check if this slot is booked
                let is_booked = booked_slots.iter().any(|(booked_start, booked_end)| {
                    let booked_end_time = booked_end.as_ref().unwrap_or(booked_start);
                    // Simple overlap check
                    !(slot_end <= *booked_start || slot_start >= *booked_end_time)
                });

                time_slots.push(TimeSlot {
                    start_time: slot_start,
                    end_time: slot_end,
                    is_available: !is_booked,
                });
            }

            Ok(time_slots)
        }

        match query_time_slots(artist_id, date).await {
            Ok(slots) => Ok(slots),
            Err(e) => Err(ServerFnError::new(format!(
                "Failed to get time slots: {}",
                e
            ))),
        }
    }
    #[cfg(not(feature = "ssr"))]
    {
        Ok(vec![])
    }
}

// ============================================================================
// Admin Style Management Functions
// ============================================================================

/// Helper function to extract user info from JWT token on server side
#[cfg(feature = "ssr")]
fn extract_user_from_token(token: &str) -> Option<(i64, String)> {
    use jsonwebtoken::{decode, DecodingKey, Validation};

    #[derive(Debug, Serialize, Deserialize)]
    struct Claims {
        sub: String,
        exp: usize,
        user_type: String,
        user_id: i64,
    }

    let secret = "tatteau-jwt-secret-key-change-in-production";
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_ref()),
        &Validation::default(),
    )
    .ok()?;

    Some((token_data.claims.user_id, token_data.claims.user_type))
}

/// Adds a style tag to an image (admin only)
#[cfg_attr(feature = "ssr", instrument(skip(token), err, level = "info"))]
#[server]
pub async fn add_style_to_image(
    image_id: i64,
    style_id: i64,
    token: String,
) -> Result<(), ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use sqlx::Row;

        // Verify admin role
        let (user_id, user_type) = extract_user_from_token(&token)
            .ok_or_else(|| ServerFnError::new("Invalid or expired token".to_string()))?;

        if user_type != "admin" {
            return Err(ServerFnError::new(
                "Unauthorized: Admin access required".to_string(),
            ));
        }

        let pool = crate::db::pool::get_pool();

        // Insert the style association with admin tracking
        let result = sqlx::query(
            "INSERT INTO artists_images_styles
             (artists_images_id, style_id, is_admin_corrected, corrected_by, corrected_at)
             VALUES ($1, $2, true, $3, CURRENT_TIMESTAMP)
             ON CONFLICT (artists_images_id, style_id)
             DO UPDATE SET
                is_admin_corrected = true,
                corrected_by = $3,
                corrected_at = CURRENT_TIMESTAMP",
        )
        .bind(image_id)
        .bind(style_id)
        .bind(user_id)
        .execute(pool)
        .await;

        match result {
            Ok(_) => Ok(()),
            Err(e) => Err(ServerFnError::new(format!(
                "Failed to add style to image: {}",
                e
            ))),
        }
    }
    #[cfg(not(feature = "ssr"))]
    {
        Err(ServerFnError::new("Not available on client".to_string()))
    }
}

/// Removes a style tag from an image (admin only)
#[cfg_attr(feature = "ssr", instrument(skip(token), err, level = "info"))]
#[server]
pub async fn remove_style_from_image(
    image_id: i64,
    style_id: i64,
    token: String,
) -> Result<(), ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        // Verify admin role
        let (_user_id, user_type) = extract_user_from_token(&token)
            .ok_or_else(|| ServerFnError::new("Invalid or expired token".to_string()))?;

        if user_type != "admin" {
            return Err(ServerFnError::new(
                "Unauthorized: Admin access required".to_string(),
            ));
        }

        let pool = crate::db::pool::get_pool();

        // Delete the style association
        let result = sqlx::query(
            "DELETE FROM artists_images_styles
             WHERE artists_images_id = $1 AND style_id = $2",
        )
        .bind(image_id)
        .bind(style_id)
        .execute(pool)
        .await;

        match result {
            Ok(_) => Ok(()),
            Err(e) => Err(ServerFnError::new(format!(
                "Failed to remove style from image: {}",
                e
            ))),
        }
    }
    #[cfg(not(feature = "ssr"))]
    {
        Err(ServerFnError::new("Not available on client".to_string()))
    }
}

/// Gets all available styles for the admin modal
#[cfg_attr(feature = "ssr", instrument(err, level = "info"))]
#[server]
pub async fn get_all_styles_for_admin() -> Result<Vec<Style>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use sqlx::Row;

        let pool = crate::db::pool::get_pool();

        let styles: Vec<Style> = sqlx::query("SELECT id, name FROM styles ORDER BY name ASC")
            .fetch_all(pool)
            .await
            .map_err(|e| ServerFnError::new(format!("Failed to fetch styles: {}", e)))?
            .into_iter()
            .map(|row| Style {
                id: row.get::<i64, _>("id") as i32,
                name: row.get("name"),
            })
            .collect();

        Ok(styles)
    }
    #[cfg(not(feature = "ssr"))]
    {
        Ok(vec![])
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ImageStyleMetadata {
    pub image_id: i64,
    pub current_styles: Vec<Style>,
    pub llm_recommended_styles: Vec<Style>,
    pub admin_corrected_count: i32,
}

/// Gets style metadata for an image including LLM recommendations and admin corrections
#[cfg_attr(feature = "ssr", instrument(err, level = "info"))]
#[server]
pub async fn get_image_style_metadata(image_id: i64) -> Result<ImageStyleMetadata, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use sqlx::Row;

        let pool = crate::db::pool::get_pool();

        // Get current styles
        let current_styles: Vec<Style> = sqlx::query(
            "SELECT s.id, s.name
             FROM styles s
             JOIN artists_images_styles ais ON s.id = ais.style_id
             WHERE ais.artists_images_id = $1
             ORDER BY s.name",
        )
        .bind(image_id)
        .fetch_all(pool)
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to fetch current styles: {}", e)))?
        .into_iter()
        .map(|row| Style {
            id: row.get::<i64, _>("id") as i32,
            name: row.get("name"),
        })
        .collect();

        // Get LLM recommended styles (original)
        let llm_recommended_styles: Vec<Style> = sqlx::query(
            "SELECT s.id, s.name
             FROM styles s
             JOIN image_style_llm_recommendations isr ON s.id = isr.style_id
             WHERE isr.artists_images_id = $1
             ORDER BY s.name",
        )
        .bind(image_id)
        .fetch_all(pool)
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to fetch LLM recommendations: {}", e)))?
        .into_iter()
        .map(|row| Style {
            id: row.get::<i64, _>("id") as i32,
            name: row.get("name"),
        })
        .collect();

        // Count admin corrections
        let admin_corrected_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM artists_images_styles
             WHERE artists_images_id = $1 AND is_admin_corrected = true",
        )
        .bind(image_id)
        .fetch_one(pool)
        .await
        .unwrap_or(0);

        Ok(ImageStyleMetadata {
            image_id,
            current_styles,
            llm_recommended_styles,
            admin_corrected_count: admin_corrected_count as i32,
        })
    }
    #[cfg(not(feature = "ssr"))]
    {
        Ok(ImageStyleMetadata {
            image_id,
            current_styles: vec![],
            llm_recommended_styles: vec![],
            admin_corrected_count: 0,
        })
    }
}
