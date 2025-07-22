use std::{env, path::Path};

use data_fetcher::fetch_data;
use data_parser::{parse_data, ParsedLocationData};
use dotenv::dotenv;
use repository::{fetch_county_boundaries, mark_county_ingested, upsert_locations};
use rusqlite::Connection;
use serde_json::Value;
use shared_types::CountyBoundary;

pub mod data_fetcher;
pub mod data_parser;
pub mod repository;
pub mod scraper;

enum IngestAction {
    Scrape,
    GoogleApi,
}

impl IngestAction {
    fn new(action: &str) -> Self {
        match action {
            "SCRAPE_HTML" => IngestAction::Scrape,
            "GOOGLE_API" => IngestAction::GoogleApi,
            _ => panic!("Invalid action"),
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    let action: String = env::var("ACTION").expect("Action to be set");
    let db_path = Path::new("tatteau.db");
    let conn: Connection = Connection::open(db_path).expect("Database should load");

    match IngestAction::new(&action) {
        IngestAction::Scrape => scraper::scrape(conn).await,
        IngestAction::GoogleApi => ingest_google(&conn).await,
    }
}

async fn ingest_google(conn: &Connection) -> Result<(), Box<dyn std::error::Error>> {
    let limit_results_to: i8 = 20;
    let max_iter: i8 = 10;

    let county_limit: i16 = 3500;
    let days_till_refetch: i16 = 160;
    let county_boundaries: Vec<CountyBoundary> =
        fetch_county_boundaries(conn, county_limit, days_till_refetch)
            .expect("County boundaries should be fetched");

    if county_boundaries.is_empty() {
        println!("No county boundaries found, exiting.");
        return Ok(());
    }

    for county_boundary in county_boundaries {
        println!("Processing county: {}", county_boundary.name);

        if let Err(e) = process_county(conn, &county_boundary, limit_results_to, max_iter).await {
            println!("Error processing county {}: {}", county_boundary.name, e);
        }

        mark_county_ingested(conn, &county_boundary)?;
    }
    Ok(())
}
async fn process_county(
    conn: &Connection,
    county_boundary: &CountyBoundary,
    limit_results_to: i8,
    max_iter: i8,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut current_token: Option<String> = None;
    let mut curr_iter = 0;
    while curr_iter < max_iter {
        curr_iter += 1;

        let res: Value = fetch_data(county_boundary, limit_results_to, &current_token).await;
        let parsed_data_opt: Option<ParsedLocationData> = parse_data(&res);
        if let Some(parsed_data) = parsed_data_opt {
            let ParsedLocationData {
                next_token,
                location_info,
                filtered_count,
            } = parsed_data;
            println!(
                "Found {} and filtered {} results out of {}",
                location_info.len(),
                filtered_count,
                limit_results_to
            );

            current_token = next_token.map(|s| s.to_string());
            let _ = upsert_locations(conn, &location_info);
            println!("Inserted {} locations", location_info.len());
        }

        if current_token.is_none() {
            break;
        }
    }

    Ok(())
}
