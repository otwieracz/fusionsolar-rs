use crate::api::response::DeviceTypeId;

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
pub struct StringInverterRealKpi {
    pub id: u64,
    pub temperature: f64,
    pub active_power: f64,
}

pub enum DeviceRealKpi {
    StringInverterRealKpi(StringInverterRealKpi),
}

pub fn station(code: String, name: String, capacity: KWh) -> Station {
    Station {
        code,
        name,
        capacity,
    }
}

pub fn station_real_kpi(code: String, day_power: KWh) -> StationRealKpi {
    StationRealKpi { code, day_power }
}

pub fn device(type_id: DeviceTypeId, id: u64) -> Device {
    Device { type_id, id }
}
