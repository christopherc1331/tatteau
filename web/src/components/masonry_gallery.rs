use leptos::prelude::*;
use web_sys::window;

#[derive(Debug, Clone, PartialEq)]
pub struct ImageItem {
    pub id: String,
    pub url: String,
    pub width: u32,
    pub height: u32,
    pub alt: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MasonryItem {
    pub image: ImageItem,
    pub row_start: u32,
    pub row_end: u32,
    pub column: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MasonryLayout {
    pub items: Vec<MasonryItem>,
    pub total_rows: u32,
}

impl MasonryLayout {
    pub fn calculate(images: &[ImageItem], columns: u32, gap: u32, column_width: u32) -> Self {
        let mut column_heights = vec![1u32; columns as usize];
        let mut items = Vec::new();
        
        for image in images {
            // Find the shortest column
            let shortest_col = column_heights
                .iter()
                .enumerate()
                .min_by_key(|(_, &height)| height)
                .map(|(idx, _)| idx)
                .unwrap_or(0);
            
            // Calculate aspect ratio and height for this column width
            let aspect_ratio = image.height as f64 / image.width as f64;
            let display_height = (column_width as f64 * aspect_ratio) as u32;
            
            // Convert to grid units (assuming each grid unit = 5px for tighter packing)
            let grid_height = (display_height / 5).max(1);
            
            let row_start = column_heights[shortest_col];
            let row_span = grid_height + (gap / 5); // Adjust gap for 5px units
            
            items.push(MasonryItem {
                image: image.clone(),
                row_start,
                row_end: row_span, // Now using this as span instead
                column: shortest_col as u32 + 1, // CSS Grid is 1-indexed
            });
            
            // Update column height
            column_heights[shortest_col] = row_start + row_span;
        }
        
        let total_rows = column_heights.into_iter().max().unwrap_or(1);
        
        Self { items, total_rows }
    }
}

// Sample data generator for Unsplash images
pub fn generate_sample_images() -> Vec<ImageItem> {
    vec![
        ImageItem {
            id: "1".to_string(),
            url: "https://picsum.photos/400/600?random=1".to_string(),
            width: 400,
            height: 600,
            alt: "Random image 1".to_string(),
        },
        ImageItem {
            id: "2".to_string(),
            url: "https://picsum.photos/400/400?random=2".to_string(),
            width: 400,
            height: 400,
            alt: "Random image 2".to_string(),
        },
        ImageItem {
            id: "3".to_string(),
            url: "https://picsum.photos/400/800?random=3".to_string(),
            width: 400,
            height: 800,
            alt: "Random image 3".to_string(),
        },
        ImageItem {
            id: "4".to_string(),
            url: "https://picsum.photos/400/500?random=4".to_string(),
            width: 400,
            height: 500,
            alt: "Random image 4".to_string(),
        },
        ImageItem {
            id: "5".to_string(),
            url: "https://picsum.photos/400/700?random=5".to_string(),
            width: 400,
            height: 700,
            alt: "Random image 5".to_string(),
        },
        ImageItem {
            id: "6".to_string(),
            url: "https://picsum.photos/400/450?random=6".to_string(),
            width: 400,
            height: 450,
            alt: "Random image 6".to_string(),
        },
        ImageItem {
            id: "7".to_string(),
            url: "https://picsum.photos/400/650?random=7".to_string(),
            width: 400,
            height: 650,
            alt: "Random image 7".to_string(),
        },
        ImageItem {
            id: "8".to_string(),
            url: "https://picsum.photos/400/550?random=8".to_string(),
            width: 400,
            height: 550,
            alt: "Random image 8".to_string(),
        },
        ImageItem {
            id: "9".to_string(),
            url: "https://picsum.photos/400/750?random=9".to_string(),
            width: 400,
            height: 750,
            alt: "Random image 9".to_string(),
        },
        ImageItem {
            id: "10".to_string(),
            url: "https://picsum.photos/400/600?random=10".to_string(),
            width: 400,
            height: 600,
            alt: "Random image 10".to_string(),
        },
        ImageItem {
            id: "11".to_string(),
            url: "https://picsum.photos/400/480?random=11".to_string(),
            width: 400,
            height: 480,
            alt: "Random image 11".to_string(),
        },
        ImageItem {
            id: "12".to_string(),
            url: "https://picsum.photos/400/520?random=12".to_string(),
            width: 400,
            height: 520,
            alt: "Random image 12".to_string(),
        },
    ]
}

#[component]
pub fn MasonryGallery() -> impl IntoView {
    let (screen_width, set_screen_width) = signal(1200u32);
    let images = generate_sample_images();
    
    // Calculate responsive columns
    let column_count = Memo::new(move |_| {
        let width = screen_width.get();
        if width >= 1200 {
            4
        } else if width >= 768 {
            3
        } else if width >= 480 {
            2
        } else {
            1
        }
    });
    
    // Set initial width
    Effect::new(move |_| {
        if let Some(window) = window() {
            let width = window.inner_width().unwrap().as_f64().unwrap() as u32;
            set_screen_width.set(width);
        }
    });
    
    view! {
        <div class="masonry-gallery">
            <h1>"Masonry Gallery"</h1>
            <div 
                class="masonry-grid"
                style:column-count=move || column_count.get().to_string()
            >
                <For
                    each=move || images.clone()
                    key=|item| item.id.clone()
                    children=move |item: ImageItem| {
                        view! {
                            <div class="masonry-item">
                                <img 
                                    src=item.url
                                    alt=item.alt
                                    loading="lazy"
                                />
                            </div>
                        }
                    }
                />
            </div>
        </div>
    }
}