use super::entities::{Artist, ArtistImage, ArtistImageStyle, ArtistStyle, CityCoords, Location, Style};
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
            website_uri,
            _id
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
                _id: row.get(13)?,
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
}

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

#[cfg(feature = "ssr")]
pub fn get_artist_by_id(artist_id: i32) -> SqliteResult<Artist> {
    use rusqlite::params;
    
    let db_path = Path::new("tatteau.db");
    let conn = Connection::open(db_path)?;
    
    let mut stmt = conn.prepare(
        "SELECT id, name, location_id, social_links, email, phone, years_experience, styles_extracted
         FROM artists
         WHERE id = ?1"
    )?;
    
    stmt.query_row(params![artist_id], |row| {
        Ok(Artist {
            id: row.get(0)?,
            name: row.get(1)?,
            location_id: row.get(2)?,
            social_links: row.get(3)?,
            email: row.get(4)?,
            phone: row.get(5)?,
            years_experience: row.get(6)?,
            styles_extracted: row.get(7)?,
        })
    })
}

#[cfg(feature = "ssr")]
pub fn get_artist_location(location_id: i32) -> SqliteResult<Location> {
    use rusqlite::params;
    
    let db_path = Path::new("tatteau.db");
    let conn = Connection::open(db_path)?;
    
    let mut stmt = conn.prepare(
        "SELECT id, name, lat, long, city, state, address
         FROM locations
         WHERE id = ?1"
    )?;
    
    stmt.query_row(params![location_id], |row| {
        Ok(Location {
            id: row.get(0)?,
            name: row.get(1)?,
            lat: row.get(2)?,
            long: row.get(3)?,
            city: row.get(4)?,
            state: row.get(5)?,
            address: row.get(6)?,
        })
    })
}

#[cfg(feature = "ssr")]
pub fn get_artist_styles(artist_id: i32) -> SqliteResult<Vec<Style>> {
    use rusqlite::params;
    
    let db_path = Path::new("tatteau.db");
    let conn = Connection::open(db_path)?;
    
    let mut stmt = conn.prepare(
        "SELECT s.id, s.name
         FROM styles s
         JOIN artists_styles ast ON s.id = ast.style_id
         WHERE ast.artist_id = ?1"
    )?;
    
    let styles = stmt.query_map(params![artist_id], |row| {
        Ok(Style {
            id: row.get(0)?,
            name: row.get(1)?,
        })
    })?;
    
    styles.collect()
}

#[cfg(feature = "ssr")]
pub fn get_artist_images_with_styles(artist_id: i32) -> SqliteResult<Vec<(ArtistImage, Vec<Style>)>> {
    use rusqlite::params;
    
    let db_path = Path::new("tatteau.db");
    let conn = Connection::open(db_path)?;
    
    // First get all images for the artist
    let mut images_stmt = conn.prepare(
        "SELECT id, short_code, artist_id
         FROM artists_images
         WHERE artist_id = ?1"
    )?;
    
    let images = images_stmt.query_map(params![artist_id], |row| {
        Ok(ArtistImage {
            id: row.get(0)?,
            short_code: row.get(1)?,
            artist_id: row.get(2)?,
        })
    })?;
    
    let mut result = Vec::new();
    
    // For each image, get its styles
    for image in images {
        let img = image?;
        let img_id = img.id;
        
        let mut styles_stmt = conn.prepare(
            "SELECT s.id, s.name
             FROM styles s
             JOIN artists_images_styles ais ON s.id = ais.style_id
             WHERE ais.artists_images_id = ?1"
        )?;
        
        let styles = styles_stmt.query_map(params![img_id], |row| {
            Ok(Style {
                id: row.get(0)?,
                name: row.get(1)?,
            })
        })?;
        
        let styles_vec: SqliteResult<Vec<Style>> = styles.collect();
        result.push((img, styles_vec?));
    }
    
    Ok(result)
}

#[cfg(feature = "ssr")]
pub fn get_location_by_id(location_id: i32) -> SqliteResult<Location> {
    use rusqlite::params;
    
    let db_path = Path::new("tatteau.db");
    let conn = Connection::open(db_path)?;
    
    let mut stmt = conn.prepare(
        "SELECT id, name, lat, long, city, state, address
         FROM locations
         WHERE id = ?1"
    )?;
    
    stmt.query_row(params![location_id], |row| {
        Ok(Location {
            id: row.get(0)?,
            name: row.get(1)?,
            lat: row.get(2)?,
            long: row.get(3)?,
            city: row.get(4)?,
            state: row.get(5)?,
            address: row.get(6)?,
        })
    })
}

#[cfg(feature = "ssr")]
pub fn get_artists_by_location(location_id: i32) -> SqliteResult<Vec<Artist>> {
    use rusqlite::params;
    
    let db_path = Path::new("tatteau.db");
    let conn = Connection::open(db_path)?;
    
    // Updated query to join with locations and ensure we're getting artists for actual shops, not persons
    let mut stmt = conn.prepare(
        "SELECT a.id, a.name, a.location_id, a.social_links, a.email, a.phone, a.years_experience, a.styles_extracted
         FROM artists a
         JOIN locations l ON a.location_id = l.id
         WHERE a.location_id = ?1 
         AND (l.is_person IS NULL OR l.is_person != 1)
         AND a.name IS NOT NULL
         AND a.name != ''"
    )?;
    
    let artists = stmt.query_map(params![location_id], |row| {
        Ok(Artist {
            id: row.get(0)?,
            name: row.get(1)?,
            location_id: row.get(2)?,
            social_links: row.get(3)?,
            email: row.get(4)?,
            phone: row.get(5)?,
            years_experience: row.get(6)?,
            styles_extracted: row.get(7)?,
        })
    })?;
    
    artists.collect()
}

#[cfg(feature = "ssr")]
pub fn get_all_styles_by_location(location_id: i32) -> SqliteResult<Vec<Style>> {
    use rusqlite::params;
    
    let db_path = Path::new("tatteau.db");
    let conn = Connection::open(db_path)?;
    
    let mut stmt = conn.prepare(
        "SELECT DISTINCT s.id, s.name
         FROM styles s
         JOIN artists_styles ast ON s.id = ast.style_id
         JOIN artists a ON ast.artist_id = a.id
         JOIN locations l ON a.location_id = l.id
         WHERE a.location_id = ?1
         AND (l.is_person IS NULL OR l.is_person != 1)
         AND a.name IS NOT NULL
         AND a.name != ''
         ORDER BY s.name"
    )?;
    
    let styles = stmt.query_map(params![location_id], |row| {
        Ok(Style {
            id: row.get(0)?,
            name: row.get(1)?,
        })
    })?;
    
    styles.collect()
}

#[cfg(feature = "ssr")]
pub fn get_all_images_with_styles_by_location(location_id: i32) -> SqliteResult<Vec<(ArtistImage, Vec<Style>, Artist)>> {
    use rusqlite::params;
    
    let db_path = Path::new("tatteau.db");
    let conn = Connection::open(db_path)?;
    
    // First get all images for artists at this location, filtering out person locations
    let mut stmt = conn.prepare(
        "SELECT ai.id, ai.short_code, ai.artist_id, a.id, a.name, a.location_id, a.social_links, a.email, a.phone, a.years_experience, a.styles_extracted
         FROM artists_images ai
         JOIN artists a ON ai.artist_id = a.id
         JOIN locations l ON a.location_id = l.id
         WHERE a.location_id = ?1
         AND (l.is_person IS NULL OR l.is_person != 1)
         AND a.name IS NOT NULL
         AND a.name != ''"
    )?;
    
    let images = stmt.query_map(params![location_id], |row| {
        let image = ArtistImage {
            id: row.get(0)?,
            short_code: row.get(1)?,
            artist_id: row.get(2)?,
        };
        
        let artist = Artist {
            id: row.get(3)?,
            name: row.get(4)?,
            location_id: row.get(5)?,
            social_links: row.get(6)?,
            email: row.get(7)?,
            phone: row.get(8)?,
            years_experience: row.get(9)?,
            styles_extracted: row.get(10)?,
        };
        
        Ok((image, artist))
    })?;
    
    let mut result = Vec::new();
    
    // For each image, get its styles
    for item in images {
        let (image, artist) = item?;
        let img_id = image.id;
        
        let mut styles_stmt = conn.prepare(
            "SELECT s.id, s.name
             FROM styles s
             JOIN artists_images_styles ais ON s.id = ais.style_id
             WHERE ais.artists_images_id = ?1"
        )?;
        
        let styles = styles_stmt.query_map(params![img_id], |row| {
            Ok(Style {
                id: row.get(0)?,
                name: row.get(1)?,
            })
        })?;
        
        let styles_vec: SqliteResult<Vec<Style>> = styles.collect();
        result.push((image, styles_vec?, artist));
    }
    
    Ok(result)
}
