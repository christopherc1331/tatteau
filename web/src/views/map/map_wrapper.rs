use leptos::prelude::*;
use leptos_leaflet::{leaflet::LatLngBounds, prelude::JsRwSignal};
use thaw::Flex;

use crate::{
    components::loading::LoadingView,
    db::entities::CityCoords,
    server::get_cities,
    views::map::{
        drop_down_cities::DropDownCities, drop_down_states::DropDownStates,
        map_renderer::MapRenderer,
    },
};

#[component]
pub fn DiscoveryMap() -> impl IntoView {
    let state = RwSignal::new("Texas".to_string());
    let default_location = CityCoords {
        city: "Dallas".to_string(),
        state: "Texas".to_string(),
        lat: 32.855895000000004,
        long: -96.8662097,
    };
    let city = RwSignal::new(default_location.clone().city);
    let cities = Resource::new(
        move || state.get(),
        move |state| async move { get_cities(state).await },
    );

    view! {
        <Suspense fallback=|| view! {
            <LoadingView message=Some("Fetching locations...".to_string()) />
        }>
            <Flex attr:style="padding-bottom: 8px">
                <DropDownStates state=state />
                <DropDownCities city=city cities=cities/>
            </Flex>
            <MapRenderer state=state city=city default_location=default_location cities=cities/>
        </Suspense>
    }
}
