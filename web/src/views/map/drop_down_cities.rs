use leptos::prelude::*;
use thaw::{Combobox, ComboboxOption, Flex, FlexAlign, Label};
use thaw_utils::Model;

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

    let model: Model<String> = Model::from(city);

    view! {
        {move ||
            match cities.get() {
                Some(Ok(cities)) => view! {
                    <Flex vertical=true align=FlexAlign::Start>
                        <Label>"City"</Label>
                        <Combobox
                            value=model
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
