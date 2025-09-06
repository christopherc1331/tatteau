use super::entities::{
    Artist, ArtistImage, Style, CityCoords, Location,
    QuestionnaireQuestion, ArtistQuestionnaire, BookingQuestionnaireResponse,
    ClientQuestionnaireForm, ClientQuestionnaireQuestion,
    ErrorLog, CreateErrorLog,
};
#[cfg(feature = "ssr")]
use rusqlite::{Connection, Result as SqliteResult};
#[cfg(feature = "ssr")]
use serde_json;
use shared_types::{LocationInfo, MapBounds};
#[cfg(feature = "ssr")]
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
         WHERE id = ?1",
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
         WHERE ast.artist_id = ?1",
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
pub fn get_artist_images_with_styles(
    artist_id: i32,
) -> SqliteResult<Vec<(ArtistImage, Vec<Style>)>> {
    use rusqlite::params;

    let db_path = Path::new("tatteau.db");
    let conn = Connection::open(db_path)?;

    // First get all images for the artist
    let mut images_stmt = conn.prepare(
        "SELECT id, short_code, artist_id
         FROM artists_images
         WHERE artist_id = ?1",
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
             WHERE ais.artists_images_id = ?1",
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
         WHERE id = ?1",
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
         ORDER BY s.name",
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
pub fn get_all_images_with_styles_by_location(
    location_id: i32,
) -> SqliteResult<Vec<(ArtistImage, Vec<Style>, Artist)>> {
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
             WHERE ais.artists_images_id = ?1",
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
pub fn get_location_stats_for_city(
    city: String,
    state: String,
) -> SqliteResult<crate::server::LocationStats> {
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
         AND (l.is_person IS NULL OR l.is_person != 1)",
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
         ORDER BY artist_count DESC, s.name ASC",
    )?;

    let styles = stmt.query_map([], |row| {
        Ok(crate::server::StyleWithCount {
            id: row.get(0)?,
            name: row.get(1)?,
            description: None,
            artist_count: row.get(2)?,
            sample_images: None,
        })
    })?;

    styles.collect()
}

#[cfg(feature = "ssr")]
pub fn get_styles_with_counts_in_bounds(
    bounds: shared_types::MapBounds,
) -> SqliteResult<Vec<crate::server::StyleWithCount>> {
    let db_path = Path::new("tatteau.db");
    let conn = Connection::open(db_path)?;

    let mut stmt = conn.prepare(
        "SELECT 
            s.id,
            s.name,
            COUNT(DISTINCT ast.artist_id) as artist_count
         FROM styles s
         INNER JOIN artists_styles ast ON s.id = ast.style_id
         INNER JOIN artists a ON ast.artist_id = a.id
         INNER JOIN locations l ON a.location_id = l.id
         WHERE l.lat BETWEEN ?1 AND ?2
           AND l.long BETWEEN ?3 AND ?4
         GROUP BY s.id, s.name
         HAVING artist_count > 0
         ORDER BY artist_count DESC, s.name ASC",
    )?;

    let styles = stmt.query_map(
        [
            bounds.south_west.lat,
            bounds.north_east.lat,
            bounds.south_west.long,
            bounds.north_east.long,
        ],
        |row| {
            Ok(crate::server::StyleWithCount {
                id: row.get(0)?,
                name: row.get(1)?,
                description: None,
                artist_count: row.get(2)?,
                sample_images: None,
            })
        },
    )?;

    styles.collect()
}

#[cfg(feature = "ssr")]
fn map_location_row(row: &rusqlite::Row) -> rusqlite::Result<(shared_types::LocationInfo, i32)> {
    use shared_types::LocationInfo;
    let location_id: i32 = row.get(0)?;
    let location_info = LocationInfo {
        id: location_id as i32,
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
            CASE WHEN COUNT(DISTINCT a.id) > 0 THEN 1 ELSE 0 END as has_artists,
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
            CASE WHEN COUNT(DISTINCT a.id) > 0 THEN 1 ELSE 0 END as has_artists,
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
             WHERE a.location_id = ?1",
        )?;

        let image_count: i32 =
            image_count_stmt.query_row(params![location_info.id], |row| row.get(0))?;

        let mut styles_stmt = conn.prepare(
            "SELECT DISTINCT s.name
             FROM styles s
             JOIN artists_styles ast ON s.id = ast.style_id
             JOIN artists a ON ast.artist_id = a.id
             WHERE a.location_id = ?1
             LIMIT 5",
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
pub fn query_matched_artists(
    style_preferences: Vec<String>,
    location: String,
    price_range: Option<(f64, f64)>,
) -> SqliteResult<Vec<crate::server::MatchedArtist>> {
    use rusqlite::params;
    let db_path = std::path::Path::new("tatteau.db");
    let conn = Connection::open(db_path)?;

    // Start with a simple query to get artists, then we'll add filtering later
    let query = "
        SELECT DISTINCT 
            a.id,
            a.name,
            l.city,
            l.state,
            l.name as location_name,
            a.years_experience,
            COUNT(DISTINCT ai.id) as image_count
        FROM artists a
        LEFT JOIN locations l ON a.location_id = l.id
        LEFT JOIN artists_images ai ON a.id = ai.artist_id
        WHERE (l.is_person IS NULL OR l.is_person != 1)
        AND a.name IS NOT NULL
        AND a.name != ''
        GROUP BY a.id, a.name, l.city, l.state, l.name, a.years_experience
        ORDER BY image_count DESC, a.name ASC 
        LIMIT 10
    ";

    let mut stmt = conn.prepare(query)?;

    let artist_iter = stmt.query_map([], |row| {
        let artist_id: i64 = row.get(0)?;
        let artist_name: String = row.get(1)?;
        let city: String = row.get(2).unwrap_or_else(|_| "Unknown".to_string());
        let state: String = row.get(3).unwrap_or_else(|_| "Unknown".to_string());
        let location_name: String = row.get(4).unwrap_or_else(|_| "Unknown Studio".to_string());
        let years_experience: Option<i32> = row.get(5).ok();
        let image_count: i32 = row.get(6).unwrap_or(0);

        // Get styles for this artist
        let styles = get_artist_styles_by_id(&conn, artist_id).unwrap_or_default();

        // Get portfolio images for this artist
        let portfolio_images =
            get_artist_portfolio_images_by_id(&conn, artist_id).unwrap_or_default();

        // Calculate match score based on style overlap and image count
        let match_score = calculate_match_score(&styles, &style_preferences, image_count);

        Ok(crate::server::MatchedArtist {
            id: artist_id,
            name: artist_name,
            all_styles: styles.clone(),
            portfolio_images,
            avatar_url: None, // TODO: Add avatar support in database
            years_experience,
            min_price: Some(150.0), // TODO: Get from artist pricing table
            max_price: Some(400.0), // TODO: Get from artist pricing table
            avg_rating: 4.2,        // TODO: Calculate from reviews
            image_count,
            match_score,
            city,
            state,
            location_name,
            primary_style: styles.first().unwrap_or(&"Various".to_string()).clone(),
        })
    })?;

    artist_iter.collect()
}

#[cfg(feature = "ssr")]
fn get_artist_styles_by_id(conn: &Connection, artist_id: i64) -> SqliteResult<Vec<String>> {
    let mut stmt = conn.prepare(
        "SELECT s.name FROM styles s 
         JOIN artists_styles ars ON s.id = ars.style_id 
         WHERE ars.artist_id = ?",
    )?;

    let style_iter = stmt.query_map([artist_id], |row| Ok(row.get::<_, String>(0)?))?;

    style_iter.collect()
}

#[cfg(feature = "ssr")]
fn get_artist_portfolio_images_by_id(
    conn: &Connection,
    artist_id: i64,
) -> SqliteResult<Vec<String>> {
    let mut stmt = conn.prepare(
        "SELECT short_code FROM artists_images 
         WHERE artist_id = ? 
         ORDER BY id DESC 
         LIMIT 4",
    )?;

    let image_iter = stmt.query_map([artist_id], |row| {
        let short_code: String = row.get(0)?;
        // Return the Instagram post URL so we can create proper embeds
        // We'll handle the embedding in the frontend component
        Ok(format!(
            "https://www.instagram.com/p/{}/",
            short_code
        ))
    })?;

    image_iter.collect()
}

#[cfg(feature = "ssr")]
fn calculate_match_score(
    artist_styles: &[String],
    user_preferences: &[String],
    image_count: i32,
) -> i32 {
    let mut score = 60; // Base score

    // Add points for image count
    score += std::cmp::min(20, image_count * 2);

    // Add points for style matches
    if !user_preferences.is_empty() {
        let matches = artist_styles
            .iter()
            .filter(|style| {
                user_preferences.iter().any(|pref| {
                    style.to_lowercase().contains(&pref.to_lowercase())
                        || pref.to_lowercase().contains(&style.to_lowercase())
                })
            })
            .count();

        score += (matches as f32 / user_preferences.len() as f32 * 20.0) as i32;
    } else {
        score += 10; // Slight bonus when no preferences (shows all artists)
    }

    score.max(50).min(95) // Ensure reasonable score range
}

#[cfg(feature = "ssr")]
pub fn get_coords_by_postal_code(postal_code: String) -> SqliteResult<CityCoords> {
    use rusqlite::params;
    let db_path = Path::new("tatteau.db");
    let conn = Connection::open(db_path)?;

    // First check if the postal code exists
    let mut check_stmt = conn.prepare("SELECT COUNT(*) FROM locations WHERE postal_code = ?1")?;

    let count: i32 = check_stmt.query_row(params![postal_code], |row| row.get(0))?;

    if count == 0 {
        return Err(rusqlite::Error::QueryReturnedNoRows);
    }

    let mut stmt = conn.prepare(
        "SELECT DISTINCT city, state, lat, long
         FROM locations
         WHERE postal_code = ?1
         AND (is_person IS NULL OR is_person != 1)
         LIMIT 1",
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

#[cfg(feature = "ssr")]
pub fn get_location_with_artist_details(
    location_id: i32,
) -> SqliteResult<crate::server::LocationDetailInfo> {
    use crate::server::{ArtistThumbnail, LocationDetailInfo};
    use rusqlite::params;
    let db_path = std::path::Path::new("tatteau.db");
    let conn = rusqlite::Connection::open(db_path)?;

    // Get location info
    let location_query = "
        SELECT id, name, lat, long, city, county, state, country_code, 
               postal_code, is_open, address, category, website_uri, _id
        FROM locations 
        WHERE id = ?1
    ";

    let location = conn.query_row(location_query, params![location_id], |row| {
        Ok(shared_types::LocationInfo {
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
            has_artists: None,
            artist_images_count: None,
        })
    })?;

    // Get artists with their primary image and style
    let artists_query = "
        SELECT DISTINCT a.id, a.name,
               (SELECT ai.short_code 
                FROM artists_images ai 
                WHERE ai.artist_id = a.id 
                LIMIT 1) as image_url,
               (SELECT s.name 
                FROM styles s 
                JOIN artists_styles ast ON s.id = ast.style_id 
                WHERE ast.artist_id = a.id 
                LIMIT 1) as primary_style
        FROM artists a
        WHERE a.location_id = ?1
        ORDER BY a.name
        LIMIT 4
    ";

    let mut stmt = conn.prepare(artists_query)?;
    let artist_rows = stmt.query_map(params![location_id], |row| {
        Ok(ArtistThumbnail {
            artist_id: row.get(0)?,
            artist_name: row.get(1)?,
            image_url: row.get(2)?,
            primary_style: row.get(3)?,
        })
    })?;

    let artists: Vec<ArtistThumbnail> = artist_rows.collect::<Result<Vec<_>, _>>()?;

    // Get counts and stats
    let stats_query = "
        SELECT 
            COUNT(DISTINCT a.id) as artist_count,
            COUNT(DISTINCT ai.id) as image_count
        FROM artists a
        LEFT JOIN artists_images ai ON a.id = ai.artist_id
        WHERE a.location_id = ?1
    ";

    let (artist_count, image_count): (i32, i32) =
        conn.query_row(stats_query, params![location_id], |row| {
            Ok((row.get(0)?, row.get(1)?))
        })?;

    // Get styles
    let styles_query = "
        SELECT DISTINCT s.name
        FROM styles s
        JOIN artists_styles ast ON s.id = ast.style_id
        JOIN artists a ON ast.artist_id = a.id
        WHERE a.location_id = ?1
        ORDER BY s.name
        LIMIT 5
    ";

    let mut styles_stmt = conn.prepare(styles_query)?;
    let style_rows = styles_stmt.query_map(params![location_id], |row| row.get::<_, String>(0))?;
    let styles: Vec<String> = style_rows.collect::<Result<Vec<_>, _>>()?;

    Ok(LocationDetailInfo {
        location,
        artist_count,
        image_count,
        styles,
        artists,
        min_price: None, // TODO: Add when pricing table exists
        max_price: None, // TODO: Add when pricing table exists
        average_rating: None, // TODO: Add when artist_reviews table exists
    })
}

// Questionnaire System Repository Functions

#[cfg(feature = "ssr")]
pub fn get_artist_questionnaire(artist_id: i32) -> SqliteResult<ClientQuestionnaireForm> {
    use rusqlite::params;
    
    let db_path = Path::new("tatteau.db");
    let conn = Connection::open(db_path)?;

    let mut stmt = conn.prepare("
        SELECT q.id, q.question_type, q.question_text, aq.is_required, 
               COALESCE(aq.custom_options, q.options_data) as options,
               q.validation_rules
        FROM questionnaire_questions q
        JOIN artist_questionnaires aq ON q.id = aq.question_id
        WHERE aq.artist_id = ?1
        ORDER BY aq.display_order
    ")?;

    let question_rows = stmt.query_map(params![artist_id], |row| {
        let options_str: Option<String> = row.get(4)?;
        let options = match options_str {
            Some(json_str) => serde_json::from_str(&json_str).unwrap_or_default(),
            None => vec![],
        };

        Ok(ClientQuestionnaireQuestion {
            id: row.get(0)?,
            question_type: row.get(1)?,
            question_text: row.get(2)?,
            is_required: row.get(3)?,
            options,
            validation_rules: row.get(5)?,
        })
    })?;

    let questions: Result<Vec<_>, _> = question_rows.collect();
    
    Ok(ClientQuestionnaireForm {
        artist_id,
        questions: questions?,
    })
}

#[cfg(feature = "ssr")]
pub fn get_all_default_questions() -> SqliteResult<Vec<QuestionnaireQuestion>> {
    use rusqlite::params;
    
    let db_path = Path::new("tatteau.db");
    let conn = Connection::open(db_path)?;

    let mut stmt = conn.prepare("
        SELECT id, question_type, question_text, is_default, 
               options_data, validation_rules, created_at
        FROM questionnaire_questions
        WHERE is_default = true
        ORDER BY id
    ")?;

    let question_rows = stmt.query_map([], |row| {
        Ok(QuestionnaireQuestion {
            id: row.get(0)?,
            question_type: row.get(1)?,
            question_text: row.get(2)?,
            is_default: row.get(3)?,
            options_data: row.get(4)?,
            validation_rules: row.get(5)?,
            created_at: row.get(6)?,
        })
    })?;

    question_rows.collect()
}

#[cfg(feature = "ssr")]
pub fn get_artist_questionnaire_config(artist_id: i32) -> SqliteResult<Vec<ArtistQuestionnaire>> {
    use rusqlite::params;
    
    let db_path = Path::new("tatteau.db");
    let conn = Connection::open(db_path)?;

    let mut stmt = conn.prepare("
        SELECT id, artist_id, question_id, is_required, display_order, custom_options
        FROM artist_questionnaires
        WHERE artist_id = ?1
        ORDER BY display_order
    ")?;

    let config_rows = stmt.query_map(params![artist_id], |row| {
        Ok(ArtistQuestionnaire {
            id: row.get(0)?,
            artist_id: row.get(1)?,
            question_id: row.get(2)?,
            is_required: row.get(3)?,
            display_order: row.get(4)?,
            custom_options: row.get(5)?,
        })
    })?;

    config_rows.collect()
}

#[cfg(feature = "ssr")]
pub fn update_artist_questionnaire_config(
    artist_id: i32,
    config: Vec<ArtistQuestionnaire>
) -> SqliteResult<()> {
    use rusqlite::params;
    
    let db_path = Path::new("tatteau.db");
    let conn = Connection::open(db_path)?;
    
    // Remove existing config
    conn.execute(
        "DELETE FROM artist_questionnaires WHERE artist_id = ?1",
        params![artist_id],
    )?;
    
    // Insert new config
    for item in config {
        conn.execute(
            "INSERT INTO artist_questionnaires 
             (artist_id, question_id, is_required, display_order, custom_options)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                item.artist_id,
                item.question_id,
                item.is_required,
                item.display_order,
                item.custom_options
            ],
        )?;
    }
    
    Ok(())
}

#[cfg(feature = "ssr")]
pub fn get_artist_id_from_user_id(user_id: i32) -> SqliteResult<Option<i32>> {
    use rusqlite::params;
    
    let db_path = Path::new("tatteau.db");
    let conn = Connection::open(db_path)?;
    
    let mut stmt = conn.prepare(
        "SELECT artist_id FROM artist_users WHERE id = ?1"
    )?;
    
    let artist_id: Result<i32, rusqlite::Error> = stmt.query_row(
        params![user_id],
        |row| Ok(row.get(0)?)
    );
    
    match artist_id {
        Ok(id) => Ok(Some(id)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e),
    }
}

#[cfg(feature = "ssr")]
pub fn delete_artist_question(
    artist_id: i32,
    question_id: i32
) -> SqliteResult<()> {
    use rusqlite::params;
    
    let db_path = Path::new("tatteau.db");
    let conn = Connection::open(db_path)?;
    
    // Delete the specific question from artist's configuration
    conn.execute(
        "DELETE FROM artist_questionnaires WHERE artist_id = ?1 AND question_id = ?2",
        params![artist_id, question_id],
    )?;
    
    Ok(())
}

#[cfg(feature = "ssr")]
pub fn save_questionnaire_responses(
    booking_request_id: i32,
    responses: Vec<BookingQuestionnaireResponse>
) -> SqliteResult<()> {
    use rusqlite::params;
    
    let db_path = Path::new("tatteau.db");
    let conn = Connection::open(db_path)?;
    
    for response in responses {
        conn.execute(
            "INSERT INTO booking_questionnaire_responses 
             (booking_request_id, question_id, response_text, response_data)
             VALUES (?1, ?2, ?3, ?4)",
            params![
                booking_request_id,
                response.question_id,
                response.response_text,
                response.response_data
            ],
        )?;
    }
    
    Ok(())
}

#[cfg(feature = "ssr")]
pub fn get_booking_questionnaire_responses(
    booking_request_id: i32
) -> SqliteResult<Vec<BookingQuestionnaireResponse>> {
    use rusqlite::params;
    
    let db_path = Path::new("tatteau.db");
    let conn = Connection::open(db_path)?;

    let mut stmt = conn.prepare("
        SELECT id, booking_request_id, question_id, response_text, response_data, created_at
        FROM booking_questionnaire_responses
        WHERE booking_request_id = ?1
        ORDER BY id
    ")?;

    let response_rows = stmt.query_map(params![booking_request_id], |row| {
        Ok(BookingQuestionnaireResponse {
            id: row.get(0)?,
            booking_request_id: row.get(1)?,
            question_id: row.get(2)?,
            response_text: row.get(3)?,
            response_data: row.get(4)?,
            created_at: row.get(5)?,
        })
    })?;

    response_rows.collect()
}

// Error Logging Functions
#[cfg(feature = "ssr")]
pub fn log_error(error_data: CreateErrorLog) -> SqliteResult<i64> {
    use rusqlite::params;
    
    let db_path = Path::new("tatteau.db");
    let conn = Connection::open(db_path)?;
    
    conn.execute(
        "INSERT INTO error_logs 
         (error_type, error_level, error_message, error_stack, url_path, 
          user_agent, user_id, session_id, request_headers, additional_context)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
        params![
            error_data.error_type,
            error_data.error_level,
            error_data.error_message,
            error_data.error_stack,
            error_data.url_path,
            error_data.user_agent,
            error_data.user_id,
            error_data.session_id,
            error_data.request_headers,
            error_data.additional_context
        ],
    )?;

    Ok(conn.last_insert_rowid())
}

#[cfg(feature = "ssr")]
pub fn get_recent_errors(limit: i32) -> SqliteResult<Vec<ErrorLog>> {
    use rusqlite::params;
    
    let db_path = Path::new("tatteau.db");
    let conn = Connection::open(db_path)?;

    let mut stmt = conn.prepare("
        SELECT id, error_type, error_level, error_message, error_stack, 
               url_path, user_agent, user_id, session_id, timestamp,
               request_headers, additional_context
        FROM error_logs
        ORDER BY timestamp DESC
        LIMIT ?1
    ")?;

    let error_rows = stmt.query_map(params![limit], |row| {
        Ok(ErrorLog {
            id: row.get(0)?,
            error_type: row.get(1)?,
            error_level: row.get(2)?,
            error_message: row.get(3)?,
            error_stack: row.get(4)?,
            url_path: row.get(5)?,
            user_agent: row.get(6)?,
            user_id: row.get(7)?,
            session_id: row.get(8)?,
            timestamp: row.get(9)?,
            request_headers: row.get(10)?,
            additional_context: row.get(11)?,
        })
    })?;

    error_rows.collect()
}

#[cfg(feature = "ssr")]
pub fn get_errors_by_type(error_type: String, limit: i32) -> SqliteResult<Vec<ErrorLog>> {
    use rusqlite::params;
    
    let db_path = Path::new("tatteau.db");
    let conn = Connection::open(db_path)?;

    let mut stmt = conn.prepare("
        SELECT id, error_type, error_level, error_message, error_stack, 
               url_path, user_agent, user_id, session_id, timestamp,
               request_headers, additional_context
        FROM error_logs
        WHERE error_type = ?1
        ORDER BY timestamp DESC
        LIMIT ?2
    ")?;

    let error_rows = stmt.query_map(params![error_type, limit], |row| {
        Ok(ErrorLog {
            id: row.get(0)?,
            error_type: row.get(1)?,
            error_level: row.get(2)?,
            error_message: row.get(3)?,
            error_stack: row.get(4)?,
            url_path: row.get(5)?,
            user_agent: row.get(6)?,
            user_id: row.get(7)?,
            session_id: row.get(8)?,
            timestamp: row.get(9)?,
            request_headers: row.get(10)?,
            additional_context: row.get(11)?,
        })
    })?;

    error_rows.collect()
}
