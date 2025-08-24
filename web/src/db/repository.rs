use super::entities::{Artist, ArtistImage, ArtistImageStyle, ArtistStyle, CityCoords, Location, Style};
#[cfg(feature = "ssr")]
use rusqlite::{Connection, Result as SqliteResult};
use shared_types::{LocationInfo, MapBounds};
use std::path::Path;
#[cfg(feature = "ssr")]
use serde_json;

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
            l.id,
            l.name,
            l.lat,
            l.long,
            l.city,
            l.county,
            l.state,
            l.country_code,
            l.postal_code,
            l.is_open,
            l.address,
            l.category,
            l.website_uri,
            l._id,
            CASE WHEN COUNT(DISTINCT a.id) > 0 THEN 1 ELSE 0 END as has_artists,
            COUNT(DISTINCT ai.id) as artist_images_count
        FROM locations l
        LEFT JOIN artists a ON l.id = a.location_id
        LEFT JOIN artists_images ai ON a.id = ai.artist_id
        WHERE 
            l.lat BETWEEN ?1 AND ?2
            AND l.long BETWEEN ?3 AND ?4
            AND (l.is_person IS NULL OR l.is_person != 1)
        GROUP BY l.id, l.name, l.lat, l.long, l.city, l.county, l.state, l.country_code, l.postal_code, l.is_open, l.address, l.category, l.website_uri, l._id
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
                has_artists: Some(row.get::<_, i32>(14)? == 1),
                artist_images_count: Some(row.get(15)?),
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
        "SELECT id, name, lat, long, city, state, address, website_uri
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
            website_uri: row.get(7)?,
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
        "SELECT id, name, lat, long, city, state, address, website_uri
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
            website_uri: row.get(7)?,
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

#[cfg(feature = "ssr")]
pub fn save_quiz_session(
    style_preference: String,
    body_placement: String,
    pain_tolerance: i32,
    budget_min: f64,
    budget_max: f64,
    vibe_preference: String,
) -> SqliteResult<i64> {
    use rusqlite::params;
    let db_path = Path::new("tatteau.db");
    let conn = Connection::open(db_path)?;

    conn.execute(
        "INSERT INTO client_quiz_sessions (style_preference, body_placement, pain_tolerance, budget_min, budget_max, vibe_preference)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![style_preference, body_placement, pain_tolerance, budget_min, budget_max, vibe_preference],
    )?;

    Ok(conn.last_insert_rowid())
}

#[cfg(feature = "ssr")]
pub fn get_location_stats_for_city(city: String, state: String) -> SqliteResult<crate::server::LocationStats> {
    use rusqlite::params;
    let db_path = Path::new("tatteau.db");
    let conn = Connection::open(db_path)?;
    
    let mut stmt = conn.prepare(
        "SELECT 
            COUNT(DISTINCT l.id) as shop_count,
            COUNT(DISTINCT a.id) as artist_count,
            COUNT(DISTINCT s.id) as styles_available
         FROM locations l
         LEFT JOIN artists a ON l.id = a.location_id
         LEFT JOIN artists_styles ast ON a.id = ast.artist_id
         LEFT JOIN styles s ON ast.style_id = s.id
         WHERE l.city = ?1 AND l.state = ?2
         AND (l.is_person IS NULL OR l.is_person != 1)"
    )?;
    
    stmt.query_row(params![city, state], |row| {
        Ok(crate::server::LocationStats {
            shop_count: row.get(0)?,
            artist_count: row.get(1)?,
            styles_available: row.get(2)?,
        })
    })
}

#[cfg(feature = "ssr")]
pub fn get_all_styles_with_counts() -> SqliteResult<Vec<crate::server::StyleWithCount>> {
    let db_path = Path::new("tatteau.db");
    let conn = Connection::open(db_path)?;
    
    let mut stmt = conn.prepare(
        "SELECT 
            s.id,
            s.name,
            COUNT(DISTINCT ast.artist_id) as artist_count
         FROM styles s
         LEFT JOIN artists_styles ast ON s.id = ast.style_id
         GROUP BY s.id, s.name
         ORDER BY artist_count DESC, s.name ASC"
    )?;
    
    let styles = stmt.query_map([], |row| {
        Ok(crate::server::StyleWithCount {
            id: row.get(0)?,
            name: row.get(1)?,
            artist_count: row.get(2)?,
        })
    })?;
    
    styles.collect()
}

#[cfg(feature = "ssr")]
fn map_location_row(row: &rusqlite::Row) -> rusqlite::Result<(shared_types::LocationInfo, i32)> {
    use shared_types::LocationInfo;
    let location_id: i64 = row.get(0)?;
    let location_info = LocationInfo {
        id: location_id,
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
        has_artists: Some(row.get::<_, i32>(15)? == 1),
        artist_images_count: Some(row.get(16)?),
    };
    Ok((location_info, row.get::<_, i32>(14)?))
}

#[cfg(feature = "ssr")]
pub fn query_locations_with_details(
    state: String,
    city: String,
    bounds: MapBounds,
    style_filter: Option<Vec<i32>>,
) -> SqliteResult<Vec<crate::server::EnhancedLocationInfo>> {
    use rusqlite::params;
    let db_path = Path::new("tatteau.db");
    let conn = Connection::open(db_path)?;
    
    // Build the query based on whether we have style filters
    let query = if style_filter.is_some() && !style_filter.as_ref().unwrap().is_empty() {
        "SELECT DISTINCT
            l.id, l.name, l.lat, l.long, l.city, l.county, l.state, 
            l.country_code, l.postal_code, l.is_open, l.address, 
            l.category, l.website_uri, l._id,
            COUNT(DISTINCT a.id) as artist_count,
            CASE WHEN COUNT(DISTINCT ai.id) > 0 THEN 1 ELSE 0 END as has_artists,
            COUNT(DISTINCT ai.id) as artist_images_count
         FROM locations l
         LEFT JOIN artists a ON l.id = a.location_id
         LEFT JOIN artists_images ai ON a.id = ai.artist_id
         LEFT JOIN artists_styles ast ON a.id = ast.artist_id
         WHERE l.lat BETWEEN ?1 AND ?2
         AND l.long BETWEEN ?3 AND ?4
         AND (l.is_person IS NULL OR l.is_person != 1)
         AND ast.style_id IN (SELECT value FROM json_each(?5))
         GROUP BY l.id"
    } else {
        "SELECT 
            l.id, l.name, l.lat, l.long, l.city, l.county, l.state, 
            l.country_code, l.postal_code, l.is_open, l.address, 
            l.category, l.website_uri, l._id,
            COUNT(DISTINCT a.id) as artist_count,
            CASE WHEN COUNT(DISTINCT ai.id) > 0 THEN 1 ELSE 0 END as has_artists,
            COUNT(DISTINCT ai.id) as artist_images_count
         FROM locations l
         LEFT JOIN artists a ON l.id = a.location_id
         LEFT JOIN artists_images ai ON a.id = ai.artist_id
         WHERE l.lat BETWEEN ?1 AND ?2
         AND l.long BETWEEN ?3 AND ?4
         AND (l.is_person IS NULL OR l.is_person != 1)
         GROUP BY l.id"
    };
    
    let mut stmt = conn.prepare(query)?;
    
    let locations = if let Some(ref styles) = style_filter {
        if !styles.is_empty() {
            let style_json = serde_json::to_string(&styles).unwrap();
            stmt.query_map(
                params![
                    bounds.south_west.lat,
                    bounds.north_east.lat,
                    bounds.south_west.long,
                    bounds.north_east.long,
                    style_json
                ],
                map_location_row,
            )?
        } else {
            stmt.query_map(
                params![
                    bounds.south_west.lat,
                    bounds.north_east.lat,
                    bounds.south_west.long,
                    bounds.north_east.long
                ],
                map_location_row,
            )?
        }
    } else {
        stmt.query_map(
            params![
                bounds.south_west.lat,
                bounds.north_east.lat,
                bounds.south_west.long,
                bounds.north_east.long
            ],
            map_location_row,
        )?
    };
    
    let mut result = Vec::new();
    for location_result in locations {
        let (location_info, artist_count) = location_result?;
        
        // Get image count and styles for this location
        let mut image_count_stmt = conn.prepare(
            "SELECT COUNT(DISTINCT ai.id)
             FROM artists_images ai
             JOIN artists a ON ai.artist_id = a.id
             WHERE a.location_id = ?1"
        )?;
        
        let image_count: i32 = image_count_stmt
            .query_row(params![location_info.id], |row| row.get(0))?;
        
        let mut styles_stmt = conn.prepare(
            "SELECT DISTINCT s.name
             FROM styles s
             JOIN artists_styles ast ON s.id = ast.style_id
             JOIN artists a ON ast.artist_id = a.id
             WHERE a.location_id = ?1
             LIMIT 5"
        )?;
        
        let styles: Vec<String> = styles_stmt
            .query_map(params![location_info.id], |row| row.get(0))?
            .filter_map(Result::ok)
            .collect();
        
        result.push(crate::server::EnhancedLocationInfo {
            location: location_info,
            artist_count,
            image_count,
            styles,
            min_price: None, // TODO: Add when artist_pricing table exists
            max_price: None, // TODO: Add when artist_pricing table exists
        });
    }
    
    Ok(result)
}

#[cfg(feature = "ssr")]
pub fn get_coords_by_postal_code(postal_code: String) -> SqliteResult<CityCoords> {
    use rusqlite::params;
    let db_path = Path::new("tatteau.db");
    let conn = Connection::open(db_path)?;
    
    let mut stmt = conn.prepare(
        "SELECT DISTINCT city, state, lat, long
         FROM locations
         WHERE postal_code = ?1
         AND (is_person IS NULL OR is_person != 1)
         LIMIT 1"
    )?;
    
    stmt.query_row(params![postal_code], |row| {
        Ok(CityCoords {
            city: row.get(0)?,
            state: row.get(1)?,
            lat: row.get(2)?,
            long: row.get(3)?,
        })
    })
}
