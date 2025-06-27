use super::entities::CityCoords;
#[cfg(feature = "ssr")]
use rusqlite::{Connection, Result as SqliteResult};
use shared_types::{LocationInfo, MapBounds};
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

    city_coords
        .into_iter()
        .filter(|c| c.as_ref().unwrap().city.parse::<f64>().is_err())
        .collect()
}

#[cfg(feature = "ssr")]
pub fn query_locations(
    state: String,
    city: String,
    bounds: MapBounds,
) -> SqliteResult<Vec<LocationInfo>> {
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
            lat BETWEEN ?1 AND ?2
            AND long BETWEEN ?3 AND ?4
        AND (is_person IS NULL OR is_person != 1)
    ",
    )?;

    // Map the results to LocationInfo structs
    let locations = stmt.query_map(
        params![
            bounds.south_west.lat,
            bounds.north_east.lat,
            bounds.south_west.long,
            bounds.north_east.long
        ],
        |row| {
            Ok(LocationInfo {
                _id: row.get(0)?,
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
        },
    )?;

    // Collect all results into a vector
    let mut result = Vec::new();
    for location in locations {
        result.push(location?);
    }

    Ok(result)
}

pub struct LocationState {
    pub state: String,
}
#[cfg(feature = "ssr")]
pub fn get_states() -> SqliteResult<Vec<LocationState>> {
    use std::error::Error;

    use rusqlite::MappedRows;

    let db_path = Path::new("tatteau.db");
    let conn = Connection::open(db_path)?;
    let mut stmt = conn.prepare(
        "
        SELECT DISTINCT state
        FROM locations
        WHERE country_code = 'United States'
        ORDER BY state ASC
    ",
    )?;

    // let mut result = Vec::new();

    let res = stmt.query_map([], |r| Ok(LocationState { state: r.get(0)? }))?;

    res.into_iter().collect()

    // println!("row length: {:?}", rows.size_hint());
    // let mut i = 1;
    // while let Some(row) = rows.next()? {
    //     // println!("Processing row: {}", i);
    //     // i += 1;
    //     let s: String = row.get(0)?;
    //     if s.is_empty() {
    //         continue;
    //     }
    //     result.push(s);
    // }
    // println!("row length: {:?}", result.len());
    // println!("rows: {:?}", result);
    // Ok(result)

    // Ok(vec![
    //     "Texas".to_string(),
    //     "California".to_string(),
    //     "New York".to_string(),
    // ])
    // Ok(vec![
    //     "Wyoming".to_string(),
    //     "Wisconsin".to_string(),
    //     "West Virginia".to_string(),
    //     "Washington".to_string(),
    //     "Virginia".to_string(),
    //     "Vermont".to_string(),
    //     "Utah".to_string(),
    //     "Texas".to_string(),
    //     "Tennessee".to_string(),
    //     "South Dakota".to_string(),
    //     "South Carolina".to_string(),
    //     "Rhode Island".to_string(),
    //     "Pennsylvania".to_string(),
    //     "Oregon".to_string(),
    //     "Oklahoma".to_string(),
    //     "Ohio".to_string(),
    //     "North Dakota".to_string(),
    //     "North Carolina".to_string(),
    //     "New York".to_string(),
    //     "New Mexico".to_string(),
    //     "New Jersey".to_string(),
    //     "New Hampshire".to_string(),
    //     "Nevada".to_string(),
    //     "Nebraska".to_string(),
    //     "Montana".to_string(),
    //     "Missouri".to_string(),
    //     "Mississippi".to_string(),
    //     "Minnesota".to_string(),
    //     "Michigan".to_string(),
    //     "Massachusetts".to_string(),
    //     "Maryland".to_string(),
    //     "Maine".to_string(),
    //     "MI".to_string(),
    //     "Louisiana".to_string(),
    //     "Kentucky".to_string(),
    //     "Kansas".to_string(),
    //     "Iowa".to_string(),
    //     "Indiana".to_string(),
    //     "Illinois".to_string(),
    //     "Idaho".to_string(),
    //     "IL".to_string(),
    //     "Hawaii".to_string(),
    //     "Georgia".to_string(),
    //     "Florida".to_string(),
    //     "District of Columbia".to_string(),
    //     "Delaware".to_string(),
    //     "Connecticut".to_string(),
    //     "Colorado".to_string(),
    //     "California".to_string(),
    //     "Arkansas".to_string(),
    //     "Arizona".to_string(),
    //     "Alaska".to_string(),
    //     "Alabama".to_string(),
    // ])
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
