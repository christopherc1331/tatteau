use leptos::prelude::*;
use thaw_utils::Model;

use crate::{
    components::loading::LoadingView,
    map::{
        drop_down_cities::DropDownCities, drop_down_states::DropDownStates,
        map_renderer::MapRenderer,
    },
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

    view! {
        <Suspense fallback=move || view! {
                    <LoadingView message=Some("Fetching locations...".to_string()) />
                }>
            <DropDownStates states=states_resource state_model=state_model/>
            <DropDownCities cities=cities_resource selected_city=city/>
            <MapRenderer locations=locations_resource selected_city_coords=selected_city_coords/>
        </Suspense>
    }
}
