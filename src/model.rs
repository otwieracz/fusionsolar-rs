use crate::api::response::device_type::DeviceTypeId;

type KWh = f64;

#[derive(Debug, Clone)]
pub struct Api {
    pub api_url: String,
    pub username: String,
    pub password: String,
}

#[derive(Debug)]
pub struct LoggedInApi {
    pub api_url: String,
    pub xsrf_token: String,
    pub client: reqwest::Client,
}

#[derive(Debug)]
pub struct Station {
    pub capacity: KWh,
    pub name: String,
    pub code: String,
}

#[derive(Debug)]
pub struct Device {
    pub type_id: DeviceTypeId,
    pub id: u64,
}

#[derive(Debug)]
pub struct StationRealKpi {
    pub code: String,
    pub day_power: KWh,
}

#[derive(Debug)]
pub struct DeviceRealKpi {
    pub id: u64,
    pub temperature: Option<f64>,
    pub active_power: Option<f64>,
}
