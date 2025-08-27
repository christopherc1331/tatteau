use leptos::prelude::*;
use leptos_router::components::A;
use thaw::*;

#[component]
pub fn ArtistSettings() -> impl IntoView {
    let auto_reply = RwSignal::new(true);
    let availability = RwSignal::new(true);
    let base_price = RwSignal::new("150.0".to_string());
    let hourly_rate = RwSignal::new("200.0".to_string());

    view! {
        <div class="artist-dashboard-container">
            <div class="dashboard-header">
                <A href="/artist/dashboard">
                    <div class="back-button">"← Back to Dashboard"</div>
                </A>
                <h1>"Artist Settings"</h1>
                <p class="dashboard-subtitle">"Configure your preferences and pricing"</p>
            </div>

            <div class="settings-grid">
                <div class="settings-card">
                    <h2>"Availability Settings"</h2>
                    
                    <div class="setting-group">
                        <label class="setting-label">
                            <Switch
                                checked=availability
                            />
                            <span>"Accept New Bookings"</span>
                        </label>
                        <p class="setting-description">"Turn off to stop receiving new booking requests"</p>
                    </div>

                    <div class="setting-group">
                        <label class="setting-label">
                            <Switch
                                checked=auto_reply
                            />
                            <span>"Auto-Reply to Messages"</span>
                        </label>
                        <p class="setting-description">"Automatically send a response to new client messages"</p>
                    </div>
                </div>

                <div class="settings-card">
                    <h2>"Pricing Configuration"</h2>
                    
                    <div class="setting-group">
                        <label class="setting-label">"Base Price (Small Tattoos)"</label>
                        <div class="price-input">
                            <span class="currency">"$"</span>
                            <Input
                                value=base_price
                                placeholder="150"
                            />
                        </div>
                        <p class="setting-description">"Starting price for small tattoos (2-3 inches)"</p>
                    </div>

                    <div class="setting-group">
                        <label class="setting-label">"Hourly Rate"</label>
                        <div class="price-input">
                            <span class="currency">"$"</span>
                            <Input
                                value=hourly_rate
                                placeholder="200"
                            />
                        </div>
                        <p class="setting-description">"Rate per hour for larger, custom pieces"</p>
                    </div>

                    <div class="setting-actions">
                        <button class="btn btn-primary">"Save Pricing"</button>
                    </div>
                </div>

                <div class="settings-card">
                    <h2>"Business Hours"</h2>
                    
                    <div class="hours-grid">
                        <div class="day-setting">
                            <span class="day-label">"Monday"</span>
                            <div class="time-inputs">
                                <Input placeholder="9:00 AM" />
                                <span>"-"</span>
                                <Input placeholder="6:00 PM" />
                            </div>
                        </div>
                        
                        <div class="day-setting">
                            <span class="day-label">"Tuesday"</span>
                            <div class="time-inputs">
                                <Input placeholder="9:00 AM" />
                                <span>"-"</span>
                                <Input placeholder="6:00 PM" />
                            </div>
                        </div>
                        
                        <div class="day-setting">
                            <span class="day-label">"Wednesday"</span>
                            <div class="time-inputs">
                                <Input placeholder="9:00 AM" />
                                <span>"-"</span>
                                <Input placeholder="6:00 PM" />
                            </div>
                        </div>
                        
                        <div class="day-setting">
                            <span class="day-label">"Thursday"</span>
                            <div class="time-inputs">
                                <Input placeholder="9:00 AM" />
                                <span>"-"</span>
                                <Input placeholder="6:00 PM" />
                            </div>
                        </div>
                        
                        <div class="day-setting">
                            <span class="day-label">"Friday"</span>
                            <div class="time-inputs">
                                <Input placeholder="9:00 AM" />
                                <span>"-"</span>
                                <Input placeholder="6:00 PM" />
                            </div>
                        </div>
                        
                        <div class="day-setting">
                            <span class="day-label">"Saturday"</span>
                            <div class="time-inputs">
                                <Input placeholder="Closed" />
                                <span>"-"</span>
                                <Input placeholder="Closed" />
                            </div>
                        </div>
                        
                        <div class="day-setting">
                            <span class="day-label">"Sunday"</span>
                            <div class="time-inputs">
                                <Input placeholder="Closed" />
                                <span>"-"</span>
                                <Input placeholder="Closed" />
                            </div>
                        </div>
                    </div>

                    <div class="setting-actions">
                        <button class="btn btn-primary">"Save Hours"</button>
                    </div>
                </div>

                <div class="settings-card">
                    <h2>"Profile Settings"</h2>
                    
                    <div class="coming-soon-card">
                        <div class="coming-soon-icon">"⚙️"</div>
                        <h3>"Advanced Settings Coming Soon"</h3>
                        <p>"Additional configuration options in development:"</p>
                        <ul class="feature-list">
                            <li>"Portfolio management"</li>
                            <li>"Style specialization tags"</li>
                            <li>"Notification preferences"</li>
                            <li>"Payment method setup"</li>
                            <li>"Social media integration"</li>
                        </ul>
                    </div>
                </div>
            </div>
        </div>
    }
}