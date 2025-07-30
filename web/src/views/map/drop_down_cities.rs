use leptos::prelude::*;
use thaw::{Combobox, ComboboxOption, Flex, FlexAlign, Label};

use crate::{
    components::{error::ErrorView, loading::LoadingView},
    db::entities::CityCoords,
};

#[component]
pub fn DropDownCities(
    city: RwSignal<String>,
    cities: Resource<Result<Vec<CityCoords>, ServerFnError>>,
) -> impl IntoView {
    let selected_options: RwSignal<Option<String>> = RwSignal::new(Some(city.get_untracked()));
    Effect::new(move |_| {
        if let Some(val) = selected_options.get() {
            city.set(val);
        }
    });

    view! {
        {move ||
            match cities.get() {
                Some(Ok(cities)) => view! {
                    <Flex vertical=true align=FlexAlign::Start>
                        <Label>"City"</Label>
                        <Combobox selected_options placeholder="Select a city">
                            {cities.into_iter().map(|city| {
                                let city_name = city.city.clone();
                                view! {
                                    <ComboboxOption value=city_name.clone() text=city_name />
                                }
                            }).collect_view()}
                        </Combobox>
                    </Flex>
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
