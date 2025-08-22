use leptos::leptos_dom::logging::console_log;
use leptos::prelude::*;
use leptos::server;
use leptos_leaflet::leaflet::LatLngBounds;
use shared_types::LocationInfo;
use shared_types::MapBounds;

use crate::db::entities::{Artist, ArtistImage, CityCoords, Location, Style};
#[cfg(feature = "ssr")]
use crate::db::repository::{
    get_artist_by_id, get_artist_images_with_styles, get_artist_location, get_artist_styles,
    get_cities_and_coords, get_city_coordinates, get_states, query_locations,
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
