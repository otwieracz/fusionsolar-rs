use serde::Deserialize;

#[derive(Deserialize)]
pub struct Data {
    #[serde(rename = "stationCode")]
    pub station_code: String,
    #[serde(rename = "stationName")]
    pub station_name: String,
    pub capacity: f64,
}

#[derive(Deserialize)]
pub struct GetStationsList {
    pub data: Vec<Data>,
}
