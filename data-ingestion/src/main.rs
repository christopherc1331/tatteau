use std::{env, path::Path};

use dotenv::dotenv;
use rusqlite::Connection;

pub mod actions;
pub mod repository;

enum IngestAction {
    Scrape,
    GoogleApi,
    ExtractStyles,
}

impl IngestAction {
    fn new(action: &str) -> Self {
        match action {
            "SCRAPE_HTML" => Self::Scrape,
            "GOOGLE_API" => Self::GoogleApi,
            "EXTRACT_STYLES" => Self::ExtractStyles,
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
        IngestAction::Scrape => actions::scraper::scrape(conn).await,
        IngestAction::GoogleApi => {
            actions::google_api_ingestion::driver::ingest_google(&conn).await
        }
        IngestAction::ExtractStyles => actions::style_extraction::extract_styles(conn).await,
    }
}
