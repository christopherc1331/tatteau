use leptos::prelude::*;
use leptos::server;
use shared_types::LocationInfo;
use shared_types::MapBounds;

use crate::db::entities::{Artist, ArtistImage, CityCoords, Location, Style, AvailabilitySlot, BookingRequest, BookingMessage, AvailabilityUpdate, RecurringRule};
use crate::db::search_repository::{SearchResult};
use serde::{Deserialize, Serialize};

#[cfg(feature = "ssr")]
use chrono::{NaiveDateTime, Utc};

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
    get_all_images_with_styles_by_location, get_all_styles_by_location, get_artist_by_id,
    get_artist_images_with_styles, get_artist_location, get_artist_styles, get_artists_by_location,
    get_cities_and_coords, get_city_coordinates, get_location_by_id, get_states, query_locations,
};

#[server]
pub async fn fetch_locations(
    state: String,
    city: String,
    bounds: MapBounds,
) -> Result<Vec<LocationInfo>, ServerFnError> {
    match query_locations(state, city, bounds) {
        Ok(locations) => Ok(locations),
        Err(e) => Err(ServerFnError::new(format!("Database error: {}", e))),
    }
}

#[server]
pub async fn get_cities(state: String) -> Result<Vec<CityCoords>, ServerFnError> {
    match get_cities_and_coords(state) {
        Ok(cities) => Ok(cities),
        Err(e) => Err(ServerFnError::new(format!("Database error: {}", e))),
    }
}

#[server]
pub async fn get_states_list() -> Result<Vec<String>, ServerFnError> {
    match get_states() {
        Ok(states) => Ok(states.into_iter().map(|s| s.state).collect()),
        Err(e) => Err(ServerFnError::new(format!("Database error: {}", e))),
    }
    // Ok(vec![
    //     "Texas".to_string(),
    //     "California".to_string(),
    //     "New York".to_string(),
    // ])
}

#[server]
pub async fn get_center_coordinates_for_cities(
    cities: Vec<CityCoords>,
) -> Result<CityCoords, ServerFnError> {
    let city_name = &cities
        .first()
        .expect("At least one city should be passed")
        .city;

    match get_city_coordinates(city_name.to_string()) {
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
    pub images_with_styles: Vec<(ArtistImage, Vec<Style>)>,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct ShopData {
    pub location: Location,
    pub artists: Vec<Artist>,
    pub all_styles: Vec<Style>,
    pub all_images_with_styles: Vec<(ArtistImage, Vec<Style>, Artist)>,
}

#[server]
pub async fn fetch_artist_data(artist_id: i32) -> Result<ArtistData, ServerFnError> {
    let artist = get_artist_by_id(artist_id)
        .map_err(|e| ServerFnError::new(format!("Failed to fetch artist: {}", e)))?;

    let location = get_artist_location(artist.location_id)
        .map_err(|e| ServerFnError::new(format!("Failed to fetch location: {}", e)))?;

    let styles = get_artist_styles(artist_id)
        .map_err(|e| ServerFnError::new(format!("Failed to fetch styles: {}", e)))?;

    let images_with_styles = get_artist_images_with_styles(artist_id)
        .map_err(|e| ServerFnError::new(format!("Failed to fetch images: {}", e)))?;

    Ok(ArtistData {
        artist,
        location,
        styles,
        images_with_styles,
    })
}

#[server]
pub async fn fetch_shop_data(location_id: i32) -> Result<ShopData, ServerFnError> {
    let location = get_location_by_id(location_id)
        .map_err(|e| ServerFnError::new(format!("Failed to fetch location: {}", e)))?;

    let artists = get_artists_by_location(location_id)
        .map_err(|e| ServerFnError::new(format!("Failed to fetch artists: {}", e)))?;

    let all_styles = get_all_styles_by_location(location_id)
        .map_err(|e| ServerFnError::new(format!("Failed to fetch styles: {}", e)))?;

    let all_images_with_styles = get_all_images_with_styles_by_location(location_id)
        .map_err(|e| ServerFnError::new(format!("Failed to fetch images: {}", e)))?;

    Ok(ShopData {
        location,
        artists,
        all_styles,
        all_images_with_styles,
    })
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct LocationStats {
    pub shop_count: i32,
    pub artist_count: i32,
    pub styles_available: i32,
}

#[server]
pub async fn get_location_stats(
    city: String,
    state: String,
) -> Result<LocationStats, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use crate::db::repository::get_location_stats_for_city;
        match get_location_stats_for_city(city, state) {
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


#[server]
pub async fn get_available_styles() -> Result<Vec<StyleWithCount>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use crate::db::repository::get_all_styles_with_counts;
        match get_all_styles_with_counts() {
            Ok(styles) => Ok(styles),
            Err(e) => Err(ServerFnError::new(format!("Failed to fetch styles: {}", e))),
        }
    }
    #[cfg(not(feature = "ssr"))]
    {
        Ok(vec![])
    }
}

#[server]
pub async fn get_styles_in_bounds(bounds: MapBounds) -> Result<Vec<StyleWithCount>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use crate::db::repository::get_styles_with_counts_in_bounds;
        match get_styles_with_counts_in_bounds(bounds) {
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

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct EnhancedLocationInfo {
    pub location: LocationInfo,
    pub artist_count: i32,
    pub image_count: i32,
    pub styles: Vec<String>,
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

#[server]
pub async fn get_locations_with_details(
    state: String,
    city: String,
    bounds: MapBounds,
    style_filter: Option<Vec<i32>>,
) -> Result<Vec<EnhancedLocationInfo>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use crate::db::repository::query_locations_with_details;
        match query_locations_with_details(state, city, bounds, style_filter) {
            Ok(locations) => Ok(locations),
            Err(e) => Err(ServerFnError::new(format!(
                "Failed to fetch locations: {}",
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

#[server]
pub async fn get_matched_artists(
    style_preferences: Vec<String>,
    location: String,
    price_range: Option<(f64, f64)>,
) -> Result<Vec<MatchedArtist>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use crate::db::repository::query_matched_artists;
        match query_matched_artists(style_preferences, location, price_range) {
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

#[server]
pub async fn get_location_details(location_id: i64) -> Result<LocationDetailInfo, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use crate::db::repository::get_location_with_artist_details;
        match get_location_with_artist_details(location_id) {
            Ok(details) => Ok(details),
            Err(e) => Err(ServerFnError::new(format!(
                "Failed to get location details: {}",
                e
            ))),
        }
    }
    #[cfg(not(feature = "ssr"))]
    {
        Err(ServerFnError::new(
            "Server-side rendering not available".to_string(),
        ))
    }
}

#[server]
pub async fn search_by_postal_code(postal_code: String) -> Result<CityCoords, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use crate::db::repository::get_coords_by_postal_code;
        match get_coords_by_postal_code(postal_code) {
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

#[server]
pub async fn universal_search(query: String) -> Result<Vec<SearchResult>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use crate::db::search_repository::universal_location_search;
        match universal_location_search(query) {
            Ok(results) => Ok(results),
            Err(e) => Err(ServerFnError::new(format!("Search failed: {}", e))),
        }
    }
    #[cfg(not(feature = "ssr"))]
    {
        Ok(vec![])
    }
}

#[server]
pub async fn get_search_suggestions(query: String) -> Result<Vec<String>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use crate::db::search_repository::get_search_suggestions as get_suggestions;
        match get_suggestions(query, 10) {
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

#[server]
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

#[server]
pub async fn get_artist_dashboard_data(artist_id: i32) -> Result<ArtistDashboardData, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use rusqlite::{Connection, Result as SqliteResult};
        use std::path::Path;

        fn query_dashboard_data(artist_id: i32) -> SqliteResult<ArtistDashboardData> {
            let db_path = Path::new("tatteau.db");
            let conn = Connection::open(db_path)?;

            // Get today's bookings count
            let today = Utc::now().naive_utc().date();
            let today_str = today.to_string();
            let mut stmt = conn.prepare(
                "SELECT COUNT(*) FROM bookings 
                 WHERE artist_id = ?1 AND DATE(created_at) = ?2"
            )?;
            let todays_bookings: i32 = stmt.query_row([&artist_id.to_string(), &today_str], |row| {
                row.get(0)
            }).unwrap_or(0);

            // Get pending sketch requests count (placeholder - would need sketch_requests table)
            let pending_sketches = 3; // Placeholder

            // Get unread messages count (placeholder - would need messages table)
            let unread_messages = 7; // Placeholder

            // Get monthly revenue (placeholder calculation)
            let monthly_revenue = 1250.0; // Placeholder

            // Get recent bookings
            let mut recent_stmt = conn.prepare(
                "SELECT b.id, b.client_name, b.placement, b.created_at
                 FROM bookings b
                 WHERE b.artist_id = ?1
                 ORDER BY b.created_at DESC
                 LIMIT 5"
            )?;

            let recent_booking_iter = recent_stmt.query_map([&artist_id], |row| {
                let created_at_str: String = row.get(3)?;
                // Try to parse the date string and format it, fallback to original if parsing fails
                let formatted_date = if let Ok(naive_date) = NaiveDateTime::parse_from_str(&created_at_str, "%Y-%m-%d %H:%M:%S") {
                    naive_date.format("%B %d, %Y").to_string()
                } else {
                    created_at_str.clone()
                };
                
                Ok(RecentBooking {
                    id: row.get(0)?,
                    client_name: row.get(1)?,
                    placement: row.get(2)?,
                    created_at: formatted_date,
                })
            })?;

            let mut recent_bookings = Vec::new();
            for booking in recent_booking_iter {
                if let Ok(booking) = booking {
                    recent_bookings.push(booking);
                }
            }

            Ok(ArtistDashboardData {
                todays_bookings,
                pending_sketches,
                unread_messages,
                monthly_revenue,
                recent_bookings,
            })
        }

        match query_dashboard_data(artist_id) {
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

#[server]
pub async fn log_match_impression(
    session_id: Option<i32>,
    artist_id: i64,
    impression_type: String, // "view" or "click"
) -> Result<(), ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use rusqlite::{Connection, params};
        use std::path::Path;
        use std::time::{SystemTime, UNIX_EPOCH};

        // Use session ID if available, otherwise create a temp session ID based on timestamp
        let session_id = session_id.unwrap_or_else(|| {
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs() as i32
        });

        let db_path = Path::new("tatteau.db");
        let conn = Connection::open(db_path).map_err(|e| {
            ServerFnError::new(format!("Database connection error: {}", e))
        })?;

        conn.execute(
            "INSERT INTO client_match_impressions (session_id, artist_id, impression_type)
             VALUES (?1, ?2, ?3)",
            params![session_id, artist_id, impression_type],
        ).map_err(|e| {
            ServerFnError::new(format!("Failed to log impression: {}", e))
        })?;
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

#[server]
pub async fn get_all_styles_with_counts() -> Result<Vec<StyleWithCount>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use rusqlite::{Connection, params};
        use std::path::Path;

        let db_path = Path::new("tatteau.db");
        let conn = Connection::open(db_path).map_err(|e| {
            ServerFnError::new(format!("Database connection error: {}", e))
        })?;

        let mut stmt = conn.prepare("
            SELECT 
                s.id,
                s.name,
                s.description,
                COUNT(DISTINCT ast.artist_id) as artist_count,
                GROUP_CONCAT(DISTINCT ai.instagram_url, '|') as sample_images
            FROM styles s
            LEFT JOIN artists_styles ast ON s.id = ast.style_id
            LEFT JOIN artists a ON ast.artist_id = a.id
            LEFT JOIN artists_images ai ON a.id = ai.artist_id
            GROUP BY s.id, s.name, s.description
            HAVING artist_count > 0
            ORDER BY artist_count DESC, s.name ASC
        ").map_err(|e| {
            ServerFnError::new(format!("Failed to prepare statement: {}", e))
        })?;

        let styles = stmt.query_map([], |row| {
            let sample_images_str: Option<String> = row.get(4).ok();
            let sample_images = sample_images_str
                .and_then(|s| if s.trim().is_empty() { None } else { Some(s) })
                .map(|s| s.split('|').map(|url| url.trim().to_string()).filter(|url| !url.is_empty()).collect::<Vec<_>>());

            Ok(StyleWithCount {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2).ok(),
                artist_count: row.get(3)?,
                sample_images,
            })
        }).map_err(|e| {
            ServerFnError::new(format!("Failed to query styles: {}", e))
        })?;

        let mut result = Vec::new();
        for style in styles {
            match style {
                Ok(style_info) => result.push(style_info),
                Err(e) => return Err(ServerFnError::new(format!("Row error: {}", e))),
            }
        }

        Ok(result)
    }
    
    #[cfg(not(feature = "ssr"))]
    {
        // Return placeholder data for client-side
        Ok(vec![
            StyleWithCount {
                id: 1,
                name: "Traditional".to_string(),
                description: Some("Classic American tattoo style with bold lines and bright colors".to_string()),
                artist_count: 12,
                sample_images: None,
            },
            StyleWithCount {
                id: 2,
                name: "Neo-Traditional".to_string(),
                description: Some("Modern take on traditional tattoos with enhanced detail and color".to_string()),
                artist_count: 8,
                sample_images: None,
            }
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
}

#[server]
pub async fn get_tattoo_posts_by_style(
    style_names: Vec<String>,
    limit: Option<i64>,
) -> Result<Vec<TattooPost>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use rusqlite::{Connection, params};
        use std::path::Path;

        let db_path = Path::new("tatteau.db");
        let conn = Connection::open(db_path).map_err(|e| {
            ServerFnError::new(format!("Database connection error: {}", e))
        })?;

        let limit_value = limit.unwrap_or(100);

        // Simple approach: just use the first style for now to get it working
        let target_style = style_names.first().unwrap_or(&"japanese".to_string()).to_lowercase();

        let query = "
            SELECT DISTINCT 
                ai.id,
                ai.short_code,
                ai.artist_id,
                a.name as artist_name,
                a.instagram_handle as artist_instagram
            FROM artists_images ai
            JOIN artists a ON ai.artist_id = a.id
            JOIN artists_images_styles ais ON ai.id = ais.artists_images_id
            JOIN styles s ON ais.style_id = s.id
            WHERE LOWER(s.name) = ?
            ORDER BY ai.id DESC
            LIMIT ?
        ";

        let mut stmt = conn.prepare(query).map_err(|e| {
            ServerFnError::new(format!("Failed to prepare statement: {}", e))
        })?;

        let post_iter = stmt.query_map(params![target_style, limit_value], |row| {
            let image_id: i64 = row.get(0)?;
            let short_code: String = row.get(1)?;
            let artist_id: i64 = row.get(2)?;
            let artist_name: String = row.get(3)?;
            let artist_instagram: Option<String> = row.get(4)?;

            Ok((image_id, short_code, artist_id, artist_name, artist_instagram))
        }).map_err(|e| {
            ServerFnError::new(format!("Failed to query posts: {}", e))
        })?;

        let mut posts = Vec::new();
        for post_result in post_iter {
            match post_result {
                Ok((image_id, short_code, artist_id, artist_name, artist_instagram)) => {
                    // Get styles for this specific image
                    let style_query = "
                        SELECT s.name
                        FROM styles s
                        JOIN artists_images_styles ais ON s.id = ais.style_id
                        WHERE ais.artists_images_id = ?
                    ";
                    
                    let mut style_stmt = conn.prepare(style_query).map_err(|e| {
                        ServerFnError::new(format!("Failed to prepare style query: {}", e))
                    })?;
                    
                    let style_iter = style_stmt.query_map([image_id], |row| {
                        Ok(row.get::<_, String>(0)?)
                    }).map_err(|e| {
                        ServerFnError::new(format!("Failed to query styles: {}", e))
                    })?;
                    
                    let mut styles = Vec::new();
                    for style_result in style_iter {
                        if let Ok(style_name) = style_result {
                            styles.push(style_name);
                        }
                    }

                    posts.push(TattooPost {
                        id: image_id,
                        short_code,
                        artist_id,
                        artist_name,
                        artist_instagram,
                        styles,
                    });
                },
                Err(e) => return Err(ServerFnError::new(format!("Row error: {}", e))),
            }
        }

        Ok(posts)
    }

    #[cfg(not(feature = "ssr"))]
    {
        // Return placeholder data for client-side
        Ok(vec![
            TattooPost {
                id: 1,
                short_code: "ABC123".to_string(),
                artist_id: 1,
                artist_name: "Sample Artist".to_string(),
                artist_instagram: Some("sample_artist".to_string()),
                styles: vec!["Japanese".to_string(), "Traditional".to_string()],
            }
        ])
    }
}

#[server]
pub async fn get_artist_availability(
    artist_id: i32,
    start_date: String,
    end_date: String,
) -> Result<Vec<AvailabilitySlot>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use rusqlite::{params, Connection, Result as SqliteResult};
        use std::path::Path;

        fn query_availability(artist_id: i32, start_date: String, end_date: String) -> SqliteResult<Vec<AvailabilitySlot>> {
            let db_path = Path::new("tatteau.db");
            let conn = Connection::open(db_path)?;
            
            let mut stmt = conn.prepare("
                SELECT id, artist_id, day_of_week, specific_date, start_time, end_time, 
                       is_available, is_recurring, created_at
                FROM artist_availability 
                WHERE artist_id = ?1 
                AND (specific_date IS NULL OR (specific_date >= ?2 AND specific_date <= ?3))
                ORDER BY day_of_week, start_time
            ")?;
            
            let availability_iter = stmt.query_map(params![artist_id, start_date, end_date], |row| {
                Ok(AvailabilitySlot {
                    id: row.get(0)?,
                    artist_id: row.get(1)?,
                    day_of_week: row.get(2)?,
                    specific_date: row.get(3)?,
                    start_time: row.get(4)?,
                    end_time: row.get(5)?,
                    is_available: row.get(6)?,
                    is_recurring: row.get(7)?,
                    created_at: row.get(8)?,
                })
            })?;
            
            let mut slots = Vec::new();
            for slot in availability_iter {
                slots.push(slot?);
            }
            
            Ok(slots)
        }

        match query_availability(artist_id, start_date, end_date) {
            Ok(availability) => Ok(availability),
            Err(e) => Err(ServerFnError::new(format!("Failed to get availability: {}", e))),
        }
    }
    #[cfg(not(feature = "ssr"))]
    {
        Ok(vec![])
    }
}

#[server]
pub async fn set_artist_availability(
    availability: AvailabilityUpdate,
) -> Result<(), ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use rusqlite::{Connection, Result as SqliteResult};
        use std::path::Path;

        fn update_availability(availability: AvailabilityUpdate) -> SqliteResult<()> {
            let db_path = Path::new("tatteau.db");
            let conn = Connection::open(db_path)?;
            
            conn.execute("
                INSERT INTO artist_availability 
                (artist_id, day_of_week, specific_date, start_time, end_time, is_available, is_recurring)
                VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
            ", (
                availability.artist_id,
                availability.day_of_week,
                availability.date,
                availability.start_time,
                availability.end_time,
                availability.is_available,
                availability.is_recurring,
            ))?;
            
            Ok(())
        }

        match update_availability(availability) {
            Ok(_) => Ok(()),
            Err(e) => Err(ServerFnError::new(format!("Failed to set availability: {}", e))),
        }
    }
    #[cfg(not(feature = "ssr"))]
    {
        Ok(())
    }
}

#[server]
pub async fn get_booking_requests(artist_id: i32) -> Result<Vec<BookingRequest>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use rusqlite::{Connection, Result as SqliteResult};
        use std::path::Path;

        fn query_bookings(artist_id: i32) -> SqliteResult<Vec<BookingRequest>> {
            let db_path = Path::new("tatteau.db");
            let conn = Connection::open(db_path)?;
            
            let mut stmt = conn.prepare("
                SELECT id, artist_id, client_name, client_email, client_phone,
                       requested_date, requested_start_time, requested_end_time,
                       tattoo_description, placement, size_inches, reference_images,
                       message_from_client, status, artist_response, estimated_price,
                       created_at, updated_at
                FROM booking_requests 
                WHERE artist_id = ?1 
                ORDER BY created_at DESC
            ")?;
            
            let booking_iter = stmt.query_map([artist_id], |row| {
                Ok(BookingRequest {
                    id: row.get(0)?,
                    artist_id: row.get(1)?,
                    client_name: row.get(2)?,
                    client_email: row.get(3)?,
                    client_phone: row.get(4)?,
                    requested_date: row.get(5)?,
                    requested_start_time: row.get(6)?,
                    requested_end_time: row.get(7)?,
                    tattoo_description: row.get(8)?,
                    placement: row.get(9)?,
                    size_inches: row.get(10)?,
                    reference_images: row.get(11)?,
                    message_from_client: row.get(12)?,
                    status: row.get(13)?,
                    artist_response: row.get(14)?,
                    estimated_price: row.get(15)?,
                    created_at: row.get(16)?,
                    updated_at: row.get(17)?,
                })
            })?;
            
            let mut bookings = Vec::new();
            for booking in booking_iter {
                bookings.push(booking?);
            }
            
            Ok(bookings)
        }

        match query_bookings(artist_id) {
            Ok(bookings) => Ok(bookings),
            Err(e) => Err(ServerFnError::new(format!("Failed to get booking requests: {}", e))),
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
}

#[server]
pub async fn respond_to_booking(response: BookingResponse) -> Result<(), ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use rusqlite::{Connection, Result as SqliteResult};
        use std::path::Path;

        fn update_booking(response: BookingResponse) -> SqliteResult<()> {
            let db_path = Path::new("tatteau.db");
            let conn = Connection::open(db_path)?;
            
            conn.execute("
                UPDATE booking_requests 
                SET status = ?1, artist_response = ?2, estimated_price = ?3, updated_at = CURRENT_TIMESTAMP
                WHERE id = ?4
            ", (
                response.status,
                response.artist_response,
                response.estimated_price,
                response.booking_id,
            ))?;
            
            Ok(())
        }

        match update_booking(response) {
            Ok(_) => Ok(()),
            Err(e) => Err(ServerFnError::new(format!("Failed to respond to booking: {}", e))),
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

#[server]
pub async fn send_booking_message(message_data: NewBookingMessage) -> Result<(), ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use rusqlite::{Connection, Result as SqliteResult};
        use std::path::Path;

        fn insert_message(message_data: NewBookingMessage) -> SqliteResult<()> {
            let db_path = Path::new("tatteau.db");
            let conn = Connection::open(db_path)?;
            
            conn.execute("
                INSERT INTO booking_messages (booking_request_id, sender_type, message)
                VALUES (?1, ?2, ?3)
            ", (
                message_data.booking_request_id,
                message_data.sender_type,
                message_data.message,
            ))?;
            
            Ok(())
        }

        match insert_message(message_data) {
            Ok(_) => Ok(()),
            Err(e) => Err(ServerFnError::new(format!("Failed to send message: {}", e))),
        }
    }
    #[cfg(not(feature = "ssr"))]
    {
        Ok(())
    }
}

#[server]
pub async fn get_booking_messages(booking_request_id: i32) -> Result<Vec<BookingMessage>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use rusqlite::{Connection, Result as SqliteResult};
        use std::path::Path;

        fn query_messages(booking_request_id: i32) -> SqliteResult<Vec<BookingMessage>> {
            let db_path = Path::new("tatteau.db");
            let conn = Connection::open(db_path)?;
            
            let mut stmt = conn.prepare("
                SELECT id, booking_request_id, sender_type, message, created_at
                FROM booking_messages 
                WHERE booking_request_id = ?1 
                ORDER BY created_at ASC
            ")?;
            
            let message_iter = stmt.query_map([booking_request_id], |row| {
                Ok(BookingMessage {
                    id: row.get(0)?,
                    booking_request_id: row.get(1)?,
                    sender_type: row.get(2)?,
                    message: row.get(3)?,
                    created_at: row.get(4)?,
                })
            })?;
            
            let mut messages = Vec::new();
            for message in message_iter {
                messages.push(message?);
            }
            
            Ok(messages)
        }

        match query_messages(booking_request_id) {
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

#[server]
pub async fn get_recurring_rules(artist_id: i32) -> Result<Vec<RecurringRule>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use rusqlite::{Connection, Result as SqliteResult};
        use std::path::Path;

        fn query_recurring_rules(artist_id: i32) -> SqliteResult<Vec<RecurringRule>> {
            let db_path = Path::new("tatteau.db");
            let conn = Connection::open(db_path)?;
            
            // Disable foreign key constraints
            conn.execute("PRAGMA foreign_keys = OFF;", [])?;
            
            // Create table if it doesn't exist
            conn.execute("
                CREATE TABLE IF NOT EXISTS recurring_rules (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    artist_id INTEGER NOT NULL,
                    name TEXT NOT NULL,
                    rule_type TEXT NOT NULL,
                    pattern TEXT NOT NULL,
                    action TEXT NOT NULL,
                    start_time TEXT,
                    end_time TEXT,
                    active INTEGER DEFAULT 1,
                    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
                )
            ", [])?;
            
            let mut stmt = conn.prepare("
                SELECT id, artist_id, name, rule_type, pattern, action, 
                       start_time, end_time, active, created_at
                FROM recurring_rules 
                WHERE artist_id = ?1 
                ORDER BY created_at DESC
            ")?;
            
            let rule_iter = stmt.query_map([artist_id], |row| {
                Ok(RecurringRule {
                    id: row.get(0)?,
                    artist_id: row.get(1)?,
                    name: row.get(2)?,
                    rule_type: row.get(3)?,
                    pattern: row.get(4)?,
                    action: row.get(5)?,
                    start_time: row.get(6)?,
                    end_time: row.get(7)?,
                    active: row.get(8)?,
                    created_at: row.get(9)?,
                })
            })?;
            
            let mut rules = Vec::new();
            for rule in rule_iter {
                rules.push(rule?);
            }
            
            Ok(rules)
        }

        match query_recurring_rules(artist_id) {
            Ok(rules) => Ok(rules),
            Err(e) => Err(ServerFnError::new(format!("Failed to get recurring rules: {}", e))),
        }
    }
    #[cfg(not(feature = "ssr"))]
    {
        Ok(vec![])
    }
}

#[server]
pub async fn create_recurring_rule(
    artist_id: i32,
    name: String,
    rule_type: String,
    pattern: String,
    action: String,
    start_time: Option<String>,
    end_time: Option<String>
) -> Result<i32, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use rusqlite::{Connection, Result as SqliteResult};
        use std::path::Path;

        fn insert_recurring_rule(
            artist_id: i32,
            name: String,
            rule_type: String,
            pattern: String,
            action: String,
            start_time: Option<String>,
            end_time: Option<String>
        ) -> SqliteResult<i32> {
            let db_path = Path::new("tatteau.db");
            let conn = Connection::open(db_path)?;
            
            // Disable foreign key constraints
            conn.execute("PRAGMA foreign_keys = OFF;", [])?;
            
            // Create recurring_rules table if it doesn't exist
            conn.execute("
                CREATE TABLE IF NOT EXISTS recurring_rules (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    artist_id INTEGER NOT NULL,
                    name TEXT NOT NULL,
                    rule_type TEXT NOT NULL,
                    pattern TEXT NOT NULL,
                    action TEXT NOT NULL,
                    start_time TEXT,
                    end_time TEXT,
                    active INTEGER DEFAULT 1,
                    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
                )
            ", [])?;
            
            let mut stmt = conn.prepare("
                INSERT INTO recurring_rules (artist_id, name, rule_type, pattern, action, start_time, end_time)
                VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
            ")?;
            
            stmt.execute([
                &artist_id.to_string(),
                &name,
                &rule_type,
                &pattern,
                &action,
                &start_time.unwrap_or_default(),
                &end_time.unwrap_or_default(),
            ])?;
            
            Ok(conn.last_insert_rowid() as i32)
        }

        match insert_recurring_rule(artist_id, name, rule_type, pattern.clone(), action, start_time, end_time) {
            Ok(id) => Ok(id),
            Err(e) => Err(ServerFnError::new(format!("Failed to create recurring rule - Error: {} - Pattern: {} - Artist ID: {}", e, pattern, artist_id))),
        }
    }
    #[cfg(not(feature = "ssr"))]
    {
        Ok(0)
    }
}

#[server]
pub async fn update_recurring_rule(
    id: i32,
    name: Option<String>,
    pattern: Option<String>,
    action: Option<String>,
    start_time: Option<String>,
    end_time: Option<String>,
    active: Option<bool>
) -> Result<(), ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use rusqlite::{Connection, Result as SqliteResult};
        use std::path::Path;

        fn update_rule(
            id: i32,
            name: Option<String>,
            pattern: Option<String>,
            action: Option<String>,
            start_time: Option<String>,
            end_time: Option<String>,
            active: Option<bool>
        ) -> SqliteResult<()> {
            let db_path = Path::new("tatteau.db");
            let conn = Connection::open(db_path)?;
            
            let mut updates = Vec::new();
            let mut params = Vec::new();
            
            if let Some(name) = &name {
                updates.push("name = ?");
                params.push(name.as_str());
            }
            if let Some(pattern) = &pattern {
                updates.push("pattern = ?");
                params.push(pattern.as_str());
            }
            if let Some(action) = &action {
                updates.push("action = ?");
                params.push(action.as_str());
            }
            if let Some(start_time) = &start_time {
                updates.push("start_time = ?");
                params.push(start_time.as_str());
            }
            if let Some(end_time) = &end_time {
                updates.push("end_time = ?");
                params.push(end_time.as_str());
            }
            if let Some(active) = active {
                updates.push("active = ?");
                params.push(if active { "1" } else { "0" });
            }
            
            if updates.is_empty() {
                return Ok(());
            }
            
            let sql = format!("UPDATE recurring_rules SET {} WHERE id = ?", updates.join(", "));
            let id_string = id.to_string();
            params.push(&id_string);
            
            conn.execute(&sql, rusqlite::params_from_iter(params))?;
            Ok(())
        }

        match update_rule(id, name, pattern, action, start_time, end_time, active) {
            Ok(_) => Ok(()),
            Err(e) => Err(ServerFnError::new(format!("Failed to update recurring rule: {}", e))),
        }
    }
    #[cfg(not(feature = "ssr"))]
    {
        Ok(())
    }
}

#[server]
pub async fn get_effective_availability(
    artist_id: i32,
    date: String
) -> Result<bool, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use rusqlite::{Connection, Result as SqliteResult};
        use std::path::Path;
        use chrono::{NaiveDate, Weekday, Datelike};

        fn check_effective_availability(artist_id: i32, date: String) -> SqliteResult<bool> {
            let db_path = Path::new("tatteau.db");
            let conn = Connection::open(db_path)?;
            
            // Create recurring_rules table if it doesn't exist
            conn.execute("
                CREATE TABLE IF NOT EXISTS recurring_rules (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    artist_id INTEGER NOT NULL,
                    name TEXT NOT NULL,
                    rule_type TEXT NOT NULL,
                    pattern TEXT NOT NULL,
                    action TEXT NOT NULL,
                    start_time TEXT,
                    end_time TEXT,
                    active INTEGER DEFAULT 1,
                    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
                )
            ", [])?;
            
            // First check if there's an explicit availability record for this date
            let mut stmt = conn.prepare("
                SELECT is_available FROM availability_slots 
                WHERE artist_id = ?1 AND specific_date = ?2
            ")?;
            
            let mut rows = stmt.query_map([&artist_id.to_string(), &date], |row| {
                Ok(row.get::<_, bool>(0)?)
            })?;
            
            if let Some(explicit_availability) = rows.next() {
                return explicit_availability;
            }
            
            // No explicit record, check recurring rules
            let parsed_date = match NaiveDate::parse_from_str(&date, "%Y-%m-%d") {
                Ok(d) => d,
                Err(_) => return Ok(true), // Default to available if date parsing fails
            };
            
            // Get all active recurring rules for this artist
            let mut rules_stmt = conn.prepare("
                SELECT rule_type, pattern, action FROM recurring_rules 
                WHERE artist_id = ?1 AND active = 1
            ")?;
            
            let rules = rules_stmt.query_map([&artist_id.to_string()], |row| {
                Ok((
                    row.get::<_, String>(0)?, // rule_type
                    row.get::<_, String>(1)?, // pattern
                    row.get::<_, String>(2)?, // action
                ))
            })?;
            
            for rule_result in rules {
                let (rule_type, pattern, action) = rule_result?;
                
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
                    },
                    "dates" => {
                        let month_day = parsed_date.format("%B %e").to_string().trim().to_string();
                        pattern.contains(&month_day)
                    },
                    "monthly" => {
                        // Simple pattern matching for common monthly patterns
                        if pattern.contains("1st") && parsed_date.day() <= 7 {
                            let first_weekday_of_month = parsed_date.with_day(1).unwrap().weekday();
                            let target_day = parsed_date.weekday();
                            (parsed_date.day() - 1) / 7 == 0 && first_weekday_of_month == target_day
                        } else {
                            false // More complex patterns would need more parsing
                        }
                    },
                    _ => false
                };
                
                if matches {
                    return Ok(action == "available");
                }
            }
            
            // Default to available if no rules match
            Ok(true)
        }

        match check_effective_availability(artist_id, date) {
            Ok(is_available) => Ok(is_available),
            Err(e) => Err(ServerFnError::new(format!("Failed to check effective availability: {}", e))),
        }
    }
    #[cfg(not(feature = "ssr"))]
    {
        Ok(true)
    }
}

#[server]
pub async fn delete_recurring_rule(rule_id: i32) -> Result<(), ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use rusqlite::{Connection, Result as SqliteResult};
        use std::path::Path;

        fn delete_rule(rule_id: i32) -> SqliteResult<()> {
            let db_path = Path::new("tatteau.db");
            let conn = Connection::open(db_path)?;
            
            conn.execute("DELETE FROM recurring_rules WHERE id = ?1", [rule_id])?;
            Ok(())
        }

        match delete_rule(rule_id) {
            Ok(_) => Ok(()),
            Err(e) => Err(ServerFnError::new(format!("Failed to delete recurring rule: {}", e))),
        }
    }
    #[cfg(not(feature = "ssr"))]
    {
        Ok(())
    }
}

#[server]
pub async fn get_booking_request_by_id(booking_id: i32) -> Result<BookingRequest, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use rusqlite::{Connection, Result as SqliteResult};
        use std::path::Path;
        
        fn query_booking_by_id(booking_id: i32) -> SqliteResult<BookingRequest> {
            let db_path = Path::new("tatteau.db");
            let conn = Connection::open(db_path)?;
            
            let mut stmt = conn.prepare("
                SELECT id, artist_id, client_name, client_email, client_phone,
                       requested_date, requested_start_time, requested_end_time,
                       tattoo_description, placement, size_inches, reference_images,
                       message_from_client, status, artist_response, estimated_price,
                       created_at, updated_at
                FROM booking_requests 
                WHERE id = ?1
            ")?;
            
            stmt.query_row([booking_id], |row| {
                Ok(BookingRequest {
                    id: row.get(0)?,
                    artist_id: row.get(1)?,
                    client_name: row.get(2)?,
                    client_email: row.get(3)?,
                    client_phone: row.get(4)?,
                    requested_date: row.get(5)?,
                    requested_start_time: row.get(6)?,
                    requested_end_time: row.get(7)?,
                    tattoo_description: row.get(8)?,
                    placement: row.get(9)?,
                    size_inches: row.get(10)?,
                    reference_images: row.get(11)?,
                    message_from_client: row.get(12)?,
                    status: row.get(13)?,
                    artist_response: row.get(14)?,
                    estimated_price: row.get(15)?,
                    created_at: row.get(16)?,
                    updated_at: row.get(17)?,
                })
            })
        }
        
        match query_booking_by_id(booking_id) {
            Ok(booking) => Ok(booking),
            Err(e) => Err(ServerFnError::new(format!("Failed to get booking: {}", e))),
        }
    }
    #[cfg(not(feature = "ssr"))]
    {
        Err(ServerFnError::new("Not available on client".to_string()))
    }
}
