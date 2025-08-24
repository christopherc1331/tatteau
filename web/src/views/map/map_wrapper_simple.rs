use leptos::{prelude::*, task::spawn_local};
use thaw::{Button, ButtonSize, Checkbox, CheckboxGroup, Flex, FlexAlign};

use crate::{
    components::loading::LoadingView,
    db::entities::CityCoords,
    server::{get_available_styles, get_cities, get_location_stats, search_by_postal_code, LocationStats, StyleWithCount},
    views::map::{
        drop_down_cities::DropDownCities, drop_down_states::DropDownStates,
        map_renderer::MapRenderer,
    },
};

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
    let cities = LocalResource::new(
        move || state.get(),
        move |state| async move { get_cities(state).await },
    );
    
    // New state for enhanced features
    let search_input = RwSignal::new(String::new());
    let selected_styles = RwSignal::new(Vec::<i32>::new());
    let sidebar_collapsed = RwSignal::new(false);
    
    // Fetch location stats (use LocalResource to avoid hydration issues)
    let location_stats = LocalResource::new(
        move || (city.get(), state.get()),
        move |(city, state)| async move { 
            get_location_stats(city, state).await.unwrap_or_default()
        },
    );
    
    // Fetch available styles (use LocalResource to avoid hydration issues)
    let available_styles = LocalResource::new(
        || (),
        |_| async move { 
            get_available_styles().await.unwrap_or_default()
        },
    );
    
    let handle_search = move |_ev: web_sys::MouseEvent| {
        let search_value = search_input.get();
        if !search_value.is_empty() {
            // Check if it's a zip code
            if search_value.chars().all(|c| c.is_numeric()) && search_value.len() == 5 {
                spawn_local(async move {
                    if let Ok(coords) = search_by_postal_code(search_value).await {
                        city.set(coords.city.clone());
                        state.set(coords.state);
                    }
                });
            }
        }
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
                    
                    <div class="search-section">
                        <input
                            type="text"
                            class="search-input"
                            placeholder="Search by city or zip code..."
                            on:input=move |ev| {
                                let value = event_target_value(&ev);
                                search_input.set(value);
                            }
                        />
                        <button class="search-button" on:click=handle_search>
                            "Search"
                        </button>
                    </div>
                    
                    <div class="location-stats">
                        <Suspense fallback=|| view! { <span>"Loading stats..."</span> }>
                            {move || {
                                location_stats.get().map(|stats| {
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
                                }
                            }).unwrap_or_else(|| view! { <span>"No stats available"</span> })}
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
                        
                        // Style filters (simplified)
                        <div class="filter-section style-filters">
                            <h3>"Tattoo Styles"</h3>
                            <div>"Style filtering coming soon"</div>
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
                        default_location=default_location
                        cities=cities
                        selected_styles=selected_styles
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