use leptos::prelude::*;
use leptos_meta::{provide_meta_context, MetaTags, Stylesheet, Title};
use leptos_router::{
    components::{Route, Router, Routes},
    ParamSegment, StaticSegment,
};
use thaw::ssr::SSRMountStyleProvider;
use thaw::*;

use crate::components::{masonry_gallery::MasonryGallery, ArtistAuthGuard, ErrorBoundary, Navbar};
use crate::views::admin_login::AdminLoginPage;
use crate::views::artist_dashboard::{
    ArtistCalendar, ArtistHome, ArtistRecurring, ArtistRequests, ArtistSettings, BookingDetails,
    QuestionnaireBuilder,
};
use crate::views::artist_highlight::ArtistHighlight;
use crate::views::artist_login_prompt::ArtistLoginPrompt;
use crate::views::auth::{LoginPage, SignupPage};
use crate::views::booking::{ArtistBooking, ShopBooking};
use crate::views::booking_confirmation::BookingConfirmation;
use crate::views::favorites::FavoritesPage;
use crate::views::home::HomePage;
use crate::views::map::map_wrapper::DiscoveryMap;
use crate::views::match_results::MatchResults;
use crate::views::not_found::NotFoundPage;
use crate::views::quiz::GetMatchedQuiz;
use crate::views::shop::Shop;
use crate::views::styles::StylesShowcase;
use crate::views::subscription_tiers::SubscriptionTiersPage;

pub fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
        <SSRMountStyleProvider>
            <!DOCTYPE html>
            <html lang="en">
                <head>
                    <meta charset="utf-8"/>
                    <meta name="viewport" content="width=device-width, initial-scale=1"/>
                    <AutoReload options=options.clone() />
                    <HydrationScripts options/>
                    <MetaTags/>
                </head>
                <link
                    rel="stylesheet"
                    href="https://unpkg.com/leaflet@1.9.3/dist/leaflet.css"
                />
                <script
                    src="https://unpkg.com/leaflet@1.9.3/dist/leaflet.js"
                    defer
                ></script>
                // Add Instagram embed script
                <script async defer src="//www.instagram.com/embed.js"></script>
                <body>
                    <App/>
                </body>
            </html>
        </SSRMountStyleProvider>
    }
}

#[component]
pub fn App() -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context();

    view! {
        // injects a stylesheet into the document <head>
        // id=leptos means cargo-leptos will hot-reload this stylesheet
        <Stylesheet id="leptos" href="/pkg/web.css"/>

        // sets the document title
        <Title text="tatteau"/>

        // content for this welcome page
        <ConfigProvider>
            <ErrorBoundary>
                <Router>
                    <Navbar />
                    <main>
                    <Routes fallback=|| view! { <NotFoundPage /> }.into_view()>
                        // Public routes
                        <Route path=StaticSegment("") view=HomePage/>
                        <Route path=StaticSegment("login") view=LoginPage/>
                        <Route path=StaticSegment("signup") view=SignupPage/>
                        <Route path=(StaticSegment("admin"), StaticSegment("login")) view=AdminLoginPage/>
                        <Route path=StaticSegment("explore") view=ExplorePage/>
                        <Route path=StaticSegment("favorites") view=FavoritesPage/>
                        // <Route path=StaticSegment("artist-login-required") view=ArtistLoginPrompt/>
                        // <Route path=(StaticSegment("subscription"), StaticSegment("tiers")) view=SubscriptionTiersPage/>
                        <Route path=StaticSegment("match") view=GetMatchedQuiz/>
                        <Route path=(StaticSegment("match"), StaticSegment("results")) view=MatchResults/>
                        // <Route path=StaticSegment("styles") view=StylesPage/>
                        // <Route path=StaticSegment("gallery") view=GalleryPage/>

                        // Protected artist dashboard routes (authentication required) - MUST come before artist profile routes
                        // <Route path=(StaticSegment("artist"), StaticSegment("dashboard")) view=ProtectedArtistHome/>
                        // <Route path=(StaticSegment("artist"), StaticSegment("dashboard"), StaticSegment("calendar")) view=ProtectedArtistCalendar/>
                        // <Route path=(StaticSegment("artist"), StaticSegment("dashboard"), StaticSegment("requests")) view=ProtectedArtistRequests/>
                        // <Route path=(StaticSegment("artist"), StaticSegment("dashboard"), StaticSegment("settings")) view=ProtectedArtistSettings/>
                        // <Route path=(StaticSegment("artist"), StaticSegment("dashboard"), StaticSegment("recurring")) view=ProtectedArtistRecurring/>
                        // <Route path=(StaticSegment("artist"), StaticSegment("dashboard"), StaticSegment("questionnaire")) view=ProtectedQuestionnaireBuilder/>
                        // <Route path=(StaticSegment("artist"), StaticSegment("dashboard"), StaticSegment("booking"), ParamSegment("id")) view=ProtectedBookingDetailsPage/>

                        // Public artist profile pages (no authentication required)
                        <Route path=(StaticSegment("artist"), ParamSegment("id")) view=ArtistHighlight/>
                        <Route path=(StaticSegment("shop"), ParamSegment("id")) view=Shop/>
                        // <Route path=(StaticSegment("book"), StaticSegment("artist"), ParamSegment("id")) view=ArtistBooking/>
                        // <Route path=(StaticSegment("book"), StaticSegment("shop"), ParamSegment("id")) view=ShopBooking/>
                        // <Route path=(StaticSegment("booking"), StaticSegment("confirmation")) view=BookingConfirmation/>
                    </Routes>
                </main>
            </Router>
            </ErrorBoundary>
        </ConfigProvider>
    }
}

/// Renders the explore page with map discovery
#[component]
fn ExplorePage() -> impl IntoView {
    view! {
        <DiscoveryMap/>
    }
}

/// Renders the styles page
#[component]
fn StylesPage() -> impl IntoView {
    view! {
        <StylesShowcase />
    }
}

/// Renders the gallery page of your application.
#[component]
fn GalleryPage() -> impl IntoView {
    view! {
        <MasonryGallery/>
    }
}

/// Renders the booking details page
#[component]
fn BookingDetailsPage() -> impl IntoView {
    let params = leptos_router::hooks::use_params_map();
    let booking_id = move || {
        params
            .get()
            .get("id")
            .and_then(|id| id.parse::<i32>().ok())
            .unwrap_or(0)
    };

    // Get the booking_id once at component creation time
    let id = booking_id();

    view! {
        <BookingDetails booking_id=id />
    }
}

// Protected artist dashboard components wrapped with authentication guards

#[component]
fn ProtectedArtistHome() -> impl IntoView {
    view! {
        <ArtistAuthGuard>
            <ArtistHome />
        </ArtistAuthGuard>
    }
}

#[component]
fn ProtectedArtistCalendar() -> impl IntoView {
    view! {
        <ArtistAuthGuard>
            <ArtistCalendar />
        </ArtistAuthGuard>
    }
}

#[component]
fn ProtectedArtistRequests() -> impl IntoView {
    view! {
        <ArtistAuthGuard>
            <ArtistRequests />
        </ArtistAuthGuard>
    }
}

#[component]
fn ProtectedArtistSettings() -> impl IntoView {
    view! {
        <ArtistAuthGuard>
            <ArtistSettings />
        </ArtistAuthGuard>
    }
}

#[component]
fn ProtectedArtistRecurring() -> impl IntoView {
    view! {
        <ArtistAuthGuard>
            <ArtistRecurring />
        </ArtistAuthGuard>
    }
}

#[component]
fn ProtectedQuestionnaireBuilder() -> impl IntoView {
    view! {
        <ArtistAuthGuard>
            <QuestionnaireBuilder />
        </ArtistAuthGuard>
    }
}

#[component]
fn ProtectedBookingDetailsPage() -> impl IntoView {
    let params = leptos_router::hooks::use_params_map();
    let booking_id = move || {
        params
            .get()
            .get("id")
            .and_then(|id| id.parse::<i32>().ok())
            .unwrap_or(0)
    };

    let id = booking_id();

    view! {
        <ArtistAuthGuard>
            <BookingDetails booking_id=id />
        </ArtistAuthGuard>
    }
}
