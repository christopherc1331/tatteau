use super::entities::{Artist, ArtistImage, CreateUserFavorite, Style, UserFavorite};
#[cfg(feature = "ssr")]
use sqlx::{PgPool, Row};

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct FavoritePostWithDetails {
    pub image: ArtistImage,
    pub artist: Option<Artist>,
    pub styles: Vec<Style>,
}

#[cfg(feature = "ssr")]
type DbResult<T> = Result<T, sqlx::Error>;

/// Add a favorite for a user
#[cfg(feature = "ssr")]
pub async fn add_favorite(user_id: i32, artists_images_id: i32) -> DbResult<i64> {
    let pool = crate::db::pool::get_pool();

    let result = sqlx::query(
        "INSERT INTO user_favorites (user_id, artists_images_id)
         VALUES ($1, $2)
         ON CONFLICT (user_id, artists_images_id) DO NOTHING
         RETURNING id",
    )
    .bind(user_id)
    .bind(artists_images_id)
    .fetch_optional(pool)
    .await?;

    if let Some(row) = result {
        Ok(row.get::<i64, _>("id"))
    } else {
        // If conflict occurred, get the existing id
        let row = sqlx::query(
            "SELECT id FROM user_favorites
             WHERE user_id = $1 AND artists_images_id = $2",
        )
        .bind(user_id)
        .bind(artists_images_id)
        .fetch_one(pool)
        .await?;

        Ok(row.get::<i64, _>("id"))
    }
}

/// Remove a favorite for a user
#[cfg(feature = "ssr")]
pub async fn remove_favorite(user_id: i32, artists_images_id: i32) -> DbResult<()> {
    let pool = crate::db::pool::get_pool();

    sqlx::query(
        "DELETE FROM user_favorites
         WHERE user_id = $1 AND artists_images_id = $2",
    )
    .bind(user_id)
    .bind(artists_images_id)
    .execute(pool)
    .await?;

    Ok(())
}

/// Check if an image is favorited by a user
#[cfg(feature = "ssr")]
pub async fn is_favorited(user_id: i32, artists_images_id: i32) -> DbResult<bool> {
    let pool = crate::db::pool::get_pool();

    let row = sqlx::query(
        "SELECT COUNT(*) as count FROM user_favorites
         WHERE user_id = $1 AND artists_images_id = $2",
    )
    .bind(user_id)
    .bind(artists_images_id)
    .fetch_one(pool)
    .await?;

    let count: i64 = row.get("count");
    Ok(count > 0)
}

/// Get all favorite image IDs for a user
#[cfg(feature = "ssr")]
pub async fn get_user_favorites(user_id: i32) -> DbResult<Vec<i32>> {
    let pool = crate::db::pool::get_pool();

    let rows = sqlx::query(
        "SELECT artists_images_id FROM user_favorites
         WHERE user_id = $1
         ORDER BY created_at DESC",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;

    let favorites = rows
        .into_iter()
        .map(|row| row.try_get::<i64, _>("artists_images_id").unwrap_or(0) as i32)
        .collect();

    Ok(favorites)
}

/// Get the count of favorites for a user
#[cfg(feature = "ssr")]
pub async fn get_user_favorite_count(user_id: i32) -> DbResult<i32> {
    let pool = crate::db::pool::get_pool();

    let row = sqlx::query("SELECT COUNT(*) as count FROM user_favorites WHERE user_id = $1")
        .bind(user_id)
        .fetch_one(pool)
        .await?;

    Ok(row.get::<i64, _>("count") as i32)
}

/// Toggle favorite status (add if not exists, remove if exists)
#[cfg(feature = "ssr")]
pub async fn toggle_favorite(user_id: i32, artists_images_id: i32) -> DbResult<bool> {
    let is_fav = is_favorited(user_id, artists_images_id).await?;

    if is_fav {
        remove_favorite(user_id, artists_images_id).await?;
        Ok(false)
    } else {
        add_favorite(user_id, artists_images_id).await?;
        Ok(true)
    }
}

/// Get user's favorite posts with full details (image, artist, styles)
#[cfg(feature = "ssr")]
pub async fn get_user_favorites_with_details(
    user_id: i32,
) -> DbResult<Vec<FavoritePostWithDetails>> {
    let pool = crate::db::pool::get_pool();

    // Get all favorited image IDs
    let image_ids = get_user_favorites(user_id).await?;
    let mut posts = Vec::new();

    for image_id in image_ids {
        // Get image details
        let image_row = sqlx::query(
            "SELECT id, short_code, artist_id, post_date
             FROM artists_images WHERE id = $1",
        )
        .bind(image_id)
        .fetch_optional(pool)
        .await?;

        if let Some(image_row) = image_row {
            let image = ArtistImage {
                id: image_row.try_get::<i64, _>("id").unwrap_or(0) as i32,
                short_code: image_row.get("short_code"),
                artist_id: image_row.try_get::<i64, _>("artist_id").unwrap_or(0) as i32,
                post_date: image_row.try_get("post_date").ok(),
                validated: image_row.try_get("validated").ok(),
            };

            // Get artist details
            let artist_row = sqlx::query(
                "SELECT id, name, location_id, social_links, instagram_handle, email, phone, years_experience, styles_extracted
                 FROM artists WHERE id = $1",
            )
            .bind(image.artist_id)
            .fetch_optional(pool)
            .await?;

            let artist = artist_row.map(|row| Artist {
                id: row.try_get::<i64, _>("id").unwrap_or(0) as i32,
                name: row.get("name"),
                location_id: row.try_get::<i64, _>("location_id").unwrap_or(0) as i32,
                social_links: row.get("social_links"),
                instagram_handle: row.get("instagram_handle"),
                email: row.get("email"),
                phone: row.get("phone"),
                years_experience: row
                    .try_get::<i64, _>("years_experience")
                    .ok()
                    .map(|v| v as i32),
                styles_extracted: row
                    .try_get::<i64, _>("styles_extracted")
                    .ok()
                    .map(|v| v as i32),
                shop_validated: row.try_get("shop_validated").ok(),
            });

            // Get styles for this image
            let style_rows = sqlx::query(
                "SELECT s.id, s.name
                 FROM styles s
                 JOIN artists_images_styles ais ON s.id = ais.style_id
                 WHERE ais.artists_images_id = $1",
            )
            .bind(image.id)
            .fetch_all(pool)
            .await?;

            let styles: Vec<Style> = style_rows
                .into_iter()
                .map(|row| Style {
                    id: row.try_get::<i64, _>("id").unwrap_or(0) as i32,
                    name: row.get("name"),
                })
                .collect();

            posts.push(FavoritePostWithDetails {
                image,
                artist,
                styles,
            });
        }
    }

    Ok(posts)
}
