# Tatteau Platform — End-to-End Product Strategy

## 🎯 Goal
Build a global-first tattoo artist discovery and booking platform that genuinely solves pain points for clients, artists, and shops.

---

## 🧠 Dev Context

- DB: `tatteau.db` (SQLite)
- Access: via `rusqlite`
- UI: Use **Thaw UI components exclusively**
- App: Extend **existing codebase**
- Some pages already initialized:  
  - ✅ Map Discovery  
  - ✅ Artist Highlight  
  - ✅ Shop Highlight  
- ✅ Agent must:  
  - Build on existing pages (not overwrite)  
  - Compile app after every task  
  - Track work in `agent_state.json`  
  - Log DB/schema changes in `schema_snapshot.txt`

---

## 🧑‍🎨 Client Experience Flow — Page-by-Page Requirements

### Page 1: Homepage / Entry
- CTA buttons using `thaw::Button`:  
  `[Explore Artists]`, `[Get Matched]`, `[See Styles]`
- Style tags in `thaw::ChipGroup`  
- Routes: `/explore`, `/match`, `/styles`

### Page 2: Get Matched Quiz
- Use `thaw::FormControl`, `Select`, `RangeSlider`, `CheckboxGroup`
- Fields: Style, Body Placement, Pain Tolerance, Budget, Vibe
- On submit:
  - Save to app state and `client_quiz_sessions`
  - Redirect to `/match/results`

### Page 3: Match Summary
- Query matching artists from DB
- Render `ArtistCard`: avatar, style tags, pricing meter, thumbnails
- Log match impression to `client_match_impressions`

### Page 4: Artist Profile View
- Pull from `artists`, `artist_styles`, `artist_images`
- 3-column `Grid` portfolio
- Pricing display: `thaw::Progress`
- CTA: `Book Now` (opens booking modal)

### Page 5: Booking Modal
- Use: `thaw::Modal`, `FileUpload`, `TextInput`, `Select`
- Fields: placement, size (inches), notes, image
- Insert into `bookings` table

### Page 6: Confirmation + Onboarding
- Display booking summary
- CTA: `[Add to Calendar]`, `[Home]`, `[Match Again]`
- Aftercare opt-in → inserts to `aftercare_signups`

---

## 🎨 Artist Dashboard Flow — Page-by-Page Requirements

### Page 1: Artist Home
- Tiles: Today’s Bookings, Sketch Requests, Unread Messages

### Page 2: Booking Calendar
- Drag-to-reschedule booking → update `bookings.date`
- Highlight buffer time

### Page 3: Incoming Requests
- Tabs for matches, sketches, bookings
- CTAs: Accept, Decline, Request Changes

### Page 4: Design Intake & Revisions
- Left: client ref
- Right: sketch upload
- “Lock” disables further uploads

### Page 5: Settings
- Toggles: auto-reply, availability
- Pricing config: `artist_pricing`

### Page 6: Metrics
- Booking total, avg income, repeat clients, charts

---

## 🗺️ Map Discovery Page *(Initialized — Extend)*

- Geolocation or zip search
- Artist pins → MiniCards
- Route to `/artist/:id`

---

## ✨ Artist Highlight Page *(Initialized — Extend)*

- Detailed artist bio, animated section, portfolio
- CTA: `Book Now` → pre-selects artist in flow

---

## 🏬 Shop Highlight Page *(Initialized — Extend)*

- Banner, shop info, team grid, booking CTAs

---

## 🏪 Shop Owner Portal — Page-by-Page Requirements

### Page 1: Shop Overview
- Roster, status, bookings, CTAs

### Page 2: Shop Inbox
- Threads by artist or client
- Message types tagged

### Page 3: Shop Profile Editor
- Update hours, tags, bio, images

### Page 4: Artist Management
- Toggle availability, edit roles

### Page 5: Merch / Services
- CRUD for shop offerings

### Page 6: Trends Dashboard
- Style views, artist stats, revenue

---

## 💰 Monetization Strategy

### Artists
- Subscription tiers
- Sketch Previews
- Boosted Discovery

### Clients
- Concierge Matching
- Sketch Feedback

### Shops
- Merch rev share
- Sponsored listings
- Team onboarding

---

## 🧪 MVP Roadmap

### Phase 1
- Quiz, Discovery
- Booking flow
- Artist dashboard (core)
- Schema & rusqlite setup

### Phase 2
- Sketch feedback
- Aftercare
- Shop portal
- Trends

---

## ✅ Agent Execution Rules

Before doing any work:

1. **Review the current app.**
   - Inspect routes, components, and schema

2. **Update the spec if needed.**
   - If features exist, adjust goals dynamically

3. **Avoid duplicating functionality.**

4. **Only add tables if clearly needed.**

5. **Compile after each feature.**

6. **Use Thaw UI exclusively.**

7. **Log everything:**
   - Progress → `agent_state.json`
   - DB → `schema_snapshot.txt`
   - Builds → `build_log.txt`

---

Let’s build the tattoo platform everyone wishes existed—intelligently.
