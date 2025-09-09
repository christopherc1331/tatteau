use leptos::prelude::*;
use leptos_router::hooks::use_navigate;
use crate::utils::auth::get_authenticated_artist_id;

/// Hook to check artist authentication status using proper JWT validation
pub fn use_artist_auth() -> (Signal<bool>, Signal<bool>) {
    let is_authenticated = RwSignal::new(false);
    let is_loading = RwSignal::new(true);
    
    Effect::new(move |_| {
        // Use the proper authentication function that validates JWT and checks user_type
        let artist_id = get_authenticated_artist_id();
        is_authenticated.set(artist_id.is_some());
        is_loading.set(false);
    });
    
    (is_authenticated.into(), is_loading.into())
}

#[component]
pub fn LoadingState() -> impl IntoView {
    view! {
        <div class="auth-guard-container">
            <div class="auth-guard-content">
                <div class="auth-guard-loading-title">
                    "🔐 Verifying access..."
                </div>
                <div class="auth-guard-loading-subtitle">
                    "Please wait while we check your credentials"
                </div>
            </div>
        </div>
    }
}

#[component]
pub fn AccessDeniedState() -> impl IntoView {
    let navigate = use_navigate();
    
    Effect::new(move |_| {
        navigate("/artist-login-required", Default::default());
    });
    
    view! {
        <div class="auth-guard-container">
            <div class="auth-guard-content">
                <div class="auth-guard-denied-title">
                    "🚫 Access Denied"
                </div>
                <div class="auth-guard-denied-subtitle">
                    "Redirecting to login..."
                </div>
            </div>
        </div>
    }
}

/// Authentication guard component using proper ChildrenFn pattern
#[component] 
pub fn ArtistAuthGuard(children: ChildrenFn) -> impl IntoView {
    let (is_authenticated, is_loading) = use_artist_auth();
    
    view! {
        <Show
            when=move || !is_loading.get()
            fallback=move || view! { <LoadingState/> }
        >
            <Show
                when=move || is_authenticated.get()
                fallback=move || view! { <AccessDeniedState/> }
                clone:children
            >
                {children()}
            </Show>
        </Show>
    }
}