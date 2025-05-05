use leptos::prelude::*;
use leptos_leaflet::{leaflet::Map, prelude::*};
use thaw::{Label, LabelSize, MessageBar, MessageBarIntent, Select, Spinner, SpinnerSize};
use thaw_utils::Model;

use crate::server::{fetch_locations, get_cities, get_states_list, CityCoords};

#[component]
pub fn LoadingView(message: Option<String>) -> impl IntoView {
    view! {
        <div class="flex flex-col items-center justify-center p-4">
            <Spinner size=SpinnerSize::Large />
            <p class="mt-2 text-gray-600">
                {message.unwrap_or_else(|| "Loading, please wait...".to_string())}
            </p>
        </div>
    }
}

#[component]
pub fn ErrorView(message: Option<String>) -> impl IntoView {
    view! {
        <MessageBar intent=MessageBarIntent::Error>
            {message.unwrap_or_else(|| "An error occurred. Please try again.".to_string())}
        </MessageBar>
    }
}

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
    // TODO: add create effect that searches for city coord record
    // based on currently selected city and assigns found result to selected city signal
    // with the selected city coord we can set the default position that the map centers on
    //
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
        if let Some(Ok(city_coords_list)) = cities_resource.get() {
            println!("Selected city coordinates list: {:?}", city_coords_list);
            let matching_city = city_coords_list.into_iter().find(|c| c.city == city.get());
            if let Some(found_city) = matching_city {
                selected_city_coords.set(found_city);
            }
        }
    });

    Effect::new(move |_| {
        println!(
            "Selected city coordinates: {:?}",
            selected_city_coords.get()
        )
    });

    Effect::new(move |_| println!("City changed: {:?}", city.get()));

    // let center: Memo<Position> = Memo::new(move |_| {
    //     let coords = selected_city_coords.get();
    //     Position::new(coords.lat, coords.long)
    // });

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
            <MapContainer style="height: 70vh" center=Position::new(32.855895000000004, -96.8662097) zoom=12.0 set_view=true>
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
