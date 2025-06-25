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
    Effect::new(move |_| {
        if let Some(Ok(cities)) = cities.get() {
            city.set(cities[0].clone().city);
        }
    });

    let selected_options = RwSignal::new(None::<String>);
    Effect::new(move || {
        if let Some(selected_opt) = selected_options.get() {
            city.set(selected_opt);
        }
    });

    view! {
        {move ||
            match cities.get() {
                Some(Ok(cities)) => view! {
                    <Flex vertical=true align=FlexAlign::Start>
                        <Label>"City"</Label>
                        <Combobox
                            selected_options
                        >
                            {cities.into_iter().map(|city| {
                                let city_ref = &city.clone().city;
                                view! {
                                    <ComboboxOption value=city_ref text=city_ref />
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
