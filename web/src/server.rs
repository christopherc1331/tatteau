use leptos::prelude::*;
use leptos::server;
#[cfg(feature = "ssr")]
use rusqlite::{Connection, Result as SqliteResult};
use shared_types::LocationInfo;
use std::path::Path;

#[server]
pub async fn fetch_locations(city: String) -> Result<Vec<LocationInfo>, ServerFnError> {
    // This function will be executed on the server
    match query_locations(city) {
        Ok(locations) => Ok(locations),
        Err(e) => Err(ServerFnError::new(format!("Database error: {}", e))),
    }
}

#[cfg(feature = "ssr")]
fn query_locations(city: String) -> SqliteResult<Vec<LocationInfo>> {
    // Path to your SQLite database file

    use rusqlite::params;
    let db_path = Path::new("web/tatteau.db");

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
        WHERE LOWER(city) LIKE ?1
    ",
    )?;

    // Map the results to LocationInfo structs
    let locations = stmt.query_map(params![format!("%{}%", city)], |row| {
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
