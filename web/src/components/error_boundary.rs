use leptos::prelude::*;
use leptos::task::spawn_local;
use thaw::{MessageBar, MessageBarIntent, Button};
use crate::server::log_client_error;
use serde_json;

stylance::import_crate_style!(style, "src/components/error_boundary.module.css");

#[component]
pub fn ErrorBoundary(children: Children) -> impl IntoView {
    // Simple error boundary that just wraps children
    // Advanced error handling can be added later when needed
    view! {
        {children()}
    }
}

// Manual error logging function for components
pub fn log_component_error(
    error_message: String,
    component_name: String,
    error_stack: Option<String>,
) {
    spawn_local(async move {
        let additional_context = serde_json::json!({
            "component": component_name,
            "type": "component_error"
        }).to_string();
        
        let _ = log_client_error(
            "client".to_string(),
            "error".to_string(),
            error_message,
            error_stack,
            None, // URL
            None, // User agent
            None, // Session ID
            Some(additional_context),
        ).await;
    });
}