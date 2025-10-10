use crate::server::{login_user, signup_user};
use leptos::{prelude::*, task::spawn_local};
use leptos_router::{components::A, hooks::{use_query_map, use_navigate}};
use serde::{Deserialize, Serialize};
use thaw::*;

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
    let query_map = use_query_map();
    let navigate = use_navigate();
    
    // Initialize user_type from query parameter or default to "client"  
    let initial_user_type = query_map.get().get("user_type").unwrap_or_else(|| "client".to_string());
    let user_type = RwSignal::new(initial_user_type);
    
    let email = RwSignal::new(String::new());
    let password = RwSignal::new(String::new());
    let loading = RwSignal::new(false);
    let error_message = RwSignal::new(Option::<String>::None);
    let success_message = RwSignal::new(Option::<String>::None);
    
    // Check for success query parameter
    Effect::new({
        let success_message = success_message.clone();
        move |_| {
            let query = query_map.get();
            if query.get("success").as_deref() == Some("signup") {
                success_message.set(Some("Account created successfully! Please log in.".to_string()));
            }
        }
    });
    
    // Update user_type when URL changes
    Effect::new({
        let user_type = user_type.clone();
        move |_| {
            let query = query_map.get();
            if let Some(url_user_type) = query.get("user_type") {
                if url_user_type != user_type.get_untracked() {
                    user_type.set(url_user_type.clone());
                }
            }
        }
    });

    let is_button_disabled =
        Memo::new(move |_| email.get().is_empty() || password.get().is_empty());

    let submit_login = move |_| {
        loading.set(true);
        error_message.set(None);

        let login_data = LoginData {
            email: email.get(),
            password: password.get(),
            user_type: user_type.get(),
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
                        
                        // Redirect to appropriate page
                        if let Some(user_type) = auth_response.user_type {
                            // Check for redirect/return_url parameter first
                            let query = query_map.get();
                            let mut redirect_url = if let Some(redirect) = query.get("redirect") {
                                // Use the redirect URL if provided
                                redirect.clone()
                            } else if let Some(return_url) = query.get("return_url") {
                                // Use the return URL if provided (backwards compatibility)
                                return_url.clone()
                            } else if user_type == "artist" {
                                "/artist/dashboard".to_string()
                            } else {
                                "/explore".to_string() // Client goes to explore page
                            };

                            // Append favorite parameter if present
                            if let Some(favorite) = query.get("favorite") {
                                redirect_url = format!("{}?favorite={}", redirect_url, favorite);
                            }

                            if let Some(window) = web_sys::window() {
                                let _ = window.location().set_href(&redirect_url);
                            }
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
                    <h1>"Welcome Back"</h1>
                    <p>"Sign in to your Tatteau account"</p>
                </div>

                <div class="auth-user-type-toggle">
                    <div class="auth-toggle-buttons">
                        <button
                            class=move || if user_type.get() == "client" { "auth-toggle-btn auth-active" } else { "auth-toggle-btn" }
                            on:click={
                                let navigate = navigate.clone();
                                move |_| {
                                    user_type.set("client".to_string());

                                    // Build query string with user_type and preserve success/redirect/favorite if present
                                    let current_query = query_map.get();
                                    let mut query_parts = vec!["user_type=client".to_string()];

                                    if current_query.get("success").is_some() {
                                        query_parts.push("success=signup".to_string());
                                    }

                                    if let Some(redirect) = current_query.get("redirect") {
                                        query_parts.push(format!("redirect={}", redirect));
                                    }

                                    if let Some(favorite) = current_query.get("favorite") {
                                        query_parts.push(format!("favorite={}", favorite));
                                    }

                                    let query_string = format!("?{}", query_parts.join("&"));
                                    navigate(&format!("/login{}", query_string), Default::default());
                                }
                            }
                        >
                            "I'm a Client"
                        </button>
                        <button
                            class=move || if user_type.get() == "artist" { "auth-toggle-btn auth-active" } else { "auth-toggle-btn" }
                            on:click={
                                let navigate = navigate.clone();
                                move |_| {
                                    user_type.set("artist".to_string());

                                    // Build query string with user_type and preserve success/redirect/favorite if present
                                    let current_query = query_map.get();
                                    let mut query_parts = vec!["user_type=artist".to_string()];

                                    if current_query.get("success").is_some() {
                                        query_parts.push("success=signup".to_string());
                                    }

                                    if let Some(redirect) = current_query.get("redirect") {
                                        query_parts.push(format!("redirect={}", redirect));
                                    }

                                    if let Some(favorite) = current_query.get("favorite") {
                                        query_parts.push(format!("favorite={}", favorite));
                                    }

                                    let query_string = format!("?{}", query_parts.join("&"));
                                    navigate(&format!("/login{}", query_string), Default::default());
                                }
                            }
                        >
                            "I'm an Artist"
                        </button>
                    </div>
                </div>

                {move || {
                    if let Some(msg) = success_message.get() {
                        view! {
                            <div class="auth-success-message">
                                <span class="auth-success-icon">"✓"</span>
                                <p>{msg}</p>
                            </div>
                        }.into_any()
                    } else {
                        view! {}.into_any()
                    }
                }}

                <form on:submit=move |ev| {
                    ev.prevent_default();
                    submit_login(());
                }>
                    <div class="auth-form-group">
                        <Input
                            placeholder="Email"
                            input_type=InputType::Email
                            value=email
                        />
                        <Input
                            placeholder="Password"
                            input_type=InputType::Password
                            value=password
                        />
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
    let query_map = use_query_map();
    let navigate = use_navigate();
    
    // Initialize user_type from query parameter or default to "client"
    let initial_user_type = query_map.get().get("user_type").unwrap_or_else(|| "client".to_string());
    let user_type = RwSignal::new(initial_user_type);
    
    let first_name = RwSignal::new(String::new());
    let last_name = RwSignal::new(String::new());
    let email = RwSignal::new(String::new());
    let phone = RwSignal::new(String::new());
    let password = RwSignal::new(String::new());
    let confirm_password = RwSignal::new(String::new());
    let loading = RwSignal::new(false);
    let error_message = RwSignal::new(Option::<String>::None);

    let is_button_disabled = Memo::new(move |_| {
        first_name.get().is_empty()
            || last_name.get().is_empty()
            || email.get().is_empty()
            || password.get().is_empty()
            || confirm_password.get().is_empty()
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
            phone: if phone.get().is_empty() {
                None
            } else {
                Some(phone.get())
            },
            password: password.get(),
            user_type: user_type.get(),
        };

        spawn_local(async move {
            match signup_user(signup_data).await {
                Ok(auth_response) => {
                    if auth_response.success {
                        // Redirect to login page with success message
                        if let Some(user_type) = auth_response.user_type {
                            let login_url = if user_type == "artist" {
                                "/login?success=signup&user_type=artist"
                            } else {
                                "/login?success=signup&user_type=client"
                            };

                            if let Some(window) = web_sys::window() {
                                let _ = window.location().set_href(login_url);
                            }
                        }
                    } else {
                        error_message.set(auth_response.error);
                    }
                }
                Err(e) => {
                    error_message.set(Some(format!("Signup failed: {}", e)));
                }
            }
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

                <div class="auth-user-type-toggle">
                    <div class="auth-toggle-buttons">
                        <button
                            class=move || if user_type.get() == "client" { "auth-toggle-btn auth-active" } else { "auth-toggle-btn" }
                            on:click={
                                let navigate = navigate.clone();
                                move |_| {
                                    user_type.set("client".to_string());
                                    
                                    // Build query string with user_type and preserve success if present
                                    let current_query = query_map.get();
                                    let mut query_parts = vec!["user_type=client"];
                                    
                                    if current_query.get("success").is_some() {
                                        query_parts.push("success=signup");
                                    }
                                    
                                    let query_string = format!("?{}", query_parts.join("&"));
                                    navigate(&format!("/signup{}", query_string), Default::default());
                                }
                            }
                        >
                            "I'm a Client"
                        </button>
                        <button
                            class=move || if user_type.get() == "artist" { "auth-toggle-btn auth-active" } else { "auth-toggle-btn" }
                            on:click={
                                let navigate = navigate.clone();
                                move |_| {
                                    user_type.set("artist".to_string());
                                    
                                    // Build query string with user_type and preserve success if present
                                    let current_query = query_map.get();
                                    let mut query_parts = vec!["user_type=artist"];
                                    
                                    if current_query.get("success").is_some() {
                                        query_parts.push("success=signup");
                                    }
                                    
                                    let query_string = format!("?{}", query_parts.join("&"));
                                    navigate(&format!("/signup{}", query_string), Default::default());
                                }
                            }
                        >
                            "I'm an Artist"
                        </button>
                    </div>
                </div>

                {move || if user_type.get() == "artist" {
                    view! {
                        <div class="auth-artist-notice">
                            <span class="auth-info-icon">"ℹ"</span>
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
                    <div class="auth-form-row">
                        <div class="auth-form-group">
                            <Input
                                placeholder="First Name"
                                value=first_name
                            />
                        </div>
                        <div class="auth-form-group">
                            <Input
                                placeholder="Last Name"
                                value=last_name
                            />
                        </div>
                    </div>

                    <div class="auth-form-group">
                        <Input
                            placeholder="Email"
                            input_type=InputType::Email
                            value=email
                        />
                    </div>

                    <div class="auth-form-group">
                        <Input
                            placeholder="Phone (optional)"
                            input_type=InputType::Tel
                            value=phone
                        />
                    </div>

                    <div class="auth-form-group">
                        <Input
                            placeholder="Password"
                            input_type=InputType::Password
                            value=password
                        />
                    </div>

                    <div class="auth-form-group">
                        <Input
                            placeholder="Confirm Password"
                            input_type=InputType::Password
                            value=confirm_password
                        />
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

