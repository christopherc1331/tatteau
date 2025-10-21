use crate::db::entities::Style;
use crate::server::{add_style_to_image, get_all_styles_for_admin, remove_style_from_image};
use crate::utils::auth::is_admin;
use leptos::prelude::*;
use leptos::task::spawn_local;
use thaw::*;

#[component]
pub fn StyleTagManager(
    image_id: i64,
    #[prop(into)] current_styles: Signal<Vec<Style>>,
    #[prop(optional)] on_styles_changed: Option<Callback<Vec<Style>>>,
) -> impl IntoView {
    let show_modal = RwSignal::new(false);
    let all_styles = RwSignal::new(Vec::<Style>::new());
    let loading = RwSignal::new(false);
    let error_message = RwSignal::new(Option::<String>::None);
    let search_filter = RwSignal::new(String::new());

    // Track selected style IDs in modal (using HashSet for O(1) lookups)
    let selected_style_ids = RwSignal::new(std::collections::HashSet::<i32>::new());

    // Get auth token from localStorage
    let get_token = move || -> Option<String> {
        #[cfg(feature = "hydrate")]
        {
            use wasm_bindgen::prelude::*;

            #[wasm_bindgen]
            extern "C" {
                #[wasm_bindgen(js_namespace = localStorage)]
                fn getItem(key: &str) -> Option<String>;
            }

            getItem("tatteau_auth_token")
        }

        #[cfg(not(feature = "hydrate"))]
        {
            None
        }
    };

    // Open modal and fetch all styles
    let open_modal = move |_| {
        show_modal.set(true);
        error_message.set(None);
        search_filter.set(String::new()); // Reset search filter

        // Initialize selected styles with current styles
        let current_ids: std::collections::HashSet<i32> =
            current_styles.get().iter().map(|s| s.id).collect();
        selected_style_ids.set(current_ids);

        // Fetch all available styles
        spawn_local(async move {
            match get_all_styles_for_admin().await {
                Ok(styles) => all_styles.set(styles),
                Err(e) => error_message.set(Some(format!("Failed to load styles: {}", e))),
            }
        });
    };

    // Toggle style selection in modal
    let toggle_style = move |style_id: i32| {
        let mut ids = selected_style_ids.get();
        if ids.contains(&style_id) {
            ids.remove(&style_id);
        } else {
            ids.insert(style_id);
        }
        selected_style_ids.set(ids);
    };

    // Save changes - add/remove styles as needed
    let save_changes = move |_| {
        let token = match get_token() {
            Some(t) => t,
            None => {
                error_message.set(Some("Not authenticated".to_string()));
                return;
            }
        };

        loading.set(true);
        error_message.set(None);

        let current_ids: std::collections::HashSet<i32> =
            current_styles.get().iter().map(|s| s.id).collect();
        let selected_ids = selected_style_ids.get();

        // Determine which styles to add and remove
        let to_add: Vec<i32> = selected_ids.difference(&current_ids).copied().collect();
        let to_remove: Vec<i32> = current_ids.difference(&selected_ids).copied().collect();

        spawn_local(async move {
            let mut success = true;

            // Remove styles
            for style_id in to_remove {
                if let Err(e) =
                    remove_style_from_image(image_id, style_id as i64, token.clone()).await
                {
                    error_message.set(Some(format!("Failed to remove style: {}", e)));
                    success = false;
                    break;
                }
            }

            // Add styles
            if success {
                for style_id in to_add {
                    if let Err(e) =
                        add_style_to_image(image_id, style_id as i64, token.clone()).await
                    {
                        error_message.set(Some(format!("Failed to add style: {}", e)));
                        success = false;
                        break;
                    }
                }
            }

            loading.set(false);

            if success {
                // Update current_styles with new selection
                let all_styles_list = all_styles.get();
                let selected_ids_set = selected_style_ids.get();
                let new_styles: Vec<Style> = all_styles_list
                    .into_iter()
                    .filter(|s| selected_ids_set.contains(&s.id))
                    .collect();

                if let Some(callback) = on_styles_changed {
                    callback.run(new_styles);
                }

                show_modal.set(false);
            }
        });
    };

    // Filtered and sorted styles for the modal
    let filtered_sorted_styles = Memo::new(move |_| {
        let styles = all_styles.get();
        let filter = search_filter.get().to_lowercase();
        let selected = selected_style_ids.get();

        // Filter by search term
        let mut filtered: Vec<Style> = styles
            .into_iter()
            .filter(|s| s.name.to_lowercase().contains(&filter))
            .collect();

        // Sort: selected first (alphabetically), then non-selected (alphabetically)
        filtered.sort_by(|a, b| {
            let a_selected = selected.contains(&a.id);
            let b_selected = selected.contains(&b.id);

            match (a_selected, b_selected) {
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
            }
        });

        filtered
    });

    view! {
        <div class="style-tag-manager" style=move || if !is_admin() { "display: none;" } else { "" }>
            <button
                class="style-tag-manager__edit-btn"
                on:click=open_modal
            >
                "Edit Tags"
            </button>

            <Show when=move || show_modal.get()>
                <div class="style-tag-manager__modal-overlay" on:click=move |_| {
                    if !loading.get() {
                        show_modal.set(false);
                    }
                }>
                    <div class="style-tag-manager__modal" on:click=|ev| ev.stop_propagation()>
                        <div class="style-tag-manager__modal-header">
                            <h2>"Edit Style Tags"</h2>
                            <button
                                class="style-tag-manager__modal-close"
                                on:click=move |_| show_modal.set(false)
                                disabled=loading
                            >
                                "\u{00D7}"
                            </button>
                        </div>

                        <div class="style-tag-manager__modal-body">
                            <Show when=move || error_message.get().is_some()>
                                <div class="style-tag-manager__error">
                                    {move || error_message.get().unwrap_or_default()}
                                </div>
                            </Show>

                            <input
                                type="text"
                                class="style-tag-manager__search"
                                placeholder="Search styles..."
                                on:input=move |ev| {
                                    search_filter.set(event_target_value(&ev));
                                }
                                prop:value=move || search_filter.get()
                            />

                            <div class="style-tag-manager__checkbox-grid">
                                <For
                                    each=move || filtered_sorted_styles.get()
                                    key=|style| style.id
                                    children=move |style| {
                                        let style_id = style.id;
                                        let is_selected = move || {
                                            selected_style_ids.get().contains(&style_id)
                                        };

                                        view! {
                                            <label class="style-tag-manager__checkbox-item">
                                                <input
                                                    type="checkbox"
                                                    checked=is_selected
                                                    on:change=move |_| toggle_style(style_id)
                                                />
                                                <span>{style.name.clone()}</span>
                                            </label>
                                        }
                                    }
                                />
                            </div>
                        </div>

                        <div class="style-tag-manager__modal-footer">
                            <Button
                                appearance=ButtonAppearance::Primary
                                on_click=save_changes
                                disabled=loading
                            >
                                {move || if loading.get() { "Saving..." } else { "Save Changes" }}
                            </Button>
                            <Button
                                appearance=ButtonAppearance::Secondary
                                on_click=move |_| show_modal.set(false)
                                disabled=loading
                            >
                                "Cancel"
                            </Button>
                        </div>
                    </div>
                </div>
            </Show>
        </div>
    }
}
