use crate::{
    components::{error::ErrorView, loading::LoadingView},
    db::entities::CityCoords,
    server::fetch_locations,
};
use leptos::{leptos_dom::logging::console_log, prelude::*};
use leptos_leaflet::{
    leaflet::{LatLng, LatLngBounds, Map},
    prelude::*,
};
use shared_types::{LatLong, MapBounds};
use thaw::{Label, LabelSize};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{Event, EventTarget};

#[component]
pub fn MapRenderer(
    state: RwSignal<String>,
    city: RwSignal<String>,
    default_location: CityCoords,
    cities: Resource<Result<Vec<CityCoords>, ServerFnError>>,
) -> impl IntoView {
    let selected_city_coords = RwSignal::new(default_location.clone());

    let center: Memo<Position> = Memo::new(move |_| {
        let CityCoords { lat, long, .. } = selected_city_coords.get();
        Position::new(lat, long)
    });

    let bounds: RwSignal<MapBounds> = RwSignal::new(MapBounds::default());
    let map: JsRwSignal<Option<Map>> = JsRwSignal::new_local(None::<Map>);
    let update_bounds = move |_| {
        if let Some(map) = map.get_untracked() {
            let map_bounds: LatLngBounds = map.get_bounds();
            let north_east: LatLng = map_bounds.get_north_east();
            let south_west: LatLng = map_bounds.get_south_west();
            bounds.set(MapBounds {
                north_east: LatLong {
                    lat: north_east.lat(),
                    long: north_east.lng(),
                },
                south_west: LatLong {
                    lat: south_west.lat(),
                    long: south_west.lng(),
                },
            })
        }
    };

    Effect::new(move |_| {
        let new_pos = center.get();
        if let Some(map) = map.get_untracked() {
            map.set_view(&new_pos.as_lat_lng(), map.get_zoom());
            update_bounds(());
        }
    });

    Effect::new(move |_| {
        let Some(map_instance) = map.get() else {
            return;
        };

        let cb: Closure<dyn FnMut(Event)> = Closure::wrap(Box::new(move |_event| {
            update_bounds(());
        }));

        let raw_map: &EventTarget = map_instance.unchecked_ref();
        raw_map
            .add_event_listener_with_callback("moveend", cb.as_ref().unchecked_ref())
            .expect("Failed to attach");

        cb.forget();
    });

    let locations = Resource::new(
        move || (state.get(), city.get(), bounds.get()),
        move |(state, city, bounds)| async move { fetch_locations(state, city, bounds).await },
    );

    Effect::new(move |_| {
        if let Some(Ok(city_coords_list)) = cities.get() {
            let matching_city = city_coords_list.into_iter().find(|c| c.city == city.get());
            if let Some(found_city) = matching_city {
                selected_city_coords.set(found_city);
            }
        }
    });

    view! {
        <MapContainer
            style="height: 100%; width: 100%; flex: 1"
            center=center.get()
            zoom=12.0
            set_view=true
            map=map.write_only()
        >
            <TileLayer
                url="https://tile.openstreetmap.org/{z}/{x}/{y}.png"
                attribution="&copy; <a href=\"https://www.openstreetmap.org/copyright\">OpenStreetMap</a> contributors"
            />
            {move ||
                match locations.get() {
                    Some(Ok(locations)) => view! {
                            {locations.into_iter().map(|loc| {
                                // Balanced visibility - vibrant enough to see, elegant enough to look good
                                let fill_color = if !loc.has_artists {
                                    "%236b7280" // Darker gray for better visibility
                                } else if loc.artist_images_count == 0 {
                                    "%23f97316" // Vibrant orange
                                } else {
                                    "%235b21b6" // Rich purple (matches your site theme)
                                };
                                
                                // Clean design with better contrast and subtle shadow - slightly larger
                                let icon_svg = format!(
                                    "data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='28' height='42' viewBox='0 0 28 42'%3E%3Cdefs%3E%3Cfilter id='shadow' x='-50%25' y='-50%25' width='200%25' height='200%25'%3E%3CfeDropShadow dx='0' dy='1' stdDeviation='1.5' flood-color='%23000' flood-opacity='0.25'/%3E%3C/filter%3E%3C/defs%3E%3Cpath fill='{}' stroke='%23ffffff' stroke-width='1.5' filter='url(%23shadow)' d='M14 2C8.5 2 4 6.5 4 12c0 8.5 10 26 10 26s10-17.5 10-26c0-5.5-4.5-10-10-10zm0 13.5c-1.9 0-3.5-1.6-3.5-3.5s1.6-3.5 3.5-3.5 3.5 1.6 3.5 3.5-1.6 3.5-3.5 3.5z'/%3E%3C/svg%3E",
                                    fill_color
                                );
                                
                                view! {
                                    <Marker 
                                        position=Position::new(loc.lat, loc.long) 
                                        draggable=false
                                        icon_url=Some(icon_svg)
                                        icon_size=Some((28.0, 42.0))
                                        icon_anchor=Some((14.0, 42.0))
                                    >
                                        <Popup>
                                            <Label size=LabelSize::Large>{loc.name.clone()}</Label>
                                            <p>{format!("Address: {}", loc.address)}</p>
                                            <div style="margin: 0.5rem 0; display: flex; flex-direction: column; gap: 0.5rem;">
                                                {if !loc.has_artists {
                                                    view! {
                                                        <a href=loc.website_uri target="_blank" 
                                                           style="background: #667eea; color: white; padding: 0.5rem 1rem; border-radius: 6px; text-decoration: none; text-align: center; font-weight: 600;">
                                                            "Visit Website"
                                                        </a>
                                                    }.into_any()
                                                } else if loc.artist_images_count == 0 {
                                                    view! {
                                                        <a href={format!("/shop/{}", loc.id)} 
                                                           style="background: #667eea; color: white; padding: 0.5rem 1rem; border-radius: 6px; text-decoration: none; text-align: center; font-weight: 600;">
                                                            "View Shop Info"
                                                        </a>
                                                    }.into_any()
                                                } else {
                                                    view! {
                                                        <a href={format!("/shop/{}", loc.id)} 
                                                           style="background: #667eea; color: white; padding: 0.5rem 1rem; border-radius: 6px; text-decoration: none; text-align: center; font-weight: 600;">
                                                            "View Shop Portfolio"
                                                        </a>
                                                    }.into_any()
                                                }}
                                            </div>
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
