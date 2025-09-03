use crate::server::{create_artist_subscription, get_subscription_tiers};
use leptos::{prelude::*, task::spawn_local};
use leptos_router::components::A;
use serde::{Deserialize, Serialize};
use thaw::*;

fn snake_case_to_title_case(snake_str: &str) -> String {
    snake_str
        .split('_')
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().collect::<String>() + &chars.as_str().to_lowercase(),
            }
        })
        .collect::<Vec<String>>()
        .join(" ")
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TierSelection {
    pub tier_id: i32,
    pub artist_id: i32,
}

#[component]
pub fn SubscriptionTiersPage() -> impl IntoView {
    let selected_tier = RwSignal::new(Option::<i32>::None);
    let loading = RwSignal::new(false);
    let error_message = RwSignal::new(Option::<String>::None);
    let success_message = RwSignal::new(Option::<String>::None);
    
    // Mock artist ID for now - this would come from authentication context
    let artist_id = 1; // TODO: Get from authentication context

    let tiers_resource = Resource::new(
        move || {},
        |_| async move {
            match get_subscription_tiers().await {
                Ok(tiers) => tiers,
                Err(_) => vec![],
            }
        },
    );

    let select_tier = move |tier_id: i32| {
        selected_tier.set(Some(tier_id));
    };

    let subscribe_action = move |_| {
        if let Some(tier_id) = selected_tier.get() {
            loading.set(true);
            error_message.set(None);

            spawn_local(async move {
                match create_artist_subscription(
                    artist_id,
                    tier_id,
                    "active".to_string(),
                    Some("mock_payment".to_string()),
                ).await {
                    Ok(_) => {
                        success_message.set(Some("Subscription created successfully! Redirecting to artist dashboard...".to_string()));
                        
                        // Redirect to artist dashboard immediately for now
                        // TODO: Add proper delay redirect later
                        if let Some(window) = web_sys::window() {
                            let _ = window.location().set_href("/artist/dashboard");
                        }
                    }
                    Err(e) => {
                        error_message.set(Some(format!("Subscription failed: {}", e)));
                    }
                }
                loading.set(false);
            });
        }
    };

    view! {
        <div class="subscription-container">
            <div class="subscription-header">
                <h1>"Choose Your Artist Plan"</h1>
                <p>"Select the perfect subscription tier to unlock the full potential of your tattoo business"</p>
            </div>

            <Suspense fallback=move || {
                view! {
                    <div class="loading-container">
                        <p>"Loading subscription plans..."</p>
                    </div>
                }
            }>
                {move || {
                    tiers_resource.get().map(|tiers| {
                        view! {
                            <div class="tiers-grid">
                                {tiers.into_iter().map(|tier| {
                                    let tier_id = tier.id;
                                    let is_selected = move || selected_tier.get() == Some(tier_id);
                                    let features: Vec<String> = if let Some(features_json) = &tier.features_json {
                                        serde_json::from_str(features_json).unwrap_or_else(|_| vec![])
                                    } else {
                                        vec![]
                                    };

                                    view! {
                                        <div class=move || if is_selected() { "tier-card selected" } else { "tier-card" }>
                                            <div class="tier-header">
                                                <h3>{tier.tier_name.clone()}</h3>
                                                <div class="tier-price">
                                                    <span class="price">"$"{tier.price_monthly}</span>
                                                    <span class="period">"/month"</span>
                                                </div>
                                            </div>
                                            
                                            <div class="tier-features">
                                                <h4>"Features Included:"</h4>
                                                <ul>
                                                    {features.into_iter().map(|feature| {
                                                        let display_name = match feature.as_str() {
                                                            "basic_profile" => "Claim Profile".to_string(),
                                                            "booking_feature" => "Booking Management".to_string(),
                                                            _ => snake_case_to_title_case(&feature),
                                                        };
                                                        view! {
                                                            <li>{display_name}</li>
                                                        }
                                                    }).collect::<Vec<_>>()}
                                                </ul>
                                            </div>

                                            <Button
                                                class="tier-select-btn"
                                                on_click=move |_| select_tier(tier_id)
                                            >
                                                {move || if is_selected() { "Selected" } else { "Select Plan" }}
                                            </Button>
                                        </div>
                                    }
                                }).collect::<Vec<_>>()}
                            </div>
                        }
                    })
                }}
            </Suspense>

            <Show when=move || selected_tier.get().is_some()>
                <div class="subscription-actions">
                    <div class="payment-notice">
                        <p>"Payment processing is not yet implemented. This is a placeholder for the subscription flow."</p>
                    </div>
                    
                    <div class="action-buttons">
                        <Button
                            loading=Signal::from(loading)
                            disabled=Signal::from(loading)
                            on_click=move |_| subscribe_action(())
                        >
                            "Subscribe Now"
                        </Button>
                        
                        <A href="/signup">
                            <Button>
                                "Back to Signup"
                            </Button>
                        </A>
                    </div>
                </div>
            </Show>

            <Show when=move || error_message.get().is_some()>
                <div class="error-message">
                    {move || error_message.get()}
                </div>
            </Show>

            <Show when=move || success_message.get().is_some()>
                <div class="success-message">
                    {move || success_message.get()}
                </div>
            </Show>
        </div>
    }
}