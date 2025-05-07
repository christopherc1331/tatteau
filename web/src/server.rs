use leptos::prelude::*;
use leptos::server;
use shared_types::LocationInfo;

use crate::db::entities::CityCoords;
#[cfg(feature = "ssr")]
use crate::db::repository::{
    get_cities_and_coords, get_city_coordinates, get_states, query_locations,
};

#[server]
pub async fn fetch_locations(city: String) -> Result<Vec<LocationInfo>, ServerFnError> {
    // This function will be executed on the server
    match query_locations(city) {
        Ok(locations) => Ok(locations),
        Err(e) => Err(ServerFnError::new(format!("Database error: {}", e))),
    }
}

#[server]
pub async fn get_cities(state: String) -> Result<Vec<CityCoords>, ServerFnError> {
    // This function will be executed on the server
    match get_cities_and_coords(state) {
        Ok(cities) => Ok(cities),
        Err(e) => Err(ServerFnError::new(format!("Database error: {}", e))),
    }
}

#[server]
pub async fn get_states_list() -> Result<Vec<String>, ServerFnError> {
    match get_states() {
        Ok(states) => Ok(states),
        Err(e) => Err(ServerFnError::new(format!("Database error: {}", e))),
    }
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
