use crate::{
    components::{error::ErrorView, loading::LoadingView},
    db::entities::CityCoords,
    server::fetch_locations,
    views::map::map_marker::MapMarker,
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
) -> impl IntoView {
    let selected_city_coords = RwSignal::new(default_location.clone());

    let center: Memo<Position> = Memo::new(move |_| {
        let CityCoords { lat, long, .. } = selected_city_coords.get();
        Position::new(lat, long)
    });

    let bounds: RwSignal<MapBounds> = RwSignal::new(MapBounds::default());
    let map: JsRwSignal<Option<Map>> = JsRwSignal::new_local(None::<Map>);
    let update_bounds = move |_| {
        if let Some(map) = map.get_untracked() {
            let map_bounds: LatLngBounds = map.get_bounds();
            let north_east: LatLng = map_bounds.get_north_east();
            let south_west: LatLng = map_bounds.get_south_west();
            bounds.set(MapBounds {
                north_east: LatLong {
                    lat: north_east.lat(),
                    long: north_east.lng(),
                },
                south_west: LatLong {
                    lat: south_west.lat(),
                    long: south_west.lng(),
                },
            })
        }
    };

    Effect::new(move |_| {
        let new_pos = center.get();
        if let Some(map) = map.get_untracked() {
            map.set_view(&new_pos.as_lat_lng(), map.get_zoom());
            update_bounds(());
        }
    });

    Effect::new(move |_| {
        let Some(map_instance) = map.get() else {
            return;
        };

        let cb: Closure<dyn FnMut(Event)> = Closure::wrap(Box::new(move |_event| {
            update_bounds(());
        }));

        let raw_map: &EventTarget = map_instance.unchecked_ref();
        raw_map
            .add_event_listener_with_callback("moveend", cb.as_ref().unchecked_ref())
            .expect("Failed to attach");

        cb.forget();
    });

    let locations = Resource::new(
        move || (state.get(), city.get(), bounds.get()),
        move |(state, city, bounds)| async move { fetch_locations(state, city, bounds).await },
    );

    Effect::new(move |_| {
        if let Some(Ok(city_coords_list)) = cities.get() {
            let matching_city = city_coords_list.into_iter().find(|c| c.city == city.get());
            if let Some(found_city) = matching_city {
                selected_city_coords.set(found_city);
            }
        }
    });

    let map_ready = RwSignal::new(false);

    Effect::new(move |_| {
        let closure = Closure::wrap(Box::new(move || {
            map_ready.set(true);
        }) as Box<dyn FnMut()>);

        let window = web_sys::window().expect("Failed to get window");
        window
            .set_timeout_with_callback_and_timeout_and_arguments_0(
                closure.as_ref().unchecked_ref(),
                50,
            )
            .expect("Failed to set timeout");

        closure.forget();
    });

    Effect::new(move |_| {
        if map.get().is_some() {
            // Give the map a moment to render, then update bounds
            let closure = Closure::wrap(Box::new(move || {
                update_bounds(());
            }) as Box<dyn FnMut()>);

            let window = web_sys::window().expect("Failed to get window");
            window
                .set_timeout_with_callback_and_timeout_and_arguments_0(
                    closure.as_ref().unchecked_ref(),
                    100,
                )
                .expect("Failed to set timeout");

            closure.forget();
        }
    });

    view! {
        {move || if map_ready.get() {
            view! {
                <MapContainer
                    style="height: 100%; width: 100%; flex: 1"
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
                            Some(Ok(locations)) => view! {
                                    {locations.into_iter().map(|loc| {
                                        view! {
                                            <MapMarker location=loc />
                                        }
                                    }).collect_view()}
                            }.into_any(),
                            Some(Err(err)) => {
                                println!("Error occurred while fetching locations: {}", err);
                                view! {
                                    <ErrorView message=Some("Error fetching locations...".to_string()) />
                                }.into_any()
                            },
                            None => view! {
                                <LoadingView message=Some("Fetching locations...".to_string()) />
                            }.into_any(),
                        }
                    }
                </MapContainer>
            }.into_any()
        } else {
            view! {
                <div style="height: 100%; width: 100%; flex: 1; display: flex; align-items: center; justify-content: center;">
                    <LoadingView message=Some("Initializing map...".to_string()) />
                </div>
            }.into_any()
        }}
    }
}
