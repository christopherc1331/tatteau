use std::time::Duration;

use leptos::{prelude::ServerFnError, *};
use leptos_leaflet::prelude::*;
use shared_types::LocationInfo;
use thaw::{Label, LabelSize};

#[server]
pub async fn fetch_locations() -> Result<Vec<LocationInfo>, ServerFnError> {
    // TODO: Implement actual data fetching from your backend
    Ok(Vec::new())
}

#[component]
pub fn DiscoveryMap() -> impl IntoView {
    // let locations = create_resource(|| (), |_| async move { fetch_locations().await });

    view! {
        <MapContainer style="height: 400px" center=Position::new(51.505, -0.09) zoom=13.0 set_view=true>
            <TileLayer url="https://tile.openstreetmap.org/{z}/{x}/{y}.png" attribution="&copy; <a href=\"https://www.openstreetmap.org/copyright\">OpenStreetMap</a> contributors"/>
            <Marker position=position!(51.5, -0.065) draggable=true>
                <Popup>
                    <Label size=LabelSize::Large>{"A pretty CSS3 popup"}</Label>
                </Popup>
            </Marker>
        </MapContainer>
    }
}
