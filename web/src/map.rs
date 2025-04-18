use std::time::Duration;

use leptos::{
    prelude::{set_interval_with_handle, Effect, ServerFnError, Update},
    server::Resource,
    *,
};
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
    let locations = Resource::new(|| (), |_| async move { fetch_locations().await });

    let (marker_position, set_marker_position) =
        JsRwSignal::new_local(Position::new(51.49, -0.08)).split();

    Effect::new(move |_| {
        set_interval_with_handle(
            move || {
                set_marker_position.update(|pos| {
                    pos.lat += 0.001;
                    pos.lng += 0.001;
                });
            },
            Duration::from_millis(200),
        )
        .ok()
    });

    view! {
          <MapContainer style="height: 400px" center=Position::new(51.505, -0.09) zoom=13.0 set_view=true>
              <TileLayer url="https://tile.openstreetmap.org/{z}/{x}/{y}.png" attribution="&copy; <a href=\"https://www.openstreetmap.org/copyright\">OpenStreetMap</a> contributors"/>
                <Marker position=position!(51.5, -0.065) draggable=true >
                  <Popup>
                      <Label size=LabelSize::Large>{"A pretty CSS3 popup"}</Label>
                  </Popup>
              </Marker>
        </MapContainer>
    }
}
