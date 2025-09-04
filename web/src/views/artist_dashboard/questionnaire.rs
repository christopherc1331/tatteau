use leptos::prelude::*;
use leptos::task::spawn_local;
use server_fn::ServerFnError;
use thaw::*;
use crate::db::entities::{QuestionnaireQuestion, ArtistQuestionnaire};
use crate::server::{get_default_questions, get_artist_questionnaire_configuration, update_artist_questionnaire_configuration};
use crate::utils::auth::use_authenticated_artist_id;

#[component]
pub fn QuestionnaireBuilder() -> impl IntoView {
    let artist_id = use_authenticated_artist_id();
    
    // Load default questions
    let default_questions = Resource::new_blocking(
        move || (),
        move |_| async move {
            get_default_questions().await
        }
    );
    
    // Load current artist configuration
    let artist_config = Resource::new_blocking(
        move || artist_id.get(),
        move |id| async move {
            match id {
                Some(artist_id) => get_artist_questionnaire_configuration(artist_id).await,
                None => Err(ServerFnError::new("Artist not authenticated".to_string())),
            }
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
                                    <InteractiveQuestionnaireBuilder 
                                        questions=questions
                                        config=config
                                        artist_id=artist_id.get().unwrap_or(-1)
                                        on_config_updated=move |_| {
                                            artist_config.refetch();
                                        }
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
fn InteractiveQuestionnaireBuilder(
    questions: Vec<QuestionnaireQuestion>,
    config: Vec<ArtistQuestionnaire>,
    artist_id: i32,
    on_config_updated: impl Fn(()) + Clone + Send + Sync + 'static
) -> impl IntoView {
    // Create local state for question configurations
    let (current_config, set_current_config) = RwSignal::new(config).split();
    let (is_saving, set_is_saving) = RwSignal::new(false).split();
    let (save_message, set_save_message) = RwSignal::new(None::<String>).split();
    let (has_changes, set_has_changes) = RwSignal::new(false).split();
    
    // Save configuration to server
    let save_config = {
        let on_updated = on_config_updated.clone();
        move |new_config: Vec<ArtistQuestionnaire>| {
            let config_clone = new_config.clone();
            let on_updated = on_updated.clone();
            let set_is_saving = set_is_saving.clone();
            let set_save_message = set_save_message.clone();
            let set_has_changes = set_has_changes.clone();
            
            spawn_local(async move {
                set_is_saving.set(true);
                set_save_message.set(None);
                
                match update_artist_questionnaire_configuration(artist_id, config_clone).await {
                    Ok(_) => {
                        set_save_message.set(Some("Configuration saved successfully!".to_string()));
                        set_has_changes.set(false);
                        on_updated(());
                    }
                    Err(e) => {
                        set_save_message.set(Some(format!("Error saving: {}", e)));
                    }
                }
                set_is_saving.set(false);
            });
        }
    };
    
    view! {
        <div class="questionnaire-form">
            <div class="questions-list">
                <h3>"Available Questions"</h3>
                <p class="questions-subtitle">"Toggle questions on/off and configure requirements for your booking form."</p>
                
                {questions.into_iter().enumerate().map(|(index, question)| {
                    let question_id = question.id;
                    let question_type = question.question_type.clone();
                    let question_text = question.question_text.clone();
                    let options_data = question.options_data.clone();
                    
                    // Find current config for this question
                    let initial_config = current_config.get().into_iter()
                        .find(|c| c.question_id == question_id);
                    
                    let is_enabled = RwSignal::new(initial_config.is_some());
                    let is_required = RwSignal::new(initial_config.map(|c| c.is_required).unwrap_or(true));
                    
                    // Update handlers
                    let update_enabled = {
                        let set_current_config = set_current_config.clone();
                        let set_has_changes = set_has_changes.clone();
                        move |enabled: bool| {
                            set_current_config.update(|config| {
                                if enabled {
                                    // Add question to config
                                    if !config.iter().any(|c| c.question_id == question_id) {
                                        config.push(ArtistQuestionnaire {
                                            id: 0, // Will be set by database
                                            artist_id,
                                            question_id,
                                            is_required: true,
                                            display_order: (index + 1) as i32,
                                            custom_options: None,
                                        });
                                    }
                                } else {
                                    // Remove question from config
                                    config.retain(|c| c.question_id != question_id);
                                }
                            });
                            set_has_changes.set(true);
                        }
                    };
                    
                    let update_required = {
                        let set_current_config = set_current_config.clone();
                        let set_has_changes = set_has_changes.clone();
                        move |required: bool| {
                            set_current_config.update(|config| {
                                if let Some(item) = config.iter_mut().find(|c| c.question_id == question_id) {
                                    item.is_required = required;
                                }
                            });
                            set_has_changes.set(true);
                        }
                    };
                    
                    // Watch for enabled state changes - skip initial run to avoid duplicate effects
                    let update_enabled_clone = update_enabled.clone();
                    let initial_enabled = is_enabled.get_untracked();
                    Effect::new(move |prev_run: Option<bool>| {
                        let enabled = is_enabled.get();
                        if let Some(_) = prev_run {
                            // Only update after the first run to avoid initial trigger
                            if enabled != initial_enabled {
                                update_enabled_clone(enabled);
                            }
                        }
                        enabled
                    });
                    
                    // Watch for required state changes - skip initial run to avoid duplicate effects
                    let update_required_clone = update_required.clone();
                    let initial_required = is_required.get_untracked();
                    Effect::new(move |prev_run: Option<bool>| {
                        let required = is_required.get();
                        if let Some(_) = prev_run {
                            // Only update after the first run to avoid initial trigger
                            if required != initial_required {
                                update_required_clone(required);
                            }
                        }
                        required
                    });
                    
                    view! {
                        <div class="question-config-item">
                            <div class="question-header">
                                <div class="question-controls">
                                    <Switch 
                                        checked=is_enabled
                                    />
                                    <div class="question-status-text">
                                        {move || if is_enabled.get() { "Enabled" } else { "Disabled" }}
                                    </div>
                                </div>
                                <div class="question-info">
                                    <h4>{question_text}</h4>
                                    <span class="question-type">{format!("Type: {}", question_type)}</span>
                                </div>
                            </div>
                            
                            {move || {
                                if is_enabled.get() {
                                    view! {
                                        <div class="question-options">
                                            <div class="option-row">
                                                <label>"Required:"</label>
                                                <Switch 
                                                    checked=is_required
                                                />
                                            </div>
                                            
                                            {if question_type == "multiselect" {
                                                view! {
                                                    <div class="option-row">
                                                        <label>"Available Options:"</label>
                                                        <div class="options-display">
                                                            {match &options_data {
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
                                }
                            }}
                        </div>
                    }
                }).collect::<Vec<_>>()}
            </div>
            
            <div class="save-section">
                {move || {
                    if let Some(message) = save_message.get() {
                        let class = if message.contains("Error") { "error-message" } else { "success-message" };
                        view! {
                            <div class=class>{message}</div>
                        }.into_any()
                    } else {
                        view! { <div></div> }.into_any()
                    }
                }}
                
                <button 
                    class="thaw-button thaw-button--primary"
                    disabled=move || !has_changes.get() || is_saving.get()
                    on:click=move |_| {
                        let config = current_config.get();
                        save_config(config);
                    }
                >
                    {move || if is_saving.get() { "Saving..." } else { "Save Configuration" }}
                </button>
                
                <p class="status-text">
                    {move || {
                        let enabled_count = current_config.get().len();
                        format!("{} questions enabled for new bookings", enabled_count)
                    }}
                </p>
            </div>
        </div>
    }
}