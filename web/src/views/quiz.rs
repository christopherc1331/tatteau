use leptos::prelude::*;
use leptos_router::hooks::use_navigate;
use thaw::*;
use std::collections::HashSet;
use crate::server::{get_styles_by_location_filter, get_states_list, get_cities, StyleWithCount};
use crate::db::entities::CityCoords;

#[component]
pub fn GetMatchedQuiz() -> impl IntoView {
    let navigate = use_navigate();

    // Location filters
    let selected_state = RwSignal::new(Option::<String>::None);
    let selected_city = RwSignal::new(Option::<String>::None);

    // Style selections
    let selected_styles = RwSignal::new(HashSet::<i32>::new());

    // Available data
    let states_resource = Resource::new(|| (), |_| async {
        get_states_list().await.unwrap_or_default()
    });

    let cities_resource = Resource::new(
        move || selected_state.get(),
        |state| async move {
            match state {
                Some(s) => get_cities(s).await.unwrap_or_default(),
                None => vec![],
            }
        },
    );

    let styles_resource = Resource::new(
        move || (selected_state.get(), selected_city.get()),
        |(state, city)| async move {
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

        let location = selected_state.get().unwrap_or_else(|| "".to_string());

        // Navigate with query parameters to pass data to match results
        let styles_param = styles_vec.join(",");
        let navigate_url = if !location.is_empty() {
            if let Some(city) = selected_city.get() {
                format!(
                    "/match/results?styles={}&location={}&city={}",
                    urlencoding::encode(&styles_param),
                    urlencoding::encode(&location),
                    urlencoding::encode(&city)
                )
            } else {
                format!(
                    "/match/results?styles={}&location={}",
                    urlencoding::encode(&styles_param),
                    urlencoding::encode(&location)
                )
            }
        } else {
            format!(
                "/match/results?styles={}",
                urlencoding::encode(&styles_param)
            )
        };

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
                                <label>"State"</label>
                                <Suspense fallback=move || view! { <div>"Loading states..."</div> }>
                                    {move || {
                                        states_resource.get().map(|states| {
                                            view! {
                                                <select
                                                    class="quiz-form-input"
                                                    on:change=move |ev| {
                                                        let value = event_target_value(&ev);
                                                        if value.is_empty() {
                                                            selected_state.set(None);
                                                        } else {
                                                            selected_state.set(Some(value));
                                                        }
                                                        selected_city.set(None);
                                                    }
                                                >
                                                    <option value="">"All States"</option>
                                                    {states.into_iter().map(|state| {
                                                        let state_val = state.clone();
                                                        view! {
                                                            <option value={state_val}>{state}</option>
                                                        }
                                                    }).collect_view()}
                                                </select>
                                            }
                                        })
                                    }}
                                </Suspense>
                            </div>

                            <div class="quiz-question quiz-location-field">
                                <label>"City"</label>
                                <Suspense fallback=move || view! { <div>"Loading cities..."</div> }>
                                    {move || {
                                        let cities = cities_resource.get().unwrap_or_default();
                                        let has_state = selected_state.get().is_some();

                                        view! {
                                            <select
                                                class="quiz-form-input"
                                                disabled=!has_state
                                                on:change=move |ev| {
                                                    let value = event_target_value(&ev);
                                                    if value.is_empty() {
                                                        selected_city.set(None);
                                                    } else {
                                                        selected_city.set(Some(value));
                                                    }
                                                }
                                            >
                                                <option value="">
                                                    {if has_state { "All Cities" } else { "Select state first" }}
                                                </option>
                                                {cities.into_iter().map(|city| {
                                                    let city_name = city.city.clone();
                                                    let city_val = city_name.clone();
                                                    view! {
                                                        <option value={city_val}>{city_name}</option>
                                                    }
                                                }).collect_view()}
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