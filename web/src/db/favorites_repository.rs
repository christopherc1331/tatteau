use super::entities::{UserFavorite, CreateUserFavorite, ArtistImage, Artist, Style};
#[cfg(feature = "ssr")]
use rusqlite::{Connection, Result as SqliteResult};
#[cfg(feature = "ssr")]
use std::path::PathBuf;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct FavoritePostWithDetails {
    pub image: ArtistImage,
    pub artist: Option<Artist>,
    pub styles: Vec<Style>,
}

/// Get the database path from environment variable or use default
#[cfg(feature = "ssr")]
fn get_db_path() -> PathBuf {
    std::env::var("DATABASE_PATH")
        .unwrap_or_else(|_| "tatteau.db".to_string())
        .into()
}

/// Add a favorite for a user
#[cfg(feature = "ssr")]
pub fn add_favorite(user_id: i32, artists_images_id: i32) -> SqliteResult<i64> {
    use rusqlite::params;

    let db_path = get_db_path();
    let conn = Connection::open(db_path)?;

    conn.execute(
        "INSERT OR IGNORE INTO user_favorites (user_id, artists_images_id)
         VALUES (?1, ?2)",
        params![user_id, artists_images_id],
    )?;

    Ok(conn.last_insert_rowid())
}

/// Remove a favorite for a user
#[cfg(feature = "ssr")]
pub fn remove_favorite(user_id: i32, artists_images_id: i32) -> SqliteResult<()> {
    use rusqlite::params;

    let db_path = get_db_path();
    let conn = Connection::open(db_path)?;

    conn.execute(
        "DELETE FROM user_favorites
         WHERE user_id = ?1 AND artists_images_id = ?2",
        params![user_id, artists_images_id],
    )?;

    Ok(())
}

/// Check if an image is favorited by a user
#[cfg(feature = "ssr")]
pub fn is_favorited(user_id: i32, artists_images_id: i32) -> SqliteResult<bool> {
    use rusqlite::params;

    let db_path = get_db_path();
    let conn = Connection::open(db_path)?;

    let count: i32 = conn.query_row(
        "SELECT COUNT(*) FROM user_favorites
         WHERE user_id = ?1 AND artists_images_id = ?2",
        params![user_id, artists_images_id],
        |row| row.get(0),
    )?;

    Ok(count > 0)
}

/// Get all favorite image IDs for a user
#[cfg(feature = "ssr")]
pub fn get_user_favorites(user_id: i32) -> SqliteResult<Vec<i32>> {
    use rusqlite::params;

    let db_path = get_db_path();
    let conn = Connection::open(db_path)?;

    let mut stmt = conn.prepare(
        "SELECT artists_images_id FROM user_favorites
         WHERE user_id = ?1
         ORDER BY created_at DESC",
    )?;

    let favorites = stmt.query_map(params![user_id], |row| row.get(0))?;

    favorites.collect()
}

/// Get the count of favorites for a user
#[cfg(feature = "ssr")]
pub fn get_user_favorite_count(user_id: i32) -> SqliteResult<i32> {
    use rusqlite::params;

    let db_path = get_db_path();
    let conn = Connection::open(db_path)?;

    conn.query_row(
        "SELECT COUNT(*) FROM user_favorites WHERE user_id = ?1",
        params![user_id],
        |row| row.get(0),
    )
}

/// Toggle favorite status (add if not exists, remove if exists)
#[cfg(feature = "ssr")]
pub fn toggle_favorite(user_id: i32, artists_images_id: i32) -> SqliteResult<bool> {
    let is_fav = is_favorited(user_id, artists_images_id)?;

    if is_fav {
        remove_favorite(user_id, artists_images_id)?;
        Ok(false)
    } else {
        add_favorite(user_id, artists_images_id)?;
        Ok(true)
    }
}

/// Get user's favorite posts with full details (image, artist, styles)
#[cfg(feature = "ssr")]
pub fn get_user_favorites_with_details(user_id: i32) -> SqliteResult<Vec<FavoritePostWithDetails>> {
    use rusqlite::params;
    use rusqlite::OptionalExtension;

    let db_path = get_db_path();
    let conn = Connection::open(db_path)?;

    // Get all favorited image IDs
    let image_ids = get_user_favorites(user_id)?;
    let mut posts = Vec::new();

    for image_id in image_ids {
        // Get image details
        let image: Option<ArtistImage> = conn
            .query_row(
                "SELECT id, short_code, artist_id
                 FROM artists_images WHERE id = ?1",
                params![image_id],
                |row| {
                    Ok(ArtistImage {
                        id: row.get(0)?,
                        short_code: row.get(1)?,
                        artist_id: row.get(2)?,
                    })
                },
            )
            .optional()?;

        if let Some(image) = image {
            // Get artist details
            let artist: Option<Artist> = conn
                .query_row(
                    "SELECT id, name, location_id, social_links, email, phone, years_experience, styles_extracted
                     FROM artists WHERE id = ?1",
                    params![image.artist_id],
                    |row| {
                        Ok(Artist {
                            id: row.get(0)?,
                            name: row.get(1)?,
                            location_id: row.get(2)?,
                            social_links: row.get(3)?,
                            email: row.get(4)?,
                            phone: row.get(5)?,
                            years_experience: row.get(6)?,
                            styles_extracted: row.get(7)?,
                        })
                    },
                )
                .optional()?;

            // Get styles for this image
            let mut stmt = conn.prepare(
                "SELECT s.id, s.name
                 FROM styles s
                 JOIN artists_images_styles ais ON s.id = ais.style_id
                 WHERE ais.artists_images_id = ?1",
            )?;

            let styles: Vec<Style> = stmt
                .query_map(params![image.id], |row| {
                    Ok(Style {
                        id: row.get(0)?,
                        name: row.get(1)?,
                    })
                })?
                .collect::<Result<Vec<_>, _>>()?;

            posts.push(FavoritePostWithDetails {
                image,
                artist,
                styles,
            });
        }
    }

    Ok(posts)
}
