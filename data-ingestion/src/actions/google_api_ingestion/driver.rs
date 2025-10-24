use crate::repository::{fetch_county_boundaries, mark_county_ingested, upsert_locations};
use crate::services::google_places::{parse_places_to_locations, search_text_in_rectangle, LocationBounds};
use shared_types::CountyBoundary;
use sqlx::PgPool;

pub async fn ingest_google(pool: &PgPool) -> Result<(), Box<dyn std::error::Error>> {
    let limit_results_to: i8 = 20;
    let max_iter: i8 = 10;

    let county_limit: i16 = 3500;
    let days_till_refetch: i16 = 160;
    let county_boundaries: Vec<CountyBoundary> =
        fetch_county_boundaries(pool, county_limit, days_till_refetch)
            .await
            .expect("County boundaries should be fetched");

    if county_boundaries.is_empty() {
        println!("No county boundaries found, exiting.");
        return Ok(());
    }

    for county_boundary in county_boundaries {
        println!("Processing county: {}", county_boundary.name);

        if let Err(e) = process_county(pool, &county_boundary, limit_results_to, max_iter).await {
            println!("Error processing county {}: {}", county_boundary.name, e);
        }

        mark_county_ingested(pool, &county_boundary).await?;
    }
    Ok(())
}

async fn process_county(
    pool: &PgPool,
    county_boundary: &CountyBoundary,
    limit_results_to: i8,
    max_iter: i8,
) -> Result<(), Box<dyn std::error::Error>> {
    // Convert CountyBoundary to LocationBounds for service module
    let bounds = LocationBounds {
        low_lat: county_boundary.low_lat as f32,
        low_long: county_boundary.low_long as f32,
        high_lat: county_boundary.high_lat as f32,
        high_long: county_boundary.high_long as f32,
    };

    let mut current_token: Option<String> = None;
    let mut curr_iter = 0;
    while curr_iter < max_iter {
        curr_iter += 1;

        // Use service module for Google Places API call
        let res = search_text_in_rectangle(
            "Tattoo",
            &bounds,
            limit_results_to,
            current_token.as_deref(),
        )
        .await?;

        // Parse response to LocationInfo with filtering
        let location_info = parse_places_to_locations(&res);
        let total_results = res
            .get("places")
            .and_then(|p| p.as_array())
            .map(|a| a.len())
            .unwrap_or(0);
        let filtered_count = total_results - location_info.len();

        println!(
            "Found {} and filtered {} results out of {}",
            location_info.len(),
            filtered_count,
            limit_results_to
        );

        // Extract next page token
        current_token = res
            .get("nextPageToken")
            .and_then(|t| t.as_str())
            .map(|s| s.to_string());

        let _ = upsert_locations(pool, &location_info).await;
        println!("Inserted {} locations", location_info.len());

        if current_token.is_none() {
            break;
        }
    }

    Ok(())
}
