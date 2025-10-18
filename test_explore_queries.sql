-- EXPLORE PAGE SQL QUERIES TEST
-- Testing with: city=Spokane, state=Washington
-- Coordinates: lat=47.6578118, long=-117.4186315
-- Map bounds: NE(47.67,-117.40), SW(47.64,-117.43)

-- ============================================================
-- QUERY 1: get_cities_and_coords(state='Washington')
-- Purpose: Fetch all cities in Washington state for dropdown
-- ============================================================
SELECT
    city,
    state,
    AVG(lat) as lat,
    AVG(long) as long
FROM locations
WHERE
    state = 'Washington'
AND (is_person IS NULL OR is_person = 0)
GROUP BY city, state;

-- ============================================================
-- QUERY 2: get_location_stats_for_city(city='Spokane', state='Washington')
-- Purpose: Get shop count, artist count, styles available for stats display
-- ============================================================
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

-- ============================================================
-- QUERY 3A: get_all_styles_with_counts()
-- Purpose: Get all tattoo styles with artist counts (used initially before bounds)
-- ============================================================
SELECT
    s.id,
    s.name,
    COUNT(DISTINCT ast.artist_id) as artist_count
FROM styles s
LEFT JOIN artists_styles ast ON s.id = ast.style_id
GROUP BY s.id, s.name
ORDER BY artist_count DESC, s.name ASC;

-- ============================================================
-- QUERY 3B: get_styles_with_counts_in_bounds(bounds)
-- Purpose: Get tattoo styles within map bounds with artist counts
-- Uses: bounds.south_west.lat=47.64, bounds.north_east.lat=47.67
--       bounds.south_west.long=-117.43, bounds.north_east.long=-117.40
-- ============================================================
SELECT
    s.id,
    s.name,
    COUNT(DISTINCT ast.artist_id) as artist_count
FROM styles s
INNER JOIN artists_styles ast ON s.id = ast.style_id
INNER JOIN artists a ON ast.artist_id = a.id
INNER JOIN locations l ON a.location_id = l.id
WHERE l.lat BETWEEN 47.64 AND 47.67
  AND l.long BETWEEN -117.43 AND -117.40
GROUP BY s.id, s.name
HAVING COUNT(DISTINCT ast.artist_id) > 0
ORDER BY artist_count DESC, s.name ASC;

-- ============================================================
-- QUERY 4A: query_locations_with_details (NO style filter)
-- Purpose: Get all shop locations within bounds for map markers
-- ============================================================
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
WHERE l.lat BETWEEN 47.64 AND 47.67
AND l.long BETWEEN -117.43 AND -117.40
AND (l.is_person IS NULL OR l.is_person = 0)
GROUP BY l.id, l.name, l.lat, l.long, l.city, l.county, l.state, l.country_code, l.postal_code, l.is_open, l.address, l.category, l.website_uri, l._id;

-- ============================================================
-- QUERY 4B: query_locations_with_details (WITH style filter)
-- Purpose: Get shop locations filtered by style (e.g., style_ids=[1,2,3])
-- Example with style_id = 1 (traditional)
-- ============================================================
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
WHERE l.lat BETWEEN 47.64 AND 47.67
AND l.long BETWEEN -117.43 AND -117.40
AND (l.is_person IS NULL OR l.is_person = 0)
AND ast.style_id = ANY(ARRAY[1])
GROUP BY l.id, l.name, l.lat, l.long, l.city, l.county, l.state, l.country_code, l.postal_code, l.is_open, l.address, l.category, l.website_uri, l._id;
