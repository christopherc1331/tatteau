use chrono::{DateTime, Utc};
use rusqlite::{params, Connection, Error};

use shared_types::{CountyBoundary, LocationInfo};

pub fn upsert_locations(conn: &Connection, locations: &[LocationInfo]) -> Result<(), Error> {
    let mut stmt = conn.prepare_cached(
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
                    VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)
            ON CONFLICT DO UPDATE
            SET 
                city = excluded.city,
                county = excluded.county,
                state = excluded.state,
                country_code = excluded.country_code,
                postal_code = excluded.postal_code,
                is_open = excluded.is_open,
                address = excluded.address,
                category = excluded.category,
                name = excluded.name,
                website_uri = excluded.website_uri,
                lat = excluded.lat,
                long = excluded.long
        ",
    )?;
    let t = conn.unchecked_transaction()?;
    locations.iter().for_each(|li| {
        stmt.execute(params![
            li.city,
            li.county,
            li.state,
            li.country_code,
            li.postal_code,
            li.is_open,
            li.address,
            li._id,
            li.category,
            li.name,
            li.website_uri,
            li.lat,
            li.long,
        ])
        .expect("query to execute");
    });
    t.commit()
}

pub fn fetch_county_boundaries(
    conn: &Connection,
    limit: i16,
    days_till_refetch: i16,
) -> Result<Vec<CountyBoundary>, Error> {
    let now: DateTime<Utc> = Utc::now();
    let date_cutoff: DateTime<Utc> = now - chrono::Duration::days(days_till_refetch as i64);
    let date_cutoff_timestamp: i64 = date_cutoff.timestamp();
    let mut stmt = conn.prepare(
        "
            SELECT 
                name,
                low_lat,
                low_long,
                high_lat,
                high_long,
                date_utc_last_ingested
            FROM county_boundaries
            WHERE date_utc_last_ingested IS NULL OR date_utc_last_ingested < ?1
            LIMIT ?2
        ",
    )?;

    let county_boundaries = stmt.query_map(params![date_cutoff_timestamp, limit], |row| {
        Ok(CountyBoundary {
            name: row.get(0)?,
            low_lat: row.get(1)?,
            low_long: row.get(2)?,
            high_lat: row.get(3)?,
            high_long: row.get(4)?,
            date_utc_last_ingested: row.get(5)?,
        })
    });

    county_boundaries
        .map(|res| res.collect::<Result<Vec<CountyBoundary>, Error>>())
        .expect("County boundaries to be fetched")
}

pub fn mark_county_ingested(
    conn: &Connection,
    county_boundary: &CountyBoundary,
) -> Result<(), Error> {
    let now: DateTime<Utc> = Utc::now();
    let now_timestamp: i64 = now.timestamp();
    let mut stmt = conn.prepare(
        "
            UPDATE county_boundaries
            SET date_utc_last_ingested = ?1
            WHERE name = ?2;
        ",
    )?;

    stmt.execute(params![now_timestamp, county_boundary.name])?;

    Ok(())
}

pub fn mark_locations_scraped(conn: &Connection, ids: Vec<i64>) -> Result<(), Error> {
    let mut stmt = conn.prepare(
        "
            UPDATE locations
            SET scraped_html = 1
            WHERE id IN (?2);
        ",
    )?;

    stmt.execute(params![ids
        .iter()
        .map(|i| i.to_string())
        .collect::<Vec<String>>()
        .join(",")])?;

    Ok(())
}

#[derive(Debug)]
pub struct Artist {
    pub id: i64,
    pub name: String,
    pub images_dir: String,
}

pub struct LocationUris {
    pub id: i64,
    website_uri: String,
}

pub fn get_locations_to_scrape(conn: &Connection, limit: i16) -> Result<Vec<LocationUris>, Error> {
    let mut stmt = conn.prepare(
        "
            SELECT id, website_uri
            FROM locations
            WHERE website_uri IS NOT NULL
              AND TRIM(website_uri) != ''
              AND scraped_html = 0
            LIMIT ?1
        ",
    )?;

    let location_uris = stmt.query_map(params![limit], |row| {
        Ok(LocationUris {
            id: row.get(0)?,
            website_uri: row.get(1)?,
        })
    });

    location_uris
        .map(|res| res.collect::<Result<Vec<LocationUris>, Error>>())
        .expect("Location URIs to be fetched")
}

pub fn get_artists_for_style_extraction(conn: &Connection, limit: i16) -> Result<Vec<Artist>, Error> {
    let mut stmt = conn.prepare(
        "
            SELECT id, name, images_dir
            FROM artists 
            WHERE images_dir IS NOT NULL 
              AND TRIM(images_dir) != ''
              AND (styles_extracted IS NULL OR styles_extracted != 1)
            LIMIT ?1
        ",
    )?;

    let artists = stmt.query_map(params![limit], |row| {
        Ok(Artist {
            id: row.get(0)?,
            name: row.get(1)?,
            images_dir: row.get(2)?,
        })
    });

    artists
        .map(|res| res.collect::<Result<Vec<Artist>, Error>>())
        .expect("Artists to be fetched")
}

pub fn get_or_create_style_ids(conn: &Connection, style_names: &[String]) -> Result<Vec<i64>, Error> {
    let mut style_ids = Vec::new();
    
    for style_name in style_names {
        // First try to find existing style
        let mut stmt = conn.prepare("SELECT id FROM styles WHERE name = ?1")?;
        let existing_id: Result<i64, Error> = stmt.query_row(params![style_name], |row| {
            Ok(row.get(0)?)
        });
        
        match existing_id {
            Ok(id) => style_ids.push(id),
            Err(_) => {
                // Style doesn't exist, create it
                let mut insert_stmt = conn.prepare("INSERT INTO styles (name) VALUES (?1)")?;
                insert_stmt.execute(params![style_name])?;
                style_ids.push(conn.last_insert_rowid());
            }
        }
    }
    
    Ok(style_ids)
}

pub fn upsert_artist_styles(conn: &Connection, artist_id: i64, style_ids: &[i64]) -> Result<(), Error> {
    // First, delete existing styles for this artist to avoid duplicates
    let mut delete_stmt = conn.prepare("DELETE FROM artists_styles WHERE artist_id = ?1")?;
    delete_stmt.execute(params![artist_id])?;
    
    // Then insert all the new styles
    let mut insert_stmt = conn.prepare("INSERT INTO artists_styles (artist_id, style_id) VALUES (?1, ?2)")?;
    for style_id in style_ids {
        insert_stmt.execute(params![artist_id, style_id])?;
    }
    
    Ok(())
}

pub fn mark_artist_styles_extracted(conn: &Connection, artist_id: i64) -> Result<(), Error> {
    let mut stmt = conn.prepare("UPDATE artists SET styles_extracted = 1 WHERE id = ?1")?;
    stmt.execute(params![artist_id])?;
    Ok(())
}
