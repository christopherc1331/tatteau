use leptos::prelude::*;
use leptos_router::hooks::use_navigate;
use thaw::*;
use thaw_utils::VecModel;
use std::collections::HashSet;
use crate::server::{get_styles_by_location_filter, get_states_list, get_cities, StyleWithCount};
use crate::db::entities::CityCoords;

#[component]
pub fn GetMatchedQuiz() -> impl IntoView {
    let navigate = use_navigate();

    // Location filters - state is single-select, cities are multi-select
    let selected_state = RwSignal::new(Option::<String>::None);
    let selected_cities = RwSignal::new(Vec::<String>::new());

    // Search filters for states, cities, and styles
    let state_search = RwSignal::new(String::new());
    let city_search = RwSignal::new(String::new());
    let style_search = RwSignal::new(String::new());

    // Style selections
    let selected_styles = RwSignal::new(HashSet::<i32>::new());

    // Available data
    let states_resource = Resource::new(|| (), |_| async {
        get_states_list().await.unwrap_or_default()
    });

    // Fetch cities for selected state
    let cities_resource = Resource::new(
        move || selected_state.get(),
        |state| async move {
            if let Some(state) = state {
                get_cities(state).await.unwrap_or_default()
            } else {
                vec![]
            }
        },
    );

    // Filter styles based on selected state/cities
    let styles_resource = Resource::new(
        move || (selected_state.get(), selected_cities.get()),
        |(state, cities)| async move {
            // Pass selected state and cities
            let states_opt = state.map(|s| vec![s]);
            let cities_opt = if cities.is_empty() { None } else { Some(cities) };
            get_styles_by_location_filter(states_opt, cities_opt).await.unwrap_or_default()
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

        let state_opt = selected_state.get();
        let cities: Vec<String> = selected_cities.get();

        // Build query parameters
        let styles_param = styles_vec.join(",");
        let mut query_parts = vec![format!("styles={}", urlencoding::encode(&styles_param))];

        if let Some(state) = state_opt {
            query_parts.push(format!("states={}", urlencoding::encode(&state)));
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
                                <Suspense fallback=move || view! { <div>"Loading states..."</div> }>
                                    {move || {
                                        states_resource.get().map(|states| {
                                            // Create effect to clear cities when state changes
                                            Effect::new(move |_| {
                                                let _ = selected_state.get();
                                                selected_cities.set(vec![]);
                                            });

                                            view! {
                                                <div class="quiz-location-select-wrapper">
                                                    <Label>"State (Select one)"</Label>
                                                    <select
                                                        class="quiz-location-select"
                                                        on:change=move |ev| {
                                                            let value = event_target_value(&ev);
                                                            if value.is_empty() {
                                                                selected_state.set(None);
                                                            } else {
                                                                selected_state.set(Some(value));
                                                            }
                                                        }
                                                    >
                                                        <option value="">"Select a state..."</option>
                                                        {states.into_iter().map(|state| {
                                                            let state_val = state.clone();
                                                            view! {
                                                                <option value=state_val>{state}</option>
                                                            }
                                                        }).collect_view()}
                                                    </select>
                                                </div>
                                            }
                                        })
                                    }}
                                </Suspense>
                            </div>

                            <div class="quiz-question quiz-location-field">
                                <Suspense fallback=move || view! { <div>"Loading cities..."</div> }>
                                    {move || {
                                        let cities = cities_resource.get().unwrap_or_default();
                                        let has_state = selected_state.get().is_some();
                                        let cities_clone = cities.clone();

                                        // Filter cities based on search
                                        let filtered_cities = Signal::derive(move || {
                                            let search = city_search.get().to_lowercase();
                                            if search.is_empty() {
                                                cities_clone.clone()
                                            } else {
                                                cities_clone.iter()
                                                    .filter(|c| c.city.to_lowercase().contains(&search))
                                                    .cloned()
                                                    .collect()
                                            }
                                        });

                                        view! {
                                            <div class="quiz-multi-select-wrapper">
                                                <Label>"City (Select one or more)"</Label>
                                                {if !has_state {
                                                    view! {
                                                        <p class="quiz-location-hint">"Select a state first"</p>
                                                    }.into_any()
                                                } else if cities.is_empty() {
                                                    view! {
                                                        <p class="quiz-location-hint">"No cities available"</p>
                                                    }.into_any()
                                                } else {
                                                    view! {
                                                        <input
                                                            type="text"
                                                            class="quiz-search-input"
                                                            placeholder="Search cities..."
                                                            prop:value=move || city_search.get()
                                                            on:input=move |ev| {
                                                                city_search.set(event_target_value(&ev));
                                                            }
                                                        />
                                                        <div class="quiz-chip-grid">
                                                            {move || filtered_cities.get().into_iter().map(|city| {
                                                                let city_name = city.city.clone();
                                                                let city_val = city_name.clone();
                                                                let city_val_signal = city_val.clone();
                                                                let city_val_click = city_val.clone();

                                                                let is_selected = Signal::derive(move || {
                                                                    selected_cities.get().contains(&city_val_signal)
                                                                });

                                                                view! {
                                                                    <button
                                                                        type="button"
                                                                        class="quiz-location-chip"
                                                                        class:quiz-location-chip-selected=is_selected
                                                                        on:click=move |_| {
                                                                            let mut current = selected_cities.get();
                                                                            if current.contains(&city_val_click) {
                                                                                current.retain(|c| c != &city_val_click);
                                                                            } else {
                                                                                current.push(city_val_click.clone());
                                                                            }
                                                                            selected_cities.set(current);
                                                                        }
                                                                    >
                                                                        {city_name}
                                                                    </button>
                                                                }
                                                            }).collect_view()}
                                                        </div>
                                                        {move || {
                                                            let count = selected_cities.get().len();
                                                            if count > 0 {
                                                                view! {
                                                                    <p class="quiz-selection-count">
                                                                        {format!("{} cit{} selected", count, if count == 1 { "y" } else { "ies" })}
                                                                    </p>
                                                                }.into_any()
                                                            } else {
                                                                view! {}.into_any()
                                                            }
                                                        }}
                                                    }.into_any()
                                                }}
                                            </div>
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
                                        let styles_clone = styles.clone();

                                        // Filter styles based on search
                                        let filtered_styles = Signal::derive(move || {
                                            let search = style_search.get().to_lowercase();
                                            if search.is_empty() {
                                                styles_clone.clone()
                                            } else {
                                                styles_clone.iter()
                                                    .filter(|s| s.name.to_lowercase().contains(&search))
                                                    .cloned()
                                                    .collect()
                                            }
                                        });

                                        let styles_count = styles.len();
                                        view! {
                                            <div class="quiz-styles-info">
                                                {format!("{} styles available", styles_count)}
                                            </div>
                                            <input
                                                type="text"
                                                class="quiz-search-input"
                                                placeholder="Search styles..."
                                                prop:value=move || style_search.get()
                                                on:input=move |ev| {
                                                    style_search.set(event_target_value(&ev));
                                                }
                                            />
                                            <div class="quiz-style-grid">
                                                {move || filtered_styles.get().into_iter().map(|style| {
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