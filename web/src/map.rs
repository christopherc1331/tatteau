use leptos::prelude::*;
use leptos_leaflet::prelude::*;
use shared_types::LocationInfo;
use thaw::{Icon, Label, LabelSize, MessageBar, MessageBarIntent, Spinner, SpinnerSize};

use crate::server::fetch_locations;

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
    let (city, _) = signal("Dallas".to_string());

    let locations_resource = Resource::new(
        move || city.read().clone(),
        move |city| async move { fetch_locations(city).await },
    );

    view! {
        <Suspense fallback=move || view! {
                    <LoadingView message=Some("Fetching locations...".to_string()) />
                }>
            {move ||
                match locations_resource.get() {
                    Some(Ok(locations)) => view! {
                        <MapContainer style="height: 70vh" center=Position::new(32.482209, -96.994499) zoom=12.0 set_view=true>
                            <TileLayer
                                url="https://tile.openstreetmap.org/{z}/{x}/{y}.png"
                                attribution="&copy; <a href=\"https://www.openstreetmap.org/copyright\">OpenStreetMap</a> contributors"
                            />

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
                        </MapContainer>
                    }.into_any(),
                    Some(Err(err)) => view! {
                        <ErrorView message=Some(err.to_string()) />
                    }.into_any(),
                    None => view! {
                        <LoadingView message=Some("Fetching locations...".to_string()) />
                    }.into_any(),
                }
            }
        </Suspense>
    }
}
