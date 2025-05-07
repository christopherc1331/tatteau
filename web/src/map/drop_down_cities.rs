use leptos::prelude::*;
use thaw::Select;

use crate::{
    components::{error::ErrorView, loading::LoadingView},
    db::entities::CityCoords,
};

#[component]
pub fn DropDownCities(
    city: RwSignal<String>,
    cities: Resource<Result<Vec<CityCoords>, ServerFnError>>,
) -> impl IntoView {
    Effect::new(move |_| {
        if let Some(Ok(cities)) = cities.get() {
            city.set(cities[0].clone().city);
        }
    });

    view! {
        {move ||
            match cities.get() {
                Some(Ok(cities)) => view! {
                    <Select
                        on:change=move |ev| {
                            let value = event_target_value(&ev);
                            city.set(value);
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
