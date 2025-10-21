use crate::{
    components::loading::LoadingView, db::entities::CityCoords, server::get_locations_with_details,
    views::map::enhanced_map_marker::EnhancedMapMarker,
};
use leptos::prelude::*;
use leptos_leaflet::prelude::*;
use shared_types::MapBounds;

#[cfg(not(feature = "ssr"))]
use leptos_leaflet::leaflet::Map;

#[cfg(not(feature = "ssr"))]
use wasm_bindgen::JsCast;

#[component]
pub fn MapRendererV2(
    state: RwSignal<String>,
    city: RwSignal<String>,
    default_location: CityCoords,
    cities: Resource<Result<Vec<CityCoords>, ServerFnError>>,
    selected_styles: RwSignal<Vec<i32>>,
    map_bounds: RwSignal<MapBounds>,
) -> impl IntoView {
    // Track selected city coordinates
    let selected_city_coords = RwSignal::new(default_location.clone());

    // Create center memo
    let center: Memo<Position> = Memo::new(move |_| {
        let CityCoords { lat, long, .. } = selected_city_coords.get();
        Position::new(lat, long)
    });

    // Map signal - only on client
    #[cfg(not(feature = "ssr"))]
    let map = JsRwSignal::new_local(None::<Map>);

    // Track if map is ready to render (avoid hydration issues)
    let map_ready = RwSignal::new(false);

    #[cfg(not(feature = "ssr"))]
    {
        // Delay map rendering until after hydration
        Effect::new(move |_| {
            let window = web_sys::window().expect("no global `window` exists");
            let _ = window.request_animation_frame(
                wasm_bindgen::closure::Closure::once_into_js(move || {
                    map_ready.set(true);
                })
                .as_ref()
                .unchecked_ref(),
            );
        });

        // Set up map bounds tracking when map is ready
        Effect::new(move |_| {
            if let Some(map_instance) = map.read_only().get() {
                // Create closure for moveend event
                let bounds_signal = map_bounds;
                let closure = wasm_bindgen::closure::Closure::wrap(Box::new(move || {
                    if let Some(map_inst) = map.read_only().get() {
                        let leaflet_bounds = map_inst.get_bounds();
                        let ne = leaflet_bounds.get_north_east();
                        let sw = leaflet_bounds.get_south_west();

                        let new_bounds = MapBounds {
                            north_east: shared_types::LatLong {
                                lat: ne.lat(),
                                long: ne.lng(),
                            },
                            south_west: shared_types::LatLong {
                                lat: sw.lat(),
                                long: sw.lng(),
                            },
                        };

                        leptos::logging::log!(
                            "Map bounds updated: NE({}, {}), SW({}, {})",
                            new_bounds.north_east.lat,
                            new_bounds.north_east.long,
                            new_bounds.south_west.lat,
                            new_bounds.south_west.long
                        );
                        bounds_signal.set(new_bounds);
                    }
                })
                    as Box<dyn FnMut()>);

                // Add event listener
                map_instance.on("moveend", closure.as_ref().unchecked_ref());

                // Keep closure alive
                closure.forget();

                // Set initial bounds
                let leaflet_bounds = map_instance.get_bounds();
                let ne = leaflet_bounds.get_north_east();
                let sw = leaflet_bounds.get_south_west();

                let initial_bounds = MapBounds {
                    north_east: shared_types::LatLong {
                        lat: ne.lat(),
                        long: ne.lng(),
                    },
                    south_west: shared_types::LatLong {
                        lat: sw.lat(),
                        long: sw.lng(),
                    },
                };

                leptos::logging::log!(
                    "Initial map bounds: NE({}, {}), SW({}, {})",
                    initial_bounds.north_east.lat,
                    initial_bounds.north_east.long,
                    initial_bounds.south_west.lat,
                    initial_bounds.south_west.long
                );
                map_bounds.set(initial_bounds);
            }
        });
    }

    // Fetch locations based on current map bounds
    let locations = LocalResource::new(move || async move {
        let current_state = state.get();
        let current_city = city.get();
        let current_styles = selected_styles.get();
        let current_bounds = map_bounds.get();

        // Only fetch if we have valid bounds (not the default 0,0)
        if current_bounds.north_east.lat == 0.0 && current_bounds.south_west.lat == 0.0 {
            leptos::logging::log!("Map bounds not initialized yet, skipping location fetch");
            return Ok(vec![]);
        }

        leptos::logging::log!(
            "Fetching locations for bounds: NE({}, {}), SW({}, {})",
            current_bounds.north_east.lat,
            current_bounds.north_east.long,
            current_bounds.south_west.lat,
            current_bounds.south_west.long
        );

        get_locations_with_details(
            current_state,
            current_city,
            current_bounds,
            if current_styles.is_empty() {
                None
            } else {
                Some(current_styles)
            },
        )
        .await
    });

    // Update city coordinates when cities resource changes
    Effect::new(move |_| {
        if let Some(Ok(city_coords_list)) = cities.get() {
            let current_city = city.get();
            let matching_city = city_coords_list
                .into_iter()
                .find(|c| c.city == current_city);
            if let Some(found_city) = matching_city {
                selected_city_coords.set(found_city);
            }
        }
    });

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

                                {move || {
                                    match locations.get() {
                                        Some(result) => {
                                            match result.as_ref() {
                                                Ok(locs) => {
                                                    leptos::logging::log!("Rendering {} markers", locs.len());
                                                    let current_styles = selected_styles.get();
                                                    let styles_opt = if current_styles.is_empty() {
                                                        None
                                                    } else {
                                                        Some(current_styles)
                                                    };
                                                    view! {
                                                        {locs.iter().map(move |enhanced_loc| {
                                                            view! {
                                                                <EnhancedMapMarker location=enhanced_loc.clone() selected_styles=styles_opt.clone() />
                                                            }
                                                        }).collect_view()}
                                                    }.into_any()
                                                },
                                                Err(err) => {
                                                    leptos::logging::log!("Error loading locations: {:?}", err);
                                                    view! { <></> }.into_any()
                                                }
                                            }
                                        },
                                        None => {
                                            leptos::logging::log!("Locations resource not ready yet");
                                            view! { <></> }.into_any()
                                        }
                                    }
                                }}
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
        </div>
    }
}
