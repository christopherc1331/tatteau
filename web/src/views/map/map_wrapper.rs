use leptos::{prelude::*, task::spawn_local};
use thaw::{Button, ButtonSize, Checkbox, CheckboxGroup, Flex, FlexAlign, Slider};

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
    
    // Price range filter state (min, max)
    let price_range = RwSignal::new((0.0, 500.0));
    let selected_price_min = RwSignal::new(50.0);
    let selected_price_max = RwSignal::new(300.0);
    
    // Distance filter state
    let distance_radius = RwSignal::new(25.0); // Default 25 miles
    let distance_unit = RwSignal::new("miles".to_string());

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
        selected_price_min.set(50.0);
        selected_price_max.set(300.0);
        distance_radius.set(25.0);
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
                        
                        // Price range filter
                        <div class="filter-section price-filter">
                            <h3>"Price Range"</h3>
                            <div class="price-range-container">
                                <div class="price-display">
                                    <span class="price-min">"$"{move || selected_price_min.get() as i32}</span>
                                    <span class="price-separator">" - "</span>
                                    <span class="price-max">"$"{move || selected_price_max.get() as i32}</span>
                                </div>
                                
                                <div class="slider-group">
                                    <label class="slider-label">"Minimum"</label>
                                    <Slider 
                                        value=selected_price_min
                                        min=0.0
                                        max=500.0
                                        step=10.0
                                    />
                                    
                                    <label class="slider-label">"Maximum"</label>
                                    <Slider 
                                        value=selected_price_max
                                        min=0.0
                                        max=1000.0
                                        step=10.0
                                    />
                                </div>
                                
                                <div class="price-presets">
                                    <button 
                                        class="preset-btn"
                                        on:click=move |_| {
                                            selected_price_min.set(0.0);
                                            selected_price_max.set(150.0);
                                        }
                                    >"Budget"</button>
                                    <button 
                                        class="preset-btn"
                                        on:click=move |_| {
                                            selected_price_min.set(150.0);
                                            selected_price_max.set(400.0);
                                        }
                                    >"Mid-range"</button>
                                    <button 
                                        class="preset-btn"
                                        on:click=move |_| {
                                            selected_price_min.set(400.0);
                                            selected_price_max.set(1000.0);
                                        }
                                    >"Premium"</button>
                                </div>
                            </div>
                        </div>
                        
                        // Distance filter
                        <div class="filter-section distance-filter">
                            <h3>"Distance"</h3>
                            <div class="distance-container">
                                <div class="distance-display">
                                    <span class="distance-value">{move || distance_radius.get() as i32}</span>
                                    <span class="distance-unit">" miles"</span>
                                </div>
                                
                                <Slider 
                                    value=distance_radius
                                    min=5.0
                                    max=100.0
                                    step=5.0
                                />
                                
                                <div class="distance-presets">
                                    <button 
                                        class="preset-btn"
                                        on:click=move |_| distance_radius.set(10.0)
                                    >"10 mi"</button>
                                    <button 
                                        class="preset-btn"
                                        on:click=move |_| distance_radius.set(25.0)
                                    >"25 mi"</button>
                                    <button 
                                        class="preset-btn"
                                        on:click=move |_| distance_radius.set(50.0)
                                    >"50 mi"</button>
                                    <button 
                                        class="preset-btn"
                                        on:click=move |_| distance_radius.set(100.0)
                                    >"100 mi"</button>
                                </div>
                            </div>
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
                        price_min=selected_price_min
                        price_max=selected_price_max
                        distance_radius=distance_radius
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

