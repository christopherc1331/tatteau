use leptos::prelude::*;

#[component]
pub fn StyleTag(
    #[prop(into)] name: String,
) -> impl IntoView {
    view! {
        <span class="shop-masonry-gallery__style-tag">
            {name}
        </span>
    }
}
