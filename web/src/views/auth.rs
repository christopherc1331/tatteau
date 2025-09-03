use leptos::{prelude::*, task::spawn_local};
use leptos_router::components::A;
use thaw::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginData {
    pub email: String,
    pub password: String,
    pub user_type: String, // "client" or "artist"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignupData {
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub phone: Option<String>,
    pub password: String,
    pub user_type: String, // "client" or "artist"
}

#[component]
pub fn LoginPage() -> impl IntoView {
    let user_type = RwSignal::new("client".to_string());
    let email = RwSignal::new(String::new());
    let password = RwSignal::new(String::new());
    let loading = RwSignal::new(false);
    let error_message = RwSignal::new(Option::<String>::None);

    let is_button_disabled = Memo::new(move |_| {
        email.get().is_empty() || password.get().is_empty()
    });

    let submit_login = move |_| {
        loading.set(true);
        error_message.set(None);
        
        let login_data = LoginData {
            email: email.get(),
            password: password.get(),
            user_type: user_type.get(),
        };
        
        // TODO: Implement actual login logic
        spawn_local(async move {
            // Placeholder - will implement server function later
            loading.set(false);
        });
    };

    view! {
        <div class="auth-container">
            <div class="auth-card">
                <div class="auth-header">
                    <h1>"Welcome Back"</h1>
                    <p>"Sign in to your Tatteau account"</p>
                </div>

                <div class="user-type-toggle">
                    <div class="toggle-buttons">
                        <button 
                            class=move || if user_type.get() == "client" { "toggle-btn active" } else { "toggle-btn" }
                            on:click=move |_| user_type.set("client".to_string())
                        >
                            "I'm a Client"
                        </button>
                        <button 
                            class=move || if user_type.get() == "artist" { "toggle-btn active" } else { "toggle-btn" }
                            on:click=move |_| user_type.set("artist".to_string())
                        >
                            "I'm an Artist"
                        </button>
                    </div>
                </div>

                <form on:submit=move |ev| {
                    ev.prevent_default();
                    submit_login(());
                }>
                    <div class="form-group">
                        <Input
                            placeholder="Email"
                            input_type=InputType::Email
                            value=email
                        />
                    </div>

                    <div class="form-group">
                        <Input
                            placeholder="Password"
                            input_type=InputType::Password
                            value=password
                        />
                    </div>

                    {move || error_message.get().map(|msg| view! {
                        <div class="error-message">{msg}</div>
                    })}

                    <Button
                        class="auth-submit-btn"
                        button_type=ButtonType::Submit
                        loading=Signal::from(loading)
                        disabled=Signal::from(is_button_disabled)
                    >
                        "Sign In"
                    </Button>
                </form>

                <div class="auth-footer">
                    <p>
                        "Don't have an account? "
                        <A href="/signup">"Sign up here"</A>
                    </p>
                </div>
            </div>
        </div>
    }
}

#[component]
pub fn SignupPage() -> impl IntoView {
    let user_type = RwSignal::new("client".to_string());
    let first_name = RwSignal::new(String::new());
    let last_name = RwSignal::new(String::new());
    let email = RwSignal::new(String::new());
    let phone = RwSignal::new(String::new());
    let password = RwSignal::new(String::new());
    let confirm_password = RwSignal::new(String::new());
    let loading = RwSignal::new(false);
    let error_message = RwSignal::new(Option::<String>::None);

    let is_button_disabled = Memo::new(move |_| {
        first_name.get().is_empty() || 
        last_name.get().is_empty() || 
        email.get().is_empty() || 
        password.get().is_empty() ||
        confirm_password.get().is_empty()
    });

    let submit_signup = move |_| {
        loading.set(true);
        error_message.set(None);

        // Validate passwords match
        if password.get() != confirm_password.get() {
            error_message.set(Some("Passwords do not match".to_string()));
            loading.set(false);
            return;
        }

        let signup_data = SignupData {
            first_name: first_name.get(),
            last_name: last_name.get(),
            email: email.get(),
            phone: if phone.get().is_empty() { None } else { Some(phone.get()) },
            password: password.get(),
            user_type: user_type.get(),
        };
        
        // TODO: Implement actual signup logic
        spawn_local(async move {
            // Placeholder - will implement server function later
            loading.set(false);
        });
    };

    view! {
        <div class="auth-container">
            <div class="auth-card">
                <div class="auth-header">
                    <h1>"Create Your Account"</h1>
                    <p>"Join the Tatteau community"</p>
                </div>

                <div class="user-type-toggle">
                    <div class="toggle-buttons">
                        <button 
                            class=move || if user_type.get() == "client" { "toggle-btn active" } else { "toggle-btn" }
                            on:click=move |_| user_type.set("client".to_string())
                        >
                            "I'm a Client"
                        </button>
                        <button 
                            class=move || if user_type.get() == "artist" { "toggle-btn active" } else { "toggle-btn" }
                            on:click=move |_| user_type.set("artist".to_string())
                        >
                            "I'm an Artist"
                        </button>
                    </div>
                </div>

                {move || if user_type.get() == "artist" {
                    view! {
                        <div class="artist-notice">
                            <span class="info-icon">"ℹ"</span>
                            <p>"After signing up, you'll choose a subscription tier to access booking features."</p>
                        </div>
                    }.into_any()
                } else {
                    view! {}.into_any()
                }}

                <form on:submit=move |ev| {
                    ev.prevent_default();
                    submit_signup(());
                }>
                    <div class="form-row">
                        <div class="form-group">
                            <Input
                                placeholder="First Name"
                                value=first_name
                            />
                        </div>
                        <div class="form-group">
                            <Input
                                placeholder="Last Name"
                                value=last_name
                            />
                        </div>
                    </div>

                    <div class="form-group">
                        <Input
                            placeholder="Email"
                            input_type=InputType::Email
                            value=email
                        />
                    </div>

                    <div class="form-group">
                        <Input
                            placeholder="Phone (optional)"
                            input_type=InputType::Tel
                            value=phone
                        />
                    </div>

                    <div class="form-group">
                        <Input
                            placeholder="Password"
                            input_type=InputType::Password
                            value=password
                        />
                    </div>

                    <div class="form-group">
                        <Input
                            placeholder="Confirm Password"
                            input_type=InputType::Password
                            value=confirm_password
                        />
                    </div>

                    {move || error_message.get().map(|msg| view! {
                        <div class="error-message">{msg}</div>
                    })}

                    <Button
                        class="auth-submit-btn"
                        button_type=ButtonType::Submit
                        loading=Signal::from(loading)
                        disabled=Signal::from(is_button_disabled)
                    >
                        {move || if user_type.get() == "artist" { 
                            "Create Artist Account" 
                        } else { 
                            "Create Account" 
                        }}
                    </Button>
                </form>

                <div class="auth-footer">
                    <p>
                        "Already have an account? "
                        <A href="/login">"Sign in here"</A>
                    </p>
                </div>
            </div>
        </div>
    }
}