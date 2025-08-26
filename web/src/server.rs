use leptos::leptos_dom::logging::console_log;
use leptos::prelude::*;
use leptos::server;
use leptos_leaflet::leaflet::LatLngBounds;
use shared_types::LocationInfo;
use shared_types::MapBounds;

use crate::db::entities::{Artist, ArtistImage, CityCoords, Location, Style};
use crate::db::search_repository::{SearchResult, SearchResultType};
use serde::{Deserialize, Serialize};

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

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct StyleWithCount {
    pub id: i32,
    pub name: String,
    pub artist_count: i32,
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
