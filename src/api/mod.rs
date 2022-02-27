pub mod endpoint;
pub mod error;
pub mod response;

use crate::model;
pub use error::Error;
use reqwest::Response;
use response::get_device_list::GetDevicesList;
use response::get_device_real_kpi;
use response::get_station_real_kpi::GetStationRealKpi;
use response::get_stations_list::GetStationsList;
use serde_json::Value;

use std::collections::HashMap;

const XSRF_TOKEN: &str = "XSRF-TOKEN";

pub fn api(api_url: String, username: String, password: String) -> model::Api {
    model::Api {
        api_url,
        username,
        password,
    }
}

fn extract_xsrf_token(response: Response) -> Result<String, Error> {
    response
        .cookies()
        .find(|cookie| cookie.name() == XSRF_TOKEN)
        .ok_or_else(|| {
            Error::LoginError(format!(
                "No XSRF-TOKEN received (server responded {})",
                response.status()
            ))
        })
        .map(|cookie| String::from(cookie.value()))
}

/// Map Non-200 API response to Error
fn map_api_err(error: reqwest::Error) -> Error {
    match error.status() {
        Some(http::StatusCode::TOO_MANY_REQUESTS) => Error::RateExceeded(error.to_string()),
        Some(http::StatusCode::UNAUTHORIZED) => Error::LoginError(error.to_string()),
        _ => Error::ApiError(error.to_string()),
    }
}

/// Process value of valid HTTP response (2xx) to identify potential API-level error indicated
/// with non-true `success`. Return specific or generic error in that case or carry the `value`
/// forward if it is identified as successful response.
fn map_response_status(value: Value) -> Result<Value, Error> {
    let success = value
        .get("success")
        .and_then(Value::as_bool)
        .unwrap_or(false);

    let fail_code = value.get("failCode").and_then(Value::as_u64);

    if success {
        Ok(value)
    } else {
        match fail_code.and_then(num::FromPrimitive::from_u64) {
            /* {"data":"ACCESS_FREQUENCY_IS_TOO_HIGH","failCode":407,"params":null,"success":false} */
            Some(response::FailCode::AccessFrequencyIsTooHigh) => {
                Err(Error::RateExceeded(value.to_string()))
            }
            _ => Err(Error::ApiError(value.to_string())),
        }
    }
}

pub async fn login(api: &model::Api) -> Result<model::LoggedInApi, Error> {
    let client = reqwest::ClientBuilder::new()
        .cookie_store(true)
        .build()
        .or(Err(Error::InternalError))?;
    let url = format!("{}{}", api.api_url, endpoint::LOGIN);

    let request_body = HashMap::from([
        ("userName", api.username.to_owned()),
        ("systemCode", api.password.to_owned()),
    ]);

    client
        .post(url)
        .json(&request_body)
        .send()
        .await
        .map_err(map_api_err)
        .map(extract_xsrf_token)?
        .map(|token| model::LoggedInApi {
            api_url: api.api_url.to_owned(),
            xsrf_token: token,
            client,
        })
}

async fn post(
    api: &model::LoggedInApi,
    endpoint: &endpoint::Endpoint,
    data: Option<&HashMap<&str, String>>,
) -> Result<Value, Error> {
    let url = format!("{}{}", api.api_url, endpoint);

    let request = match data {
        Some(data) => api.client.post(url.clone()).json(data),
        None => api.client.post(url.clone()),
    }
    .header(XSRF_TOKEN, api.xsrf_token.to_owned());

    request
        .send()
        .await
        .map_err(map_api_err)
        .map(|r| r.text())?
        .await
        .map_err(|e| Error::ApiError(format!("Error reading API response: {}", e)))
        .map(|s| {
            serde_json::from_str::<Value>(&s).map_err(|e| Error::InvalidResponse(s, e.to_string()))
        })?
        .map(map_response_status)?
}

pub async fn stations(api: &model::LoggedInApi) -> Result<Vec<model::Station>, Error> {
    post(api, endpoint::STATIONS, None)
        .await
        .map(serde_json::from_value::<GetStationsList>)?
        .or(Err(Error::UnexpectedApiResponse))
        .map(|response| {
            let stations = response
                .data
                .into_iter()
                .map(|sta_resp| model::Station {
                    code: sta_resp.station_code,
                    name: sta_resp.station_name,
                    /* convert MWh to kWh */
                    capacity: sta_resp.capacity * 1000.0,
                })
                .collect();
            Ok(stations)
        })?
}

/// Read KPI of specified station.
pub async fn station_real_kpi(
    api: &model::LoggedInApi,
    station: &model::Station,
) -> Result<Vec<model::StationRealKpi>, Error> {
    let request_body = HashMap::from([("stationCodes", station.code.to_owned())]);

    post(api, endpoint::STATION_REAL_KPI, Some(&request_body))
        .await
        .map(serde_json::from_value::<GetStationRealKpi>)?
        .or(Err(Error::UnexpectedApiResponse))
        .map(|response| {
            let stations = response
                .data
                .into_iter()
                .map(|resp| model::StationRealKpi {
                    code: resp.station_code,
                    day_power: resp.data_item_map.day_power,
                })
                .collect();
            Ok(stations)
        })?
}

/// List all devices for `station`
pub async fn devices(
    api: &model::LoggedInApi,
    station: &model::Station,
) -> Result<Vec<model::Device>, Error> {
    let request_body = HashMap::from([("stationCodes", station.code.to_owned())]);

    post(api, endpoint::DEVICES, Some(&request_body))
        .await
        .map(serde_json::from_value::<GetDevicesList>)?
        .or(Err(Error::UnexpectedApiResponse))
        .map(|response| {
            let devices = response
                .data
                .iter()
                .map(|resp| model::Device {
                    type_id: resp.dev_type_id,
                    id: resp.id,
                })
                .collect();
            Ok(devices)
        })?
}

/// Takes `device: Device` and if `device.type_id` is supported, reads KPI for that device.
pub async fn device_real_kpi(
    api: &model::LoggedInApi,
    device: &model::Device,
) -> Result<Vec<model::DeviceRealKpi>, Error> {
    match num::FromPrimitive::from_u64(device.type_id) as Option<model::DeviceTypeId> {
        Some(type_id) => {
            let request_body = HashMap::from([
                ("devIds", device.id.to_string()),
                ("devTypeId", device.type_id.to_string()),
            ]);

            let value = post(api, endpoint::DEVICE_REAL_KPI, Some(&request_body))
                .await
                .or(Err(Error::UnexpectedApiResponse))?;

            match type_id {
                model::DeviceTypeId::StringInverter => {
                    serde_json::from_value::<get_device_real_kpi::StringInverter>(value)
                        .or(Err(Error::UnexpectedApiResponse))
                        .map(|response| {
                            let devices = response
                                .data
                                .iter()
                                .map(|resp| model::DeviceRealKpi {
                                    id: resp.dev_id,
                                    temperature: Some(resp.data_item_map.temperature),
                                    active_power: Some(resp.data_item_map.active_power),
                                })
                                .collect();
                            Ok(devices)
                        })?
                }
            }
        }
        None => Err(Error::UnknownDeviceType(device.type_id)),
    }
}

/// Dump devices KPI
///
/// Iterate through all stations and all devices within those stations. Collect raw JSON output
/// of KPI for feature reporting purposes.
///
/// For the sake of simplicity, it's intentionally allowed to panic.
pub async fn dump_devices(api: &model::LoggedInApi) -> Result<HashMap<u64, Value>, Error> {
    let stations = stations(api).await?;
    let mut dump: HashMap<u64, Value> = HashMap::new();

    for station in stations {
        if let Ok(devices) = devices(api, &station).await {
            for device in devices {
                let request_body = HashMap::from([
                    ("devIds", device.id.to_string()),
                    ("devTypeId", device.type_id.to_string()),
                ]);

                let response = post(api, endpoint::DEVICE_REAL_KPI, Some(&request_body)).await?;
                if let Ok(value) = serde_json::from_value::<Value>(response.clone()) {
                    if let Some(data_item_map) = value
                        .get("data")
                        .and_then(|v| v.get(0))
                        .and_then(|v| v.get("dataItemMap"))
                    {
                        dump.insert(device.type_id, data_item_map.to_owned());
                    } else {
                        log::warn!(
                            "No dataItemMap returned for device {}: {}: {}",
                            device.type_id,
                            device.id,
                            response.to_string()
                        );
                    }
                }
            }
        }
    }

    Ok(dump)
}
