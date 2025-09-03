use leptos::prelude::*;
use leptos_router::components::A;

use crate::{
    components::loading::LoadingView,
    server::get_artist_dashboard_data,
};

#[component]
pub fn ArtistHome() -> impl IntoView {
    // In a real app, this would come from authentication context
    let artist_id = RwSignal::new(1i32); // Placeholder - would come from auth

    let dashboard_data = Resource::new(
        move || artist_id.get(),
        move |id| async move {
            get_artist_dashboard_data(id).await
        }
    );

    view! {
        <div class="artist-dashboard-container">
            <div class="dashboard-header">
                <h1>"Artist Dashboard"</h1>
                <p class="dashboard-subtitle">"Welcome back! Here's what's happening with your bookings."</p>
            </div>

            <Suspense fallback=move || view! { 
                <LoadingView message=Some("Loading your dashboard...".to_string()) /> 
            }>
                {move || {
                    match dashboard_data.get() {
                        Some(Ok(data)) => {
                            view! {
                                <div class="dashboard-grid">
                                    <DashboardTile
                                        title="Today's Bookings".to_string()
                                        value=data.todays_bookings.to_string()
                                        subtitle="appointments scheduled".to_string()
                                        color="purple".to_string()
                                        icon="📅".to_string()
                                        link="/artist/dashboard/calendar".to_string()
                                    />
                                    
                                    <DashboardTile
                                        title="Sketch Requests".to_string()
                                        value=data.pending_sketches.to_string()
                                        subtitle="awaiting your response".to_string()
                                        color="orange".to_string()
                                        icon="✏️".to_string()
                                        link="/artist/dashboard/requests".to_string()
                                    />
                                    
                                    <DashboardTile
                                        title="Unread Messages".to_string()
                                        value=data.unread_messages.to_string()
                                        subtitle="from clients".to_string()
                                        color="blue".to_string()
                                        icon="💬".to_string()
                                        link="/artist/dashboard/requests".to_string()
                                    />
                                    
                                    <DashboardTile
                                        title="This Month".to_string()
                                        value=format!("${}", data.monthly_revenue)
                                        subtitle="total revenue".to_string()
                                        color="green".to_string()
                                        icon="💰".to_string()
                                        link="/artist/dashboard/calendar".to_string()
                                    />
                                </div>

                                <div class="recent-activity">
                                    <h2>"Recent Activity"</h2>
                                    <div class="activity-list">
                                        {data.recent_bookings.clone().into_iter().map(|booking| {
                                            view! {
                                                <div class="activity-item">
                                                    <div class="activity-icon">"📅"</div>
                                                    <div class="activity-content">
                                                        <div class="activity-title">
                                                            {format!("New booking from {}", booking.client_name.unwrap_or_else(|| "Unknown Client".to_string()))}
                                                        </div>
                                                        <div class="activity-subtitle">
                                                            {booking.placement.unwrap_or_else(|| "No placement specified".to_string())}
                                                        </div>
                                                        <div class="activity-time">
                                                            {booking.created_at.clone()}
                                                        </div>
                                                    </div>
                                                </div>
                                            }
                                        }).collect_view()}
                                        
                                        {if data.recent_bookings.is_empty() {
                                            view! {
                                                <div class="empty-state">
                                                    <p>"No recent activity to show."</p>
                                                </div>
                                            }.into_any()
                                        } else {
                                            view! { <div></div> }.into_any()
                                        }}
                                    </div>
                                </div>

                                <div class="quick-actions">
                                    <h2>"Quick Actions"</h2>
                                    <div class="actions-grid">
                                        <A href="/artist/dashboard/calendar">
                                            <div class="action-button">
                                                <div class="action-icon">"📅"</div>
                                                <div class="action-text">"View Calendar"</div>
                                            </div>
                                        </A>
                                        
                                        <A href="/artist/dashboard/requests">
                                            <div class="action-button">
                                                <div class="action-icon">"📋"</div>
                                                <div class="action-text">"Manage Requests"</div>
                                            </div>
                                        </A>
                                        
                                        <A href="/artist/dashboard/settings">
                                            <div class="action-button">
                                                <div class="action-icon">"⚙️"</div>
                                                <div class="action-text">"Settings"</div>
                                            </div>
                                        </A>
                                    </div>
                                </div>
                            }.into_any()
                        },
                        Some(Err(_)) => view! {
                            <div class="error-state">
                                <h3>"Unable to load dashboard"</h3>
                                <p>"Please try refreshing the page or contact support if the problem persists."</p>
                            </div>
                        }.into_any(),
                        None => view! {
                            <LoadingView message=Some("Loading dashboard...".to_string()) />
                        }.into_any(),
                    }
                }}
            </Suspense>
        </div>
    }
}

#[component]
fn DashboardTile(
    title: String,
    value: String,
    subtitle: String,
    color: String,
    icon: String,
    link: String,
) -> impl IntoView {
    view! {
        <A href=link>
            <div class={format!("dashboard-tile tile-{}", color)}>
                <div class="tile-header">
                    <div class="tile-icon">{icon}</div>
                    <div class="tile-title">{title}</div>
                </div>
                <div class="tile-value">{value}</div>
                <div class="tile-subtitle">{subtitle}</div>
            </div>
        </A>
    }
}