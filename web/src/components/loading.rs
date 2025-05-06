use leptos::prelude::*;
use thaw::{Spinner, SpinnerSize};

#[component]
pub fn LoadingView(message: Option<String>) -> impl IntoView {
    view! {
        <div class="flex flex-col items-center justify-center p-4">
            <Spinner size=SpinnerSize::Large />
            <p class="mt-2 text-gray-600">
                {message.unwrap_or_else(|| "Loading, please wait...".to_string())}
            </p>
        </div>
    }
}
