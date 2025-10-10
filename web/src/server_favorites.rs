use leptos::prelude::*;
use serde::{Deserialize, Serialize};

#[cfg(feature = "ssr")]
use leptos::server_fn::error::ServerFnError;

// Helper function to extract user_id from JWT token
#[cfg(feature = "ssr")]
fn extract_user_id_from_token(token: &str) -> Result<i32, ServerFnError> {
    use jsonwebtoken::{decode, DecodingKey, Validation};

    #[derive(Debug, Serialize, Deserialize)]
    struct Claims {
        sub: String,
        exp: usize,
        user_type: String,
        user_id: i32,
    }

    let secret = "tatteau-jwt-secret-key-change-in-production";
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_ref()),
        &Validation::default(),
    )
    .map_err(|e| ServerFnError::new(format!("Invalid token: {}", e)))?;

    Ok(token_data.claims.user_id)
}

#[server]
pub async fn toggle_favorite(
    token: String,
    artists_images_id: i32,
) -> Result<bool, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use crate::db::favorites_repository;

        let user_id = extract_user_id_from_token(&token)?;

        favorites_repository::toggle_favorite(user_id, artists_images_id)
            .map_err(|e| ServerFnError::new(format!("Failed to toggle favorite: {}", e)))
    }
    #[cfg(not(feature = "ssr"))]
    {
        Ok(false)
    }
}

#[server]
pub async fn check_is_favorited(
    token: Option<String>,
    artists_images_id: i32,
) -> Result<bool, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use crate::db::favorites_repository;

        // If no token, not logged in, so return false
        let Some(token) = token else {
            return Ok(false);
        };

        let user_id = match extract_user_id_from_token(&token) {
            Ok(id) => id,
            Err(_) => return Ok(false), // Invalid token, treat as not logged in
        };

        favorites_repository::is_favorited(user_id, artists_images_id)
            .map_err(|e| ServerFnError::new(format!("Failed to check favorite status: {}", e)))
    }
    #[cfg(not(feature = "ssr"))]
    {
        Ok(false)
    }
}

#[server]
pub async fn get_user_favorites_list(token: String) -> Result<Vec<i32>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use crate::db::favorites_repository;

        let user_id = extract_user_id_from_token(&token)?;

        favorites_repository::get_user_favorites(user_id)
            .map_err(|e| ServerFnError::new(format!("Failed to get favorites: {}", e)))
    }
    #[cfg(not(feature = "ssr"))]
    {
        Ok(vec![])
    }
}

#[server]
pub async fn get_favorites_count(token: String) -> Result<i32, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use crate::db::favorites_repository;

        let user_id = extract_user_id_from_token(&token)?;

        favorites_repository::get_user_favorite_count(user_id)
            .map_err(|e| ServerFnError::new(format!("Failed to get favorites count: {}", e)))
    }
    #[cfg(not(feature = "ssr"))]
    {
        Ok(0)
    }
}

#[server]
pub async fn get_user_favorites_with_details(
    token: String,
) -> Result<Vec<crate::db::favorites_repository::FavoritePostWithDetails>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use crate::db::favorites_repository;

        let user_id = extract_user_id_from_token(&token)?;

        favorites_repository::get_user_favorites_with_details(user_id)
            .map_err(|e| ServerFnError::new(format!("Failed to get favorites with details: {}", e)))
    }
    #[cfg(not(feature = "ssr"))]
    {
        Ok(vec![])
    }
}
