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

pub async fn get_all_styles(pool: &PgPool) -> Result<Vec<String>, sqlx::Error> {
    let rows = sqlx::query("SELECT name FROM styles ORDER BY name")
        .fetch_all(pool)
        .await?;

    let styles: Vec<String> = rows
        .into_iter()
        .map(|row| row.get("name"))
        .collect();

    Ok(styles)
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
) -> Result<i64, sqlx::Error> {
    let row = sqlx::query(
        "INSERT INTO artists_images (short_code, artist_id) VALUES ($1, $2) RETURNING id"
    )
    .bind(short_code)
    .bind(artist_id)
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
            let existing_total: f64 = row.try_get::<f32, _>("total_cost").unwrap_or(0.0) as f64;
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
