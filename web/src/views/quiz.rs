use leptos::prelude::*;
use leptos_router::hooks::use_navigate;
use thaw::*;

#[component]
pub fn GetMatchedQuiz() -> impl IntoView {
    let navigate = use_navigate();
    
    let style_preference = RwSignal::new(String::new());
    let body_placement = RwSignal::new(String::new());
    let pain_tolerance = RwSignal::new(5);
    let budget_min = RwSignal::new(100.0);
    let budget_max = RwSignal::new(1000.0);
    let vibe_preference = RwSignal::new(String::new());

    let on_submit = move |_| {
        // TODO: Save to database via server function
        #[cfg(feature = "ssr")]
        {
            use crate::db::repository::save_quiz_session;
            let _ = save_quiz_session(
                style_preference.get(),
                body_placement.get(),
                pain_tolerance.get(),
                budget_min.get(),
                budget_max.get(),
                vibe_preference.get(),
            );
        }
        
        navigate("/match/results", Default::default());
    };

    view! {
        <div style="max-width: 800px; margin: 0 auto; padding: 2rem;">
            <h1 style="text-align: center; margin-bottom: 2rem;">"Find Your Perfect Artist"</h1>
            
            <div style="background: #f9f9f9; padding: 2rem; border-radius: 8px;">
                <form on:submit=on_submit>
                    <div style="margin-bottom: 1.5rem;">
                        <label style="display: block; margin-bottom: 0.5rem; font-weight: bold;">
                            "What style are you looking for?"
                        </label>
                        <select 
                            on:change=move |ev| {
                                style_preference.set(event_target_value(&ev));
                            }
                            class="form-input"
                        >
                            <option value="">"Select a style..."</option>
                            <option value="traditional">"Traditional"</option>
                            <option value="neo-traditional">"Neo-Traditional"</option>
                            <option value="realism">"Realism"</option>
                            <option value="watercolor">"Watercolor"</option>
                            <option value="blackwork">"Blackwork"</option>
                            <option value="japanese">"Japanese"</option>
                            <option value="minimalist">"Minimalist"</option>
                            <option value="geometric">"Geometric"</option>
                        </select>
                    </div>

                    <div style="margin-bottom: 1.5rem;">
                        <label style="display: block; margin-bottom: 0.5rem; font-weight: bold;">
                            "Where on your body?"
                        </label>
                        <select 
                            on:change=move |ev| {
                                body_placement.set(event_target_value(&ev));
                            }
                            class="form-input"
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

                    <div style="margin-bottom: 1.5rem;">
                        <label style="display: block; margin-bottom: 0.5rem; font-weight: bold;">
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
                            style="width: 100%;"
                        />
                        <p style="text-align: center; margin-top: 0.5rem;">
                            {move || pain_tolerance.get()}
                        </p>
                    </div>

                    <div style="margin-bottom: 1.5rem;">
                        <label style="display: block; margin-bottom: 0.5rem; font-weight: bold;">
                            "Budget range"
                        </label>
                        <div style="display: flex; gap: 1rem; align-items: center;">
                            <input 
                                type="number"
                                placeholder="Min"
                                value=move || budget_min.get().to_string()
                                on:input=move |ev| {
                                    if let Ok(val) = event_target_value(&ev).parse::<f64>() {
                                        budget_min.set(val);
                                    }
                                }
                                class="form-input"
                                style="flex: 1;"
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
                                class="form-input"
                                style="flex: 1;"
                            />
                        </div>
                    </div>

                    <div style="margin-bottom: 1.5rem;">
                        <label style="display: block; margin-bottom: 0.5rem; font-weight: bold;">
                            "What vibe are you looking for?"
                        </label>
                        <div style="display: flex; flex-direction: column; gap: 0.5rem;">
                            <label style="display: flex; align-items: center; gap: 0.5rem;">
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
                            <label style="display: flex; align-items: center; gap: 0.5rem;">
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
                            <label style="display: flex; align-items: center; gap: 0.5rem;">
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
                            <label style="display: flex; align-items: center; gap: 0.5rem;">
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

                    <div style="text-align: center; margin-top: 2rem;">
                        <button type="submit" class="btn-primary">
                            "Find My Artists"
                        </button>
                    </div>
                </form>
            </div>
        </div>
    }
}