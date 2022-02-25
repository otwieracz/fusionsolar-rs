use serde::Deserialize;

/* Device Type 1: String Inverter */
pub mod string_inverter {
    use serde::Deserialize;

    #[derive(Deserialize)]
    pub struct DataItemMap {
        pub temperature: f64,
        pub active_power: f64,
        pub mppt_power: f64,
    }

    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Data {
        pub dev_id: u64,
        pub data_item_map: DataItemMap,
    }
}

#[derive(Deserialize)]
pub struct StringInverter {
    pub data: Vec<string_inverter::Data>,
}
