use chrono::{DateTime, Utc};
use rusqlite::{params, Connection, Error};

use shared_types::{CountyBoundary, LocationInfo};

pub fn upsert_locations(conn: &Connection, locations: &[LocationInfo]) -> Result<(), Error> {
    let mut stmt = conn.prepare_cached(
        "INSERT OR REPLACE INTO locations (
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
                    VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
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

pub fn get_locations(conn: &Connection, county_boundary: &CountyBoundary) -> Result<(), Error> {
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
