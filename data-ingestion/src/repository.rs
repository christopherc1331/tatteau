use chrono::{DateTime, Utc};
use sqlx::{PgPool, Row};

use shared_types::{CountyBoundary, LocationInfo};

pub async fn upsert_locations(pool: &PgPool, locations: &[LocationInfo]) -> Result<(), sqlx::Error> {
    for li in locations {
        sqlx::query(
            "
                INSERT INTO locations (
                            city,
                            county,
                            state,
                            country_code,
                            postal_code,
                            is_open,
                            address,
                            _id,
                            category,
                            name,
                            website_uri,
                            lat,
                            long
                        )
                        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
                ON CONFLICT (_id) DO UPDATE
                SET
                    city = EXCLUDED.city,
                    county = EXCLUDED.county,
                    state = EXCLUDED.state,
                    country_code = EXCLUDED.country_code,
                    postal_code = EXCLUDED.postal_code,
                    is_open = EXCLUDED.is_open,
                    address = EXCLUDED.address,
                    category = EXCLUDED.category,
                    name = EXCLUDED.name,
                    website_uri = EXCLUDED.website_uri,
                    lat = EXCLUDED.lat,
                    long = EXCLUDED.long
            ",
        )
        .bind(&li.city)
        .bind(&li.county)
        .bind(&li.state)
        .bind(&li.country_code)
        .bind(&li.postal_code)
        .bind(li.is_open)
        .bind(&li.address)
        .bind(&li._id)
        .bind(&li.category)
        .bind(&li.name)
        .bind(&li.website_uri)
        .bind(li.lat)
        .bind(li.long)
        .execute(pool)
        .await?;
    }

    Ok(())
}

pub async fn fetch_county_boundaries(
    pool: &PgPool,
    limit: i16,
    days_till_refetch: i16,
) -> Result<Vec<CountyBoundary>, sqlx::Error> {
    let now: DateTime<Utc> = Utc::now();
    let date_cutoff: DateTime<Utc> = now - chrono::Duration::days(days_till_refetch as i64);
    let date_cutoff_timestamp: i64 = date_cutoff.timestamp();

    let rows = sqlx::query(
        "
            SELECT
                name,
                low_lat,
                low_long,
                high_lat,
                high_long,
                date_utc_last_ingested
            FROM county_boundaries
            WHERE date_utc_last_ingested IS NULL OR date_utc_last_ingested < $1
            LIMIT $2
        ",
    )
    .bind(date_cutoff_timestamp)
    .bind(limit as i32)
    .fetch_all(pool)
    .await?;

    let county_boundaries: Vec<CountyBoundary> = rows
        .into_iter()
        .map(|row| CountyBoundary {
            name: row.get("name"),
            low_lat: row.get("low_lat"),
            low_long: row.get("low_long"),
            high_lat: row.get("high_lat"),
            high_long: row.get("high_long"),
            date_utc_last_ingested: row.get("date_utc_last_ingested"),
        })
        .collect();

    Ok(county_boundaries)
}

pub async fn mark_county_ingested(
    pool: &PgPool,
    county_boundary: &CountyBoundary,
) -> Result<(), sqlx::Error> {
    let now: DateTime<Utc> = Utc::now();
    let now_timestamp: i64 = now.timestamp();

    sqlx::query(
        "
            UPDATE county_boundaries
            SET date_utc_last_ingested = $1
            WHERE name = $2
        ",
    )
    .bind(now_timestamp)
    .bind(&county_boundary.name)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn mark_locations_scraped(pool: &PgPool, ids: Vec<i64>) -> Result<(), sqlx::Error> {
    if ids.is_empty() {
        return Ok(());
    }

    sqlx::query(
        "
            UPDATE locations
            SET scraped_html = 1
            WHERE id = ANY($1)
        ",
    )
    .bind(&ids)
    .execute(pool)
    .await?;

    Ok(())
}

#[derive(Debug)]
pub struct Artist {
    pub id: i64,
    pub name: String,
    pub ig_username: Option<String>,
}

pub struct LocationUris {
    pub id: i64,
    pub website_uri: String,
}

pub async fn get_locations_to_scrape(pool: &PgPool, limit: i16) -> Result<Vec<LocationUris>, sqlx::Error> {
    let rows = sqlx::query(
        "
            SELECT id, website_uri
            FROM locations
            WHERE website_uri IS NOT NULL
              AND TRIM(website_uri) != ''
              AND scraped_html = 0
            LIMIT $1
        ",
    )
    .bind(limit as i32)
    .fetch_all(pool)
    .await?;

    let location_uris: Vec<LocationUris> = rows
        .into_iter()
        .map(|row| LocationUris {
            id: row.get("id"),
            website_uri: row.get("website_uri"),
        })
        .collect();

    Ok(location_uris)
}

fn extract_instagram_username(social_links: &str) -> Option<String> {
    for url in social_links.split(',') {
        let url = url.trim();
        let url_lower = url.to_lowercase();
        if let Some(start) = url_lower.find("instagram.com/") {
            // Use the original URL to extract the username (preserving case)
            let after_domain = &url[start + 14..];

            // Find the end of username - stop at '/', '?' or end of string
            let end = after_domain
                .find(|c| c == '/' || c == '?')
                .unwrap_or(after_domain.len());

            let username = &after_domain[..end].trim();

            // Validate username (no query params, not empty)
            if !username.is_empty() && !username.contains('&') && !username.contains('=') {
                return Some(username.to_string());
            }
        }
    }
    None
}

pub async fn get_artists_for_style_extraction(
    pool: &PgPool,
    limit: i16,
) -> Result<Vec<Artist>, sqlx::Error> {
    let rows = sqlx::query(
        "
            SELECT id, name, social_links
            FROM artists
            WHERE social_links IS NOT NULL
              AND TRIM(social_links) != ''
              AND (LOWER(social_links) LIKE '%instagram.com%')
              AND (styles_extracted IS NULL OR styles_extracted = 0)
            ORDER BY id
            LIMIT $1
        ",
    )
    .bind(limit as i32)
    .fetch_all(pool)
    .await?;

    let artists: Vec<Artist> = rows
        .into_iter()
        .map(|row| {
            let social_links: String = row.get("social_links");
            Artist {
                id: row.get("id"),
                name: row.get("name"),
                ig_username: extract_instagram_username(&social_links),
            }
        })
        .collect();

    Ok(artists)
}

pub async fn get_all_styles(pool: &PgPool) -> Result<std::collections::HashMap<String, Vec<String>>, sqlx::Error> {
    let rows = sqlx::query("SELECT name, type FROM styles ORDER BY type, name")
        .fetch_all(pool)
        .await?;

    let mut styles_by_type: std::collections::HashMap<String, Vec<String>> = std::collections::HashMap::new();

    for row in rows {
        let name: String = row.get("name");
        let style_type: Option<String> = row.get("type");

        if let Some(type_name) = style_type {
            styles_by_type
                .entry(type_name)
                .or_insert_with(Vec::new)
                .push(name);
        }
    }

    Ok(styles_by_type)
}

pub async fn get_style_ids(pool: &PgPool, style_names: &[String]) -> Result<Vec<i64>, sqlx::Error> {
    let mut style_ids = Vec::new();

    for style_name in style_names {
        let result = sqlx::query("SELECT id FROM styles WHERE LOWER(name) = LOWER($1)")
            .bind(style_name)
            .fetch_optional(pool)
            .await?;

        if let Some(row) = result {
            style_ids.push(row.get("id"));
        }
    }

    Ok(style_ids)
}

pub async fn upsert_artist_styles(
    pool: &PgPool,
    artist_id: i64,
    style_ids: &[i64],
) -> Result<(), sqlx::Error> {
    for style_id in style_ids {
        sqlx::query(
            "INSERT INTO artists_styles (artist_id, style_id) VALUES ($1, $2)
             ON CONFLICT (artist_id, style_id) DO NOTHING"
        )
        .bind(artist_id)
        .bind(style_id)
        .execute(pool)
        .await?;
    }

    Ok(())
}

pub async fn mark_artist_styles_extracted(pool: &PgPool, artist_id: i64) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE artists SET styles_extracted = 1 WHERE id = $1")
        .bind(artist_id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn mark_artist_styles_extraction_failed(
    pool: &PgPool,
    artist_id: i64,
) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE artists SET styles_extracted = -1 WHERE id = $1")
        .bind(artist_id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn insert_artist_image(
    pool: &PgPool,
    short_code: &str,
    artist_id: i64,
    post_date: Option<i64>,
) -> Result<i64, sqlx::Error> {
    let row = sqlx::query(
        "INSERT INTO artists_images (short_code, artist_id, post_date) VALUES ($1, $2, $3) RETURNING id"
    )
    .bind(short_code)
    .bind(artist_id)
    .bind(post_date)
    .fetch_one(pool)
    .await?;

    Ok(row.get("id"))
}

pub async fn insert_artist_image_styles(
    pool: &PgPool,
    artist_image_id: i64,
    style_ids: &[i64],
) -> Result<(), sqlx::Error> {
    for style_id in style_ids {
        sqlx::query(
            "INSERT INTO artists_images_styles (artists_images_id, style_id) VALUES ($1, $2)
             ON CONFLICT (artists_images_id, style_id) DO NOTHING"
        )
        .bind(artist_image_id)
        .bind(style_id)
        .execute(pool)
        .await?;
    }
    Ok(())
}

pub async fn update_openai_api_costs(
    pool: &PgPool,
    action: &str,
    model: &str,
    cost: f64,
) -> Result<(), sqlx::Error> {
    let existing = sqlx::query(
        "SELECT count, total_cost FROM openai_api_costs WHERE action = $1 AND model = $2"
    )
    .bind(action)
    .bind(model)
    .fetch_optional(pool)
    .await?;

    match existing {
        Some(row) => {
            let existing_count: i64 = row.get("count");
            let existing_total: f64 = row.get("total_cost");
            let new_total = existing_total + cost;
            let new_count = existing_count + 1;
            let new_avg = new_total / new_count as f64;

            sqlx::query(
                "UPDATE openai_api_costs SET count = $1, avg_cost = $2, total_cost = $3
                 WHERE action = $4 AND model = $5"
            )
            .bind(new_count)
            .bind(new_avg)
            .bind(new_total)
            .bind(action)
            .bind(model)
            .execute(pool)
            .await?;
        }
        None => {
            sqlx::query(
                "INSERT INTO openai_api_costs (action, count, avg_cost, model, total_cost)
                 VALUES ($1, $2, $3, $4, $5)"
            )
            .bind(action)
            .bind(1_i64)
            .bind(cost)
            .bind(model)
            .bind(cost)
            .execute(pool)
            .await?;
        }
    }

    Ok(())
}

// ============================================================================
// Reddit Scraper Repository Functions
// ============================================================================

// --- City Management ---

pub struct CityToScrape {
    pub city: String,
    pub state: String,
}

pub async fn get_cities_for_scrape(
    pool: &PgPool,
    limit: Option<i16>,
    city_filter: Option<String>,
    state_filter: Option<String>,
    rescrape_days: i16,
) -> Result<Vec<CityToScrape>, sqlx::Error> {
    let mut query = String::from(
        "SELECT city, state FROM reddit_scrape_cities WHERE 1=1"
    );

    // Add filters
    if city_filter.is_some() && state_filter.is_some() {
        query.push_str(" AND city = $1 AND state = $2");
    } else if state_filter.is_some() {
        query.push_str(" AND state = $1");
    } else {
        // Get pending or stale cities
        query.push_str(" AND (status = 'pending' OR last_scraped_at IS NULL OR last_scraped_at < NOW() - INTERVAL '1 day' * $1)");
    }

    if let Some(lim) = limit {
        query.push_str(&format!(" LIMIT {}", lim));
    }

    let rows = if let (Some(city), Some(state)) = (&city_filter, &state_filter) {
        sqlx::query(&query)
            .bind(city)
            .bind(state)
            .fetch_all(pool)
            .await?
    } else if let Some(state) = &state_filter {
        sqlx::query(&query)
            .bind(state)
            .fetch_all(pool)
            .await?
    } else {
        sqlx::query(&query)
            .bind(rescrape_days as i32)
            .fetch_all(pool)
            .await?
    };

    let cities: Vec<CityToScrape> = rows
        .into_iter()
        .map(|row| CityToScrape {
            city: row.get("city"),
            state: row.get("state"),
        })
        .collect();

    Ok(cities)
}

pub struct CityStats {
    pub posts_found: i32,
    pub artists_added: i32,
    pub artists_updated: i32,
    pub artists_pending: i32,
    pub artists_added_from_shop_bios: i32,
    pub shops_scraped: i32,
}

pub async fn mark_city_scraped(
    pool: &PgPool,
    city: &str,
    state: &str,
    status: &str,
    stats: &CityStats,
    error_message: Option<&str>,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE reddit_scrape_cities
         SET last_scraped_at = NOW(),
             status = $3,
             posts_found = $4,
             artists_added = $5,
             artists_updated = $6,
             artists_pending = $7,
             artists_added_from_shop_bios = $8,
             shops_scraped = $9,
             error_message = $10
         WHERE city = $1 AND state = $2"
    )
    .bind(city)
    .bind(state)
    .bind(status)
    .bind(stats.posts_found)
    .bind(stats.artists_added)
    .bind(stats.artists_updated)
    .bind(stats.artists_pending)
    .bind(stats.artists_added_from_shop_bios)
    .bind(stats.shops_scraped)
    .bind(error_message)
    .execute(pool)
    .await?;

    Ok(())
}

// --- Shop/Location Matching ---

pub async fn find_location_by_shop_and_city(
    pool: &PgPool,
    shop_name: &str,
    city: &str,
    state: &str,
) -> Result<Option<i64>, sqlx::Error> {
    let result = sqlx::query(
        "SELECT id FROM locations
         WHERE LOWER(name) = LOWER($1)
           AND city = $2
           AND state = $3
         LIMIT 1"
    )
    .bind(shop_name)
    .bind(city)
    .bind(state)
    .fetch_optional(pool)
    .await?;

    Ok(result.map(|row| row.get("id")))
}

// --- Artist Lookups ---

pub struct ArtistWithSocial {
    pub id: i64,
    pub name: Option<String>,
    pub location_id: i64,
    pub social_links: Option<String>,
    pub instagram_handle: Option<String>,
}

pub async fn find_artist_by_instagram_globally(
    pool: &PgPool,
    handle: &str,
) -> Result<Option<ArtistWithSocial>, sqlx::Error> {
    let result = sqlx::query(
        "SELECT id, name, location_id, social_links, instagram_handle
         FROM artists
         WHERE instagram_handle = $1
            OR social_links LIKE '%instagram.com/' || $1 || '%'
         LIMIT 1"
    )
    .bind(handle)
    .fetch_optional(pool)
    .await?;

    Ok(result.map(|row| ArtistWithSocial {
        id: row.get("id"),
        name: row.get("name"),
        location_id: row.get("location_id"),
        social_links: row.get("social_links"),
        instagram_handle: row.get("instagram_handle"),
    }))
}

pub async fn find_artist_by_instagram_at_location(
    pool: &PgPool,
    handle: &str,
    location_id: i64,
) -> Result<Option<ArtistWithSocial>, sqlx::Error> {
    let result = sqlx::query(
        "SELECT id, name, location_id, social_links, instagram_handle
         FROM artists
         WHERE location_id = $1
           AND (instagram_handle = $2 OR social_links LIKE '%instagram.com/' || $2 || '%')
         LIMIT 1"
    )
    .bind(location_id)
    .bind(handle)
    .fetch_optional(pool)
    .await?;

    Ok(result.map(|row| ArtistWithSocial {
        id: row.get("id"),
        name: row.get("name"),
        location_id: row.get("location_id"),
        social_links: row.get("social_links"),
        instagram_handle: row.get("instagram_handle"),
    }))
}

pub async fn find_artist_by_name_at_location(
    pool: &PgPool,
    first: &str,
    last: &str,
    location_id: i64,
) -> Result<Option<ArtistWithSocial>, sqlx::Error> {
    let result = if !last.is_empty() {
        sqlx::query(
            "SELECT id, name, location_id, social_links, instagram_handle
             FROM artists
             WHERE location_id = $1
               AND (LOWER(name) LIKE '%' || $2 || '%' OR LOWER(name) LIKE '%' || $3 || '%')
             LIMIT 1"
        )
        .bind(location_id)
        .bind(first)
        .bind(last)
        .fetch_optional(pool)
        .await?
    } else {
        sqlx::query(
            "SELECT id, name, location_id, social_links, instagram_handle
             FROM artists
             WHERE location_id = $1
               AND LOWER(name) LIKE '%' || $2 || '%'
             LIMIT 1"
        )
        .bind(location_id)
        .bind(first)
        .fetch_optional(pool)
        .await?
    };

    Ok(result.map(|row| ArtistWithSocial {
        id: row.get("id"),
        name: row.get("name"),
        location_id: row.get("location_id"),
        social_links: row.get("social_links"),
        instagram_handle: row.get("instagram_handle"),
    }))
}

// --- Artist Insert/Update ---

pub async fn insert_artist_with_instagram(
    pool: &PgPool,
    name: Option<&str>,
    location_id: i64,
    instagram_handle: &str,
    instagram_url: &str,
) -> Result<i64, sqlx::Error> {
    let row = sqlx::query(
        "INSERT INTO artists (name, location_id, instagram_handle, social_links, styles_extracted)
         VALUES ($1, $2, $3, $4, 0)
         RETURNING id"
    )
    .bind(name)
    .bind(location_id)
    .bind(instagram_handle)
    .bind(instagram_url)
    .fetch_one(pool)
    .await?;

    Ok(row.get("id"))
}

pub async fn update_artist_add_instagram(
    pool: &PgPool,
    artist_id: i64,
    instagram_handle: &str,
    instagram_url: &str,
    existing_social_links: Option<String>,
) -> Result<(), sqlx::Error> {
    let new_social_links = if let Some(existing) = existing_social_links {
        if existing.is_empty() {
            instagram_url.to_string()
        } else {
            format!("{},{}", existing, instagram_url)
        }
    } else {
        instagram_url.to_string()
    };

    sqlx::query(
        "UPDATE artists
         SET social_links = $1,
             instagram_handle = $2
         WHERE id = $3"
    )
    .bind(&new_social_links)
    .bind(instagram_handle)
    .bind(artist_id)
    .execute(pool)
    .await?;

    Ok(())
}

// --- Pending Review ---

pub struct PendingArtistData {
    pub reddit_post_url: Option<String>,
    pub artist_name: Option<String>,
    pub instagram_handle: Option<String>,
    pub shop_name_mentioned: Option<String>,
    pub city: String,
    pub state: String,
    pub post_context: Option<String>,
    pub match_type: String,
}

pub async fn insert_reddit_artist_pending(
    pool: &PgPool,
    data: &PendingArtistData,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO reddit_artists_pending
         (reddit_post_url, artist_name, instagram_handle, shop_name_mentioned,
          city, state, post_context, match_type)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8)"
    )
    .bind(&data.reddit_post_url)
    .bind(&data.artist_name)
    .bind(&data.instagram_handle)
    .bind(&data.shop_name_mentioned)
    .bind(&data.city)
    .bind(&data.state)
    .bind(&data.post_context)
    .bind(&data.match_type)
    .execute(pool)
    .await?;

    Ok(())
}
