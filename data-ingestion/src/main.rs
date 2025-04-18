use data_fetcher::fetch_data;
use data_parser::{parse_data, ParsedLocationData};
use data_persister::upsert_locations;
use dotenv::dotenv;
use geographical_location::USCounties;
use rusqlite::Connection;
use serde_json::Value;

pub mod data_fetcher;
pub mod data_parser;
pub mod data_persister;
pub mod geographical_location;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    let location = USCounties::TXEllisCounty;
    let limit_results_to: i8 = 20;
    let mut current_token: Option<String> = None;
    let mut max_iter: i8 = 5;
    let conn: Connection = Connection::open("tatteau.db").expect("Database should load");

    while max_iter > 0 {
        max_iter -= 1;

        let res: Value = fetch_data(&location, limit_results_to, &current_token).await;
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
            let _ = upsert_locations(&conn, &location_info);
            println!("Inserted {} locations", location_info.len());
        }

        if current_token.is_none() {
            break;
        }
    }
    Ok(())
}
