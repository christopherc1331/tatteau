use std::time::Duration;

use leptos::prelude::*;
use leptos_leaflet::prelude::*;
use thaw::{Label, LabelSize};

#[server]
pub async fn fetch_locations() -> Result<(), ServerFnError> {
    todo!()
}

#[component]
pub fn DiscoveryMap() -> impl IntoView {
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
