use leptos::prelude::*;
use thaw::*;
use serde_json;
use crate::server::*;
use crate::db::entities::{RecurringRule};

#[component]
pub fn ArtistRecurring() -> impl IntoView {
    let show_add_rule_modal = RwSignal::new(false);
    let rule_type = RwSignal::new("weekdays".to_string());
    let rule_action = RwSignal::new("blocked".to_string()); // Default to blocked as per user request
    let rule_name = RwSignal::new("".to_string());
    let rule_pattern = RwSignal::new("".to_string());
    let start_time = RwSignal::new("".to_string());
    let end_time = RwSignal::new("".to_string());
    let selected_weekdays = RwSignal::new(Vec::<String>::new());
    let selected_dates = RwSignal::new("".to_string());
    let monthly_pattern = RwSignal::new("".to_string());
    
    // For now, hardcode artist_id as 1 - in real app this would come from auth context
    let artist_id = 1;
    
    // Resource to fetch recurring rules from database
    let rules_resource = Resource::new(
        move || (),
        move |_| async move {
            get_recurring_rules(artist_id).await.unwrap_or_else(|_| Vec::new())
        }
    );
    
    // Action to create a new rule
    let create_rule_action = Action::new(move |(artist_id, name, rule_type, pattern, action, start_time_param, end_time_param): &(i32, String, String, String, String, Option<String>, Option<String>)| {
        let (artist_id, name, rule_type, pattern, action, start_time_param, end_time_param) = (*artist_id, name.clone(), rule_type.clone(), pattern.clone(), action.clone(), start_time_param.clone(), end_time_param.clone());
        async move {
            match create_recurring_rule(artist_id, name, rule_type, pattern, action, start_time_param, end_time_param).await {
                Ok(_) => {
                    show_add_rule_modal.set(false);
                    rules_resource.refetch();
                    // Reset form
                    rule_name.set("".to_string());
                    rule_pattern.set("".to_string());
                    start_time.set("".to_string());
                    end_time.set("".to_string());
                    selected_weekdays.set(Vec::new());
                    selected_dates.set("".to_string());
                    monthly_pattern.set("".to_string());
                },
                Err(e) => {
                    leptos::logging::error!("Failed to create rule: {}", e);
                }
            }
        }
    });
    
    // Action to update a rule
    let update_rule_action = Action::new(move |(id, name_param, pattern_param, action_param, start_time_param, end_time_param, active_param): &(i32, Option<String>, Option<String>, Option<String>, Option<String>, Option<String>, Option<bool>)| {
        let (id, name_param, pattern_param, action_param, start_time_param, end_time_param, active_param) = (*id, name_param.clone(), pattern_param.clone(), action_param.clone(), start_time_param.clone(), end_time_param.clone(), *active_param);
        async move {
            match update_recurring_rule(id, name_param, pattern_param, action_param, start_time_param, end_time_param, active_param).await {
                Ok(_) => {
                    rules_resource.refetch();
                },
                Err(e) => {
                    leptos::logging::error!("Failed to update rule: {}", e);
                }
            }
        }
    });
    
    // Action to delete a rule
    let delete_rule_action = Action::new(move |rule_id: &i32| {
        let rule_id = *rule_id;
        async move {
            match delete_recurring_rule(rule_id).await {
                Ok(_) => {
                    rules_resource.refetch();
                },
                Err(e) => {
                    leptos::logging::error!("Failed to delete rule: {}", e);
                }
            }
        }
    });
    
    // Function to build pattern from form inputs
    let build_pattern = move || -> String {
        match rule_type.get().as_str() {
            "weekdays" => {
                let days = selected_weekdays.get();
                if days.is_empty() {
                    "[]".to_string()
                } else {
                    // Convert day names to numbers for JSON array
                    let day_numbers: Vec<i32> = days.iter().map(|day| {
                        match day.as_str() {
                            "Sunday" => 0,
                            "Monday" => 1,
                            "Tuesday" => 2,
                            "Wednesday" => 3,
                            "Thursday" => 4,
                            "Friday" => 5,
                            "Saturday" => 6,
                            _ => -1
                        }
                    }).filter(|&n| n != -1).collect();
                    
                    // Create JSON array
                    serde_json::to_string(&day_numbers).unwrap_or_else(|_| "[]".to_string())
                }
            },
            "dates" => selected_dates.get(),
            "monthly" => monthly_pattern.get(),
            _ => "".to_string()
        }
    };
    
    // Function to save the rule
    let save_rule = move |_| {
        let pattern = build_pattern();
        if rule_name.get().is_empty() || pattern.is_empty() {
            leptos::logging::warn!("Rule name and pattern are required");
            return;
        }
        
        create_rule_action.dispatch((
            artist_id,
            rule_name.get(),
            rule_type.get(),
            pattern,
            rule_action.get(),
            if start_time.get().is_empty() { None } else { Some(start_time.get()) },
            if end_time.get().is_empty() { None } else { Some(end_time.get()) },
        ));
    };
    
    // Function to toggle rule active status
    let toggle_rule_active = move |rule: RecurringRule| {
        update_rule_action.dispatch((
            rule.id,
            None,
            None,
            None,
            None,
            None,
            Some(!rule.active)
        ));
    };
    
    // Function to delete a rule
    let delete_rule = move |rule_id: i32| {
        delete_rule_action.dispatch(rule_id);
    };

    view! {
        <div class="recurring-settings">
            <div class="recurring-header">
                <div class="header-content">
                    <h1>"Recurring Availability Rules"</h1>
                    <p class="subtitle">"Set patterns that automatically apply to your calendar. You can always override these on specific dates."</p>
                </div>
                <div class="header-actions">
                    <Button appearance=ButtonAppearance::Primary on_click=move |_| show_add_rule_modal.set(true)>
                        "Add New Rule"
                    </Button>
                    <Button>
                        <a href="/artist/dashboard/calendar" style="text-decoration: none; color: inherit;">
                            "Back to Calendar"
                        </a>
                    </Button>
                </div>
            </div>
            
            <div class="recurring-content">
                <div class="rule-types-info">
                    <h2>"Rule Types"</h2>
                    <div class="rule-type-cards">
                        <div class="rule-type-card">
                            <h3>"📅 Weekday Patterns"</h3>
                            <p>"Set availability for specific days of the week (e.g., no weekends, Tuesday mornings only)"</p>
                        </div>
                        <div class="rule-type-card">
                            <h3>"🎉 Annual Dates"</h3>
                            <p>"Block specific dates every year (holidays, personal days, vacations)"</p>
                        </div>
                        <div class="rule-type-card">
                            <h3>"🗓️ Monthly Patterns"</h3>
                            <p>"Set rules based on monthly patterns (1st of month, last Friday, etc.)"</p>
                        </div>
                    </div>
                </div>
                
                <div class="existing-rules">
                    <h2>"Your Active Rules"</h2>
                    <Suspense fallback=move || view! { <div>"Loading rules..."</div> }>
                        {move || {
                            rules_resource.get().map(|rules| {
                                if rules.is_empty() {
                                    view! {
                                        <div class="no-rules">
                                            <p>"No recurring rules set up yet. Add your first rule to get started!"</p>
                                        </div>
                                    }.into_any()
                                } else {
                                    view! {
                                        <div class="rules-list">
                                            {rules.into_iter().map(|rule| {
                                                let rule_clone_toggle = rule.clone();
                                                let rule_clone_delete = rule.clone();
                                                view! {
                                                    <div class="rule-card" class:inactive=move || !rule.active>
                                                        <div class="rule-info">
                                                            <h3>{rule.name.clone()}</h3>
                                                            <div class="rule-details">
                                                                <span class="rule-pattern">{rule.pattern.clone()}</span>
                                                                <span class="rule-action" class:blocked=move || rule.action == "blocked">
                                                                    {if rule.action == "blocked" { "🚫 Blocked" } else { "✅ Available" }}
                                                                </span>
                                                                {rule.start_time.clone().map(|time| {
                                                                    view! {
                                                                        <span class="rule-time">{time}</span>
                                                                    }
                                                                })}
                                                            </div>
                                                        </div>
                                                        <div class="rule-actions">
                                                            <Button size=ButtonSize::Small>
                                                                "Edit"
                                                            </Button>
                                                            <Button size=ButtonSize::Small on_click=move |_| toggle_rule_active(rule_clone_toggle.clone())>
                                                                {if rule.active { "Disable" } else { "Enable" }}
                                                            </Button>
                                                            <Button size=ButtonSize::Small on_click=move |_| delete_rule(rule_clone_delete.id)>
                                                                "Delete"
                                                            </Button>
                                                        </div>
                                                    </div>
                                                }
                                            }).collect::<Vec<_>>()}
                                        </div>
                                    }.into_any()
                                }
                            })
                        }}
                    </Suspense>
                </div>
            </div>
            
            // Add Rule Modal
            <Show when=move || show_add_rule_modal.get()>
                <div class="modal-backdrop" on:click=move |_| show_add_rule_modal.set(false)>
                    <div class="add-rule-modal" on:click=|e| e.stop_propagation()>
                        <div class="modal-header">
                            <h2>"Add Recurring Rule"</h2>
                            <Button on_click=move |_| show_add_rule_modal.set(false)>"×"</Button>
                        </div>
                        
                        <div class="modal-content">
                            <div class="add-rule-form">
                                <div class="form-group">
                                    <label>"Rule Name"</label>
                                    <Input 
                                        placeholder="e.g., No Weekends"
                                        value=rule_name
                                        on:input=move |e| {
                                            let val = event_target_value(&e);
                                            rule_name.set(val);
                                        }
                                    />
                                </div>
                                
                                <div class="form-group">
                                    <label>"Rule Type"</label>
                                    <div class="radio-group">
                                        <label class="radio-label">
                                            <input type="radio" name="rule_type" value="weekdays" checked=move || rule_type.get() == "weekdays" on:change=move |_| rule_type.set("weekdays".to_string()) />
                                            "Weekday Pattern"
                                        </label>
                                        <label class="radio-label">
                                            <input type="radio" name="rule_type" value="dates" checked=move || rule_type.get() == "dates" on:change=move |_| rule_type.set("dates".to_string()) />
                                            "Annual Dates"
                                        </label>
                                        <label class="radio-label">
                                            <input type="radio" name="rule_type" value="monthly" checked=move || rule_type.get() == "monthly" on:change=move |_| rule_type.set("monthly".to_string()) />
                                            "Monthly Pattern"
                                        </label>
                                    </div>
                                </div>
                                
                                <div class="form-group">
                                    <label>"Action"</label>
                                    <div class="radio-group">
                                        <label class="radio-label">
                                            <input type="radio" name="rule_action" value="available" checked=move || rule_action.get() == "available" on:change=move |_| rule_action.set("available".to_string()) />
                                            "Set as Available"
                                        </label>
                                        <label class="radio-label">
                                            <input type="radio" name="rule_action" value="blocked" checked=move || rule_action.get() == "blocked" on:change=move |_| rule_action.set("blocked".to_string()) />
                                            "Block/Unavailable"
                                        </label>
                                    </div>
                                </div>
                                
                                // Dynamic form based on rule type
                                {move || match rule_type.get().as_str() {
                                    "weekdays" => view! {
                                        <div class="form-group">
                                            <label>"Select Days"</label>
                                            <div class="weekday-checkboxes">
                                                {["Sunday", "Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday"].iter().map(|day| {
                                                    let day_str = day.to_string();
                                                    let day_clone = day_str.clone();
                                                    view! {
                                                        <label class="checkbox-label">
                                                            <input 
                                                                type="checkbox" 
                                                                on:change=move |e| {
                                                                    let checked = event_target_checked(&e);
                                                                    let mut current = selected_weekdays.get();
                                                                    if checked {
                                                                        if !current.contains(&day_clone) {
                                                                            current.push(day_clone.clone());
                                                                        }
                                                                    } else {
                                                                        current.retain(|d| d != &day_clone);
                                                                    }
                                                                    selected_weekdays.set(current);
                                                                }
                                                            />
                                                            {day_str}
                                                        </label>
                                                    }
                                                }).collect::<Vec<_>>()}
                                            </div>
                                        </div>
                                    }.into_any(),
                                    "dates" => view! {
                                        <div class="form-group">
                                            <label>"Annual Dates (comma separated)"</label>
                                            <Input 
                                                placeholder="e.g., December 25, January 1"
                                                value=selected_dates
                                                on:input=move |e| {
                                                    let val = event_target_value(&e);
                                                    selected_dates.set(val);
                                                }
                                            />
                                        </div>
                                    }.into_any(),
                                    "monthly" => view! {
                                        <div class="form-group">
                                            <label>"Monthly Pattern"</label>
                                            <Input 
                                                placeholder="e.g., 1st Monday, Last Friday, 15th of month"
                                                value=monthly_pattern
                                                on:input=move |e| {
                                                    let val = event_target_value(&e);
                                                    monthly_pattern.set(val);
                                                }
                                            />
                                        </div>
                                    }.into_any(),
                                    _ => view! {}.into_any()
                                }}
                                
                                <div class="form-group">
                                    <label>"Start Time (optional)"</label>
                                    <Input 
                                        placeholder="e.g., 09:00 or leave blank for all day"
                                        value=start_time
                                        on:input=move |e| {
                                            let val = event_target_value(&e);
                                            start_time.set(val);
                                        }
                                    />
                                </div>
                                
                                <div class="form-group">
                                    <label>"End Time (optional)"</label>
                                    <Input 
                                        placeholder="e.g., 17:00 or leave blank for all day"
                                        value=end_time
                                        on:input=move |e| {
                                            let val = event_target_value(&e);
                                            end_time.set(val);
                                        }
                                    />
                                </div>
                                
                                <div class="modal-actions">
                                    <Button appearance=ButtonAppearance::Primary on_click=save_rule>
                                        "Save Rule"
                                    </Button>
                                    <Button on_click=move |_| show_add_rule_modal.set(false)>
                                        "Cancel"
                                    </Button>
                                </div>
                            </div>
                        </div>
                    </div>
                </div>
            </Show>
        </div>
    }
}