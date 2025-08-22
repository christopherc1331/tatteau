use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct CityCoords {
    pub city: String,
    pub state: String,
    pub lat: f64,
    pub long: f64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Artist {
    pub id: i32,
    pub name: Option<String>,
    pub location_id: i32,
    pub social_links: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub years_experience: Option<i32>,
    pub styles_extracted: Option<i32>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ArtistStyle {
    pub id: i32,
    pub style_id: i32,
    pub artist_id: i32,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ArtistImage {
    pub id: i32,
    pub short_code: String,
    pub artist_id: i32,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ArtistImageStyle {
    pub id: i32,
    pub artists_images_id: i32,
    pub style_id: i32,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Style {
    pub id: i32,
    pub name: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Location {
    pub id: i32,
    pub name: Option<String>,
    pub lat: Option<f64>,
    pub long: Option<f64>,
    pub city: Option<String>,
    pub state: Option<String>,
    pub address: Option<String>,
    pub website_uri: Option<String>,
}
