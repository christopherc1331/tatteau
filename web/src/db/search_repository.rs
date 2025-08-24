#[cfg(feature = "ssr")]
use rusqlite::{Connection, params};
#[cfg(feature = "ssr")]
use std::path::Path;

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
pub fn universal_location_search(query: String) -> rusqlite::Result<Vec<SearchResult>> {
    let db_path = Path::new("tatteau.db");
    let conn = Connection::open(db_path)?;
    
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
        let mut stmt = conn.prepare(
            "SELECT DISTINCT 
                l.city, l.state, l.county, l.postal_code, 
                AVG(l.lat) as lat, AVG(l.long) as long,
                COUNT(DISTINCT a.id) as artist_count,
                COUNT(DISTINCT l.id) as shop_count
             FROM locations l
             LEFT JOIN artists a ON l.id = a.location_id
             WHERE l.postal_code = ?1
             AND (l.is_person IS NULL OR l.is_person != 1)
             GROUP BY l.city, l.state, l.county, l.postal_code"
        )?;
        
        let postal_results = stmt.query_map(params![normalized_query], |row| {
            Ok(SearchResult {
                city: row.get(0)?,
                state: row.get(1)?,
                county: row.get(2)?,
                postal_code: row.get(3)?,
                lat: row.get(4)?,
                long: row.get(5)?,
                result_type: SearchResultType::PostalCode,
                artist_count: row.get(6)?,
                shop_count: row.get(7)?,
            })
        })?;
        
        for result in postal_results {
            if let Ok(r) = result {
                results.push(r);
            }
        }
    }
    
    // Search by city name
    let city_query = if let Some(ref state) = state_search {
        // Search for specific city + state combination
        "SELECT DISTINCT 
            l.city, l.state, l.county, l.postal_code,
            AVG(l.lat) as lat, AVG(l.long) as long,
            COUNT(DISTINCT a.id) as artist_count,
            COUNT(DISTINCT l.id) as shop_count
         FROM locations l
         LEFT JOIN artists a ON l.id = a.location_id
         WHERE LOWER(l.city) LIKE ?1
         AND (LOWER(l.state) = ?2 OR LOWER(l.state) LIKE ?3)
         AND (l.is_person IS NULL OR l.is_person != 1)
         GROUP BY l.city, l.state
         ORDER BY 
            CASE WHEN LOWER(l.city) = ?4 THEN 0 ELSE 1 END,
            artist_count DESC
         LIMIT 10"
    } else {
        // Search across all states
        "SELECT DISTINCT 
            l.city, l.state, l.county, l.postal_code,
            AVG(l.lat) as lat, AVG(l.long) as long,
            COUNT(DISTINCT a.id) as artist_count,
            COUNT(DISTINCT l.id) as shop_count
         FROM locations l
         LEFT JOIN artists a ON l.id = a.location_id
         WHERE LOWER(l.city) LIKE ?1
         AND (l.is_person IS NULL OR l.is_person != 1)
         GROUP BY l.city, l.state
         ORDER BY 
            CASE WHEN LOWER(l.city) = ?2 THEN 0 ELSE 1 END,
            artist_count DESC
         LIMIT 10"
    };
    
    let mut city_stmt = conn.prepare(city_query)?;
    
    let city_pattern = format!("%{}%", city_search);
    let city_results = if let Some(ref state) = state_search {
        // For state searches, also check state abbreviations
        let state_pattern = format!("%{}%", state);
        city_stmt.query_map(params![city_pattern, state, state_pattern, city_search], |row| {
            Ok(SearchResult {
                city: row.get(0)?,
                state: row.get(1)?,
                county: row.get(2)?,
                postal_code: row.get(3)?,
                lat: row.get(4)?,
                long: row.get(5)?,
                result_type: SearchResultType::City,
                artist_count: row.get(6)?,
                shop_count: row.get(7)?,
            })
        })?
    } else {
        city_stmt.query_map(params![city_pattern, city_search], |row| {
            Ok(SearchResult {
                city: row.get(0)?,
                state: row.get(1)?,
                county: row.get(2)?,
                postal_code: row.get(3)?,
                lat: row.get(4)?,
                long: row.get(5)?,
                result_type: SearchResultType::City,
                artist_count: row.get(6)?,
                shop_count: row.get(7)?,
            })
        })?
    };
    
    for result in city_results {
        if let Ok(r) = result {
            // Don't add duplicates if we already found it via postal code
            if !results.iter().any(|existing| 
                existing.city == r.city && existing.state == r.state
            ) {
                results.push(r);
            }
        }
    }
    
    // Search by county name
    let mut county_stmt = conn.prepare(
        "SELECT DISTINCT 
            l.city, l.state, l.county, l.postal_code,
            AVG(l.lat) as lat, AVG(l.long) as long,
            COUNT(DISTINCT a.id) as artist_count,
            COUNT(DISTINCT l.id) as shop_count
         FROM locations l
         LEFT JOIN artists a ON l.id = a.location_id
         WHERE LOWER(l.county) LIKE ?1
         AND (l.is_person IS NULL OR l.is_person != 1)
         GROUP BY l.county, l.state
         ORDER BY artist_count DESC
         LIMIT 5"
    )?;
    
    let county_pattern = format!("%{}%", normalized_query);
    let county_results = county_stmt.query_map(params![county_pattern], |row| {
        Ok(SearchResult {
            city: row.get(0)?,
            state: row.get(1)?,
            county: row.get(2)?,
            postal_code: row.get(3)?,
            lat: row.get(4)?,
            long: row.get(5)?,
            result_type: SearchResultType::County,
            artist_count: row.get(6)?,
            shop_count: row.get(7)?,
        })
    })?;
    
    for result in county_results {
        if let Ok(r) = result {
            // Don't add duplicates
            if !results.iter().any(|existing| 
                existing.county == r.county && existing.state == r.state
            ) {
                results.push(r);
            }
        }
    }
    
    Ok(results)
}

#[cfg(feature = "ssr")]
pub fn get_search_suggestions(query: String, limit: usize) -> rusqlite::Result<Vec<String>> {
    let db_path = Path::new("tatteau.db");
    let conn = Connection::open(db_path)?;
    
    let normalized_query = query.trim().to_lowercase();
    let mut suggestions = Vec::new();
    
    // Get city suggestions
    let mut stmt = conn.prepare(
        "SELECT DISTINCT city || ', ' || state as suggestion
         FROM locations
         WHERE LOWER(city) LIKE ?1
         AND (is_person IS NULL OR is_person != 1)
         ORDER BY LENGTH(city)
         LIMIT ?2"
    )?;
    
    let pattern = format!("{}%", normalized_query);
    let city_suggestions = stmt.query_map(params![pattern, limit as i32], |row| {
        row.get::<_, String>(0)
    })?;
    
    for suggestion in city_suggestions {
        if let Ok(s) = suggestion {
            suggestions.push(s);
            if suggestions.len() >= limit {
                break;
            }
        }
    }
    
    // If we don't have enough suggestions, add postal codes
    if suggestions.len() < limit && normalized_query.chars().all(|c| c.is_numeric()) {
        let mut postal_stmt = conn.prepare(
            "SELECT DISTINCT postal_code || ' - ' || city || ', ' || state as suggestion
             FROM locations
             WHERE postal_code LIKE ?1
             AND (is_person IS NULL OR is_person != 1)
             LIMIT ?2"
        )?;
        
        let postal_pattern = format!("{}%", normalized_query);
        let remaining = (limit - suggestions.len()) as i32;
        let postal_suggestions = postal_stmt.query_map(params![postal_pattern, remaining], |row| {
            row.get::<_, String>(0)
        })?;
        
        for suggestion in postal_suggestions {
            if let Ok(s) = suggestion {
                suggestions.push(s);
            }
        }
    }
    
    Ok(suggestions)
}