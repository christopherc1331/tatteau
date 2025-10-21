use leptos::{portal::Portal, prelude::*, task::spawn_local};
use wasm_bindgen::JsCast;
use web_sys::KeyboardEvent;

use crate::{
    db::entities::CityCoords,
    db::search_repository::SearchResult,
    server::{get_search_suggestions, universal_search},
};

#[component]
pub fn LocationSearch<F>(
    city: RwSignal<String>,
    state: RwSignal<String>,
    on_location_selected: F,
) -> impl IntoView
where
    F: Fn(CityCoords) + 'static + Copy + Send + Sync,
{
    let search_input = RwSignal::new(String::new());
    let search_results = RwSignal::new(Vec::<SearchResult>::new());
    let suggestions = RwSignal::new(Vec::<String>::new());
    let is_searching = RwSignal::new(false);
    let show_suggestions = RwSignal::new(false);
    let selected_index = RwSignal::new(0usize);
    let search_error = RwSignal::new(Option::<String>::None);
    let input_ref = NodeRef::<leptos::tachys::html::element::Input>::new();

    // Debounced search for suggestions
    let fetch_suggestions = move |query: String| {
        if query.len() < 2 {
            suggestions.set(Vec::new());
            show_suggestions.set(false);
            return;
        }

        spawn_local(async move {
            match get_search_suggestions(query).await {
                Ok(sugg) => {
                    suggestions.set(sugg);
                    show_suggestions.set(!suggestions.get().is_empty());
                }
                Err(_) => {
                    suggestions.set(Vec::new());
                    show_suggestions.set(false);
                }
            }
        });
    };

    // Perform the actual search
    let perform_search = move |query: String, update_input_with_result: bool| {
        let query = query.trim().to_string();
        if query.is_empty() {
            return;
        }

        is_searching.set(true);
        search_error.set(None);
        show_suggestions.set(false);

        spawn_local(async move {
            match universal_search(query.clone()).await {
                Ok(results) => {
                    if results.is_empty() {
                        search_error.set(Some(format!("No results found for '{}'", query)));
                    } else {
                        // Use the first result
                        let first_result = results[0].clone();
                        city.set(first_result.city.clone());
                        state.set(first_result.state.clone());

                        on_location_selected(CityCoords {
                            city: first_result.city.clone(),
                            state: first_result.state.clone(),
                            lat: first_result.lat,
                            long: first_result.long,
                        });

                        // Update input with the properly formatted result
                        if update_input_with_result {
                            search_input
                                .set(format!("{}, {}", first_result.city, first_result.state));
                        }

                        search_results.set(results);
                    }
                    is_searching.set(false);
                }
                Err(e) => {
                    search_error.set(Some(format!("Search failed: {}", e)));
                    is_searching.set(false);
                }
            }
        });
    };

    // Handle keyboard navigation
    let handle_keydown = move |ev: KeyboardEvent| {
        let key = ev.key();
        match key.as_str() {
            "Enter" => {
                ev.prevent_default();
                if show_suggestions.get() && !suggestions.get().is_empty() {
                    let sugg = suggestions.get();
                    if let Some(selected) = sugg.get(selected_index.get()) {
                        // Extract the city/state from suggestion format "City, State" or "12345 - City, State"
                        let query = if selected.contains(" - ") {
                            // It's a postal code suggestion
                            selected.split(" - ").next().unwrap_or(selected).to_string()
                        } else {
                            selected.clone()
                        };
                        search_input.set(query.clone());
                        perform_search(query, true);
                        show_suggestions.set(false);
                    }
                } else {
                    perform_search(search_input.get(), false);
                }
            }
            "ArrowDown" => {
                ev.prevent_default();
                if show_suggestions.get() {
                    let max = suggestions.get().len().saturating_sub(1);
                    selected_index.update(|i| *i = (*i + 1).min(max));
                }
            }
            "ArrowUp" => {
                ev.prevent_default();
                if show_suggestions.get() {
                    selected_index.update(|i| *i = i.saturating_sub(1));
                }
            }
            "Escape" => {
                show_suggestions.set(false);
                selected_index.set(0);
            }
            _ => {}
        }
    };

    // Handle input changes
    let handle_input = move |ev: web_sys::Event| {
        let value = event_target_value(&ev);
        search_input.set(value.clone());
        selected_index.set(0);
        fetch_suggestions(value);
    };

    // Handle suggestion click
    let handle_suggestion_click = move |suggestion: String| {
        let query = if suggestion.contains(" - ") {
            // It's a postal code suggestion
            suggestion
                .split(" - ")
                .next()
                .unwrap_or(&suggestion)
                .to_string()
        } else {
            suggestion.clone()
        };
        search_input.set(query.clone());
        perform_search(query, true);
        show_suggestions.set(false);
    };

    // Effect to position the fixed dropdown
    Effect::new(move |_| {
        if show_suggestions.get() {
            if let Some(input_element) = input_ref.get() {
                let rect = input_element.get_bounding_client_rect();
                let window = web_sys::window().unwrap();
                let document = window.document().unwrap();

                // Find the portal suggestions dropdown
                if let Some(suggestions_el) = document
                    .query_selector(".location-search-suggestions-portal")
                    .unwrap()
                {
                    let suggestions_html =
                        suggestions_el.dyn_into::<web_sys::HtmlElement>().unwrap();
                    let style = suggestions_html.style();

                    // Position below the input
                    let top = rect.bottom() + 8.0; // 8px gap
                    let left = rect.left();
                    let width = rect.width();

                    style.set_property("top", &format!("{}px", top)).unwrap();
                    style.set_property("left", &format!("{}px", left)).unwrap();
                    style
                        .set_property("width", &format!("{}px", width))
                        .unwrap();
                }
            }
        }
    });

    view! {
        <div class="location-search-container">
            <div class="location-search-input-wrapper">
                <input
                    type="text"
                    class="location-search-input"
                    placeholder="Search city, county, or zip code..."
                    value=move || search_input.get()
                    on:input=handle_input
                    on:keydown=handle_keydown
                    on:focus=move |_| {
                        if !suggestions.get().is_empty() {
                            show_suggestions.set(true);
                        }
                    }
                    on:blur=move |_| {
                        // Delay to allow click on suggestion
                        set_timeout(move || {
                            show_suggestions.set(false);
                        }, std::time::Duration::from_millis(200));
                    }
                    disabled=move || is_searching.get()
                    node_ref=input_ref
                />

                <button
                    class="location-search-button"
                    class:searching=move || is_searching.get()
                    on:click=move |_| perform_search(search_input.get(), false)
                    disabled=move || is_searching.get() || search_input.get().trim().is_empty()
                >
                    {move || if is_searching.get() {
                        "Searching..."
                    } else {
                        "Search"
                    }}
                </button>

                // GPS location button
                <button
                    class="location-search-gps-button"
                    title="Use my location"
                    on:click=move |_| {
                        leptos::logging::log!("GPS location not yet implemented");
                        // TODO: Implement GPS location
                    }
                >
                    "📍"
                </button>
            </div>

            // Suggestions dropdown - render using portal to escape stacking contexts
            <Portal>
                {move || if show_suggestions.get() && !suggestions.get().is_empty() {
                    view! {
                        <div class="location-search-suggestions-portal">
                            {suggestions.get().into_iter().enumerate().map(|(idx, suggestion)| {
                                let suggestion_clone = suggestion.clone();
                                view! {
                                    <div
                                        class="location-search-suggestion-item"
                                        class:selected=move || selected_index.get() == idx
                                        on:mousedown=move |_| {
                                            handle_suggestion_click(suggestion_clone.clone())
                                        }
                                        on:mouseenter=move |_| selected_index.set(idx)
                                    >
                                        {if suggestion.contains(" - ") {
                                            // Postal code suggestion
                                            view! {
                                                <>
                                                    <span class="location-search-suggestion-icon">"📮"</span>
                                                    <span>{suggestion.clone()}</span>
                                                </>
                                            }.into_any()
                                        } else {
                                            // City suggestion
                                            view! {
                                                <>
                                                    <span class="location-search-suggestion-icon">"📍"</span>
                                                    <span>{suggestion.clone()}</span>
                                                </>
                                            }.into_any()
                                        }}
                                    </div>
                                }
                            }).collect_view()}
                        </div>
                    }.into_any()
                } else {
                    view! { <></> }.into_any()
                }}
            </Portal>

            // Error message
            {move || if let Some(error) = search_error.get() {
                view! {
                    <div class="location-search-error">
                        {error}
                    </div>
                }.into_any()
            } else {
                view! { <></> }.into_any()
            }}

            // Quick location shortcuts
            <div class="location-search-quick-locations">
                <span class="location-search-quick-label">"Quick access: "</span>
                <button
                    class="location-search-quick-location-btn"
                    on:click=move |_| perform_search("Seattle".to_string(), true)
                >
                    "Seattle"
                </button>
                <button
                    class="location-search-quick-location-btn"
                    on:click=move |_| perform_search("Portland".to_string(), true)
                >
                    "Portland"
                </button>
                <button
                    class="location-search-quick-location-btn"
                    on:click=move |_| perform_search("Los Angeles".to_string(), true)
                >
                    "Los Angeles"
                </button>
                <button
                    class="location-search-quick-location-btn"
                    on:click=move |_| perform_search("New York".to_string(), true)
                >
                    "New York"
                </button>
            </div>
        </div>
    }
}
