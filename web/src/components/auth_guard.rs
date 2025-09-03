use leptos::prelude::*;
use leptos_router::hooks::use_navigate;
use leptos::either::Either;

/// Hook to check artist authentication status
pub fn use_artist_auth() -> (Signal<bool>, Signal<bool>) {
    let is_authenticated = RwSignal::new(false);
    let is_loading = RwSignal::new(true);
    
    Effect::new(move |_| {
        #[cfg(feature = "hydrate")]
        {
            use wasm_bindgen::prelude::*;
            
            #[wasm_bindgen]
            extern "C" {
                #[wasm_bindgen(js_namespace = localStorage)]
                fn getItem(key: &str) -> Option<String>;
            }
            
            if let Some(token) = getItem("tatteau_auth_token") {
                is_authenticated.set(!token.is_empty());
            }
            
            is_loading.set(false);
        }
        
        #[cfg(not(feature = "hydrate"))]
        {
            is_authenticated.set(false);
            is_loading.set(false);
        }
    });
    
    (is_authenticated.into(), is_loading.into())
}

#[component]
pub fn LoadingState() -> impl IntoView {
    view! {
        <div style="min-height: 100vh; display: flex; align-items: center; justify-content: center; background: #f8fafc;">
            <div style="text-align: center;">
                <div style="font-size: 1.5rem; color: #667eea; margin-bottom: 0.5rem;">
                    "🔐 Verifying access..."
                </div>
                <div style="font-size: 0.9rem; color: #6b7280;">
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
        <div style="min-height: 100vh; display: flex; align-items: center; justify-content: center; background: #f8fafc;">
            <div style="text-align: center;">
                <div style="font-size: 1.5rem; color: #f59e0b; margin-bottom: 0.5rem;">
                    "🚫 Access Denied"
                </div>
                <div style="font-size: 0.9rem; color: #6b7280;">
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