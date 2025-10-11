use super::entities::{
    Artist, ArtistImage, Style, CityCoords, Location,
    QuestionnaireQuestion, ArtistQuestionnaire, BookingQuestionnaireResponse,
    ClientQuestionnaireForm, ClientQuestionnaireQuestion,
    ErrorLog, CreateErrorLog,
};
#[cfg(feature = "ssr")]
use sqlx::{PgPool, Row, Executor};
#[cfg(feature = "ssr")]
use serde_json;
use shared_types::{LocationInfo, MapBounds};
#[cfg(feature = "ssr")]
type DbResult<T> = Result<T, sqlx::Error>;

#[cfg(feature = "ssr")]
pub async fn get_cities_and_coords(state: String) -> DbResult<Vec<CityCoords>> {
    let pool = crate::db::pool::get_pool();

    let rows = sqlx::query(
        "
            SELECT
                city,
                state,
                lat,
                long
            FROM locations
            WHERE
                state = $1
            AND (is_person IS NULL OR is_person != 1)
            GROUP BY city, state, lat, long
        ",
    )
    .bind(&state)
    .fetch_all(pool)
    .await?;

    let city_coords: Vec<CityCoords> = rows
        .into_iter()
        .map(|row| CityCoords {
            city: row.get("city"),
            state: row.get("state"),
            lat: row.get("lat"),
            long: row.get("long"),
        })
        .filter(|c| c.city.parse::<f64>().is_err())
        .collect();

    Ok(city_coords)
}

#[cfg(feature = "ssr")]
pub async fn query_locations(
    state: String,
    city: String,
    bounds: MapBounds,
) -> DbResult<Vec<LocationInfo>> {
    let pool = crate::db::pool::get_pool();

    let rows = sqlx::query(
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
            l.lat BETWEEN $1 AND $2
            AND l.long BETWEEN $3 AND $4
            AND (l.is_person IS NULL OR l.is_person != 1)
        GROUP BY l.id, l.name, l.lat, l.long, l.city, l.county, l.state, l.country_code, l.postal_code, l.is_open, l.address, l.category, l.website_uri, l._id
    ",
    )
    .bind(bounds.south_west.lat)
    .bind(bounds.north_east.lat)
    .bind(bounds.south_west.long)
    .bind(bounds.north_east.long)
    .fetch_all(pool)
    .await?;

    let locations: Vec<LocationInfo> = rows
        .into_iter()
        .map(|row| {
            let has_artists: i64 = row.get("has_artists");
            let artist_images_count: i64 = row.get("artist_images_count");
            LocationInfo {
                id: row.get("id"),
                name: row.get("name"),
                lat: row.get("lat"),
                long: row.get("long"),
                city: row.get("city"),
                county: row.get("county"),
                state: row.get("state"),
                country_code: row.get("country_code"),
                postal_code: row.get("postal_code"),
                is_open: row.get("is_open"),
                address: row.get("address"),
                category: row.get("category"),
                website_uri: row.get("website_uri"),
                _id: row.get("_id"),
                has_artists: Some(has_artists == 1),
                artist_images_count: Some(artist_images_count as i32),
            }
        })
        .collect();

    Ok(locations)
}

pub struct LocationState {
    pub state: String,
}
#[cfg(feature = "ssr")]
pub async fn get_states() -> DbResult<Vec<LocationState>> {
    let pool = crate::db::pool::get_pool();

    let rows = sqlx::query(
        "
        SELECT DISTINCT state
        FROM locations
        WHERE country_code = 'United States'
        ORDER BY state ASC
    ",
    )
    .fetch_all(pool)
    .await?;

    let states = rows
        .into_iter()
        .map(|r| LocationState { state: r.get("state") })
        .collect();

    Ok(states)
}

#[cfg(feature = "ssr")]
pub async fn get_city_coordinates(city_name: String) -> DbResult<CityCoords> {
    let pool = crate::db::pool::get_pool();

    let rows = sqlx::query(
        "
            SELECT
                state_name,
                city,
                latitude,
                longitude
            FROM cities
            WHERE LOWER(REPLACE(city, '''', ' ')) LIKE $1
        ",
    )
    .bind(format!("%{}%", city_name))
    .fetch_all(pool)
    .await?;

    let city_coords: Vec<CityCoords> = rows
        .into_iter()
        .map(|row| CityCoords {
            city: row.get("city"),
            state: row.get("state_name"),
            lat: row.get("latitude"),
            long: row.get("longitude"),
        })
        .collect();

    city_coords.first()
        .cloned()
        .ok_or_else(|| sqlx::Error::RowNotFound)
}

#[cfg(feature = "ssr")]
pub async fn get_artist_by_id(artist_id: i32) -> DbResult<Artist> {
    let pool = crate::db::pool::get_pool();

    let row = sqlx::query(
        "SELECT id, name, location_id, social_links, email, phone, years_experience, styles_extracted
         FROM artists
         WHERE id = $1"
    )
    .bind(artist_id)
    .fetch_one(pool)
    .await?;

    Ok(Artist {
        id: row.get("id"),
        name: row.get("name"),
        location_id: row.get("location_id"),
        social_links: row.get("social_links"),
        email: row.get("email"),
        phone: row.get("phone"),
        years_experience: row.get("years_experience"),
        styles_extracted: row.get("styles_extracted"),
    })
}

#[cfg(feature = "ssr")]
pub async fn get_artist_location(location_id: i32) -> DbResult<Location> {
    let pool = crate::db::pool::get_pool();

    let row = sqlx::query(
        "SELECT id, name, lat, long, city, state, address, website_uri
         FROM locations
         WHERE id = $1",
    )
    .bind(location_id)
    .fetch_one(pool)
    .await?;

    Ok(Location {
        id: row.get("id"),
        name: row.get("name"),
        lat: row.get("lat"),
        long: row.get("long"),
        city: row.get("city"),
        state: row.get("state"),
        address: row.get("address"),
        website_uri: row.get("website_uri"),
    })
}

#[cfg(feature = "ssr")]
pub async fn get_artist_styles(artist_id: i32) -> DbResult<Vec<Style>> {
    let pool = crate::db::pool::get_pool();

    let rows = sqlx::query(
        "SELECT s.id, s.name
         FROM styles s
         JOIN artists_styles ast ON s.id = ast.style_id
         WHERE ast.artist_id = $1",
    )
    .bind(artist_id)
    .fetch_all(pool)
    .await?;

    let styles = rows
        .into_iter()
        .map(|row| Style {
            id: row.get("id"),
            name: row.get("name"),
        })
        .collect();

    Ok(styles)
}

#[cfg(feature = "ssr")]
pub async fn get_artist_images_with_styles(
    artist_id: i32,
) -> DbResult<Vec<(ArtistImage, Vec<Style>)>> {
    let pool = crate::db::pool::get_pool();

    // First get all images for the artist
    let image_rows = sqlx::query(
        "SELECT id, short_code, artist_id
         FROM artists_images
         WHERE artist_id = $1",
    )
    .bind(artist_id)
    .fetch_all(pool)
    .await?;

    let mut result = Vec::new();

    // For each image, get its styles
    for image_row in image_rows {
        let img = ArtistImage {
            id: image_row.get("id"),
            short_code: image_row.get("short_code"),
            artist_id: image_row.get("artist_id"),
        };
        let img_id = img.id;

        let style_rows = sqlx::query(
            "SELECT s.id, s.name
             FROM styles s
             JOIN artists_images_styles ais ON s.id = ais.style_id
             WHERE ais.artists_images_id = $1",
        )
        .bind(img_id)
        .fetch_all(pool)
        .await?;

        let styles: Vec<Style> = style_rows
            .into_iter()
            .map(|row| Style {
                id: row.get("id"),
                name: row.get("name"),
            })
            .collect();

        result.push((img, styles));
    }

    Ok(result)
}

#[cfg(feature = "ssr")]
pub async fn get_location_by_id(location_id: i32) -> DbResult<Location> {
    let pool = crate::db::pool::get_pool();

    let row = sqlx::query(
        "SELECT id, name, lat, long, city, state, address, website_uri
         FROM locations
         WHERE id = $1",
    )
    .bind(location_id)
    .fetch_one(pool)
    .await?;

    Ok(Location {
        id: row.get("id"),
        name: row.get("name"),
        lat: row.get("lat"),
        long: row.get("long"),
        city: row.get("city"),
        state: row.get("state"),
        address: row.get("address"),
        website_uri: row.get("website_uri"),
    })
}

#[cfg(feature = "ssr")]
pub async fn get_artists_by_location(location_id: i32) -> DbResult<Vec<Artist>> {
    let pool = crate::db::pool::get_pool();

    let rows = sqlx::query(
        "SELECT a.id, a.name, a.location_id, a.social_links, a.email, a.phone, a.years_experience, a.styles_extracted
         FROM artists a
         JOIN locations l ON a.location_id = l.id
         WHERE a.location_id = $1
         AND (l.is_person IS NULL OR l.is_person != 1)
         AND a.name IS NOT NULL
         AND a.name != ''"
    )
    .bind(location_id)
    .fetch_all(pool)
    .await?;

    let artists = rows
        .into_iter()
        .map(|row| Artist {
            id: row.get("id"),
            name: row.get("name"),
            location_id: row.get("location_id"),
            social_links: row.get("social_links"),
            email: row.get("email"),
            phone: row.get("phone"),
            years_experience: row.get("years_experience"),
            styles_extracted: row.get("styles_extracted"),
        })
        .collect();

    Ok(artists)
}

#[cfg(feature = "ssr")]
pub async fn get_all_styles_by_location(location_id: i32) -> DbResult<Vec<Style>> {
    let pool = crate::db::pool::get_pool();

    let rows = sqlx::query(
        "SELECT DISTINCT s.id, s.name
         FROM styles s
         JOIN artists_styles ast ON s.id = ast.style_id
         JOIN artists a ON ast.artist_id = a.id
         JOIN locations l ON a.location_id = l.id
         WHERE a.location_id = $1
         AND (l.is_person IS NULL OR l.is_person != 1)
         AND a.name IS NOT NULL
         AND a.name != ''
         ORDER BY s.name",
    )
    .bind(location_id)
    .fetch_all(pool)
    .await?;

    let styles = rows
        .into_iter()
        .map(|row| Style {
            id: row.get("id"),
            name: row.get("name"),
        })
        .collect();

    Ok(styles)
}

#[cfg(feature = "ssr")]
pub async fn get_all_images_with_styles_by_location(
    location_id: i32,
) -> DbResult<Vec<(ArtistImage, Vec<Style>, Artist)>> {
    let pool = crate::db::pool::get_pool();

    // First get all images for artists at this location, filtering out person locations
    let image_rows = sqlx::query(
        "SELECT ai.id, ai.short_code, ai.artist_id, a.id as a_id, a.name, a.location_id, a.social_links, a.email, a.phone, a.years_experience, a.styles_extracted
         FROM artists_images ai
         JOIN artists a ON ai.artist_id = a.id
         JOIN locations l ON a.location_id = l.id
         WHERE a.location_id = $1
         AND (l.is_person IS NULL OR l.is_person != 1)
         AND a.name IS NOT NULL
         AND a.name != ''"
    )
    .bind(location_id)
    .fetch_all(pool)
    .await?;

    let mut result = Vec::new();

    // For each image, get its styles
    for image_row in image_rows {
        let image = ArtistImage {
            id: image_row.get("id"),
            short_code: image_row.get("short_code"),
            artist_id: image_row.get("artist_id"),
        };

        let artist = Artist {
            id: image_row.get("a_id"),
            name: image_row.get("name"),
            location_id: image_row.get("location_id"),
            social_links: image_row.get("social_links"),
            email: image_row.get("email"),
            phone: image_row.get("phone"),
            years_experience: image_row.get("years_experience"),
            styles_extracted: image_row.get("styles_extracted"),
        };

        let img_id = image.id;

        let style_rows = sqlx::query(
            "SELECT s.id, s.name
             FROM styles s
             JOIN artists_images_styles ais ON s.id = ais.style_id
             WHERE ais.artists_images_id = $1",
        )
        .bind(img_id)
        .fetch_all(pool)
        .await?;

        let styles: Vec<Style> = style_rows
            .into_iter()
            .map(|row| Style {
                id: row.get("id"),
                name: row.get("name"),
            })
            .collect();

        result.push((image, styles, artist));
    }

    Ok(result)
}

#[cfg(feature = "ssr")]
pub async fn save_quiz_session(
    style_preference: String,
    body_placement: String,
    pain_tolerance: i32,
    budget_min: f64,
    budget_max: f64,
    vibe_preference: String,
) -> DbResult<i64> {
    let pool = crate::db::pool::get_pool();

    let row = sqlx::query(
        "INSERT INTO client_quiz_sessions (style_preference, body_placement, pain_tolerance, budget_min, budget_max, vibe_preference)
         VALUES ($1, $2, $3, $4, $5, $6)
         RETURNING id",
    )
    .bind(style_preference)
    .bind(body_placement)
    .bind(pain_tolerance)
    .bind(budget_min)
    .bind(budget_max)
    .bind(vibe_preference)
    .fetch_one(pool)
    .await?;

    Ok(row.get("id"))
}

#[cfg(feature = "ssr")]
pub async fn get_location_stats_for_city(
    city: String,
    state: String,
) -> DbResult<crate::server::LocationStats> {
    let pool = crate::db::pool::get_pool();

    let row = sqlx::query(
        "SELECT
            COUNT(DISTINCT l.id) as shop_count,
            COUNT(DISTINCT a.id) as artist_count,
            COUNT(DISTINCT s.id) as styles_available
         FROM locations l
         LEFT JOIN artists a ON l.id = a.location_id
         LEFT JOIN artists_styles ast ON a.id = ast.artist_id
         LEFT JOIN styles s ON ast.style_id = s.id
         WHERE l.city = $1 AND l.state = $2
         AND (l.is_person IS NULL OR l.is_person != 1)",
    )
    .bind(city)
    .bind(state)
    .fetch_one(pool)
    .await?;

    Ok(crate::server::LocationStats {
        shop_count: row.try_get::<i64, _>("shop_count").unwrap_or(0) as i32,
        artist_count: row.try_get::<i64, _>("artist_count").unwrap_or(0) as i32,
        styles_available: row.try_get::<i64, _>("styles_available").unwrap_or(0) as i32,
    })
}

#[cfg(feature = "ssr")]
pub async fn get_all_styles_with_counts() -> DbResult<Vec<crate::server::StyleWithCount>> {
    let pool = crate::db::pool::get_pool();

    let rows = sqlx::query(
        "SELECT
            s.id,
            s.name,
            COUNT(DISTINCT ast.artist_id) as artist_count
         FROM styles s
         LEFT JOIN artists_styles ast ON s.id = ast.style_id
         GROUP BY s.id, s.name
         ORDER BY artist_count DESC, s.name ASC",
    )
    .fetch_all(pool)
    .await?;

    let styles = rows
        .into_iter()
        .map(|row| crate::server::StyleWithCount {
            id: row.try_get::<i64, _>("id").unwrap_or(0) as i32,
            name: row.get("name"),
            description: None,
            artist_count: row.try_get::<i64, _>("artist_count").unwrap_or(0) as i32,
            sample_images: None,
        })
        .collect();

    Ok(styles)
}

#[cfg(feature = "ssr")]
pub async fn get_styles_with_counts_in_bounds(
    bounds: shared_types::MapBounds,
) -> DbResult<Vec<crate::server::StyleWithCount>> {
    let pool = crate::db::pool::get_pool();

    let rows = sqlx::query(
        "SELECT
            s.id,
            s.name,
            COUNT(DISTINCT ast.artist_id) as artist_count
         FROM styles s
         INNER JOIN artists_styles ast ON s.id = ast.style_id
         INNER JOIN artists a ON ast.artist_id = a.id
         INNER JOIN locations l ON a.location_id = l.id
         WHERE l.lat BETWEEN $1 AND $2
           AND l.long BETWEEN $3 AND $4
         GROUP BY s.id, s.name
         HAVING COUNT(DISTINCT ast.artist_id) > 0
         ORDER BY artist_count DESC, s.name ASC",
    )
    .bind(bounds.south_west.lat)
    .bind(bounds.north_east.lat)
    .bind(bounds.south_west.long)
    .bind(bounds.north_east.long)
    .fetch_all(pool)
    .await?;

    let styles = rows
        .into_iter()
        .map(|row| crate::server::StyleWithCount {
            id: row.try_get::<i64, _>("id").unwrap_or(0) as i32,
            name: row.get("name"),
            description: None,
            artist_count: row.try_get::<i64, _>("artist_count").unwrap_or(0) as i32,
            sample_images: None,
        })
        .collect();

    Ok(styles)
}

#[cfg(feature = "ssr")]
pub async fn get_styles_by_location(
    states: Option<Vec<String>>,
    cities: Option<Vec<String>>,
) -> DbResult<Vec<crate::server::StyleWithCount>> {
    let pool = crate::db::pool::get_pool();

    // Build query based on location filters
    let mut where_clauses = Vec::new();
    let mut bind_idx = 1;

    // Build WHERE clause
    if let Some(ref state_list) = states {
        if !state_list.is_empty() {
            let placeholders: Vec<String> = (0..state_list.len())
                .map(|i| format!("${}", bind_idx + i))
                .collect();
            where_clauses.push(format!("l.state IN ({})", placeholders.join(",")));
            bind_idx += state_list.len();
        }
    }

    if let Some(ref city_list) = cities {
        if !city_list.is_empty() {
            let placeholders: Vec<String> = (0..city_list.len())
                .map(|i| format!("${}", bind_idx + i))
                .collect();
            where_clauses.push(format!("l.city IN ({})", placeholders.join(",")));
        }
    }

    let where_clause = if !where_clauses.is_empty() {
        format!("WHERE {}", where_clauses.join(" AND "))
    } else {
        String::new()
    };

    let join_type = if where_clauses.is_empty() { "LEFT" } else { "INNER" };

    let query = format!(
        "SELECT
            s.id,
            s.name,
            COUNT(DISTINCT ai.id) as image_count
         FROM styles s
         {} JOIN artists_images_styles ais ON s.id = ais.style_id
         {} JOIN artists_images ai ON ais.artists_images_id = ai.id
         {} JOIN artists a ON ai.artist_id = a.id
         {} JOIN locations l ON a.location_id = l.id
         {}
         GROUP BY s.id, s.name
         {}
         ORDER BY s.name ASC",
        join_type,
        join_type,
        join_type,
        join_type,
        where_clause,
        if where_clauses.is_empty() { "" } else { "HAVING COUNT(DISTINCT ai.id) > 0" }
    );

    let mut sql_query = sqlx::query(&query);

    // Bind state parameters
    if let Some(ref state_list) = states {
        for state in state_list {
            sql_query = sql_query.bind(state);
        }
    }

    // Bind city parameters
    if let Some(ref city_list) = cities {
        for city in city_list {
            sql_query = sql_query.bind(city);
        }
    }

    let rows = sql_query.fetch_all(pool).await?;

    let styles = rows
        .into_iter()
        .map(|row| crate::server::StyleWithCount {
            id: row.try_get::<i64, _>("id").unwrap_or(0) as i32,
            name: row.get("name"),
            description: None,
            artist_count: row.try_get::<i64, _>("image_count").unwrap_or(0) as i32,
            sample_images: None,
        })
        .collect();

    Ok(styles)
}

#[cfg(feature = "ssr")]
pub async fn query_locations_with_details(
    state: String,
    city: String,
    bounds: MapBounds,
    style_filter: Option<Vec<i32>>,
) -> DbResult<Vec<crate::server::EnhancedLocationInfo>> {
    let pool = crate::db::pool::get_pool();

    // Build the query based on whether we have style filters
    let (query, style_json) = if let Some(ref styles) = style_filter {
        if !styles.is_empty() {
            let json = serde_json::to_value(styles).unwrap();
            (
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
                 WHERE l.lat BETWEEN $1 AND $2
                 AND l.long BETWEEN $3 AND $4
                 AND (l.is_person IS NULL OR l.is_person != 1)
                 AND ast.style_id = ANY($5::int[])
                 GROUP BY l.id, l.name, l.lat, l.long, l.city, l.county, l.state, l.country_code, l.postal_code, l.is_open, l.address, l.category, l.website_uri, l._id",
                Some(json)
            )
        } else {
            (
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
                 WHERE l.lat BETWEEN $1 AND $2
                 AND l.long BETWEEN $3 AND $4
                 AND (l.is_person IS NULL OR l.is_person != 1)
                 GROUP BY l.id, l.name, l.lat, l.long, l.city, l.county, l.state, l.country_code, l.postal_code, l.is_open, l.address, l.category, l.website_uri, l._id",
                None
            )
        }
    } else {
        (
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
             WHERE l.lat BETWEEN $1 AND $2
             AND l.long BETWEEN $3 AND $4
             AND (l.is_person IS NULL OR l.is_person != 1)
             GROUP BY l.id, l.name, l.lat, l.long, l.city, l.county, l.state, l.country_code, l.postal_code, l.is_open, l.address, l.category, l.website_uri, l._id",
            None
        )
    };

    let location_rows = if let Some(json) = style_json {
        let styles_vec: Vec<i32> = serde_json::from_value(json).unwrap();
        sqlx::query(query)
            .bind(bounds.south_west.lat)
            .bind(bounds.north_east.lat)
            .bind(bounds.south_west.long)
            .bind(bounds.north_east.long)
            .bind(&styles_vec)
            .fetch_all(pool)
            .await?
    } else {
        sqlx::query(query)
            .bind(bounds.south_west.lat)
            .bind(bounds.north_east.lat)
            .bind(bounds.south_west.long)
            .bind(bounds.north_east.long)
            .fetch_all(pool)
            .await?
    };

    let mut result = Vec::new();
    for location_row in location_rows {
        let location_id: i32 = location_row.try_get::<i64, _>("id").unwrap_or(0) as i32;
        let has_artists_val: i64 = location_row.get("has_artists");
        let artist_images_count: i64 = location_row.get("artist_images_count");

        let location_info = LocationInfo {
            id: location_id,
            name: location_row.get("name"),
            lat: location_row.get("lat"),
            long: location_row.get("long"),
            city: location_row.get("city"),
            county: location_row.get("county"),
            state: location_row.get("state"),
            country_code: location_row.get("country_code"),
            postal_code: location_row.get("postal_code"),
            is_open: location_row.get("is_open"),
            address: location_row.get("address"),
            category: location_row.get("category"),
            website_uri: location_row.get("website_uri"),
            _id: location_row.get("_id"),
            has_artists: Some(has_artists_val == 1),
            artist_images_count: Some(artist_images_count as i32),
        };

        let artist_count: i64 = location_row.get("artist_count");

        // Get image count for this location
        let image_count_row = sqlx::query(
            "SELECT COUNT(DISTINCT ai.id) as cnt
             FROM artists_images ai
             JOIN artists a ON ai.artist_id = a.id
             WHERE a.location_id = $1",
        )
        .bind(location_id)
        .fetch_one(pool)
        .await?;

        let image_count: i64 = image_count_row.get("cnt");

        // Get styles for this location
        let style_rows = sqlx::query(
            "SELECT DISTINCT s.name
             FROM styles s
             JOIN artists_styles ast ON s.id = ast.style_id
             JOIN artists a ON ast.artist_id = a.id
             WHERE a.location_id = $1
             LIMIT 5",
        )
        .bind(location_id)
        .fetch_all(pool)
        .await?;

        let styles: Vec<String> = style_rows
            .into_iter()
            .map(|row| row.get("name"))
            .collect();

        result.push(crate::server::EnhancedLocationInfo {
            location: location_info,
            artist_count: artist_count as i32,
            image_count: image_count as i32,
            styles,
            min_price: None,
            max_price: None,
        });
    }

    Ok(result)
}

#[cfg(feature = "ssr")]
pub async fn query_matched_artists(
    style_preferences: Vec<String>,
    location: String,
    price_range: Option<(f64, f64)>,
) -> DbResult<Vec<crate::server::MatchedArtist>> {
    let pool = crate::db::pool::get_pool();

    let rows = sqlx::query(
        "SELECT DISTINCT
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
        LIMIT 10"
    )
    .fetch_all(pool)
    .await?;

    let mut artists = Vec::new();

    for row in rows {
        let artist_id: i64 = row.get("id");
        let artist_name: String = row.get("name");
        let city: Option<String> = row.try_get("city").ok();
        let state: Option<String> = row.try_get("state").ok();
        let location_name: Option<String> = row.try_get("location_name").ok();
        let years_experience: Option<i32> = row.try_get("years_experience").ok();
        let image_count: i64 = row.get("image_count");

        // Get styles for this artist
        let styles = get_artist_styles_by_id(pool, artist_id).await.unwrap_or_default();

        // Get portfolio images for this artist
        let portfolio_images = get_artist_portfolio_images_by_id(pool, artist_id).await.unwrap_or_default();

        // Calculate match score based on style overlap and image count
        let match_score = calculate_match_score(&styles, &style_preferences, image_count as i32);

        artists.push(crate::server::MatchedArtist {
            id: artist_id,
            name: artist_name,
            all_styles: styles.clone(),
            portfolio_images,
            avatar_url: None,
            years_experience,
            min_price: Some(150.0),
            max_price: Some(400.0),
            avg_rating: 4.2,
            image_count: image_count as i32,
            match_score,
            city: city.unwrap_or_else(|| "Unknown".to_string()),
            state: state.unwrap_or_else(|| "Unknown".to_string()),
            location_name: location_name.unwrap_or_else(|| "Unknown Studio".to_string()),
            primary_style: styles.first().unwrap_or(&"Various".to_string()).clone(),
        });
    }

    Ok(artists)
}

#[cfg(feature = "ssr")]
async fn get_artist_styles_by_id(pool: &PgPool, artist_id: i64) -> DbResult<Vec<String>> {
    let rows = sqlx::query(
        "SELECT s.name FROM styles s
         JOIN artists_styles ars ON s.id = ars.style_id
         WHERE ars.artist_id = $1",
    )
    .bind(artist_id)
    .fetch_all(pool)
    .await?;

    let styles = rows
        .into_iter()
        .map(|row| row.get("name"))
        .collect();

    Ok(styles)
}

#[cfg(feature = "ssr")]
async fn get_artist_portfolio_images_by_id(
    pool: &PgPool,
    artist_id: i64,
) -> DbResult<Vec<String>> {
    let rows = sqlx::query(
        "SELECT short_code FROM artists_images
         WHERE artist_id = $1
         ORDER BY id DESC
         LIMIT 4",
    )
    .bind(artist_id)
    .fetch_all(pool)
    .await?;

    let images = rows
        .into_iter()
        .map(|row| {
            let short_code: String = row.get("short_code");
            format!("https://www.instagram.com/p/{}/", short_code)
        })
        .collect();

    Ok(images)
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
pub async fn get_coords_by_postal_code(postal_code: String) -> DbResult<CityCoords> {
    let pool = crate::db::pool::get_pool();

    // First check if the postal code exists
    let count_row = sqlx::query("SELECT COUNT(*) as cnt FROM locations WHERE postal_code = $1")
        .bind(&postal_code)
        .fetch_one(pool)
        .await?;

    let count: i64 = count_row.get("cnt");

    if count == 0 {
        return Err(sqlx::Error::RowNotFound);
    }

    let row = sqlx::query(
        "SELECT DISTINCT city, state, lat, long
         FROM locations
         WHERE postal_code = $1
         AND (is_person IS NULL OR is_person != 1)
         LIMIT 1",
    )
    .bind(postal_code)
    .fetch_one(pool)
    .await?;

    Ok(CityCoords {
        city: row.get("city"),
        state: row.get("state"),
        lat: row.get("lat"),
        long: row.get("long"),
    })
}

#[cfg(feature = "ssr")]
pub async fn get_location_with_artist_details(
    location_id: i32,
) -> DbResult<crate::server::LocationDetailInfo> {
    use crate::server::{ArtistThumbnail, LocationDetailInfo};

    let pool = crate::db::pool::get_pool();

    // Get location info
    let location_row = sqlx::query(
        "SELECT id, name, lat, long, city, county, state, country_code,
               postal_code, is_open, address, category, website_uri, _id
        FROM locations
        WHERE id = $1"
    )
    .bind(location_id)
    .fetch_one(pool)
    .await?;

    let location = shared_types::LocationInfo {
        id: location_row.get("id"),
        name: location_row.get("name"),
        lat: location_row.get("lat"),
        long: location_row.get("long"),
        city: location_row.get("city"),
        county: location_row.get("county"),
        state: location_row.get("state"),
        country_code: location_row.get("country_code"),
        postal_code: location_row.get("postal_code"),
        is_open: location_row.get("is_open"),
        address: location_row.get("address"),
        category: location_row.get("category"),
        website_uri: location_row.get("website_uri"),
        _id: location_row.get("_id"),
        has_artists: None,
        artist_images_count: None,
    };

    // Get artists with their primary image and style
    let artist_rows = sqlx::query(
        "SELECT DISTINCT a.id, a.name,
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
        WHERE a.location_id = $1
        ORDER BY a.name
        LIMIT 4"
    )
    .bind(location_id)
    .fetch_all(pool)
    .await?;

    let artists: Vec<ArtistThumbnail> = artist_rows
        .into_iter()
        .map(|row| ArtistThumbnail {
            artist_id: row.get("id"),
            artist_name: row.get("name"),
            image_url: row.try_get("image_url").ok(),
            primary_style: row.try_get("primary_style").ok(),
        })
        .collect();

    // Get counts and stats
    let stats_row = sqlx::query(
        "SELECT
            COUNT(DISTINCT a.id) as artist_count,
            COUNT(DISTINCT ai.id) as image_count
        FROM artists a
        LEFT JOIN artists_images ai ON a.id = ai.artist_id
        WHERE a.location_id = $1"
    )
    .bind(location_id)
    .fetch_one(pool)
    .await?;

    let artist_count: i64 = stats_row.get("artist_count");
    let image_count: i64 = stats_row.get("image_count");

    // Get styles
    let style_rows = sqlx::query(
        "SELECT DISTINCT s.name
        FROM styles s
        JOIN artists_styles ast ON s.id = ast.style_id
        JOIN artists a ON ast.artist_id = a.id
        WHERE a.location_id = $1
        ORDER BY s.name
        LIMIT 5"
    )
    .bind(location_id)
    .fetch_all(pool)
    .await?;

    let styles: Vec<String> = style_rows
        .into_iter()
        .map(|row| row.get("name"))
        .collect();

    Ok(LocationDetailInfo {
        location,
        artist_count: artist_count as i32,
        image_count: image_count as i32,
        styles,
        artists,
        min_price: None,
        max_price: None,
        average_rating: None,
    })
}

// Questionnaire System Repository Functions

#[cfg(feature = "ssr")]
pub async fn get_artist_questionnaire(artist_id: i32) -> DbResult<ClientQuestionnaireForm> {
    let pool = crate::db::pool::get_pool();

    // Get artist's questionnaire configuration
    let question_rows = sqlx::query(
        "WITH latest_configs AS (
            SELECT q.id, q.question_type, q.question_text, aq.is_required,
                   COALESCE(aq.custom_options, q.options_data) as options,
                   q.validation_rules, aq.display_order,
                   ROW_NUMBER() OVER (
                       PARTITION BY q.id
                       ORDER BY
                           CASE WHEN aq.custom_options IS NOT NULL THEN 0 ELSE 1 END,
                           aq.id DESC
                   ) as rn
            FROM questionnaire_questions q
            JOIN artist_questionnaires aq ON q.id = aq.question_id
            WHERE aq.artist_id = $1 AND aq.is_enabled = true
        )
        SELECT id, question_type, question_text, is_required, options, validation_rules
        FROM latest_configs
        WHERE rn = 1
        ORDER BY display_order"
    )
    .bind(artist_id)
    .fetch_all(pool)
    .await?;

    let mut questions: Vec<ClientQuestionnaireQuestion> = question_rows
        .into_iter()
        .map(|row| {
            let options_str: Option<String> = row.try_get("options").ok().flatten();
            let options = match options_str {
                Some(json_str) => serde_json::from_str(&json_str).unwrap_or_default(),
                None => vec![],
            };

            ClientQuestionnaireQuestion {
                id: row.get("id"),
                question_type: row.get("question_type"),
                question_text: row.get("question_text"),
                is_required: row.get("is_required"),
                options,
                validation_rules: row.try_get("validation_rules").ok(),
            }
        })
        .collect();

    // Always append mandatory system appointment date question (question ID 6)
    let has_appointment_question = questions.iter().any(|q| q.id == 6);
    if !has_appointment_question {
        let system_question_result = sqlx::query(
            "SELECT id, question_type, question_text, validation_rules
            FROM questionnaire_questions
            WHERE id = 6"
        )
        .fetch_optional(pool)
        .await?;

        if let Some(system_row) = system_question_result {
            questions.push(ClientQuestionnaireQuestion {
                id: system_row.get("id"),
                question_type: system_row.get("question_type"),
                question_text: system_row.get("question_text"),
                is_required: true,
                options: vec![],
                validation_rules: system_row.try_get("validation_rules").ok(),
            });
        }
    }

    Ok(ClientQuestionnaireForm {
        artist_id,
        questions,
    })
}

// Artist Availability and Booking Conflict Functions
#[cfg(feature = "ssr")]
pub async fn check_artist_availability(artist_id: i32, requested_date: &str, requested_time: &str) -> DbResult<bool> {
    let pool = crate::db::pool::get_pool();

    // Check for existing bookings at the requested time
    let booking_row = sqlx::query(
        "SELECT COUNT(*) as cnt
        FROM bookings
        WHERE artist_id = $1
        AND booking_date = $2
        AND status NOT IN ('cancelled', 'rejected')"
    )
    .bind(artist_id)
    .bind(requested_date)
    .fetch_one(pool)
    .await?;

    let booking_conflicts: i64 = booking_row.get("cnt");

    if booking_conflicts > 0 {
        return Ok(false); // Time slot already booked
    }

    // Check for artist availability restrictions
    let availability_row = sqlx::query(
        "SELECT COUNT(*) as cnt
        FROM artist_availability
        WHERE artist_id = $1
        AND (specific_date = $2 OR day_of_week = EXTRACT(DOW FROM $2::date)::text)
        AND is_available = false"
    )
    .bind(artist_id)
    .bind(requested_date)
    .fetch_one(pool)
    .await?;

    let availability_blocks: i64 = availability_row.get("cnt");

    if availability_blocks > 0 {
        return Ok(false); // Time slot is blocked by artist
    }

    Ok(true) // Time slot appears to be available
}

#[cfg(feature = "ssr")]
pub async fn get_all_default_questions() -> DbResult<Vec<QuestionnaireQuestion>> {
    let pool = crate::db::pool::get_pool();

    let rows = sqlx::query(
        "SELECT id, question_type, question_text, is_default,
               options_data, validation_rules, created_at
        FROM questionnaire_questions
        WHERE is_default = true
        ORDER BY id"
    )
    .fetch_all(pool)
    .await?;

    let questions = rows
        .into_iter()
        .map(|row| QuestionnaireQuestion {
            id: row.get("id"),
            question_type: row.get("question_type"),
            question_text: row.get("question_text"),
            is_default: row.get("is_default"),
            options_data: row.try_get("options_data").ok(),
            validation_rules: row.try_get("validation_rules").ok(),
            created_at: row.get("created_at"),
        })
        .collect();

    Ok(questions)
}

#[cfg(feature = "ssr")]
pub async fn get_artist_questionnaire_config(artist_id: i32) -> DbResult<Vec<ArtistQuestionnaire>> {
    let pool = crate::db::pool::get_pool();

    let rows = sqlx::query(
        "SELECT id, artist_id, question_id, is_required, display_order, custom_options, is_enabled
        FROM artist_questionnaires
        WHERE artist_id = $1
        ORDER BY display_order"
    )
    .bind(artist_id)
    .fetch_all(pool)
    .await?;

    let config = rows
        .into_iter()
        .map(|row| ArtistQuestionnaire {
            id: row.get("id"),
            artist_id: row.get("artist_id"),
            question_id: row.get("question_id"),
            is_required: row.get("is_required"),
            display_order: row.get("display_order"),
            custom_options: row.try_get("custom_options").ok(),
            is_enabled: row.get("is_enabled"),
        })
        .collect();

    Ok(config)
}

#[cfg(feature = "ssr")]
pub async fn update_artist_questionnaire_config(
    artist_id: i32,
    config: Vec<ArtistQuestionnaire>
) -> DbResult<()> {
    let pool = crate::db::pool::get_pool();

    // Start a transaction
    let mut tx = pool.begin().await?;

    // Remove existing config
    sqlx::query("DELETE FROM artist_questionnaires WHERE artist_id = $1")
        .bind(artist_id)
        .execute(&mut *tx)
        .await?;

    // Insert new config
    for item in config {
        sqlx::query(
            "INSERT INTO artist_questionnaires
             (artist_id, question_id, is_required, display_order, custom_options, is_enabled)
             VALUES ($1, $2, $3, $4, $5, $6)"
        )
        .bind(item.artist_id)
        .bind(item.question_id)
        .bind(item.is_required)
        .bind(item.display_order)
        .bind(item.custom_options)
        .bind(item.is_enabled)
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;

    Ok(())
}

#[cfg(feature = "ssr")]
pub async fn get_artist_id_from_user_id(user_id: i32) -> DbResult<Option<i32>> {
    let pool = crate::db::pool::get_pool();

    let result = sqlx::query(
        "SELECT artist_id FROM artist_users WHERE id = $1"
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await?;

    Ok(result.map(|row| row.get("artist_id")))
}

#[cfg(feature = "ssr")]
pub async fn delete_artist_question(
    artist_id: i32,
    question_id: i32
) -> DbResult<()> {
    let pool = crate::db::pool::get_pool();

    sqlx::query("DELETE FROM artist_questionnaires WHERE artist_id = $1 AND question_id = $2")
        .bind(artist_id)
        .bind(question_id)
        .execute(pool)
        .await?;

    Ok(())
}

#[cfg(feature = "ssr")]
pub async fn save_questionnaire_responses(
    booking_request_id: i32,
    responses: Vec<BookingQuestionnaireResponse>
) -> DbResult<()> {
    let pool = crate::db::pool::get_pool();

    for response in responses {
        sqlx::query(
            "INSERT INTO booking_questionnaire_responses
             (booking_request_id, question_id, response_text, response_data)
             VALUES ($1, $2, $3, $4)"
        )
        .bind(booking_request_id)
        .bind(response.question_id)
        .bind(response.response_text)
        .bind(response.response_data)
        .execute(pool)
        .await?;
    }

    Ok(())
}

#[cfg(feature = "ssr")]
pub async fn get_booking_questionnaire_responses(
    booking_request_id: i32
) -> DbResult<Vec<BookingQuestionnaireResponse>> {
    let pool = crate::db::pool::get_pool();

    let rows = sqlx::query(
        "SELECT id, booking_request_id, question_id, response_text, response_data, created_at
        FROM booking_questionnaire_responses
        WHERE booking_request_id = $1
        ORDER BY id"
    )
    .bind(booking_request_id)
    .fetch_all(pool)
    .await?;

    let responses = rows
        .into_iter()
        .map(|row| BookingQuestionnaireResponse {
            id: row.get("id"),
            booking_request_id: row.get("booking_request_id"),
            question_id: row.get("question_id"),
            response_text: row.try_get("response_text").ok(),
            response_data: row.try_get("response_data").ok(),
            created_at: row.get("created_at"),
        })
        .collect();

    Ok(responses)
}

// Error Logging Functions
#[cfg(feature = "ssr")]
pub async fn log_error(error_data: CreateErrorLog) -> DbResult<i64> {
    let pool = crate::db::pool::get_pool();

    let row = sqlx::query(
        "INSERT INTO error_logs
         (error_type, error_level, error_message, error_stack, url_path,
          user_agent, user_id, session_id, request_headers, additional_context)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
         RETURNING id"
    )
    .bind(error_data.error_type)
    .bind(error_data.error_level)
    .bind(error_data.error_message)
    .bind(error_data.error_stack)
    .bind(error_data.url_path)
    .bind(error_data.user_agent)
    .bind(error_data.user_id)
    .bind(error_data.session_id)
    .bind(error_data.request_headers)
    .bind(error_data.additional_context)
    .fetch_one(pool)
    .await?;

    Ok(row.get("id"))
}

#[cfg(feature = "ssr")]
pub async fn get_recent_errors(limit: i32) -> DbResult<Vec<ErrorLog>> {
    let pool = crate::db::pool::get_pool();

    let rows = sqlx::query(
        "SELECT id, error_type, error_level, error_message, error_stack,
               url_path, user_agent, user_id, session_id, timestamp,
               request_headers, additional_context
        FROM error_logs
        ORDER BY timestamp DESC
        LIMIT $1"
    )
    .bind(limit)
    .fetch_all(pool)
    .await?;

    let errors = rows
        .into_iter()
        .map(|row| ErrorLog {
            id: row.get("id"),
            error_type: row.get("error_type"),
            error_level: row.get("error_level"),
            error_message: row.get("error_message"),
            error_stack: row.try_get("error_stack").ok(),
            url_path: row.try_get("url_path").ok(),
            user_agent: row.try_get("user_agent").ok(),
            user_id: row.try_get("user_id").ok(),
            session_id: row.try_get("session_id").ok(),
            timestamp: row.get("timestamp"),
            request_headers: row.try_get("request_headers").ok(),
            additional_context: row.try_get("additional_context").ok(),
        })
        .collect();

    Ok(errors)
}

#[cfg(feature = "ssr")]
pub async fn get_errors_by_type(error_type: String, limit: i32) -> DbResult<Vec<ErrorLog>> {
    let pool = crate::db::pool::get_pool();

    let rows = sqlx::query(
        "SELECT id, error_type, error_level, error_message, error_stack,
               url_path, user_agent, user_id, session_id, timestamp,
               request_headers, additional_context
        FROM error_logs
        WHERE error_type = $1
        ORDER BY timestamp DESC
        LIMIT $2"
    )
    .bind(error_type)
    .bind(limit)
    .fetch_all(pool)
    .await?;

    let errors = rows
        .into_iter()
        .map(|row| ErrorLog {
            id: row.get("id"),
            error_type: row.get("error_type"),
            error_level: row.get("error_level"),
            error_message: row.get("error_message"),
            error_stack: row.try_get("error_stack").ok(),
            url_path: row.try_get("url_path").ok(),
            user_agent: row.try_get("user_agent").ok(),
            user_id: row.try_get("user_id").ok(),
            session_id: row.try_get("session_id").ok(),
            timestamp: row.get("timestamp"),
            request_headers: row.try_get("request_headers").ok(),
            additional_context: row.try_get("additional_context").ok(),
        })
        .collect();

    Ok(errors)
}

// Paginated and filtered image queries for shop pages
#[cfg(feature = "ssr")]
pub async fn get_shop_images_paginated(
    location_id: i32,
    style_ids: Option<Vec<i32>>,
    page: i32,
    per_page: i32,
) -> DbResult<(Vec<(ArtistImage, Vec<Style>, Artist)>, i32)> {
    let pool = crate::db::pool::get_pool();

    // Build the WHERE clause for style filtering
    let (count_query, data_query) = if let Some(ref ids) = style_ids {
        if !ids.is_empty() {
            (
                format!(
                    "SELECT COUNT(DISTINCT ai.id)
                     FROM artists_images ai
                     JOIN artists a ON ai.artist_id = a.id
                     JOIN locations l ON a.location_id = l.id
                     WHERE a.location_id = $1
                     AND (l.is_person IS NULL OR l.is_person != 1)
                     AND a.name IS NOT NULL
                     AND a.name != ''
                     AND ai.id IN (SELECT ais.artists_images_id FROM artists_images_styles ais WHERE ais.style_id = ANY($2::int[]))"
                ),
                format!(
                    "SELECT ai.id, ai.short_code, ai.artist_id, a.id as a_id, a.name, a.location_id, a.social_links, a.email, a.phone, a.years_experience, a.styles_extracted
                     FROM artists_images ai
                     JOIN artists a ON ai.artist_id = a.id
                     JOIN locations l ON a.location_id = l.id
                     WHERE a.location_id = $1
                     AND (l.is_person IS NULL OR l.is_person != 1)
                     AND a.name IS NOT NULL
                     AND a.name != ''
                     AND ai.id IN (SELECT ais.artists_images_id FROM artists_images_styles ais WHERE ais.style_id = ANY($2::int[]))
                     ORDER BY ai.id DESC
                     LIMIT $3 OFFSET $4"
                )
            )
        } else {
            (
                "SELECT COUNT(DISTINCT ai.id)
                 FROM artists_images ai
                 JOIN artists a ON ai.artist_id = a.id
                 JOIN locations l ON a.location_id = l.id
                 WHERE a.location_id = $1
                 AND (l.is_person IS NULL OR l.is_person != 1)
                 AND a.name IS NOT NULL
                 AND a.name != ''".to_string(),
                "SELECT ai.id, ai.short_code, ai.artist_id, a.id as a_id, a.name, a.location_id, a.social_links, a.email, a.phone, a.years_experience, a.styles_extracted
                 FROM artists_images ai
                 JOIN artists a ON ai.artist_id = a.id
                 JOIN locations l ON a.location_id = l.id
                 WHERE a.location_id = $1
                 AND (l.is_person IS NULL OR l.is_person != 1)
                 AND a.name IS NOT NULL
                 AND a.name != ''
                 ORDER BY ai.id DESC
                 LIMIT $2 OFFSET $3".to_string()
            )
        }
    } else {
        (
            "SELECT COUNT(DISTINCT ai.id)
             FROM artists_images ai
             JOIN artists a ON ai.artist_id = a.id
             JOIN locations l ON a.location_id = l.id
             WHERE a.location_id = $1
             AND (l.is_person IS NULL OR l.is_person != 1)
             AND a.name IS NOT NULL
             AND a.name != ''".to_string(),
            "SELECT ai.id, ai.short_code, ai.artist_id, a.id as a_id, a.name, a.location_id, a.social_links, a.email, a.phone, a.years_experience, a.styles_extracted
             FROM artists_images ai
             JOIN artists a ON ai.artist_id = a.id
             JOIN locations l ON a.location_id = l.id
             WHERE a.location_id = $1
             AND (l.is_person IS NULL OR l.is_person != 1)
             AND a.name IS NOT NULL
             AND a.name != ''
             ORDER BY ai.id DESC
             LIMIT $2 OFFSET $3".to_string()
        )
    };

    // Get total count
    let total_count: i64 = if let Some(ref ids) = style_ids {
        if !ids.is_empty() {
            sqlx::query(&count_query)
                .bind(location_id)
                .bind(ids)
                .fetch_one(pool)
                .await?
                .try_get(0)?
        } else {
            sqlx::query(&count_query)
                .bind(location_id)
                .fetch_one(pool)
                .await?
                .try_get(0)?
        }
    } else {
        sqlx::query(&count_query)
            .bind(location_id)
            .fetch_one(pool)
            .await?
            .try_get(0)?
    };

    // Get paginated images
    let offset = page * per_page;
    let image_rows = if let Some(ref ids) = style_ids {
        if !ids.is_empty() {
            sqlx::query(&data_query)
                .bind(location_id)
                .bind(ids)
                .bind(per_page)
                .bind(offset)
                .fetch_all(pool)
                .await?
        } else {
            sqlx::query(&data_query)
                .bind(location_id)
                .bind(per_page)
                .bind(offset)
                .fetch_all(pool)
                .await?
        }
    } else {
        sqlx::query(&data_query)
            .bind(location_id)
            .bind(per_page)
            .bind(offset)
            .fetch_all(pool)
            .await?
    };

    let mut result: Vec<(ArtistImage, Vec<Style>, Artist)> = vec![];

    for image_row in image_rows {
        let image = ArtistImage {
            id: image_row.get("id"),
            short_code: image_row.get("short_code"),
            artist_id: image_row.get("artist_id"),
        };

        let artist = Artist {
            id: image_row.get("a_id"),
            name: image_row.get("name"),
            location_id: image_row.get("location_id"),
            social_links: image_row.get("social_links"),
            email: image_row.get("email"),
            phone: image_row.get("phone"),
            years_experience: image_row.get("years_experience"),
            styles_extracted: image_row.get("styles_extracted"),
        };

        let img_id = image.id;

        // Get styles for this image
        let style_rows = sqlx::query(
            "SELECT s.id, s.name
             FROM styles s
             JOIN artists_images_styles ais ON s.id = ais.style_id
             WHERE ais.artists_images_id = $1"
        )
        .bind(img_id)
        .fetch_all(pool)
        .await?;

        let styles: Vec<Style> = style_rows
            .into_iter()
            .map(|row| Style {
                id: row.get("id"),
                name: row.get("name"),
            })
            .collect();

        result.push((image, styles, artist));
    }

    Ok((result, total_count as i32))
}

// Paginated and filtered image queries for artist pages
#[cfg(feature = "ssr")]
pub async fn get_artist_images_paginated(
    artist_id: i32,
    style_ids: Option<Vec<i32>>,
    page: i32,
    per_page: i32,
) -> DbResult<(Vec<(ArtistImage, Vec<Style>)>, i32)> {
    let pool = crate::db::pool::get_pool();

    // Build the WHERE clause for style filtering
    let (count_query, data_query) = if let Some(ref ids) = style_ids {
        if !ids.is_empty() {
            (
                format!(
                    "SELECT COUNT(DISTINCT ai.id)
                     FROM artists_images ai
                     WHERE ai.artist_id = $1
                     AND ai.id IN (SELECT ais.artists_images_id FROM artists_images_styles ais WHERE ais.style_id = ANY($2::int[]))"
                ),
                format!(
                    "SELECT ai.id, ai.short_code, ai.artist_id
                     FROM artists_images ai
                     WHERE ai.artist_id = $1
                     AND ai.id IN (SELECT ais.artists_images_id FROM artists_images_styles ais WHERE ais.style_id = ANY($2::int[]))
                     ORDER BY ai.id DESC
                     LIMIT $3 OFFSET $4"
                )
            )
        } else {
            (
                "SELECT COUNT(DISTINCT ai.id)
                 FROM artists_images ai
                 WHERE ai.artist_id = $1".to_string(),
                "SELECT ai.id, ai.short_code, ai.artist_id
                 FROM artists_images ai
                 WHERE ai.artist_id = $1
                 ORDER BY ai.id DESC
                 LIMIT $2 OFFSET $3".to_string()
            )
        }
    } else {
        (
            "SELECT COUNT(DISTINCT ai.id)
             FROM artists_images ai
             WHERE ai.artist_id = $1".to_string(),
            "SELECT ai.id, ai.short_code, ai.artist_id
             FROM artists_images ai
             WHERE ai.artist_id = $1
             ORDER BY ai.id DESC
             LIMIT $2 OFFSET $3".to_string()
        )
    };

    // Get total count
    let total_count: i64 = if let Some(ref ids) = style_ids {
        if !ids.is_empty() {
            sqlx::query(&count_query)
                .bind(artist_id)
                .bind(ids)
                .fetch_one(pool)
                .await?
                .try_get(0)?
        } else {
            sqlx::query(&count_query)
                .bind(artist_id)
                .fetch_one(pool)
                .await?
                .try_get(0)?
        }
    } else {
        sqlx::query(&count_query)
            .bind(artist_id)
            .fetch_one(pool)
            .await?
            .try_get(0)?
    };

    // Get paginated images
    let offset = page * per_page;
    let image_rows = if let Some(ref ids) = style_ids {
        if !ids.is_empty() {
            sqlx::query(&data_query)
                .bind(artist_id)
                .bind(ids)
                .bind(per_page)
                .bind(offset)
                .fetch_all(pool)
                .await?
        } else {
            sqlx::query(&data_query)
                .bind(artist_id)
                .bind(per_page)
                .bind(offset)
                .fetch_all(pool)
                .await?
        }
    } else {
        sqlx::query(&data_query)
            .bind(artist_id)
            .bind(per_page)
            .bind(offset)
            .fetch_all(pool)
            .await?
    };

    let mut result: Vec<(ArtistImage, Vec<Style>)> = vec![];

    for image_row in image_rows {
        let image = ArtistImage {
            id: image_row.get("id"),
            short_code: image_row.get("short_code"),
            artist_id: image_row.get("artist_id"),
        };

        let img_id = image.id;

        // Get styles for this image
        let style_rows = sqlx::query(
            "SELECT s.id, s.name
             FROM styles s
             JOIN artists_images_styles ais ON s.id = ais.style_id
             WHERE ais.artists_images_id = $1"
        )
        .bind(img_id)
        .fetch_all(pool)
        .await?;

        let styles: Vec<Style> = style_rows
            .into_iter()
            .map(|row| Style {
                id: row.get("id"),
                name: row.get("name"),
            })
            .collect();

        result.push((image, styles));
    }

    Ok((result, total_count as i32))
}
