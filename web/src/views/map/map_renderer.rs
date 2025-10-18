use crate::{
    components::{error::ErrorView, loading::LoadingView},
    db::entities::CityCoords,
    server::{fetch_locations, get_locations_with_details},
    views::map::{enhanced_map_marker::EnhancedMapMarker, map_marker::MapMarker},
};
use leptos::{leptos_dom::logging::console_log, prelude::*};
use leptos_leaflet::{
    leaflet::{LatLng, LatLngBounds, Map},
    prelude::*,
};
use shared_types::{LatLong, MapBounds};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{Event, EventTarget};

#[component]
pub fn MapRenderer(
    state: RwSignal<String>,
    city: RwSignal<String>,
    default_location: CityCoords,
    cities: Resource<Result<Vec<CityCoords>, ServerFnError>>,
    selected_styles: RwSignal<Vec<i32>>,
    map_bounds: RwSignal<MapBounds>,
) -> impl IntoView {
    let selected_city_coords = RwSignal::new(default_location.clone());

    let center: Memo<Position> = Memo::new(move |_| {
        let CityCoords { lat, long, .. } = selected_city_coords.get();
        Position::new(lat, long)
    });

    // Map signal - only create on client side to avoid SendWrapper threading issues
    #[cfg(not(feature = "ssr"))]
    let map = JsRwSignal::new_local(None::<Map>);

    let locations = Resource::new(
        move || {
            (
                state.get(),
                city.get(),
                map_bounds.get(),
                selected_styles.get(),
            )
        },
        move |(state, city, bounds, styles)| async move {
            get_locations_with_details(
                state,
                city,
                bounds,
                if styles.is_empty() {
                    None
                } else {
                    Some(styles)
                },
            )
            .await
        },
    );

    Effect::new(move |_| {
        if let Some(Ok(city_coords_list)) = cities.get() {
            let matching_city = city_coords_list.into_iter().find(|c| c.city == city.get());
            if let Some(found_city) = matching_city {
                selected_city_coords.set(found_city);
            }
        }
    });

    // Map is only ready on client side (SSR should show loading state)
    let map_ready = RwSignal::new(false);

    #[cfg(not(feature = "ssr"))]
    {
        // On client side, mark map as ready immediately
        map_ready.set(true);
    }

    view! {
        <div class="map-renderer-container">
            {move || {
                #[cfg(not(feature = "ssr"))]
                {
                    if map_ready.get() {
                        view! {
                            <MapContainer
                                class="map-renderer-map-container"
                                center=center.get()
                                zoom=12.0
                                set_view=true
                                map=map.write_only()
                            >
                                <TileLayer
                                    url="https://tile.openstreetmap.org/{z}/{x}/{y}.png"
                                    attribution="&copy; <a href=\"https://www.openstreetmap.org/copyright\">OpenStreetMap</a> contributors"
                                />

                                {move ||
                                    match locations.get() {
                                        Some(Ok(locations)) => {
                                            let current_styles = selected_styles.get();
                                            let styles_opt = if current_styles.is_empty() {
                                                None
                                            } else {
                                                Some(current_styles)
                                            };
                                            view! {
                                                {locations.into_iter().map(move |enhanced_loc| {
                                                    view! {
                                                        <EnhancedMapMarker location=enhanced_loc selected_styles=styles_opt.clone() />
                                                    }
                                                }).collect_view()}
                                            }.into_any()
                                        },
                                        Some(Err(err)) => {
                                            leptos::logging::log!("Error occurred while fetching locations: {}", err);
                                            view! { <></> }.into_any()
                                        },
                                        None => view! { <></> }.into_any(),
                                    }
                                }
                            </MapContainer>
                        }.into_any()
                    } else {
                        view! {
                            <div class="map-renderer-loading-container">
                                <LoadingView message=Some("Initializing map...".to_string()) />
                            </div>
                        }.into_any()
                    }
                }

                #[cfg(feature = "ssr")]
                {
                    view! {
                        <div class="map-renderer-loading-container">
                            <LoadingView message=Some("Initializing map...".to_string()) />
                        </div>
                    }.into_any()
                }
            }}

            // Loading overlay for when locations are being fetched (positioned absolutely over the map)
            // {move || {
            //     match locations.get() {
            //         None => {
            //             // Resource is loading
            //             view! {
            //                 <div class="map-loading-overlay">
            //                     <div class="map-loading-content">
            //                         <div class="map-loading-spinner"></div>
            //                         <span>"Loading markers..."</span>
            //                     </div>
            //                 </div>
            //             }.into_any()
            //         },
            //         Some(_) => {
            //             // Resource has loaded (either success or error)
            //             view! { <></> }.into_any()
            //         }
            //     }
            // }}
        </div>
    }
}
