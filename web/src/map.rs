use crate::server::fetch_locations;
use leptos::{server::Resource, *};
use leptos_leaflet::prelude::*;
use shared_types::LocationInfo;
use thaw::{Label, LabelSize};

#[component]
pub fn DiscoveryMap() -> impl IntoView {
    let locations = Resource::new(|| (), |_| async move { fetch_locations().await });

    view! {
        <MapContainer style="height: 400px" center=Position::new(32.482209, -96.994499) zoom=12.0 set_view=true>
            <TileLayer url="https://tile.openstreetmap.org/{z}/{x}/{y}.png" attribution="&copy; <a href=\"https://www.openstreetmap.org/copyright\">OpenStreetMap</a> contributors"/>
            <Marker position=position!(51.5, -0.065) draggable=true>
                <Popup>
                    <Label size=LabelSize::Large>{"A pretty CSS3 popup"}</Label>
                </Popup>
            </Marker>
        </MapContainer>
    }
}
