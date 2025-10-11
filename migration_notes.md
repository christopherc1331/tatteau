# SQLite to PostgreSQL Migration Progress

## Completed Migrations (10/32)
1. ✅ get_artist_dashboard_data (line 572)
2. ✅ log_match_impression (line 672)
3. ✅ get_all_styles_with_counts (line 712)
4. ✅ get_tattoo_posts_by_style (line 815)
5. ✅ get_artist_availability (line 973)
6. ✅ set_artist_availability (line 1038)
7. ✅ get_booking_requests (line 1080)
8. ✅ respond_to_booking (line 1159)
9. ✅ send_booking_message (line 1206)
10. ✅ get_booking_messages (line 1245)
11. ✅ get_recurring_rules (line 1296)
12. ✅ create_recurring_rule (line 1406)

## Remaining Migrations (20/32)
13. ⏳ update_recurring_rule (line 1495)
14. ⏳ get_effective_availability (line 1575)
15. ⏳ delete_recurring_rule (line 1709)
16. ⏳ get_booking_request_by_id (line 1738)
17. ⏳ get_business_hours (line 1800)
18. ⏳ update_business_hours (line 1853)
19. ⏳ get_client_booking_history (line 1898)
20. ⏳ suggest_booking_time (line 1958)
21. ⏳ submit_booking_request (line 2015)
22. ⏳ login_user (line 2119)
23. ⏳ signup_user (line 2259)
24. ⏳ verify_token (line 2404)
25. ⏳ get_subscription_tiers (line 2492)
26. ⏳ create_artist_subscription (line 2538)
27. ⏳ get_artist_subscription (line 2574)
28. ⏳ check_artist_has_active_subscription (line 2614)
29. ⏳ get_artist_id_from_user (line 2833)
30. ⏳ get_artist_styles_by_id (line 2860)
31. ⏳ get_available_dates (line 2970)
32. ⏳ get_available_time_slots (line 3115)

## Key Conversion Patterns Used
- `?1, ?2` → `$1, $2`
- `row.get(0)?` → `row.get("column_name")`
- `WHERE column = 1` → `WHERE column = true` (boolean)
- `GROUP_CONCAT` → `STRING_AGG`
- `INTEGER PRIMARY KEY AUTOINCREMENT` → `SERIAL PRIMARY KEY`
- `RETURNING id` for getting inserted IDs
- All helper functions changed from `fn` to `async fn`
- All function calls require `.await`
