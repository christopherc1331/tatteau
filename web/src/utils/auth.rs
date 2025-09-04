use leptos::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Claims {
    sub: String, // User ID  
    exp: usize,  // Expiration time
    user_type: String, // "client" or "artist"
    user_id: i32,
}

/// Extracts the artist_id from the JWT token stored in localStorage
/// Returns None if token is invalid, missing, or user is not an artist
pub fn get_authenticated_artist_id() -> Option<i32> {
    #[cfg(feature = "hydrate")]
    {
        use wasm_bindgen::prelude::*;
        
        #[wasm_bindgen]
        extern "C" {
            #[wasm_bindgen(js_namespace = localStorage)]
            fn getItem(key: &str) -> Option<String>;
        }
        
        if let Some(token) = getItem("tatteau_auth_token") {
            if token.is_empty() {
                return None;
            }
            
            // Decode JWT token
            if let Some(artist_id) = decode_jwt_artist_id(&token) {
                return Some(artist_id);
            }
        }
    }
    
    #[cfg(not(feature = "hydrate"))]
    {
        // On server side, we don't have access to localStorage
        // This should be handled by server-side context or cookies
        return None;
    }
    
    None
}

/// Decodes JWT token and extracts artist_id if user_type is "artist"
fn decode_jwt_artist_id(token: &str) -> Option<i32> {
    if let Some(claims) = decode_jwt_token(token) {
        if claims.user_type == "artist" {
            return Some(claims.user_id);
        }
    }
    None
}

/// Hook to get the authenticated artist ID reactively
pub fn use_authenticated_artist_id() -> Signal<Option<i32>> {
    let artist_id = RwSignal::new(None::<i32>);
    
    Effect::new(move |_| {
        let id = get_authenticated_artist_id();
        artist_id.set(id);
    });
    
    artist_id.into()
}

/// Checks if any user (client or artist) is authenticated
/// Returns true if there's a valid JWT token in localStorage
pub fn is_authenticated() -> bool {
    #[cfg(feature = "hydrate")]
    {
        use wasm_bindgen::prelude::*;
        
        #[wasm_bindgen]
        extern "C" {
            #[wasm_bindgen(js_namespace = localStorage)]
            fn getItem(key: &str) -> Option<String>;
        }
        
        if let Some(token) = getItem("tatteau_auth_token") {
            if !token.is_empty() {
                // Try to decode the token to verify it's valid
                return decode_jwt_token(&token).is_some();
            }
        }
    }
    
    #[cfg(not(feature = "hydrate"))]
    {
        // On server side, we don't have access to localStorage
        return false;
    }
    
    false
}

/// Gets the authenticated user data from JWT token
/// Returns Some((user_id, user_type)) if valid, None if invalid or missing
pub fn get_authenticated_user() -> Option<(i32, String)> {
    #[cfg(feature = "hydrate")]
    {
        use wasm_bindgen::prelude::*;
        
        #[wasm_bindgen]
        extern "C" {
            #[wasm_bindgen(js_namespace = localStorage)]
            fn getItem(key: &str) -> Option<String>;
        }
        
        if let Some(token) = getItem("tatteau_auth_token") {
            if !token.is_empty() {
                if let Some(claims) = decode_jwt_token(&token) {
                    return Some((claims.user_id, claims.user_type));
                }
            }
        }
    }
    
    #[cfg(not(feature = "hydrate"))]
    {
        // On server side, we don't have access to localStorage
        return None;
    }
    
    None
}

/// Decodes JWT token and returns claims if valid
fn decode_jwt_token(token: &str) -> Option<Claims> {
    let parts: Vec<&str> = token.split('.').collect();
    if parts.len() != 3 {
        return None;
    }
    
    // Decode the payload (second part)
    let payload = parts[1];
    
    // Add padding if needed for base64 decoding
    let padded_payload = match payload.len() % 4 {
        2 => format!("{}==", payload),
        3 => format!("{}=", payload),
        _ => payload.to_string(),
    };
    
    #[cfg(feature = "hydrate")]
    {
        use wasm_bindgen::prelude::*;
        
        #[wasm_bindgen]
        extern "C" {
            #[wasm_bindgen(js_name = atob)]
            fn base64_decode(data: &str) -> String;
        }
        
        if let Ok(decoded) = std::panic::catch_unwind(|| base64_decode(&padded_payload)) {
            if let Ok(claims) = serde_json::from_str::<Claims>(&decoded) {
                return Some(claims);
            }
        }
    }
    
    None
}