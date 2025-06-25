use leptos::prelude::*;
use thaw::{Combobox, ComboboxOption, Flex, FlexAlign, Label};

pub const STATES: [&str; 50] = [
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
    let selected_options = RwSignal::new(None::<String>);
    Effect::new(move || {
        if let Some(selected_opt) = selected_options.get() {
            state.set(selected_opt);
        }
    });

    view! {
        <Flex vertical=true align=FlexAlign::Start>
            <Label>"State"</Label>
            <Combobox selected_options>
                {STATES.into_iter().map(|state| {
                    view! {
                        <ComboboxOption value=state text=state />
                    }
                }).collect_view()}
            </Combobox>
        </Flex>
    }
}
