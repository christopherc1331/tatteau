use leptos::prelude::*;
use leptos_leaflet::{leaflet::Map, prelude::*};
use thaw::{Label, LabelSize, Select};
use thaw_utils::Model;

use crate::{
    components::{error::ErrorView, loading::LoadingView},
    server::{fetch_locations, get_cities, get_states_list, CityCoords},
};

#[component]
pub fn DiscoveryMap() -> impl IntoView {
    let state = RwSignal::new("Texas".to_string());
    let default_city = CityCoords {
        city: "Dallas".to_string(),
        state: "Texas".to_string(),
        lat: 32.855895000000004,
        long: -96.8662097,
    };
    let city = RwSignal::new(default_city.clone().city);
    let selected_city_coords = RwSignal::new(default_city.clone());

    let state_model: Model<String> = state.into();

    let locations_resource = Resource::new(
        move || city.get(),
        move |city| async move { fetch_locations(city).await },
    );

    let states_resource = OnceResource::new(async move { get_states_list().await });

    let cities_resource = Resource::new(
        move || state.get(),
        move |state| async move { get_cities(state).await },
    );

    Effect::new(move |_| {
        if let Some(Ok(cities)) = cities_resource.get() {
            city.set(cities[0].clone().city);
        }
    });

    Effect::new(move |_| {
        if let Some(Ok(city_coords_list)) = cities_resource.get() {
            let matching_city = city_coords_list.into_iter().find(|c| c.city == city.get());
            if let Some(found_city) = matching_city {
                selected_city_coords.set(found_city);
            }
        }
    });

    let center: Memo<Position> = Memo::new(move |_| {
        let CityCoords { lat, long, .. } = selected_city_coords.get();
        Position::new(lat, long)
    });

    let map = JsRwSignal::new_local(None::<Map>);
    Effect::new(move |_| {
        let new_pos = center.get();
        if let Some(map) = map.get_untracked() {
            map.set_view(&new_pos.as_lat_lng(), map.get_zoom());
        }
    });

    view! {
        <Suspense fallback=move || view! {
                    <LoadingView message=Some("Fetching locations...".to_string()) />
                }>
            {move ||
                match states_resource.get() {
                    Some(Ok(states)) => view! {
                        <Select value=state_model>
                            {states.into_iter().map(|state| {
                                view! {
                                    <option>{state}</option>
                                }
                            }).collect_view()}
                        </Select>
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
            {move ||
                match cities_resource.get() {
                    Some(Ok(cities)) => view! {
                        <Select
                            on:change=move |ev| {
                                let value = event_target_value(&ev);
                                city.set(value);
                            }
                        >
                            {cities.into_iter().map(|city| {
                                view! {
                                    <option>{city.city}</option>
                                }
                            }).collect_view()}
                        </Select>
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
            <MapContainer style="height: 70vh" center=center.get() zoom=12.0 set_view=true map=map.write_only()>
                <TileLayer
                    url="https://tile.openstreetmap.org/{z}/{x}/{y}.png"
                    attribution="&copy; <a href=\"https://www.openstreetmap.org/copyright\">OpenStreetMap</a> contributors"
                />
                {move ||
                    match locations_resource.get() {
                        Some(Ok(locations)) => view! {
                                {locations.into_iter().map(|loc| {
                                    view! {
                                        <Marker position=Position::new(loc.lat, loc.long) draggable=false>
                                            <Popup>
                                                <Label size=LabelSize::Large>{loc.name.clone()}</Label>
                                                <p>{format!("Address: {}", loc.address)}</p>
                                            </Popup>
                                        </Marker>
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
        </Suspense>
    }
}
