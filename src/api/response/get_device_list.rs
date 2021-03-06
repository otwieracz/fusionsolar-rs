use serde::Deserialize;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Data {
    pub dev_name: String,
    pub id: u64,
    pub dev_type_id: u64,
}

#[derive(Deserialize)]
pub struct GetDevicesList {
    pub data: Vec<Data>,
}
