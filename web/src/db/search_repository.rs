#[cfg(feature = "ssr")]
use sqlx::{PgPool, Row};

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct SearchResult {
    pub city: String,
    pub state: String,
    pub county: Option<String>,
    pub postal_code: Option<String>,
    pub lat: f64,
    pub long: f64,
    pub result_type: SearchResultType,
    pub artist_count: i32,
    pub shop_count: i32,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub enum SearchResultType {
    City,
    County,
    PostalCode,
}

#[cfg(feature = "ssr")]
pub async fn universal_location_search(query: String) -> Result<Vec<SearchResult>, sqlx::Error> {
    let pool = crate::db::pool::get_pool();

    let normalized_query = query.trim().to_lowercase();
    let mut results = Vec::new();

    // Check if query contains both city and state (e.g., "Seattle, WA" or "Seattle, Washington")
    let (city_search, state_search) = if normalized_query.contains(',') {
        let parts: Vec<&str> = normalized_query.split(',').map(|s| s.trim()).collect();
        if parts.len() == 2 {
            (parts[0].to_string(), Some(parts[1].to_string()))
        } else {
            (normalized_query.clone(), None)
        }
    } else {
        (normalized_query.clone(), None)
    };

    // Check if it's a zip code (5 digits)
    if normalized_query.chars().all(|c| c.is_numeric()) && normalized_query.len() == 5 {
        // Search by postal code
        let postal_results = sqlx::query(
            "SELECT DISTINCT
                l.city, l.state, l.county, l.postal_code,
                AVG(l.lat) as lat, AVG(l.long) as long,
                COUNT(DISTINCT a.id) as artist_count,
                COUNT(DISTINCT l.id) as shop_count
             FROM locations l
             LEFT JOIN artists a ON l.id = a.location_id
             WHERE l.postal_code = $1
             AND (l.is_person IS NULL OR l.is_person != true)
             GROUP BY l.city, l.state, l.county, l.postal_code"
        )
        .bind(&normalized_query)
        .fetch_all(pool)
        .await?;

        for row in postal_results {
            results.push(SearchResult {
                city: row.get("city"),
                state: row.get("state"),
                county: row.get("county"),
                postal_code: row.get("postal_code"),
                lat: row.get("lat"),
                long: row.get("long"),
                result_type: SearchResultType::PostalCode,
                artist_count: row.try_get::<i64, _>("artist_count").unwrap_or(0) as i32,
                shop_count: row.try_get::<i64, _>("shop_count").unwrap_or(0) as i32,
            });
        }
    }

    // Search by city name
    let city_results = if let Some(ref state) = state_search {
        // Search for specific city + state combination
        let state_exact_pattern = state.clone();
        let state_prefix_pattern = format!("{}%", state);
        let state_fuzzy_pattern = format!("%{}%", state);

        sqlx::query(
            "SELECT DISTINCT
                l.city, l.state, l.county, l.postal_code,
                AVG(l.lat) as lat, AVG(l.long) as long,
                COUNT(DISTINCT a.id) as artist_count,
                COUNT(DISTINCT l.id) as shop_count
             FROM locations l
             LEFT JOIN artists a ON l.id = a.location_id
             WHERE LOWER(l.city) LIKE $1
             AND (LOWER(l.state) = $2 OR LOWER(l.state) LIKE $3 OR LOWER(l.state) LIKE $4)
             AND (l.is_person IS NULL OR l.is_person != true)
             GROUP BY l.city, l.state
             ORDER BY
                CASE WHEN LOWER(l.city) = $5 THEN 0 ELSE 1 END,
                artist_count DESC
             LIMIT 10"
        )
        .bind(format!("%{}%", city_search))
        .bind(&state_exact_pattern)
        .bind(&state_prefix_pattern)
        .bind(&state_fuzzy_pattern)
        .bind(&city_search)
        .fetch_all(pool)
        .await?
    } else {
        // Search across all states
        sqlx::query(
            "SELECT DISTINCT
                l.city, l.state, l.county, l.postal_code,
                AVG(l.lat) as lat, AVG(l.long) as long,
                COUNT(DISTINCT a.id) as artist_count,
                COUNT(DISTINCT l.id) as shop_count
             FROM locations l
             LEFT JOIN artists a ON l.id = a.location_id
             WHERE LOWER(l.city) LIKE $1
             AND (l.is_person IS NULL OR l.is_person != true)
             GROUP BY l.city, l.state
             ORDER BY
                CASE WHEN LOWER(l.city) = $2 THEN 0 ELSE 1 END,
                artist_count DESC
             LIMIT 10"
        )
        .bind(format!("%{}%", city_search))
        .bind(&city_search)
        .fetch_all(pool)
        .await?
    };

    for row in city_results {
        let city: String = row.get("city");
        let state: String = row.get("state");

        // Don't add duplicates if we already found it via postal code
        if !results.iter().any(|existing|
            existing.city == city && existing.state == state
        ) {
            results.push(SearchResult {
                city,
                state,
                county: row.get("county"),
                postal_code: row.get("postal_code"),
                lat: row.get("lat"),
                long: row.get("long"),
                result_type: SearchResultType::City,
                artist_count: row.try_get::<i64, _>("artist_count").unwrap_or(0) as i32,
                shop_count: row.try_get::<i64, _>("shop_count").unwrap_or(0) as i32,
            });
        }
    }

    // Search by county name
    let county_results = sqlx::query(
        "SELECT DISTINCT
            l.city, l.state, l.county, l.postal_code,
            AVG(l.lat) as lat, AVG(l.long) as long,
            COUNT(DISTINCT a.id) as artist_count,
            COUNT(DISTINCT l.id) as shop_count
         FROM locations l
         LEFT JOIN artists a ON l.id = a.location_id
         WHERE LOWER(l.county) LIKE $1
         AND (l.is_person IS NULL OR l.is_person != true)
         GROUP BY l.county, l.state
         ORDER BY artist_count DESC
         LIMIT 5"
    )
    .bind(format!("%{}%", normalized_query))
    .fetch_all(pool)
    .await?;

    for row in county_results {
        let county: Option<String> = row.get("county");
        let state: String = row.get("state");

        // Don't add duplicates
        if !results.iter().any(|existing|
            existing.county == county && existing.state == state
        ) {
            results.push(SearchResult {
                city: row.get("city"),
                state,
                county,
                postal_code: row.get("postal_code"),
                lat: row.get("lat"),
                long: row.get("long"),
                result_type: SearchResultType::County,
                artist_count: row.try_get::<i64, _>("artist_count").unwrap_or(0) as i32,
                shop_count: row.try_get::<i64, _>("shop_count").unwrap_or(0) as i32,
            });
        }
    }

    Ok(results)
}

#[cfg(feature = "ssr")]
pub async fn get_search_suggestions(query: String, limit: usize) -> Result<Vec<String>, sqlx::Error> {
    let pool = crate::db::pool::get_pool();

    let normalized_query = query.trim().to_lowercase();
    let mut suggestions = Vec::new();

    // Helper function to check if a suggestion already exists
    let suggestion_exists = |suggestions: &Vec<String>, new_suggestion: &str| {
        suggestions.iter().any(|s| s.eq_ignore_ascii_case(new_suggestion))
    };

    // 1. Get city suggestions (highest priority)
    let city_pattern = format!("{}%", normalized_query);
    let city_suggestions = sqlx::query(
        "SELECT DISTINCT city || ', ' || state as suggestion
         FROM locations
         WHERE LOWER(city) LIKE $1
         AND (is_person IS NULL OR is_person != true)
         ORDER BY
            CASE WHEN LOWER(city) = $2 THEN 0 ELSE 1 END,
            LENGTH(city)
         LIMIT $3"
    )
    .bind(&city_pattern)
    .bind(&normalized_query)
    .bind(limit as i32)
    .fetch_all(pool)
    .await?;

    for row in city_suggestions {
        let s: String = row.get("suggestion");
        if !suggestion_exists(&suggestions, &s) {
            suggestions.push(s);
            if suggestions.len() >= limit { break; }
        }
    }

    // 2. Get state suggestions
    if suggestions.len() < limit {
        let state_pattern = format!("{}%", normalized_query);
        let state_full_pattern = format!("%{}%", normalized_query);
        let remaining = (limit - suggestions.len()) as i32;

        let state_suggestions = sqlx::query(
            "SELECT DISTINCT state || ' (State)' as suggestion
             FROM locations
             WHERE (LOWER(state) LIKE $1 OR LOWER(state) LIKE $2)
             AND (is_person IS NULL OR is_person != true)
             ORDER BY LENGTH(state)
             LIMIT $3"
        )
        .bind(&state_pattern)
        .bind(&state_full_pattern)
        .bind(remaining)
        .fetch_all(pool)
        .await?;

        for row in state_suggestions {
            let s: String = row.get("suggestion");
            if !suggestion_exists(&suggestions, &s) {
                suggestions.push(s);
                if suggestions.len() >= limit { break; }
            }
        }
    }

    // 3. Get county suggestions
    if suggestions.len() < limit {
        let county_pattern = format!("{}%", normalized_query);
        let remaining = (limit - suggestions.len()) as i32;

        let county_suggestions = sqlx::query(
            "SELECT DISTINCT county || ' County, ' || state as suggestion
             FROM locations
             WHERE LOWER(county) LIKE $1
             AND county IS NOT NULL
             AND (is_person IS NULL OR is_person != true)
             ORDER BY LENGTH(county)
             LIMIT $2"
        )
        .bind(&county_pattern)
        .bind(remaining)
        .fetch_all(pool)
        .await?;

        for row in county_suggestions {
            let s: String = row.get("suggestion");
            if !suggestion_exists(&suggestions, &s) {
                suggestions.push(s);
                if suggestions.len() >= limit { break; }
            }
        }
    }

    // 4. Get postal code suggestions (both numeric and partial)
    if suggestions.len() < limit {
        let postal_condition = if normalized_query.chars().all(|c| c.is_numeric()) {
            // If query is all numeric, match postal codes that start with it
            "postal_code LIKE $1"
        } else {
            // If query contains letters, skip postal codes
            "1 = 0"
        };

        let postal_query = format!(
            "SELECT DISTINCT postal_code || ' - ' || city || ', ' || state as suggestion
             FROM locations
             WHERE {}
             AND postal_code IS NOT NULL
             AND (is_person IS NULL OR is_person != true)
             ORDER BY postal_code
             LIMIT $2", postal_condition
        );

        let postal_pattern = format!("{}%", normalized_query);
        let remaining = (limit - suggestions.len()) as i32;

        let postal_suggestions = sqlx::query(&postal_query)
            .bind(&postal_pattern)
            .bind(remaining)
            .fetch_all(pool)
            .await?;

        for row in postal_suggestions {
            let s: String = row.get("suggestion");
            if !suggestion_exists(&suggestions, &s) {
                suggestions.push(s);
                if suggestions.len() >= limit { break; }
            }
        }
    }

    Ok(suggestions)
}
