use std::env;

use dotenv::dotenv;
use sqlx::PgPool;

pub mod actions;
pub mod repository;

enum IngestAction {
    Scrape,
    GoogleApi,
    ExtractStyles,
    RedditScraper,
}

impl IngestAction {
    fn new(action: &str) -> Self {
        match action {
            "SCRAPE_HTML" => Self::Scrape,
            "GOOGLE_API" => Self::GoogleApi,
            "EXTRACT_STYLES" => Self::ExtractStyles,
            "REDDIT_SCRAPER" => Self::RedditScraper,
            _ => panic!("Invalid action"),
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    let action: String = env::var("ACTION").expect("Action to be set");
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let pool = PgPool::connect(&database_url)
        .await
        .expect("Failed to connect to PostgreSQL");

    match IngestAction::new(&action) {
        IngestAction::Scrape => actions::scraper::scrape(&pool).await,
        IngestAction::GoogleApi => {
            actions::google_api_ingestion::driver::ingest_google(&pool).await
        }
        IngestAction::ExtractStyles => actions::style_extraction::extract_styles(&pool).await,
        IngestAction::RedditScraper => actions::reddit_scraper::run_reddit_scraper(&pool).await,
    }
}
