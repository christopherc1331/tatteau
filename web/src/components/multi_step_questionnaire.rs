use leptos::prelude::*;
use thaw::*;
use crate::db::entities::ClientQuestionnaireForm;
use std::collections::HashMap;
use serde_json;

#[component]
pub fn MultiStepQuestionnaire(
    questionnaire_form: ClientQuestionnaireForm,
    responses: RwSignal<HashMap<i32, String>>,
    on_completion: impl Fn() + 'static + Copy + Send + Sync,
    on_back: impl Fn() + 'static + Copy + Send + Sync,
) -> impl IntoView {
    let current_step = RwSignal::new(0usize);
    let total_questions = questionnaire_form.questions.len();
    
    // Filter out contact info questions for logged-in users
    let filtered_questions = questionnaire_form.questions.into_iter()
        .filter(|q| {
            let text = q.question_text.to_lowercase();
            !text.contains("name") && !text.contains("email") && !text.contains("phone")
        })
        .collect::<Vec<_>>();
    
    let total_steps = filtered_questions.len();
    
    let current_question = Memo::new(move |_| {
        let step = current_step.get();
        if step < filtered_questions.len() {
            Some(filtered_questions[step].clone())
        } else {
            None
        }
    });
    
    let is_current_answered = Memo::new(move |_| {
        if let Some(question) = current_question.get() {
            if question.is_required {
                let response = responses.get().get(&question.id).cloned().unwrap_or_default();
                !response.trim().is_empty()
            } else {
                true // Optional questions can be skipped
            }
        } else {
            false
        }
    });
    
    let progress_percentage = Memo::new(move |_| {
        if total_steps == 0 { 
            100.0 
        } else { 
            (current_step.get() as f64 / total_steps as f64) * 100.0 
        }
    });
    
    let handle_next = move || {
        let current = current_step.get();
        if current + 1 >= total_steps {
            on_completion();
        } else {
            current_step.set(current + 1);
        }
    };
    
    let handle_previous = move || {
        let current = current_step.get();
        if current == 0 {
            on_back();
        } else {
            current_step.set(current - 1);
        }
    };
    
    let handle_response = move |question_id: i32, value: String| {
        responses.update(|responses| {
            responses.insert(question_id, value);
        });
    };
    
    // Auto-advance for boolean questions (immediate advance for better UX)
    let handle_boolean_response = move |question_id: i32, value: String| {
        handle_response(question_id, value);
        // Auto-advance immediately for boolean questions
        handle_next();
    };
    
    view! {
        <div class="multi-step-questionnaire">
            // Progress Bar
            <div class="progress-container">
                <div class="progress-info">
                    <h3>"Artist Questionnaire"</h3>
                    <p class="progress-text">
                        {move || format!("Question {} of {}", 
                            current_step.get() + 1, 
                            total_steps
                        )}
                    </p>
                </div>
                <div class="progress-bar">
                    <div 
                        class="progress-fill"
                        style:width=move || format!("{}%", progress_percentage.get())
                    ></div>
                </div>
            </div>
            
            // Question Content
            <div class="question-container">
                {move || {
                    if let Some(question) = current_question.get() {
                        let question_id = question.id;
                        let current_response = responses.get().get(&question_id).cloned().unwrap_or_default();
                        
                        view! {
                            <div class="question-content">
                                <div class="question-header">
                                    <h4 class="question-text">{question.question_text.clone()}</h4>
                                    {if question.question_type == "multiselect" {
                                        view! {
                                            <span 
                                                class="info-icon" 
                                                title="This is a list of options that the artist is currently accepting at this time"
                                            >{"ⓘ"}</span>
                                        }.into_any()
                                    } else {
                                        view! {}.into_any()
                                    }}
                                    {if !question.is_required {
                                        view! {
                                            <span class="optional-indicator">"(optional)"</span>
                                        }.into_any()
                                    } else {
                                        view! {}.into_any()
                                    }}
                                </div>
                                
                                <div class="answer-section">
                                    {match question.question_type.as_str() {
                                        "text" => view! {
                                            <div class="text-input-container">
                                                <Textarea
                                                    placeholder="Please provide details..."
                                                    value=RwSignal::new(current_response.clone())
                                                    on:input=move |ev| {
                                                        let value = event_target_value(&ev);
                                                        handle_response(question_id, value);
                                                    }
                                                />
                                            </div>
                                        }.into_any(),
                                        "multiselect" => {
                                            let options = question.options.clone();
                                            let selected: Vec<String> = if current_response.is_empty() {
                                                Vec::new()
                                            } else {
                                                serde_json::from_str(&current_response).unwrap_or_default()
                                            };
                                            
                                            view! {
                                                <div class="multiselect-container">
                                                    {options.into_iter().map(|option| {
                                                        let option_value = option.clone();
                                                        let is_selected = selected.contains(&option);
                                                        
                                                        view! {
                                                            <div class="option-item">
                                                                <Button
                                                                    appearance=if is_selected { 
                                                                        ButtonAppearance::Primary 
                                                                    } else { 
                                                                        ButtonAppearance::Secondary 
                                                                    }
                                                                    class="option-button"
                                                                    on_click=move |_| {
                                                                        let current_selected = responses.get()
                                                                            .get(&question_id)
                                                                            .cloned()
                                                                            .unwrap_or_default();
                                                                        
                                                                        let mut selected_list: Vec<String> = if current_selected.is_empty() {
                                                                            Vec::new()
                                                                        } else {
                                                                            serde_json::from_str(&current_selected).unwrap_or_default()
                                                                        };
                                                                        
                                                                        if selected_list.contains(&option_value) {
                                                                            selected_list.retain(|x| x != &option_value);
                                                                        } else {
                                                                            selected_list.push(option_value.clone());
                                                                        }
                                                                        
                                                                        let json_value = serde_json::to_string(&selected_list).unwrap_or_default();
                                                                        handle_response(question_id, json_value);
                                                                    }
                                                                >
                                                                    {option}
                                                                </Button>
                                                            </div>
                                                        }
                                                    }).collect::<Vec<_>>()}
                                                </div>
                                            }.into_any()
                                        },
                                        "datetime" => view! {
                                            <div class="datetime-input-container">
                                                <Input
                                                    input_type=InputType::DatetimeLocal
                                                    value=RwSignal::new(current_response.clone())
                                                    on:input=move |ev| {
                                                        let value = event_target_value(&ev);
                                                        handle_response(question_id, value);
                                                    }
                                                />
                                            </div>
                                        }.into_any(),
                                        "boolean" => view! {
                                            <div class="boolean-container">
                                                <div class="boolean-options">
                                                    <Button
                                                        appearance=if current_response == "true" { 
                                                            ButtonAppearance::Primary 
                                                        } else { 
                                                            ButtonAppearance::Secondary 
                                                        }
                                                        class="boolean-button"
                                                        on_click=move |_| {
                                                            handle_boolean_response(question_id, "true".to_string());
                                                        }
                                                    >
                                                        "Yes"
                                                    </Button>
                                                    <Button
                                                        appearance=if current_response == "false" { 
                                                            ButtonAppearance::Primary 
                                                        } else { 
                                                            ButtonAppearance::Secondary 
                                                        }
                                                        class="boolean-button"
                                                        on_click=move |_| {
                                                            handle_boolean_response(question_id, "false".to_string());
                                                        }
                                                    >
                                                        "No"
                                                    </Button>
                                                </div>
                                            </div>
                                        }.into_any(),
                                        _ => view! {
                                            <div class="default-input-container">
                                                <Input
                                                    placeholder="Your answer..."
                                                    value=RwSignal::new(current_response.clone())
                                                    on:input=move |ev| {
                                                        let value = event_target_value(&ev);
                                                        handle_response(question_id, value);
                                                    }
                                                />
                                            </div>
                                        }.into_any(),
                                    }}
                                </div>
                            </div>
                        }.into_any()
                    } else {
                        view! {
                            <div class="no-questions">
                                <p>"No questionnaire configured for this artist."</p>
                            </div>
                        }.into_any()
                    }
                }}
            </div>
            
            // Navigation Controls
            <div class="navigation-controls">
                <Button
                    appearance=ButtonAppearance::Secondary
                    on_click=move |_| handle_previous()
                >
                    {move || if current_step.get() == 0 { "Back" } else { "Previous" }}
                </Button>
                
                <Button
                    appearance=ButtonAppearance::Primary
                    disabled=MaybeSignal::derive(move || !is_current_answered.get())
                    on_click=move |_| handle_next()
                >
                    {move || if current_step.get() + 1 >= total_steps { 
                        "Complete" 
                    } else { 
                        "Next" 
                    }}
                </Button>
            </div>
        </div>
    }
}