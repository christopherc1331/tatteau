use leptos::{prelude::*, task::spawn_local};
use thaw::{Button, ButtonSize, Checkbox, CheckboxGroup, Flex, FlexAlign};

use crate::{
    components::{loading::LoadingView, location_search::LocationSearch},
    db::entities::CityCoords,
    server::{
        get_available_styles, get_styles_in_bounds, get_cities, get_location_stats, search_by_postal_code, LocationStats,
        StyleWithCount,
    },
    views::map::{
        drop_down_cities::DropDownCities, drop_down_states::DropDownStates,
        map_renderer::MapRenderer,
    },
};
use shared_types::MapBounds;

#[component]
pub fn DiscoveryMap() -> impl IntoView {
    let state = RwSignal::new("Washington".to_string());
    let default_location = CityCoords {
        city: "Spokane".to_string(),
        state: "Washington".to_string(),
        lat: 47.6578118,
        long: -117.4186315,
    };
    let city = RwSignal::new(default_location.clone().city);
    let cities = Resource::new(
        move || state.get(),
        move |state| async move { get_cities(state).await },
    );

    // New state for enhanced features
    let selected_styles = RwSignal::new(Vec::<i32>::new());
    let sidebar_collapsed = RwSignal::new(false);
    let map_center = RwSignal::new(default_location.clone());
    let map_bounds = RwSignal::new(MapBounds::default());

    // Fetch location stats (use LocalResource to avoid hydration issues)
    let location_stats = Resource::new(
        move || (city.get(), state.get()),
        move |(city, state)| async move { get_location_stats(city, state).await.unwrap_or_default() },
    );

    // Fetch available styles based on current map bounds
    let available_styles = Resource::new(
        move || map_bounds.get(),
        move |bounds| async move { 
            if bounds.north_east.lat != 0.0 || bounds.south_west.lat != 0.0 {
                // Use bounds-based query when bounds are available
                get_styles_in_bounds(bounds).await.unwrap_or_default()
            } else {
                // Fallback to all styles when bounds are not yet initialized
                get_available_styles().await.unwrap_or_default()
            }
        },
    );

    // Handle location selection from search
    let handle_location_selected = move |coords: CityCoords| {
        city.set(coords.city.clone());
        state.set(coords.state.clone());
        map_center.set(coords);
    };

    let toggle_sidebar = move |_ev: web_sys::MouseEvent| {
        sidebar_collapsed.update(|c| *c = !*c);
    };

    let clear_filters = move |_ev: web_sys::MouseEvent| {
        selected_styles.set(Vec::new());
    };

    view! {
        <div class="explore-container">
            // Header
            <div class="explore-header">
                <div class="header-content">
                    <h1>"Discover Tattoo Artists"</h1>

                    <LocationSearch 
                        city=city
                        state=state
                        on_location_selected=handle_location_selected
                    />

                    <div class="location-stats">
                        <Suspense fallback=|| view! { <span>"Loading stats..."</span> }>
                            {move || {
                                if let Some(stats) = location_stats.get() {
                                    view! {
                                        <>
                                            <div class="stat-item">
                                                <span class="stat-number">{stats.shop_count}</span>
                                                <span>"shops"</span>
                                            </div>
                                            <div class="stat-item">
                                                <span class="stat-number">{stats.artist_count}</span>
                                                <span>"artists"</span>
                                            </div>
                                            <div class="stat-item">
                                                <span class="stat-number">{stats.styles_available}</span>
                                                <span>"styles"</span>
                                            </div>
                                            <div class="stat-item">
                                                <span>"in"</span>
                                                <span class="stat-number">{city.get()}</span>
                                            </div>
                                        </>
                                    }.into_any()
                                } else {
                                    view! {
                                        <div class="stat-item">
                                            <span>"No stats available"</span>
                                        </div>
                                    }.into_any()
                                }
                            }}
                        </Suspense>
                    </div>
                </div>
            </div>

            // Main content
            <div class="explore-content">
                // Sidebar
                <div class="explore-sidebar" class:collapsed=sidebar_collapsed>
                    <button class="sidebar-toggle" on:click=toggle_sidebar>
                        {move || if sidebar_collapsed.get() { "→" } else { "←" }}
                    </button>

                    <div class="sidebar-header">
                        <h2>"Filters"</h2>
                    </div>

                    <div class="sidebar-content">
                        // Location filters
                        <div class="filter-section">
                            <h3>"Location"</h3>
                            <div class="location-selects">
                                <DropDownStates state=state />
                                <DropDownCities city=city cities=cities/>
                            </div>
                        </div>

                        // Style filters
                        <div class="filter-section style-filters">
                            <h3>"Tattoo Styles"</h3>
                            <Suspense fallback=|| view! { <div>"Loading styles..."</div> }>
                                {move || {
                                    available_styles.get().map(|styles| {
                                        if styles.is_empty() {
                                            view! {
                                                <div class="no-styles">"No styles available"</div>
                                            }.into_any()
                                        } else {
                                            view! {
                                                <div class="style-checkbox-grid">
                                                    {styles.into_iter().map(|style| {
                                                        let style_id = style.id;
                                                        let style_name = style.name.clone();
                                                        let artist_count = style.artist_count;
                                                        
                                                        view! {
                                                            <label class="style-checkbox-label">
                                                                <input
                                                                    type="checkbox"
                                                                    class="style-checkbox"
                                                                    on:change=move |ev| {
                                                                        let is_checked = event_target_checked(&ev);
                                                                        selected_styles.update(|styles| {
                                                                            if is_checked {
                                                                                if !styles.contains(&style_id) {
                                                                                    styles.push(style_id);
                                                                                }
                                                                            } else {
                                                                                styles.retain(|&id| id != style_id);
                                                                            }
                                                                        });
                                                                    }
                                                                    checked=move || selected_styles.get().contains(&style_id)
                                                                />
                                                                <span class="style-label-content">
                                                                    <span class="style-name">{style_name}</span>
                                                                    <span class="artist-count">"("{artist_count}")"</span>
                                                                </span>
                                                            </label>
                                                        }
                                                    }).collect_view()}
                                                </div>
                                            }.into_any()
                                        }
                                    })
                                }}
                            </Suspense>
                        </div>
                        
                        // Price range filter (placeholder)
                        <div class="filter-section">
                            <h3>"Price Range"</h3>
                            <div class="price-range-placeholder">"Coming soon"</div>
                        </div>
                        
                        // Distance filter (placeholder)
                        <div class="filter-section">
                            <h3>"Distance"</h3>
                            <div class="distance-placeholder">"Coming soon"</div>
                        </div>

                        <button class="clear-filters" on:click=clear_filters>
                            "Clear All Filters"
                        </button>
                    </div>
                </div>

                // Map area
                <div class="explore-map-wrapper">
                    // Map
                    <MapRenderer
                        state=state
                        city=city
                        default_location=map_center.get()
                        cities=cities
                        selected_styles=selected_styles
                        map_bounds=map_bounds
                    />

                    // Map legend
                    <div class="map-legend">
                        <h4>"Map Legend"</h4>
                        <div class="legend-items">
                            <div class="legend-item">
                                <div class="legend-marker has-portfolio"></div>
                                <span>"Has portfolio images"</span>
                            </div>
                            <div class="legend-item">
                                <div class="legend-marker no-portfolio"></div>
                                <span>"No portfolio yet"</span>
                            </div>
                            <div class="legend-item">
                                <div class="legend-marker no-artists"></div>
                                <span>"No artists listed"</span>
                            </div>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    }
}

