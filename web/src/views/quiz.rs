use leptos::prelude::*;
use leptos_router::hooks::use_navigate;
use thaw::*;
use std::collections::HashSet;

#[component]
pub fn GetMatchedQuiz() -> impl IntoView {
    let navigate = use_navigate();
    
    let style_preferences = RwSignal::new(HashSet::<String>::new());
    let body_placement = RwSignal::new(String::new());
    let pain_tolerance = RwSignal::new(5);
    let budget_min = RwSignal::new(100.0);
    let budget_max = RwSignal::new(1000.0);
    let vibe_preference = RwSignal::new(String::new());

    let on_submit = move |_| {
        let styles_vec: Vec<String> = style_preferences.get().into_iter().collect();
        let location = "Washington".to_string(); // TODO: Add location selection to form
        let price_range = Some((budget_min.get(), budget_max.get()));
        
        // Navigate with query parameters to pass data to match results
        let styles_param = styles_vec.join(",");
        let navigate_url = format!(
            "/match/results?styles={}&location={}&min_price={}&max_price={}",
            urlencoding::encode(&styles_param),
            urlencoding::encode(&location),
            budget_min.get(),
            budget_max.get()
        );
        
        navigate(&navigate_url, Default::default());
    };

    view! {
        <div class="quiz-container">
            <h1>"Find Your Perfect Artist"</h1>
            
            <div class="quiz-form-wrapper">
                <form on:submit=on_submit>
                    <div class="quiz-question">
                        <label>
                            "What styles are you looking for? (Select multiple)"
                        </label>
                        <div class="quiz-style-grid">
                            {[
                                ("Traditional", "Traditional"),
                                ("Neo-Traditional", "Neo-Traditional"),
                                ("Realism", "Realism"),
                                ("Watercolor", "Watercolor"),
                                ("Blackwork", "Blackwork"),
                                ("Japanese", "Japanese"),
                                ("Minimalist", "Minimalist"),
                                ("Geometric", "Geometric"),
                            ].iter().map(|(value, label)| {
                                let value_str = value.to_string();
                                let value_for_selected = value_str.clone();
                                let value_for_checked = value_str.clone();
                                let value_for_change = value_str.clone();
                                let value_for_input = value_str.clone();
                                view! {
                                    <label class="quiz-style-option" class:quiz-selected=move || style_preferences.get().contains(&value_for_selected)>
                                        <input 
                                            type="checkbox"
                                            value=value_for_input
                                            checked=move || style_preferences.get().contains(&value_for_checked)
                                            on:change=move |ev| {
                                                let mut current = style_preferences.get();
                                                if event_target_checked(&ev) {
                                                    current.insert(value_for_change.clone());
                                                } else {
                                                    current.remove(&value_for_change);
                                                }
                                                style_preferences.set(current);
                                            }
                                        />
                                        {*label}
                                    </label>
                                }
                            }).collect_view()}
                        </div>
                    </div>

                    <div class="quiz-question">
                        <label>
                            "Where on your body?"
                        </label>
                        <select 
                            on:change=move |ev| {
                                body_placement.set(event_target_value(&ev));
                            }
                            class="quiz-form-input"
                        >
                            <option value="">"Select placement..."</option>
                            <option value="arm">"Arm"</option>
                            <option value="leg">"Leg"</option>
                            <option value="back">"Back"</option>
                            <option value="chest">"Chest"</option>
                            <option value="shoulder">"Shoulder"</option>
                            <option value="wrist">"Wrist"</option>
                            <option value="ankle">"Ankle"</option>
                            <option value="neck">"Neck"</option>
                        </select>
                    </div>

                    <div class="quiz-question">
                        <label>
                            "Pain tolerance (1-10)"
                        </label>
                        <input 
                            type="range"
                            min="1"
                            max="10"
                            step="1"
                            value=move || pain_tolerance.get().to_string()
                            on:input=move |ev| {
                                if let Ok(val) = event_target_value(&ev).parse::<i32>() {
                                    pain_tolerance.set(val);
                                }
                            }
                            class="quiz-range-slider"
                        />
                        <p class="quiz-range-value">
                            {move || pain_tolerance.get()}
                        </p>
                    </div>

                    <div class="quiz-question">
                        <label>
                            "Budget range"
                        </label>
                        <div class="quiz-budget-inputs">
                            <input 
                                type="number"
                                placeholder="Min"
                                value=move || budget_min.get().to_string()
                                on:input=move |ev| {
                                    if let Ok(val) = event_target_value(&ev).parse::<f64>() {
                                        budget_min.set(val);
                                    }
                                }
                                class="quiz-budget-input"
                            />
                            <span>"to"</span>
                            <input 
                                type="number"
                                placeholder="Max"
                                value=move || budget_max.get().to_string()
                                on:input=move |ev| {
                                    if let Ok(val) = event_target_value(&ev).parse::<f64>() {
                                        budget_max.set(val);
                                    }
                                }
                                class="quiz-budget-input"
                            />
                        </div>
                    </div>

                    <div class="quiz-question">
                        <label>
                            "What vibe are you looking for?"
                        </label>
                        <div class="quiz-vibe-options">
                            <label>
                                <input 
                                    type="radio"
                                    name="vibe"
                                    value="professional"
                                    on:change=move |ev| {
                                        if event_target_checked(&ev) {
                                            vibe_preference.set("professional".to_string());
                                        }
                                    }
                                />
                                "Professional & Clean"
                            </label>
                            <label>
                                <input 
                                    type="radio"
                                    name="vibe"
                                    value="artistic"
                                    on:change=move |ev| {
                                        if event_target_checked(&ev) {
                                            vibe_preference.set("artistic".to_string());
                                        }
                                    }
                                />
                                "Artistic & Creative"
                            </label>
                            <label>
                                <input 
                                    type="radio"
                                    name="vibe"
                                    value="friendly"
                                    on:change=move |ev| {
                                        if event_target_checked(&ev) {
                                            vibe_preference.set("friendly".to_string());
                                        }
                                    }
                                />
                                "Friendly & Relaxed"
                            </label>
                            <label>
                                <input 
                                    type="radio"
                                    name="vibe"
                                    value="edgy"
                                    on:change=move |ev| {
                                        if event_target_checked(&ev) {
                                            vibe_preference.set("edgy".to_string());
                                        }
                                    }
                                />
                                "Edgy & Alternative"
                            </label>
                        </div>
                    </div>

                    <div class="quiz-submit-section">
                        <button type="submit" class="quiz-btn-primary">
                            "Find My Artists"
                        </button>
                    </div>
                </form>
            </div>
        </div>
    }
}