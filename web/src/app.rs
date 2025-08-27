use leptos::prelude::*;
use leptos_meta::{provide_meta_context, MetaTags, Stylesheet, Title};
use leptos_router::{
    components::{Route, Router, Routes},
    ParamSegment, StaticSegment,
};
use thaw::ssr::SSRMountStyleProvider;
use thaw::*;

use crate::components::masonry_gallery::MasonryGallery;
use crate::views::artist_dashboard::{ArtistHome, ArtistCalendar, ArtistRequests, ArtistSettings};
use crate::views::artist_highlight::ArtistHighlight;
use crate::views::booking::{ArtistBooking, ShopBooking};
use crate::views::home::HomePage;
use crate::views::map::map_wrapper::DiscoveryMap;
use crate::views::match_results::MatchResults;
use crate::views::quiz::GetMatchedQuiz;
use crate::views::shop::Shop;
use crate::views::styles::StylesShowcase;

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
            <Router>
                <main>
                    <Routes fallback=|| "Page not found.".into_view()>
                        <Route path=StaticSegment("") view=HomePage/>
                        <Route path=StaticSegment("explore") view=ExplorePage/>
                        <Route path=StaticSegment("match") view=GetMatchedQuiz/>
                        <Route path=(StaticSegment("match"), StaticSegment("results")) view=MatchResults/>
                        <Route path=StaticSegment("styles") view=StylesPage/>
                        <Route path=StaticSegment("gallery") view=GalleryPage/>
                        <Route path=(StaticSegment("artist"), StaticSegment("dashboard")) view=ArtistHome/>
                        <Route path=(StaticSegment("artist"), StaticSegment("dashboard"), StaticSegment("calendar")) view=ArtistCalendar/>
                        <Route path=(StaticSegment("artist"), StaticSegment("dashboard"), StaticSegment("requests")) view=ArtistRequests/>
                        <Route path=(StaticSegment("artist"), StaticSegment("dashboard"), StaticSegment("settings")) view=ArtistSettings/>
                        <Route path=(StaticSegment("artist"), ParamSegment("id")) view=ArtistHighlight/>
                        <Route path=(StaticSegment("shop"), ParamSegment("id")) view=Shop/>
                        <Route path=(StaticSegment("book"), StaticSegment("artist"), ParamSegment("id")) view=ArtistBooking/>
                        <Route path=(StaticSegment("book"), StaticSegment("shop"), ParamSegment("id")) view=ShopBooking/>
                    </Routes>
                </main>
            </Router>
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
