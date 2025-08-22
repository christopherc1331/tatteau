use leptos::prelude::*;
use crate::db::entities::{ArtistImage, Style, Artist};
use crate::components::instagram_embed::{InstagramEmbed, process_instagram_embeds};
use wasm_bindgen::prelude::*;

#[derive(Clone, Debug, PartialEq)]
pub struct ShopInstagramPost {
    pub image: ArtistImage,
    pub styles: Vec<Style>,
    pub artist: Artist,
}

#[component]
pub fn ShopMasonryGallery(
    shop_posts: Vec<ShopInstagramPost>,
    all_styles: Vec<Style>,
) -> impl IntoView {
    let (selected_style, set_selected_style) = signal::<Option<i32>>(None);
    let (show_grid, set_show_grid) = signal(true);
    
    // Process Instagram embeds after DOM updates
    Effect::new(move |_| {
        if show_grid.get() {
            let window = web_sys::window().unwrap();
            let closure = Closure::once(move || {
                process_instagram_embeds();
            });
            
            window.set_timeout_with_callback_and_timeout_and_arguments_0(
                closure.as_ref().unchecked_ref(),
                200
            ).ok();
            
            closure.forget();
        }
    });
    
    let filtered_posts = Memo::new(move |_| {
        let posts = shop_posts.clone();
        if let Some(style_id) = selected_style.get() {
            posts.into_iter()
                .filter(|post| post.styles.iter().any(|s| s.id == style_id))
                .collect::<Vec<_>>()
        } else {
            posts
        }
    });


    view! {
        <div>
            <div style="margin-bottom: 1.5rem;">
                <div style="display: flex; flex-wrap: wrap; gap: 0.5rem; align-items: center;">
                    <span style="font-weight: 600; color: #4a5568; margin-right: 0.5rem;">"Filter by style:"</span>
                    
                    <button 
                        on:click=move |_| {
                            set_show_grid.set(false);
                            set_selected_style.set(None);
                            set_timeout(move || set_show_grid.set(true), std::time::Duration::from_millis(50));
                        }
                        style=move || format!(
                            "background: {}; color: {}; padding: 0.25rem 0.75rem; border: 1px solid #d1d5db; border-radius: 20px; font-size: 0.8rem; cursor: pointer;",
                            if selected_style.get().is_none() { "#667eea" } else { "white" },
                            if selected_style.get().is_none() { "white" } else { "#374151" }
                        )
                    >
                        "All"
                    </button>
                    
                    {all_styles.into_iter().map(|style| {
                        let style_id = style.id;
                        let style_name = style.name.clone();
                        view! {
                            <button 
                                on:click=move |_| {
                                    set_show_grid.set(false);
                                    set_selected_style.set(Some(style_id));
                                    set_timeout(move || set_show_grid.set(true), std::time::Duration::from_millis(50));
                                }
                                style=move || format!(
                                    "background: {}; color: {}; padding: 0.25rem 0.75rem; border: 1px solid #d1d5db; border-radius: 20px; font-size: 0.8rem; cursor: pointer;",
                                    if selected_style.get() == Some(style_id) { "#667eea" } else { "white" },
                                    if selected_style.get() == Some(style_id) { "white" } else { "#374151" }
                                )
                            >
                                {style_name}
                            </button>
                        }
                    }).collect_view()}
                </div>
            </div>
            
            <style>
                {r#"
                .shop-masonry {
                    columns: 4;
                    column-gap: 1rem;
                    column-fill: balance;
                    width: 100%;
                }
                @media (max-width: 1200px) {
                    .shop-masonry { columns: 3 !important; }
                }
                @media (max-width: 768px) {
                    .shop-masonry { columns: 2 !important; }
                }
                @media (max-width: 480px) {
                    .shop-masonry { columns: 1 !important; }
                }
                .instagram-media {
                    max-width: 100% !important;
                    min-width: 280px !important;
                }
                "#}
            </style>
            {move || {
                if show_grid.get() {
                    view! {
                        <div class="shop-masonry">
                            {filtered_posts.get().into_iter().map(|post| {
                                let short_code = post.image.short_code.clone();
                                let artist_name = post.artist.name.unwrap_or_else(|| "Unknown Artist".to_string());
                                
                                view! {
                            <div style="break-inside: avoid; margin-bottom: 1rem; position: relative;">
                                <div style="background: white; border-radius: 8px; overflow: hidden; box-shadow: 0 2px 8px rgba(0,0,0,0.1); position: relative;">
                                    <div style="padding: 0.5rem; background: white;">
                                        <a href={format!("/artist/{}", post.artist.id)} 
                                           style="color: #374151; text-decoration: none; display: flex; align-items: center; gap: 0.25rem; font-size: 0.8rem; font-weight: 600;">
                                            <span>"👤"</span>
                                            <span>{artist_name}</span>
                                        </a>
                                    </div>
                                    
                                    <InstagramEmbed short_code={short_code} />
                                    
                                    {(!post.styles.is_empty()).then(|| {
                                        view! {
                                            <div style="padding: 0.5rem; background: white;">
                                                <div style="display: flex; flex-wrap: wrap; gap: 0.25rem;">
                                                    {post.styles.into_iter().map(|style| {
                                                        view! {
                                                            <span style="background: rgba(102, 126, 234, 0.9); color: white; padding: 0.125rem 0.375rem; border-radius: 10px; font-size: 0.6rem; font-weight: 500;">
                                                                {style.name}
                                                            </span>
                                                        }
                                                    }).collect_view()}
                                                </div>
                                            </div>
                                        }
                                    })}
                                </div>
                            </div>
                                }
                            }).collect_view()}
                        </div>
                        
                    }.into_any()
                } else {
                    view! {
                        <div style="height: 200px; display: flex; align-items: center; justify-content: center;">
                            <span style="color: #999;">"Loading..."</span>
                        </div>
                    }.into_any()
                }
            }}
        </div>
    }
}