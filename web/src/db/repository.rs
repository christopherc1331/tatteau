use super::entities::CityCoords;
use leptos::prelude::*;
use leptos::server;
#[cfg(feature = "ssr")]
use rusqlite::{Connection, Result as SqliteResult};
use serde::Deserialize;
use serde::Serialize;
use shared_types::LocationInfo;
use std::path::Path;

#[cfg(feature = "ssr")]
pub fn get_cities_and_coords(state: String) -> SqliteResult<Vec<CityCoords>> {
    use rusqlite::params;

    let db_path = Path::new("tatteau.db");

    // Open a connection to the database
    let conn = Connection::open(db_path)?;

    let mut stmt = conn.prepare(
        "
            SELECT
                city,
                state,
                lat,
                long
            FROM locations
            WHERE
                state = ?1
            AND (is_person IS NULL OR is_person != 1)
            GROUP BY city
        ",
    )?;

    let city_coords = stmt.query_map(params![state], |row| {
        Ok(CityCoords {
            city: row.get(0)?,
            state: row.get(1)?,
            lat: row.get(2)?,
            long: row.get(3)?,
        })
    })?;

    city_coords.into_iter().collect()
}

#[cfg(feature = "ssr")]
pub fn query_locations(city: String) -> SqliteResult<Vec<LocationInfo>> {
    use rusqlite::params;
    let db_path = Path::new("tatteau.db");

    // Open a connection to the database
    let conn = Connection::open(db_path)?;

    // Prepare and execute the query
    let mut stmt = conn.prepare(
        "
        SELECT 
            id,
            name,
            lat,
            long,
            city,
            county,
            state,
            country_code,
            postal_code,
            is_open,
            address,
            category,
            website_uri
        FROM locations
        WHERE 
            city = ?1
        AND (is_person IS NULL OR is_person != 1)
    ",
    )?;

    // Map the results to LocationInfo structs
    let locations = stmt.query_map(params![city], |row| {
        Ok(LocationInfo {
            id: row.get(0)?,
            name: row.get(1)?,
            lat: row.get(2)?,
            long: row.get(3)?,
            city: row.get(4)?,
            county: row.get(5)?,
            state: row.get(6)?,
            country_code: row.get(7)?,
            postal_code: row.get(8)?,
            is_open: row.get(9)?,
            address: row.get(10)?,
            category: row.get(11)?,
            website_uri: row.get(12)?,
        })
    })?;

    // Collect all results into a vector
    let mut result = Vec::new();
    for location in locations {
        result.push(location?);
    }

    Ok(result)
}

#[cfg(feature = "ssr")]
pub fn get_states() -> SqliteResult<Vec<String>> {
    let db_path = Path::new("tatteau.db");

    // Open a connection to the database
    let conn = Connection::open(db_path)?;

    // Prepare and execute the query
    let mut stmt = conn.prepare(
        "
        SELECT DISTINCT state
        FROM locations
    ",
    )?;

    let mut result = Vec::new();
    // Map the results to LocationInfo structs
    let states = stmt
        .query_map([], |row| {
            Ok(row.get(0).expect("Row should have a single column"))
        })
        .expect("Should fetch states successfully");

    for state in states {
        result.push(state?);
    }

    Ok(result)
}

#[cfg(feature = "ssr")]
pub fn get_city_coordinates(city_name: String) -> SqliteResult<CityCoords> {
    use rusqlite::params;

    let db_path = Path::new("tatteau.db");

    // Open a connection to the database
    let conn = Connection::open(db_path)?;

    // Prepare and execute the query
    let mut stmt = conn.prepare(
        "
            SELECT 
                state_name,
                city, 
                latitude, 
                longitude
            FROM cities
            WHERE LOWER(REPLACE(city, \"'\", \" \")) LIKE ?1
        ",
    )?;
    let mapped_rows = stmt
        .query_map(params![format!("%{}%", city_name)], |row| {
            Ok(CityCoords {
                city: row.get(1)?,
                state: row.get(0)?,
                lat: row.get(2)?,
                long: row.get(3)?,
            })
        })
        .expect("Should fetch city coordinates successfully");
    let city_coords: SqliteResult<Vec<CityCoords>> = mapped_rows.into_iter().collect();

    if let Ok(cities) = city_coords {
        if let Some(city) = cities.first() {
            Ok(city.clone())
        } else {
            Err(rusqlite::Error::QueryReturnedNoRows)
        }
    } else {
        Err(rusqlite::Error::QueryReturnedNoRows)
    }
}
