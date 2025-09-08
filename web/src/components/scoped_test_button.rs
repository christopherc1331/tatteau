use leptos::prelude::*;
use stylance::import_style;

// Import scoped styles from the CSS module
import_style!(styles, "scoped_test_button.module.css");

#[component]
pub fn ScopedTestButton(
    #[prop(optional)] text: Option<String>,
    #[prop(optional)] on_click: Option<Box<dyn Fn() + 'static>>,
    #[prop(default = false)] disabled: bool,
) -> impl IntoView {
    let button_text = text.unwrap_or_else(|| "Scoped CSS Button".to_string());
    
    let handle_click = move |_| {
        if let Some(ref callback) = on_click {
            callback();
        }
    };

    view! {
        <div class=styles::container>
            <h3 class=styles::title>"Scoped CSS Demo"</h3>
            <p class=styles::description>
                "This component uses Stylance for CSS scoping. The styles are isolated and won't conflict with other components."
            </p>
            <button 
                class=styles::button
                on:click=handle_click
                disabled=disabled
            >
                {button_text}
            </button>
        </div>
    }
}