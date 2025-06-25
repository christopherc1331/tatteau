use crate::{
    components::{error::ErrorView, loading::LoadingView},
    db::entities::CityCoords,
    server::fetch_locations,
};
use leptos::prelude::*;
use leptos_leaflet::{leaflet::Map, prelude::*};
use thaw::{Label, LabelSize};

#[component]
pub fn MapRenderer(
    city: RwSignal<String>,
    default_city: CityCoords,
    cities: Resource<Result<Vec<CityCoords>, ServerFnError>>,
) -> impl IntoView {
    let locations = Resource::new(
        move || city.get(),
        move |city| async move { fetch_locations(city).await },
    );

    let selected_city_coords = RwSignal::new(default_city.clone());

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

    Effect::new(move |_| {
        if let Some(Ok(city_coords_list)) = cities.get() {
            let matching_city = city_coords_list.into_iter().find(|c| c.city == city.get());
            if let Some(found_city) = matching_city {
                selected_city_coords.set(found_city);
            }
        }
    });

    view! {
        <MapContainer style="height: 70vh" center=center.get() zoom=12.0 set_view=true map=map.write_only()>
            <TileLayer
                url="https://tile.openstreetmap.org/{z}/{x}/{y}.png"
                attribution="&copy; <a href=\"https://www.openstreetmap.org/copyright\">OpenStreetMap</a> contributors"
            />
            {move ||
                match locations.get() {
                    Some(Ok(locations)) => view! {
                            {locations.into_iter().map(|loc| {
                                view! {
                                    <Marker position=Position::new(loc.lat, loc.long) draggable=false>
                                        <Popup>
                                            <Label size=LabelSize::Large>{loc.name.clone()}</Label>
                                            <p>{format!("Address: {}", loc.address)}</p>
                                            <a href=loc.website_uri target="_blank">{loc.website_uri.clone()}</a>
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
    }
}
