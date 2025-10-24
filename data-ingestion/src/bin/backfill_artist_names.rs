// Backfill Artist Names
// This script finds all artists with NULL names but valid Instagram handles,
// fetches their current Instagram profile info via Apify, and updates the name field.

use data_ingestion::actions::apify_scraper;
use sqlx::{PgPool, Row};
use std::env;

#[derive(Debug)]
struct ArtistToBackfill {
    id: i64,
    instagram_handle: String,
}

async fn get_artists_without_names(pool: &PgPool) -> Result<Vec<ArtistToBackfill>, sqlx::Error> {
    let rows = sqlx::query(
        "SELECT id, instagram_handle
         FROM artists
         WHERE instagram_handle IS NOT NULL
           AND name IS NULL
         ORDER BY id",
    )
    .fetch_all(pool)
    .await?;

    let artists: Vec<ArtistToBackfill> = rows
        .into_iter()
        .map(|row| ArtistToBackfill {
            id: row.get("id"),
            instagram_handle: row.get("instagram_handle"),
        })
        .collect();

    Ok(artists)
}

async fn update_artist_name(
    pool: &PgPool,
    artist_id: i64,
    name: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE artists
         SET name = $1,
             updated_at = CURRENT_TIMESTAMP
         WHERE id = $2",
    )
    .bind(name)
    .bind(artist_id)
    .execute(pool)
    .await?;

    Ok(())
}

fn normalize_instagram_handle(handle: &str) -> String {
    handle
        .trim()
        .trim_start_matches('@')
        .trim_start_matches("https://")
        .trim_start_matches("http://")
        .trim_start_matches("www.")
        .trim_start_matches("instagram.com/")
        .trim_end_matches('/')
        .split('?')
        .next()
        .unwrap_or("")
        .to_string()
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔄 Starting Artist Name Backfill Script\n");

    // Load environment variables
    dotenv::dotenv().ok();

    // Get database URL
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    // Connect to database
    println!("📦 Connecting to database...");
    let pool = PgPool::connect(&database_url).await?;
    println!("✅ Connected to database\n");

    // Get artists without names
    println!("🔍 Querying artists without names...");
    let artists = get_artists_without_names(&pool).await?;
    println!("📊 Found {} artists without names\n", artists.len());

    if artists.is_empty() {
        println!("✅ All artists have names! Nothing to backfill.");
        return Ok(());
    }

    // Process each artist
    let mut stats = BackfillStats {
        total: artists.len(),
        updated: 0,
        failed: 0,
        empty_names: 0,
    };

    for (index, artist) in artists.iter().enumerate() {
        let progress = index + 1;
        println!(
            "👤 [{}/{}] Processing artist ID {} (@{})...",
            progress, stats.total, artist.id, artist.instagram_handle
        );

        // Normalize handle
        let normalized_handle = normalize_instagram_handle(&artist.instagram_handle);
        println!("   📝 Normalized handle: @{}", normalized_handle);

        // Get Instagram profile
        print!("   📱 Fetching Instagram profile... ");
        let profile = match apify_scraper::get_instagram_profile_info(&normalized_handle).await {
            Ok(p) => {
                println!("✅");
                p
            }
            Err(e) => {
                println!("❌");
                eprintln!("   ⚠️  Error: {}", e);
                stats.failed += 1;
                println!();
                continue;
            }
        };

        // Extract and validate name
        let artist_name = match profile.full_name {
            Some(name) if !name.trim().is_empty() => name,
            Some(_) => {
                println!("   ⚠️  Instagram profile has empty fullName");
                stats.empty_names += 1;
                println!();
                continue;
            }
            None => {
                println!("   ⚠️  Instagram profile has no fullName field");
                stats.empty_names += 1;
                println!();
                continue;
            }
        };

        println!("   📝 Found name: '{}'", artist_name);

        // Update database
        print!("   💾 Updating database... ");
        match update_artist_name(&pool, artist.id, &artist_name).await {
            Ok(_) => {
                println!("✅");
                stats.updated += 1;
                println!("   ✅ Successfully updated artist ID {} with name '{}'", artist.id, artist_name);
            }
            Err(e) => {
                println!("❌");
                eprintln!("   ⚠️  Database error: {}", e);
                stats.failed += 1;
            }
        }

        println!();
    }

    // Print final summary
    println!("\n{}", "=".repeat(60));
    println!("📊 BACKFILL SUMMARY");
    println!("{}", "=".repeat(60));
    println!("Total artists processed:  {}", stats.total);
    println!("✅ Successfully updated:   {}", stats.updated);
    println!("⚠️  Empty/missing names:   {}", stats.empty_names);
    println!("❌ Failed:                 {}", stats.failed);
    println!("{}\n", "=".repeat(60));

    if stats.updated > 0 {
        println!("🎉 Backfill complete! {} artist names updated.", stats.updated);
    } else {
        println!("⚠️  No artist names were updated.");
    }

    Ok(())
}

struct BackfillStats {
    total: usize,
    updated: usize,
    failed: usize,
    empty_names: usize,
}
