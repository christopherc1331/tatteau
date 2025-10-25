use crate::server::login_user;
use crate::views::auth::LoginData;
use leptos::{prelude::*, task::spawn_local};
use thaw::*;

#[component]
pub fn AdminLoginPage() -> impl IntoView {
    let email = RwSignal::new(String::new());
    let password = RwSignal::new(String::new());
    let password_visible = RwSignal::new(false);
    let loading = RwSignal::new(false);
    let error_message = RwSignal::new(Option::<String>::None);

    let is_button_disabled =
        Memo::new(move |_| email.get().is_empty() || password.get().is_empty());

    let submit_login = move |_| {
        loading.set(true);
        error_message.set(None);

        let login_data = LoginData {
            email: email.get(),
            password: password.get(),
            user_type: "admin".to_string(),
        };

        spawn_local(async move {
            match login_user(login_data).await {
                Ok(auth_response) => {
                    if auth_response.success {
                        // Store JWT token in localStorage
                        if let Some(token) = &auth_response.token {
                            #[cfg(feature = "hydrate")]
                            {
                                use wasm_bindgen::prelude::*;

                                #[wasm_bindgen]
                                extern "C" {
                                    #[wasm_bindgen(js_namespace = localStorage)]
                                    fn setItem(key: &str, value: &str);
                                }

                                setItem("tatteau_auth_token", token);
                            }
                        }

                        // Redirect to admin dashboard
                        if let Some(window) = web_sys::window() {
                            let _ = window.location().set_href("/admin/dashboard");
                        }
                    } else {
                        error_message.set(auth_response.error);
                    }
                }
                Err(e) => {
                    error_message.set(Some(format!("Login failed: {}", e)));
                }
            }
            loading.set(false);
        });
    };

    view! {
        <div class="auth-container">
            <div class="auth-card">
                <div class="auth-header">
                    <h1>"Admin Login"</h1>
                    <p>"Sign in to the Tatteau admin panel"</p>
                </div>

                <form on:submit=move |ev| {
                    ev.prevent_default();
                    submit_login(());
                }>
                    <div class="auth-form-group">
                        <div class="auth-input-wrapper">
                            <span class="auth-input-icon">
                                <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                                    <path d="M4 4h16c1.1 0 2 .9 2 2v12c0 1.1-.9 2-2 2H4c-1.1 0-2-.9-2-2V6c0-1.1.9-2 2-2z"></path>
                                    <polyline points="22,6 12,13 2,6"></polyline>
                                </svg>
                            </span>
                            <Input
                                class="auth-input"
                                placeholder="Email"
                                input_type=InputType::Email
                                value=email
                            />
                        </div>
                    </div>

                    <div class="auth-form-group">
                        <div class="auth-input-wrapper">
                            <span class="auth-input-icon">
                                <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                                    <rect x="3" y="11" width="18" height="11" rx="2" ry="2"></rect>
                                    <path d="M7 11V7a5 5 0 0 1 10 0v4"></path>
                                </svg>
                            </span>
                            <Input
                                class="auth-input"
                                placeholder="Password"
                                input_type=Signal::derive(move || if password_visible.get() { InputType::Text } else { InputType::Password })
                                value=password
                            />
                            <button
                                type="button"
                                class="auth-password-toggle"
                                on:click=move |_| password_visible.set(!password_visible.get())
                            >
                                {move || if password_visible.get() {
                                    view! {
                                        <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                                            <path d="M17.94 17.94A10.07 10.07 0 0 1 12 20c-7 0-11-8-11-8a18.45 18.45 0 0 1 5.06-5.94M9.9 4.24A9.12 9.12 0 0 1 12 4c7 0 11 8 11 8a18.5 18.5 0 0 1-2.16 3.19m-6.72-1.07a3 3 0 1 1-4.24-4.24"></path>
                                            <line x1="1" y1="1" x2="23" y2="23"></line>
                                        </svg>
                                    }.into_any()
                                } else {
                                    view! {
                                        <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                                            <path d="M1 12s4-8 11-8 11 8 11 8-4 8-11 8-11-8-11-8z"></path>
                                            <circle cx="12" cy="12" r="3"></circle>
                                        </svg>
                                    }.into_any()
                                }}
                            </button>
                        </div>
                    </div>

                    {move || error_message.get().map(|msg| view! {
                        <div class="auth-error-message">{msg}</div>
                    })}

                    <Button
                        class="auth-submit-btn"
                        button_type=ButtonType::Submit
                        loading=Signal::from(loading)
                        disabled=Signal::from(is_button_disabled)
                    >
                        "Sign In as Admin"
                    </Button>
                </form>
            </div>
        </div>
    }
}
