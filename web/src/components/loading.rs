use leptos::prelude::*;
use thaw::{Spinner, SpinnerSize};

#[component]
pub fn LoadingView(message: Option<String>) -> impl IntoView {
    view! {
        <div class="loading-container">
            <Spinner size=SpinnerSize::Large />
            <p class="loading-message">
                {message.unwrap_or_else(|| "Loading, please wait...".to_string())}
            </p>
        </div>
    }
}
