use leptos::prelude::*;
use thaw::{Flex, FlexAlign, Label, Select};
use thaw_utils::Model;

use crate::{
    components::{error::ErrorView, loading::LoadingView},
    server::get_states_list,
};

#[component]
pub fn DropDownStates(state: RwSignal<String>) -> impl IntoView {
    let states = OnceResource::new(async move { get_states_list().await });
    let state_model: Model<String> = state.into();

    view! {
        {move ||
            match states.get() {
                Some(Ok(states)) => view! {
                    <Flex vertical=true align=FlexAlign::Start>
                        <Label>"State"</Label>
                        <Select value=state_model>
                            {states.into_iter().map(|state| {
                                view! {
                                    <option>{state}</option>
                                }
                            }).collect_view()}
                        </Select>
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
