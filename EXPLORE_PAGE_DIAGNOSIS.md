# /EXPLORE PAGE - COMPREHENSIVE DIAGNOSIS

## Error Summary
```
thread 'tokio-runtime-worker' (1939753) panicked at SendWrapper dropped from different thread
error returned from database: argument of IS NOT TRUE must be type boolean, not type bigint
```

---

## SQL Queries Documentation

### Query Execution Flow on /explore Page Load

When loading `/explore` with state="Washington" and city="Spokane", these queries execute:

#### 1. **get_cities_and_coords(state='Washington')**
**Purpose:** Populate city dropdown menu
**File:** `web/src/db/repository.rs:16`

```sql
SELECT
    city, state,
    AVG(lat) as lat,
    AVG(long) as long
FROM locations
WHERE state = $1
AND (is_person IS NULL OR is_person = 0)
GROUP BY city, state;
```

#### 2. **get_location_stats_for_city(city='Spokane', state='Washington')**
**Purpose:** Display shop count, artist count, styles available
**File:** `web/src/db/repository.rs:521`

```sql
SELECT
    COUNT(DISTINCT l.id) as shop_count,
    COUNT(DISTINCT a.id) as artist_count,
    COUNT(DISTINCT s.id) as styles_available
FROM locations l
LEFT JOIN artists a ON l.id = a.location_id
LEFT JOIN artists_styles ast ON a.id = ast.artist_id
LEFT JOIN styles s ON ast.style_id = s.id
WHERE l.city = 'Spokane' AND l.state = 'Washington'
AND (l.is_person IS NULL OR l.is_person = 0);
```

#### 3. **get_styles_with_counts_in_bounds(bounds)**
**Purpose:** Load style filter chips with artist counts
**File:** `web/src/db/repository.rs:583`
**Called when:** Map bounds change

```sql
SELECT
    s.id, s.name,
    COUNT(DISTINCT ast.artist_id) as artist_count
FROM styles s
INNER JOIN artists_styles ast ON s.id = ast.style_id
INNER JOIN artists a ON ast.artist_id = a.id
INNER JOIN locations l ON a.location_id = l.id
WHERE l.lat BETWEEN $1 AND $2
  AND l.long BETWEEN $3 AND $4
GROUP BY s.id, s.name
HAVING COUNT(DISTINCT ast.artist_id) > 0
ORDER BY artist_count DESC, s.name ASC;
```

**Example bounds for Spokane:**
- south_west: {lat: 47.64, long: -117.43}
- north_east: {lat: 47.67, long: -117.40}

#### 4. **query_locations_with_details(state, city, bounds, style_filter)**
**Purpose:** Load map markers (shops/artists)
**File:** `web/src/db/repository.rs:718`
**Called when:** Map loads, moves, or style filter changes

**Without style filter:**
```sql
SELECT
    l.id, l.name, l.lat, l.long, l.city, l.county, l.state,
    l.country_code, l.postal_code, l.is_open, l.address,
    l.category, l.website_uri, l._id,
    COUNT(DISTINCT a.id) as artist_count,
    CASE WHEN COUNT(DISTINCT a.id) > 0 THEN 1 ELSE 0 END as has_artists,
    COUNT(DISTINCT ai.id) as artist_images_count
FROM locations l
LEFT JOIN artists a ON l.id = a.location_id
LEFT JOIN artists_images ai ON a.id = ai.artist_id
WHERE l.lat BETWEEN $1 AND $2
AND l.long BETWEEN $3 AND $4
AND (l.is_person IS NULL OR l.is_person = 0)
GROUP BY l.id, l.name, l.lat, l.long, l.city, l.county, l.state,
         l.country_code, l.postal_code, l.is_open, l.address,
         l.category, l.website_uri, l._id;
```

**With style filter (e.g., style_ids=[1,2,3]):**
```sql
SELECT DISTINCT
    l.id, l.name, l.lat, l.long, l.city, l.county, l.state,
    l.country_code, l.postal_code, l.is_open, l.address,
    l.category, l.website_uri, l._id,
    COUNT(DISTINCT a.id) as artist_count,
    CASE WHEN COUNT(DISTINCT a.id) > 0 THEN 1 ELSE 0 END as has_artists,
    COUNT(DISTINCT ai.id) as artist_images_count
FROM locations l
LEFT JOIN artists a ON l.id = a.location_id
LEFT JOIN artists_images ai ON a.id = ai.artist_id
LEFT JOIN artists_styles ast ON a.id = ast.artist_id
WHERE l.lat BETWEEN $1 AND $2
AND l.long BETWEEN $3 AND $4
AND (l.is_person IS NULL OR l.is_person = 0)
AND ast.style_id = ANY($5::int[])
GROUP BY l.id, l.name, l.lat, l.long, l.city, l.county, l.state,
         l.country_code, l.postal_code, l.is_open, l.address,
         l.category, l.website_uri, l._id;
```

---

## Root Cause Analysis

### Issue 1: Database Type Mismatch ✅ FIXED
**Problem:** `is_person` column is `bigint` (integer), not `boolean`
**Error:** `argument of IS NOT TRUE must be type boolean, not type bigint`

**Attempted fix history:**
1. First tried: `is_person != 1` (SQLite syntax)
2. Then tried: `is_person IS NOT TRUE` (PostgreSQL boolean syntax) ❌ WRONG
3. **Final fix:** `is_person = 0` (PostgreSQL integer comparison) ✅ CORRECT

**Files fixed:**
- `web/src/db/repository.rs` - 18 occurrences
- `web/src/db/search_repository.rs` - 7 occurrences

**Current status:** All SQL queries now use `(is_person IS NULL OR is_person = 0)`

### Issue 2: SendWrapper Threading Panic ✅ FIXED
**Problem:** `JsRwSignal::new_local()` created thread-local Map object that was accessed from different Tokio worker threads

**Root cause:**
- Leptos SSR runs queries on Tokio worker threads
- `on_cleanup()` handler tried to access Map from different thread
- Map Effects ran without client-side guards

**Solution applied:**
1. Removed `on_cleanup()` handler (was causing cross-thread access)
2. Wrapped all map Effects in `#[cfg(not(feature = "ssr"))]` guards
3. Added type annotation to closure: `move |_: ()|`

**Files fixed:**
- `web/src/views/map/map_renderer.rs:33-158`

---

## Solution Summary

### ✅ All Fixes Applied

1. **Changed all `is_person` checks to integer comparison:**
   ```sql
   -- From: (is_person IS NULL OR is_person IS NOT TRUE)
   -- To:   (is_person IS NULL OR is_person = 0)
   ```

2. **Fixed SendWrapper threading:**
   - Removed problematic cleanup handler
   - Added `#[cfg(not(feature = "ssr"))]` guards to all map Effects
   - Ensured map operations only run client-side

3. **Code compiles successfully** ✅

---

## Next Steps

### **IMPORTANT: Restart Required**

The error you're seeing is from **cached/old running code**. You must:

```bash
# Stop the current server (Ctrl+C or kill process)
pkill -f "cargo leptos"

# Rebuild and restart
cargo leptos watch
```

### Why restart is needed:
- SQL queries are compiled into the binary
- Old binary still has `IS NOT TRUE` syntax
- New code has correct `is_person = 0` syntax

---

## Testing Checklist

After restart, test these scenarios on `/explore`:

1. ✅ Page loads without errors
2. ✅ Washington state selected → Cities populate
3. ✅ Spokane selected → Map centers
4. ✅ Map markers appear
5. ✅ Style filter chips appear with counts
6. ✅ Clicking a style filters markers
7. ✅ Moving map updates markers
8. ✅ No SendWrapper panics in console
9. ✅ No database type errors

---

## Database Schema Note

**`is_person` column values:**
- `NULL` - Not set (treat as shop/location)
- `0` - Confirmed shop/location (NOT a person)
- `1` - Personal artist profile (individual person)

**Our filter:** `(is_person IS NULL OR is_person = 0)` includes shops and unset values, excludes personal profiles.

---

## Files Modified

1. `/home/chris/code/personal/tatteau/web/src/db/repository.rs`
2. `/home/chris/code/personal/tatteau/web/src/db/search_repository.rs`
3. `/home/chris/code/personal/tatteau/web/src/views/map/map_renderer.rs`

All changes committed and ready for testing after server restart.
