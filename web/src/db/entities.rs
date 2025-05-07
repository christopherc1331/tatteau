use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct CityCoords {
    pub city: String,
    pub state: String,
    pub lat: f64,
    pub long: f64,
}
