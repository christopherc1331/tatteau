use leptos::prelude::*;
use leptos_router::components::A;

use crate::{
    components::loading::LoadingView,
    server::{get_all_styles_with_counts, StyleWithCount},
};

#[component]
pub fn StylesShowcase() -> impl IntoView {
    let styles_resource = Resource::new(
        || (),
        |_| async move {
            get_all_styles_with_counts().await
        },
    );

    view! {
        <div class="styles-showcase-container">
            <div class="page-header">
                <h1>"Tattoo Styles"</h1>
                <p class="subtitle">
                    "Explore different tattoo styles and find artists who specialize in each"
                </p>
            </div>

            <Suspense fallback=|| view! { <LoadingView message=Some("Loading styles...".to_string()) /> }>
                {move || {
                    match styles_resource.get() {
                        Some(Ok(styles)) => {
                            if styles.is_empty() {
                                view! {
                                    <div class="no-styles">
                                        <h3>"No styles found"</h3>
                                        <p>"We couldn't find any tattoo styles at the moment."</p>
                                        <A href="/">
                                            <div class="btn-primary">
                                                "Back to Home"
                                            </div>
                                        </A>
                                    </div>
                                }.into_any()
                            } else {
                                view! {
                                    <div class="styles-grid-container">
                                        <div class="styles-grid">
                                            {styles.into_iter().map(|style_info| {
                                                view! {
                                                    <StyleCard style_info=style_info />
                                                }
                                            }).collect_view()}
                                        </div>

                                        <div class="explore-section">
                                            <h2>"Find Artists by Style"</h2>
                                            <p>"Ready to find your perfect artist? Use our discovery tools to search by style, location, and more."</p>
                                            <div class="explore-buttons">
                                                <A href="/explore">
                                                    <div class="btn-primary">
                                                        "🗺️ Explore Map"
                                                    </div>
                                                </A>
                                                <A href="/match">
                                                    <div class="btn-outlined">
                                                        "✨ Take Quiz"
                                                    </div>
                                                </A>
                                            </div>
                                        </div>
                                    </div>
                                }.into_any()
                            }
                        },
                        Some(Err(_)) => view! {
                            <div class="no-styles">
                                <h3>"Something went wrong"</h3>
                                <p>"We couldn't load the styles right now. Please try again."</p>
                                <A href="/">
                                    <div class="btn-outlined">
                                        "Back to Home"
                                    </div>
                                </A>
                            </div>
                        }.into_any(),
                        None => view! {
                            <LoadingView message=Some("Loading styles...".to_string()) />
                        }.into_any(),
                    }
                }}
            </Suspense>
        </div>
    }
}

#[component]
pub fn StyleCard(style_info: StyleWithCount) -> impl IntoView {
    let style_name = style_info.name.clone();
    let artist_count = style_info.artist_count;
    
    // Get sample images for this style (placeholder for now)
    let sample_images = style_info.sample_images.unwrap_or_default();

    view! {
        <div class="style-card">
            <div class="style-card-header">
                <h3 class="style-name">{style_name.clone()}</h3>
                <div class="artist-count-badge">
                    {format!("{} artists", artist_count)}
                </div>
            </div>

            {if !sample_images.is_empty() {
                view! {
                    <div class="style-samples">
                        <div class="sample-images">
                            {sample_images.clone().into_iter().take(3).map(|image_url| {
                                view! {
                                    <div class="sample-image">
                                        <img src={image_url} alt={format!("{} style example", style_name)} />
                                    </div>
                                }
                            }).collect_view()}
                        </div>
                        {if sample_images.len() > 3 {
                            view! {
                                <div class="more-samples">
                                    {format!("+{} more examples", sample_images.len() - 3)}
                                </div>
                            }.into_any()
                        } else {
                            view! { <div></div> }.into_any()
                        }}
                    </div>
                }.into_any()
            } else {
                view! {
                    <div class="style-placeholder">
                        <div class="placeholder-icon">"🎨"</div>
                        <div class="placeholder-text">
                            {format!("Discover {} artists", artist_count)}
                        </div>
                    </div>
                }.into_any()
            }}

            <div class="style-description">
                {style_info.description.unwrap_or_else(|| 
                    format!("Explore {} tattoo style with {} talented artists in our network.", 
                           style_name, artist_count)
                )}
            </div>

            <div class="style-actions">
                <A href={format!("/explore?style={}", style_info.id)}>
                    <div class="btn-primary">
                        "Find Artists"
                    </div>
                </A>
                <A href={format!("/match?preferred_style={}", style_info.id)}>
                    <div class="btn-outlined">
                        "Get Matched"
                    </div>
                </A>
            </div>
        </div>
    }
}