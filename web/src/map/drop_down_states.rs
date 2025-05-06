use leptos::prelude::*;
use thaw::Select;
use thaw_utils::Model;

use crate::components::{error::ErrorView, loading::LoadingView};

#[component]
pub fn DropDownStates(
    states: OnceResource<Result<Vec<String>, ServerFnError>>,
    state_model: Model<String>,
) -> impl IntoView {
    view! {
        {move ||
            match states.get() {
                Some(Ok(states)) => view! {
                    <Select value=state_model>
                        {states.into_iter().map(|state| {
                            view! {
                                <option>{state}</option>
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
