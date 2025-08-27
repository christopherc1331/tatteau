use leptos::prelude::*;
use leptos_router::components::A;
use thaw::*;

#[component]
pub fn ArtistRequests() -> impl IntoView {
    view! {
        <div class="artist-dashboard-container">
            <div class="dashboard-header">
                <A href="/artist/dashboard">
                    <div class="back-button">"← Back to Dashboard"</div>
                </A>
                <h1>"Incoming Requests"</h1>
                <p class="dashboard-subtitle">"Manage booking requests, sketches, and messages"</p>
            </div>

            <div class="requests-tabs">
                <div class="tab-buttons">
                    <button class="tab-button active">"Bookings (5)"</button>
                    <button class="tab-button">"Sketches (3)"</button>
                    <button class="tab-button">"Messages (7)"</button>
                </div>

                <div class="tab-content">
                    <div class="requests-list">
                        <div class="request-item">
                            <div class="request-header">
                                <div class="client-info">
                                    <div class="client-avatar">"J"</div>
                                    <div class="client-details">
                                        <h3>"Jessica Martinez"</h3>
                                        <span class="request-type">"Booking Request"</span>
                                        <span class="request-time">"2 hours ago"</span>
                                    </div>
                                </div>
                                <div class="request-status pending">"Pending"</div>
                            </div>
                            
                            <div class="request-details">
                                <p><strong>"Placement:"</strong> " Left shoulder"</p>
                                <p><strong>"Style:"</strong> " Traditional rose"</p>
                                <p><strong>"Size:"</strong> " 4-5 inches"</p>
                                <p><strong>"Budget:"</strong> " $200-300"</p>
                                <p><strong>"Notes:"</strong> " Looking for a traditional style rose with bold lines. This would be my third tattoo."</p>
                            </div>

                            <div class="request-actions">
                                <button class="btn btn-primary">"Accept"</button>
                                <button class="btn btn-secondary">"Request Changes"</button>
                                <button class="btn btn-outline-danger">"Decline"</button>
                            </div>
                        </div>

                        <div class="request-item">
                            <div class="request-header">
                                <div class="client-info">
                                    <div class="client-avatar">"M"</div>
                                    <div class="client-details">
                                        <h3>"Mike Chen"</h3>
                                        <span class="request-type">"Booking Request"</span>
                                        <span class="request-time">"5 hours ago"</span>
                                    </div>
                                </div>
                                <div class="request-status pending">"Pending"</div>
                            </div>
                            
                            <div class="request-details">
                                <p><strong>"Placement:"</strong> " Right forearm"</p>
                                <p><strong>"Style:"</strong> " Geometric"</p>
                                <p><strong>"Size:"</strong> " 6 inches"</p>
                                <p><strong>"Budget:"</strong> " $400-500"</p>
                                <p><strong>"Notes:"</strong> " Interested in a geometric mandala design. Can provide reference images."</p>
                            </div>

                            <div class="request-actions">
                                <button class="btn btn-primary">"Accept"</button>
                                <button class="btn btn-secondary">"Request Changes"</button>
                                <button class="btn btn-outline-danger">"Decline"</button>
                            </div>
                        </div>

                        <div class="coming-soon-card">
                            <div class="coming-soon-icon">"📋"</div>
                            <h3>"Advanced Request Management"</h3>
                            <p>"Full request management system is in development. Coming features:"</p>
                            <ul class="feature-list">
                                <li>"Interactive request approval workflow"</li>
                                <li>"Integrated messaging system"</li>
                                <li>"Sketch upload and feedback tools"</li>
                                <li>"Automated booking confirmations"</li>
                                <li>"Client communication history"</li>
                            </ul>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    }
}