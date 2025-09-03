use leptos::prelude::*;
use thaw::*;
use crate::db::entities::{QuestionnaireQuestion, ArtistQuestionnaire};
use crate::server::{get_default_questions, get_artist_questionnaire_configuration};

#[component]
pub fn QuestionnaireBuilder() -> impl IntoView {
    let artist_id = -1; // Test with Frank Reynolds for now
    
    // Load default questions
    let default_questions = Resource::new_blocking(
        move || (),
        move |_| async move {
            get_default_questions().await
        }
    );
    
    // Load current artist configuration
    let artist_config = Resource::new_blocking(
        move || artist_id,
        move |id| async move {
            get_artist_questionnaire_configuration(id).await
        }
    );

    view! {
        <div class="questionnaire-builder">
            <div class="questionnaire-header">
                <h2>"Questionnaire Configuration"</h2>
                <p>"Configure which questions clients will see when booking with you."</p>
            </div>
            
            <div class="questionnaire-content">
                <Suspense fallback=move || view! { <div class="loading">"Loading questions..."</div> }>
                    {move || {
                        let questions_result = default_questions.get();
                        let config_result = artist_config.get();
                        
                        match (questions_result, config_result) {
                            (Some(Ok(questions)), Some(Ok(config))) => {
                                view! {
                                    <QuestionnaireDisplay 
                                        questions=questions
                                        config=config
                                    />
                                }.into_any()
                            }
                            _ => view! { <div class="loading">"Loading..."</div> }.into_any()
                        }
                    }}
                </Suspense>
            </div>
        </div>
    }
}

#[component]
fn QuestionnaireDisplay(
    questions: Vec<QuestionnaireQuestion>,
    config: Vec<ArtistQuestionnaire>
) -> impl IntoView {
    view! {
        <div class="questionnaire-form">
            <div class="questions-list">
                <h3>"Available Questions"</h3>
                {questions.into_iter().map(|question| {
                    let is_enabled = config.iter().any(|c| c.question_id == question.id);
                    let question_config = config.iter().find(|c| c.question_id == question.id);
                    
                    view! {
                        <div class="question-config-item">
                            <div class="question-header">
                                <div class="question-status">
                                    {if is_enabled { "✅ Enabled" } else { "❌ Disabled" }}
                                </div>
                                <div class="question-info">
                                    <h4>{question.question_text}</h4>
                                    <span class="question-type">{format!("Type: {}", question.question_type)}</span>
                                </div>
                            </div>
                            
                            {if is_enabled {
                                view! {
                                    <div class="question-options">
                                        <div class="option-row">
                                            <label>"Required:"</label>
                                            <span>{if question_config.map(|c| c.is_required).unwrap_or(true) { "Yes" } else { "No" }}</span>
                                        </div>
                                        
                                        {if question.question_type == "multiselect" {
                                            view! {
                                                <div class="option-row">
                                                    <label>"Available Options:"</label>
                                                    <div class="options-display">
                                                        {match &question.options_data {
                                                            Some(options_json) => {
                                                                if let Ok(options) = serde_json::from_str::<Vec<String>>(options_json) {
                                                                    view! {
                                                                        <div class="options-list">
                                                                            {options.into_iter().map(|opt| view! {
                                                                                <span class="option-tag">{opt}</span>
                                                                            }).collect::<Vec<_>>()}
                                                                        </div>
                                                                    }.into_any()
                                                                } else {
                                                                    view! { <span>"Invalid options format"</span> }.into_any()
                                                                }
                                                            },
                                                            None => view! { <span>"No options configured"</span> }.into_any()
                                                        }}
                                                    </div>
                                                </div>
                                            }.into_any()
                                        } else {
                                            view! { <div></div> }.into_any()
                                        }}
                                    </div>
                                }.into_any()
                            } else {
                                view! { <div></div> }.into_any()
                            }}
                        </div>
                    }
                }).collect::<Vec<_>>()}
            </div>
            
            <div class="save-section">
                <p>"This is a read-only view of the questionnaire configuration."</p>
                <p>"Frank Reynolds has {config.len()} questions configured for new bookings."</p>
            </div>
        </div>
    }
}