use super::{BookingModal, BookingSidebar, CalendarGrid};
use crate::db::entities::{AvailabilitySlot, BookingRequest};
use crate::server::{get_artist_availability, get_booking_requests};
use leptos::prelude::*;
use thaw::*;

#[component]
pub fn ArtistCalendar() -> impl IntoView {
    let artist_id = RwSignal::new(-1); // Frank Reynolds for testing
    let selected_date = RwSignal::new(String::new());
    let show_booking_modal = RwSignal::new(false);
    let selected_booking = RwSignal::new(None::<BookingRequest>);
    let sidebar_collapsed = RwSignal::new(false);

    // Get current month for availability query
    let current_date = js_sys::Date::new_0();
    let year = current_date.get_full_year();
    let month = current_date.get_month() + 1.0; // JS months are 0-indexed
    let start_date = format!("{:04}-{:02}-01", year, month as u32);
    let end_date = format!(
        "{:04}-{:02}-{:02}",
        year,
        month as u32,
        js_sys::Date::new_with_year_month_day(year, month, 0.0).get_date()
    );

    // Load availability data
    let availability_resource = Resource::new(
        move || (artist_id.get(), start_date.clone(), end_date.clone()),
        |(artist_id, start_date, end_date)| async move {
            get_artist_availability(artist_id, start_date, end_date).await
        },
    );

    // Load booking requests
    let booking_requests_resource = Resource::new(
        move || artist_id.get(),
        |artist_id| async move { get_booking_requests(artist_id).await },
    );

    let on_booking_select = move |booking: BookingRequest| {
        selected_booking.set(Some(booking));
        show_booking_modal.set(true);
    };

    let on_date_select = move |date: String| {
        selected_date.set(date);
    };

    let toggle_sidebar = move |_| {
        sidebar_collapsed.update(|collapsed| *collapsed = !*collapsed);
    };

    view! {
        <div class="artist-calendar-container">
            <div class="artist-calendar-header">
                <h1>"Artist Calendar - Frank Reynolds"</h1>
                <div class="artist-calendar-header-actions">
                    <Button
                        appearance=ButtonAppearance::Primary
                        on_click=toggle_sidebar
                    >
                        {move || if sidebar_collapsed.get() { "Show Requests" } else { "Hide Requests" }}
                    </Button>
                </div>
            </div>

            <div class="artist-calendar-layout">
                <div class="artist-calendar-main">
                    <CalendarGrid
                        availability=availability_resource
                        booking_requests=booking_requests_resource
                        on_date_select=on_date_select
                        selected_date=selected_date
                    />
                </div>

                <div class="artist-calendar-sidebar" class:artist-calendar-sidebar--collapsed=move || sidebar_collapsed.get()>
                    <BookingSidebar
                        booking_requests=booking_requests_resource
                        on_booking_select=on_booking_select
                    />
                </div>
            </div>

            <BookingModal
                show=show_booking_modal
                booking=selected_booking
                on_close=move || show_booking_modal.set(false)
            />
        </div>
    }
}

