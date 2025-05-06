use crate::{
    components::{error::ErrorView, loading::LoadingView},
    server::CityCoords,
};
use leptos::prelude::*;
use leptos_leaflet::{leaflet::Map, prelude::*};
use shared_types::LocationInfo;
use thaw::{Label, LabelSize};

#[component]
pub fn MapRenderer(
    selected_city_coords: RwSignal<CityCoords>,
    locations: Resource<Result<Vec<LocationInfo>, ServerFnError>>,
) -> impl IntoView {
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
