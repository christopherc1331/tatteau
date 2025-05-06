use leptos::prelude::*;
use thaw::{MessageBar, MessageBarIntent};

#[component]
pub fn ErrorView(message: Option<String>) -> impl IntoView {
    view! {
        <MessageBar intent=MessageBarIntent::Error>
            {message.unwrap_or_else(|| "An error occurred. Please try again.".to_string())}
        </MessageBar>
    }
}
