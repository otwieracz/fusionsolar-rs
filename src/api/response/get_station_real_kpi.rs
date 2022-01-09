#[derive(serde::Deserialize)]
pub struct DataItemMap {
    pub day_power: f64,
}
#[derive(serde::Deserialize)]
pub struct Data {
    #[serde(rename = "dataItemMap")]
    pub data_item_map: DataItemMap,
    #[serde(rename = "stationCode")]
    pub station_code: String,
}
#[derive(serde::Deserialize)]
pub struct GetStationRealKpi {
    pub data: Vec<Data>,
}
