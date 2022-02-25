use num_derive::FromPrimitive;

type KWh = f64;

pub type UnsupportedDeviceTypeId = u64;

#[derive(Debug, FromPrimitive)]
pub enum DeviceTypeId {
    StringInverter = 1,
}

#[derive(Debug, Clone)]
pub struct Api {
    pub api_url: String,
    pub username: String,
    pub password: String,
}

pub struct LoggedInApi {
    pub api_url: String,
    pub xsrf_token: String,
    pub client: reqwest::Client,
}

pub struct Station {
    pub capacity: KWh,
    pub name: String,
    pub code: String,
}

pub struct Device {
    pub type_id: u64,
    pub id: u64,
}

pub struct StationRealKpi {
    pub code: String,
    pub day_power: KWh,
}

pub struct DeviceRealKpi {
    pub id: u64,
    pub temperature: Option<f64>,
    pub active_power: Option<f64>,
}
