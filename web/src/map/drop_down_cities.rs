use leptos::prelude::*;
use thaw::Select;

use crate::{
    components::{error::ErrorView, loading::LoadingView},
    db::entities::CityCoords,
};

#[component]
pub fn DropDownCities(
    cities: Resource<Result<Vec<CityCoords>, ServerFnError>>,
    selected_city: RwSignal<String>,
) -> impl IntoView {
    view! {
        {move ||
            match cities.get() {
                Some(Ok(cities)) => view! {
                    <Select
                        on:change=move |ev| {
                            let value = event_target_value(&ev);
                            selected_city.set(value);
                        }
                    >
                        {cities.into_iter().map(|city| {
                            view! {
                                <option>{city.city}</option>
                            }
                        }).collect_view()}
                    </Select>
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
    }
}
