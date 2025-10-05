use leptos::prelude::*;
use leptos_router::hooks::use_navigate;
use thaw::*;
use std::collections::HashSet;
use wasm_bindgen::JsCast;
use crate::server::{get_styles_by_location_filter, get_states_list, get_cities, StyleWithCount};
use crate::db::entities::CityCoords;

#[component]
pub fn GetMatchedQuiz() -> impl IntoView {
    let navigate = use_navigate();

    // Location filters - now support multiple selections
    let selected_states = RwSignal::new(HashSet::<String>::new());
    let selected_cities = RwSignal::new(HashSet::<String>::new());

    // Style selections
    let selected_styles = RwSignal::new(HashSet::<i32>::new());

    // Available data
    let states_resource = Resource::new(|| (), |_| async {
        get_states_list().await.unwrap_or_default()
    });

    // Fetch cities for all selected states
    let cities_resource = Resource::new(
        move || selected_states.get(),
        |states| async move {
            if states.is_empty() {
                return vec![];
            }
            // For now, fetch cities for the first selected state
            // TODO: Could enhance to fetch cities for all selected states
            let first_state = states.iter().next().cloned();
            match first_state {
                Some(s) => get_cities(s).await.unwrap_or_default(),
                None => vec![],
            }
        },
    );

    // Filter styles based on all selected states/cities
    let styles_resource = Resource::new(
        move || (selected_states.get(), selected_cities.get()),
        |(states, cities)| async move {
            // Use first state/city for filtering
            // TODO: Could enhance to support multiple
            let state = states.iter().next().cloned();
            let city = cities.iter().next().cloned();
            get_styles_by_location_filter(state, city).await.unwrap_or_default()
        },
    );

    let on_submit = move |_| {
        let styles_vec: Vec<String> = selected_styles.get()
            .into_iter()
            .filter_map(|id| {
                styles_resource.get().and_then(|styles| {
                    styles.iter().find(|s| s.id == id).map(|s| s.name.clone())
                })
            })
            .collect();

        let states: Vec<String> = selected_states.get().into_iter().collect();
        let cities: Vec<String> = selected_cities.get().into_iter().collect();

        // Build query parameters
        let styles_param = styles_vec.join(",");
        let mut query_parts = vec![format!("styles={}", urlencoding::encode(&styles_param))];

        if !states.is_empty() {
            let states_param = states.join(",");
            query_parts.push(format!("states={}", urlencoding::encode(&states_param)));
        }

        if !cities.is_empty() {
            let cities_param = cities.join(",");
            query_parts.push(format!("cities={}", urlencoding::encode(&cities_param)));
        }

        let navigate_url = format!("/match/results?{}", query_parts.join("&"));
        navigate(&navigate_url, Default::default());
    };

    view! {
        <div class="quiz-container">
            <h1>"Find Your Perfect Artist"</h1>

            <div class="quiz-form-wrapper">
                <form on:submit=on_submit>
                    // Location Filters Section
                    <div class="quiz-location-section">
                        <h3>"Filter by Location (Optional)"</h3>
                        <p class="quiz-location-hint">"Leave blank to search nationwide"</p>

                        <div class="quiz-location-filters">
                            <div class="quiz-question quiz-location-field">
                                <label>"State (Hold Ctrl/Cmd to select multiple)"</label>
                                <Suspense fallback=move || view! { <div>"Loading states..."</div> }>
                                    {move || {
                                        states_resource.get().map(|states| {
                                            view! {
                                                <select
                                                    class="quiz-form-input"
                                                    multiple=true
                                                    size="5"
                                                    on:change=move |ev| {
                                                        let select_element = event_target::<web_sys::HtmlSelectElement>(&ev);
                                                        let options = select_element.selected_options();
                                                        let mut new_selected = HashSet::new();

                                                        for i in 0..options.length() {
                                                            if let Some(option) = options.item(i) {
                                                                if let Some(html_option) = option.dyn_ref::<web_sys::HtmlOptionElement>() {
                                                                    let value = html_option.value();
                                                                    if !value.is_empty() {
                                                                        new_selected.insert(value);
                                                                    }
                                                                }
                                                            }
                                                        }

                                                        selected_states.set(new_selected);
                                                        selected_cities.set(HashSet::new());
                                                    }
                                                >
                                                    {states.into_iter().map(|state| {
                                                        let state_val = state.clone();
                                                        view! {
                                                            <option value={state_val.clone()}>{state}</option>
                                                        }
                                                    }).collect_view()}
                                                </select>
                                            }
                                        })
                                    }}
                                </Suspense>
                            </div>

                            <div class="quiz-question quiz-location-field">
                                <label>"City (Hold Ctrl/Cmd to select multiple)"</label>
                                <Suspense fallback=move || view! { <div>"Loading cities..."</div> }>
                                    {move || {
                                        let cities = cities_resource.get().unwrap_or_default();
                                        let has_states = !selected_states.get().is_empty();

                                        view! {
                                            <select
                                                class="quiz-form-input"
                                                multiple=true
                                                size="5"
                                                disabled=!has_states
                                                on:change=move |ev| {
                                                    let select_element = event_target::<web_sys::HtmlSelectElement>(&ev);
                                                    let options = select_element.selected_options();
                                                    let mut new_selected = HashSet::new();

                                                    for i in 0..options.length() {
                                                        if let Some(option) = options.item(i) {
                                                            if let Some(html_option) = option.dyn_ref::<web_sys::HtmlOptionElement>() {
                                                                let value = html_option.value();
                                                                if !value.is_empty() {
                                                                    new_selected.insert(value);
                                                                }
                                                            }
                                                        }
                                                    }

                                                    selected_cities.set(new_selected);
                                                }
                                            >
                                                {if !has_states {
                                                    view! {
                                                        <option disabled=true>"Select state first"</option>
                                                    }.into_any()
                                                } else {
                                                    view! {
                                                        {cities.into_iter().map(|city| {
                                                            let city_name = city.city.clone();
                                                            let city_val = city_name.clone();
                                                            view! {
                                                                <option value={city_val}>{city_name}</option>
                                                            }
                                                        }).collect_view()}
                                                    }.into_any()
                                                }}
                                            </select>
                                        }
                                    }}
                                </Suspense>
                            </div>
                        </div>
                    </div>

                    // Styles Section
                    <div class="quiz-question">
                        <label>"What styles are you looking for? (Select at least one)"</label>
                        <Suspense fallback=move || view! { <div>"Loading styles..."</div> }>
                            {move || {
                                styles_resource.get().map(|styles| {
                                    if styles.is_empty() {
                                        view! {
                                            <div class="quiz-no-styles">
                                                "No artists found in this location. Try a different area."
                                            </div>
                                        }.into_any()
                                    } else {
                                        let styles_count = styles.len();
                                        view! {
                                            <div class="quiz-styles-info">
                                                {format!("{} styles available", styles_count)}
                                            </div>
                                            <div class="quiz-style-grid">
                                                {styles.into_iter().map(|style| {
                                                    let style_id = style.id;
                                                    let style_name = style.name.clone();
                                                    let artist_count = style.artist_count;

                                                    view! {
                                                        <label
                                                            class="quiz-style-option"
                                                            class:quiz-selected=move || selected_styles.get().contains(&style_id)
                                                        >
                                                            <input
                                                                type="checkbox"
                                                                checked=move || selected_styles.get().contains(&style_id)
                                                                on:change=move |ev| {
                                                                    let mut current = selected_styles.get();
                                                                    if event_target_checked(&ev) {
                                                                        current.insert(style_id);
                                                                    } else {
                                                                        current.remove(&style_id);
                                                                    }
                                                                    selected_styles.set(current);
                                                                }
                                                            />
                                                            <span class="quiz-style-name">{style_name}</span>
                                                            <span class="quiz-style-count">{format!("({})", artist_count)}</span>
                                                        </label>
                                                    }
                                                }).collect_view()}
                                            </div>
                                        }.into_any()
                                    }
                                })
                            }}
                        </Suspense>
                    </div>

                    <div class="quiz-submit-section">
                        <p class="quiz-submit-hint">
                            {move || {
                                let count = selected_styles.get().len();
                                if count == 0 {
                                    "Select at least one style to continue".to_string()
                                } else {
                                    format!("{} style{} selected", count, if count == 1 { "" } else { "s" })
                                }
                            }}
                        </p>
                        <button
                            type="submit"
                            class="quiz-btn-primary"
                            disabled=move || selected_styles.get().is_empty()
                        >
                            "Find My Artists"
                        </button>
                    </div>
                </form>
            </div>
        </div>
    }
}