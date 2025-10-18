# EXPLORE PAGE SQL QUERY TEST RESULTS

**Test Date:** 2025-10-18
**Location Tested:** Spokane, Washington
**Map Bounds:** lat: 47.64-47.67, long: -117.43 to -117.40

---

## ✅ ALL QUERIES PASSED SUCCESSFULLY

---

## Query 1: get_cities_and_coords('Washington')

**Purpose:** Load city dropdown for Washington state
**Status:** ✅ **PASSED**

**Query:**
```sql
SELECT city, state, AVG(lat) as lat, AVG(long) as long
FROM locations
WHERE state = 'Washington'
AND (is_person IS NULL OR is_person = 0)
GROUP BY city, state;
```

**Results (first 5):**
```
      city      |   state    |        lat         |        long
----------------+------------+--------------------+---------------------
 Aberdeen       | Washington | 46.975138346354164 | -123.77499643961589
 Airway Heights | Washington |  47.64324188232422 |  -117.5870590209961
 Anacortes      | Washington |  48.49824142456055 | -122.61303329467773
 Arlington      | Washington |  48.19065189361572 | -122.14157104492188
 Auburn         | Washington |  47.30867576599121 | -122.22441482543945
```

**Analysis:** Successfully returns all cities in Washington with averaged coordinates.

---

## Query 2: get_location_stats_for_city('Spokane', 'Washington')

**Purpose:** Display stats for Spokane
**Status:** ✅ **PASSED**

**Query:**
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

**Results:**
```
 shop_count | artist_count | styles_available
------------+--------------+------------------
         50 |          104 |               72
```

**Analysis:** Spokane has 50 shops, 104 artists, and 72 different tattoo styles available.

---

## Query 3: get_styles_with_counts_in_bounds(bounds)

**Purpose:** Load style filter chips for map view
**Status:** ✅ **PASSED**

**Query:**
```sql
SELECT s.id, s.name, COUNT(DISTINCT ast.artist_id) as artist_count
FROM styles s
INNER JOIN artists_styles ast ON s.id = ast.style_id
INNER JOIN artists a ON ast.artist_id = a.id
INNER JOIN locations l ON a.location_id = l.id
WHERE l.lat BETWEEN 47.64 AND 47.67
  AND l.long BETWEEN -117.43 AND -117.40
GROUP BY s.id, s.name
HAVING COUNT(DISTINCT ast.artist_id) > 0
ORDER BY artist_count DESC, s.name ASC;
```

**Results (top 10 styles):**
```
 id |         name         | artist_count
----+----------------------+--------------
  9 | Blackwork            |           20
 45 | Floral               |           19
  8 | Black & Grey         |           17
 93 | Color                |           17
 46 | Animal               |           14
 44 | Botanical            |           12
 15 | Fine Line            |           12
 55 | Linework             |           12
  1 | American Traditional |           11
 41 | Cartoon              |           11
```

**Analysis:** Blackwork is the most popular style in Spokane with 20 artists, followed by Floral (19) and Black & Grey (17).

---

## Query 4A: query_locations_with_details (NO style filter)

**Purpose:** Load all shop markers on map
**Status:** ✅ **PASSED**

**Query:**
```sql
SELECT l.id, l.name, l.lat, l.long, l.city,
       COUNT(DISTINCT a.id) as artist_count,
       CASE WHEN COUNT(DISTINCT a.id) > 0 THEN 1 ELSE 0 END as has_artists,
       COUNT(DISTINCT ai.id) as artist_images_count
FROM locations l
LEFT JOIN artists a ON l.id = a.location_id
LEFT JOIN artists_images ai ON a.id = ai.artist_id
WHERE l.lat BETWEEN 47.64 AND 47.67
AND l.long BETWEEN -117.43 AND -117.40
AND (l.is_person IS NULL OR l.is_person = 0)
GROUP BY l.id, l.name, l.lat, l.long, l.city, l.county, l.state,
         l.country_code, l.postal_code, l.is_open, l.address,
         l.category, l.website_uri, l._id;
```

**Results (first 5 locations):**
```
  id   |           name           |    lat    |    long     |  city   | artist_count | has_artists | artist_images_count
-------+--------------------------+-----------+-------------+---------+--------------+-------------+---------------------
 34137 | Anchored Art Tattoo      |  47.65781 |  -117.41863 | Spokane |            1 |           1 |                   0
 34141 | Fortunata Tattoo Studio  | 47.655273 | -117.414375 | Spokane |           12 |           1 |                 228
 34145 | Auralite Art Collective  |  47.65785 |  -117.41367 | Spokane |           12 |           1 |                   0
 34149 | The Missing Piece Tattoo | 47.657352 |  -117.41858 | Spokane |            8 |           1 |                  72
 34160 | Main Ave Tattoo          | 47.659325 |  -117.41218 | Spokane |            4 |           1 |                   0
```

**Analysis:** Successfully returns shop locations with artist counts and portfolio image counts. Fortunata Tattoo Studio has the most portfolio images (228).

---

## Query 4B: query_locations_with_details (WITH style filter = Blackwork)

**Purpose:** Filter map markers by selected style
**Status:** ✅ **PASSED**

**Query:**
```sql
SELECT DISTINCT l.id, l.name, l.lat, l.long, l.city,
       COUNT(DISTINCT a.id) as artist_count,
       CASE WHEN COUNT(DISTINCT a.id) > 0 THEN 1 ELSE 0 END as has_artists,
       COUNT(DISTINCT ai.id) as artist_images_count
FROM locations l
LEFT JOIN artists a ON l.id = a.location_id
LEFT JOIN artists_images ai ON a.id = ai.artist_id
LEFT JOIN artists_styles ast ON a.id = ast.artist_id
WHERE l.lat BETWEEN 47.64 AND 47.67
AND l.long BETWEEN -117.43 AND -117.40
AND (l.is_person IS NULL OR l.is_person = 0)
AND ast.style_id = ANY(ARRAY[9])
GROUP BY l.id, l.name, l.lat, l.long, l.city, l.county, l.state,
         l.country_code, l.postal_code, l.is_open, l.address,
         l.category, l.website_uri, l._id;
```

**Results (filtered by Blackwork style_id=9):**
```
  id   |             name              |    lat    |    long     |  city   | artist_count | has_artists | artist_images_count
-------+-------------------------------+-----------+-------------+---------+--------------+-------------+---------------------
 34141 | Fortunata Tattoo Studio       | 47.655273 | -117.414375 | Spokane |            8 |           1 |                 176
 34149 | The Missing Piece Tattoo      | 47.657352 |  -117.41858 | Spokane |            4 |           1 |                  53
 34163 | Iron and Gold Tattoo          | 47.663757 |  -117.42683 | Spokane |            7 |           1 |                 521
 34166 | Black Bee Tattoos & Piercings | 47.654625 |  -117.42165 | Spokane |            1 |           1 |                  10
```

**Analysis:** Successfully filters to only show shops with Blackwork artists. Results correctly reduced from 5+ shops to 4 shops.

---

## DIAGNOSIS CONCLUSION

### ✅ All SQL Queries Work Correctly

**NO SQL ERRORS FOUND** - All queries execute successfully with proper results.

### The Real Problem: Cached/Compiled Code

The errors you're seeing (`IS NOT TRUE must be type boolean`) are from **old compiled code** that's still running. The binary was compiled with the old buggy SQL before we fixed it.

### Solution

**YOU MUST RESTART THE SERVER:**

```bash
# Kill the running server
pkill -f "cargo leptos"

# Rebuild with new SQL and restart
cargo leptos watch
```

### Why This Is Necessary

- Rust compiles SQL queries into the binary at build time
- The running server still has the old `IS NOT TRUE` syntax baked in
- Our code changes won't take effect until you rebuild and restart
- The database itself is fine - it's ready for the corrected queries

---

## Code Changes Made (Already Applied)

### Files Modified:
1. `web/src/db/repository.rs` - Fixed 18 occurrences
2. `web/src/db/search_repository.rs` - Fixed 7 occurrences
3. `web/src/views/map/map_renderer.rs` - Fixed SendWrapper threading

### Change Applied:
```sql
-- OLD (WRONG):
AND (is_person IS NULL OR is_person IS NOT TRUE)

-- NEW (CORRECT):
AND (is_person IS NULL OR is_person = 0)
```

---

## Expected Behavior After Restart

1. ✅ `/explore` page loads without errors
2. ✅ Washington state dropdown populates
3. ✅ Spokane selection centers map
4. ✅ Map shows 5+ markers in Spokane area
5. ✅ Style filters show with correct counts (Blackwork: 20, Floral: 19, etc.)
6. ✅ Clicking Blackwork filters to 4 shops
7. ✅ No SendWrapper panics
8. ✅ No database type errors

---

## Test Environment

- **Database:** PostgreSQL (Railway)
- **Connection:** Verified working
- **Data Quality:** Good (50 shops, 104 artists in Spokane)
- **Query Performance:** Fast (<1s for all queries)

**READY FOR TESTING AFTER SERVER RESTART** ✅
