use leptos::prelude::*;
use leptos_router::hooks::use_navigate;
use thaw::*;

#[component]
pub fn AdminDashboard() -> impl IntoView {
    let navigate = use_navigate();

    view! {
        <div class="admin-dashboard">
            <div class="admin-dashboard-header">
                <h1>"Admin Dashboard"</h1>
                <p>"Manage and validate platform content"</p>
            </div>

            <div class="admin-dashboard-grid">
                <div
                    class="admin-card"
                    on:click={
                        let navigate = navigate.clone();
                        move |_| {
                            navigate("/admin/validate-posts", Default::default());
                        }
                    }
                >
                    <div class="admin-card-icon">
                        <svg xmlns="http://www.w3.org/2000/svg" width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                            <rect x="3" y="3" width="18" height="18" rx="2" ry="2"></rect>
                            <circle cx="8.5" cy="8.5" r="1.5"></circle>
                            <polyline points="21 15 16 10 5 21"></polyline>
                        </svg>
                    </div>
                    <h2>"Validate Post Tags"</h2>
                    <p>"Review and validate style tags for posts"</p>
                </div>

                <div
                    class="admin-card"
                    on:click={
                        let navigate = navigate.clone();
                        move |_| {
                            navigate("/admin/validate-artists", Default::default());
                        }
                    }
                >
                    <div class="admin-card-icon">
                        <svg xmlns="http://www.w3.org/2000/svg" width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                            <path d="M16 21v-2a4 4 0 0 0-4-4H5a4 4 0 0 0-4 4v2"></path>
                            <circle cx="8.5" cy="7" r="4"></circle>
                            <polyline points="17 11 19 13 23 9"></polyline>
                        </svg>
                    </div>
                    <h2>"Validate Artist Shops"</h2>
                    <p>"Verify artist shop assignments"</p>
                </div>
            </div>
        </div>
    }
}
