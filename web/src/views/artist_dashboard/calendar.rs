use leptos::prelude::*;
use leptos_router::components::A;
use thaw::*;

#[component]
pub fn ArtistCalendar() -> impl IntoView {
    view! {
        <div class="artist-dashboard-container">
            <div class="dashboard-header">
                <A href="/artist/dashboard">
                    <div class="back-button">"← Back to Dashboard"</div>
                </A>
                <h1>"Booking Calendar"</h1>
                <p class="dashboard-subtitle">"Manage your appointments and availability"</p>
            </div>

            <div class="calendar-container">
                <div class="calendar-header">
                    <h2>"December 2024"</h2>
                    <div class="calendar-controls">
                        <button class="btn-outlined">"← Previous"</button>
                        <button class="btn-outlined">"Today"</button>
                        <button class="btn-outlined">"Next →"</button>
                    </div>
                </div>

                <div class="calendar-placeholder">
                    <div class="coming-soon-card">
                        <div class="coming-soon-icon">"📅"</div>
                        <h3>"Calendar Coming Soon"</h3>
                        <p>"The interactive booking calendar is currently being developed. You'll be able to:"</p>
                        <ul class="feature-list">
                            <li>"View all your appointments"</li>
                            <li>"Drag and drop to reschedule"</li>
                            <li>"Set availability windows"</li>
                            <li>"Block out time for breaks"</li>
                            <li>"Sync with external calendars"</li>
                        </ul>
                        <div class="placeholder-actions">
                            <A href="/artist/dashboard/requests">
                                <div class="btn btn-primary">"View Pending Requests"</div>
                            </A>
                            <A href="/artist/dashboard/settings">
                                <div class="btn btn-secondary">"Update Availability"</div>
                            </A>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    }
}