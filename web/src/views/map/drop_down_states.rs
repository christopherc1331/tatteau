use leptos::prelude::*;
use thaw::{Flex, FlexAlign, Label, Select};
use thaw_utils::Model;

use crate::components::{error::ErrorView, loading::LoadingView};

const STATES: [&str; 50] = [
    "Alabama",
    "Alaska",
    "Arizona",
    "Arkansas",
    "California",
    "Colorado",
    "Connecticut",
    "Delaware",
    "Florida",
    "Georgia",
    "Hawaii",
    "Idaho",
    "Illinois",
    "Indiana",
    "Iowa",
    "Kansas",
    "Kentucky",
    "Louisiana",
    "Maine",
    "Maryland",
    "Massachusetts",
    "Michigan",
    "Minnesota",
    "Mississippi",
    "Missouri",
    "Montana",
    "Nebraska",
    "Nevada",
    "New Hampshire",
    "New Jersey",
    "New Mexico",
    "New York",
    "North Carolina",
    "North Dakota",
    "Ohio",
    "Oklahoma",
    "Oregon",
    "Pennsylvania",
    "Rhode Island",
    "South Carolina",
    "South Dakota",
    "Tennessee",
    "Texas",
    "Utah",
    "Vermont",
    "Virginia",
    "Washington",
    "West Virginia",
    "Wisconsin",
    "Wyoming",
];

#[component]
pub fn DropDownStates(state: RwSignal<String>) -> impl IntoView {
    let state_model: Model<String> = state.into();

    view! {
                <Flex vertical=true align=FlexAlign::Start>
                    <Label>"State"</Label>
                    <Select value=state_model>
                        {STATES.into_iter().map(|state| {
                            view! {
                                <option>{state}</option>
                            }
                        }).collect_view()}
                    </Select>
                </Flex>
    }
}
