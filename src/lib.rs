mod api;
pub mod model;

use api::endpoint;
use api::response::get_device_real_kpi::GetDeviceRealKpi;
use api::response::FusionsolarApiResponse;
use model::DeviceTypeId;
use serde_json::Value;

use std::collections::HashMap;

const XSRF_TOKEN: &str = "XSRF-TOKEN";

#[derive(Debug, Clone)]
pub enum Error {
    LoginError(String),
    ApiError(String),
    UnexpectedApiResponse,
    InvalidResponse(String, String),
    UnknownDeviceType(DeviceTypeId),
    FormatError,
    InternalError,
}

pub fn api(api_url: String, username: String, password: String) -> model::Api {
    model::Api {
        api_url,
        username,
        password,
    }
}

pub async fn login(api: &model::Api) -> Result<model::LoggedInApi, Error> {
    let client = reqwest::ClientBuilder::new()
        .cookie_store(true)
        .build()
        .or(Err(Error::InternalError))?;
    let url = format!("{}{}", api.api_url, endpoint::LOGIN);

    let mut map = HashMap::new();
    map.insert("userName", api.username.to_owned());
    map.insert("systemCode", api.password.to_owned());

    match client.post(url).json(&map).send().await {
        Ok(response) => match response
            .cookies()
            .find(|cookie| cookie.name() == XSRF_TOKEN)
        {
            Some(cookie) => {
                let api = model::LoggedInApi {
                    api_url: api.api_url.to_owned(),
                    xsrf_token: String::from(cookie.value()),
                    client,
                };
                Ok(api)
            }
            None => Err(Error::LoginError(format!(
                "No XSRF-TOKEN received (server responded {})",
                response.status()
            ))),
        },
        Err(e) => Err(Error::LoginError(e.to_string())),
    }
}

async fn post_str(
    api: &model::LoggedInApi,
    endpoint: &endpoint::Endpoint,
    data: Option<&HashMap<&str, String>>,
) -> Result<String, Error> {
    let url = format!("{}{}", api.api_url, endpoint);

    let request = match data {
        Some(data) => api.client.post(url.clone()).json(data),
        None => api.client.post(url.clone()),
    }
    .header(XSRF_TOKEN, api.xsrf_token.to_owned());

    request
        .send()
        .await
        .map_err(|e| Error::ApiError(e.to_string()))?
        .text()
        .await
        .map_err(|e| {
            Error::InvalidResponse(
                e.to_string(),
                String::from("Error reading text from API response"),
            )
        })
}

async fn post(
    api: &model::LoggedInApi,
    endpoint: &endpoint::Endpoint,
    data: Option<&HashMap<&str, String>>,
) -> Result<FusionsolarApiResponse, Error> {
    let response_text = post_str(api, endpoint, data).await?;

    log::trace!(
        "endpoint: {}, data: {:#?}, response_text: {}",
        endpoint,
        data,
        response_text
    );

    let response = serde_json::from_str(&response_text)
        .map_err(|e| Error::InvalidResponse(e.to_string(), response_text))?;

    /* Handle `FusionsolarResponse::Error`, return any other response type as Ok */
    match response {
        FusionsolarApiResponse::Error(e) => Err(Error::ApiError(format!(
            "Error {}: {}",
            e.fail_code,
            e.message
                .unwrap_or_else(|| "(no error message received)".to_string())
        ))),
        _ => Ok(response),
    }
}

pub async fn logout(api: &model::LoggedInApi) -> Result<(), Error> {
    match post(api, endpoint::LOGOUT, None).await? {
        FusionsolarApiResponse::Success(_response) => Ok(()),
        _ => Err(Error::UnexpectedApiResponse),
    }
}

pub async fn stations(api: &model::LoggedInApi) -> Result<Vec<model::Station>, Error> {
    match post(api, endpoint::STATIONS, None).await? {
        FusionsolarApiResponse::GetStationsList(response) => {
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
        }
        _ => Err(Error::UnexpectedApiResponse),
    }
}

pub async fn station_real_kpi(
    api: &model::LoggedInApi,
    station: &model::Station,
) -> Result<Vec<model::StationRealKpi>, Error> {
    let mut map = HashMap::new();
    map.insert("stationCodes", station.code.to_owned());

    match post(api, endpoint::STATION_REAL_KPI, Some(&map)).await? {
        FusionsolarApiResponse::GetStationRealKpi(response) => {
            let stations = response
                .data
                .into_iter()
                .map(|resp| model::StationRealKpi {
                    code: resp.station_code,
                    day_power: resp.data_item_map.day_power,
                })
                .collect();
            Ok(stations)
        }
        _ => Err(Error::UnexpectedApiResponse),
    }
}

pub async fn devices(
    api: &model::LoggedInApi,
    station: &model::Station,
) -> Result<Vec<model::Device>, Error> {
    let mut map = HashMap::new();
    map.insert("stationCodes", station.code.to_owned());

    match post(api, endpoint::DEVICES, Some(&map)).await? {
        FusionsolarApiResponse::GetDevicesList(response) => {
            let devices = response
                .data
                .iter()
                .map(|resp| model::Device {
                    type_id: resp.dev_type_id,
                    id: resp.id,
                })
                .collect();
            Ok(devices)
        }
        _ => Err(Error::UnexpectedApiResponse),
    }
}

pub async fn device_real_kpi(
    api: &model::LoggedInApi,
    device: &model::Device,
) -> Result<Vec<model::DeviceRealKpi>, Error> {
    match device.type_id {
        DeviceTypeId::UnsupportedDeviceTypeId(_) => Err(Error::UnknownDeviceType(device.type_id)),
        DeviceTypeId::SupportedDeviceTypeId(type_id) => {
            let mut map = HashMap::new();
            map.insert("devIds", device.id.to_string());
            map.insert("devTypeId", (type_id as u64).to_string());

            match post(api, endpoint::DEVICE_REAL_KPI, Some(&map)).await? {
                FusionsolarApiResponse::GetDeviceRealKpi(response) => match response {
                    GetDeviceRealKpi::StringInverter(response) => {
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
                    }
                },
                _ => Err(Error::UnexpectedApiResponse),
            }
        }
    }
}

/*
Dump devices KPI

Iterate through all stations and all devices within those stations. Collect raw JSON output
of KPI for feature rporting purposes.

For the sake of simplicity, it's intentionally allowed to panic.
*/

pub async fn dump_devices(api: &model::LoggedInApi) -> Result<HashMap<u64, Value>, Error> {
    let stations = stations(api).await?;
    let mut dump: HashMap<u64, Value> = HashMap::new();

    for station in stations {
        if let Ok(devices) = devices(api, &station).await {
            for device in devices {
                let mut map = HashMap::new();

                let type_id = match device.type_id {
                    DeviceTypeId::SupportedDeviceTypeId(v) => v as u64,
                    DeviceTypeId::UnsupportedDeviceTypeId(v) => v,
                };

                map.insert("devIds", device.id.to_string());
                map.insert("devTypeId", type_id.to_string());

                let response = post_str(api, endpoint::DEVICE_REAL_KPI, Some(&map)).await?;
                if let Ok(value) = serde_json::from_str::<Value>(&response) {
                    if let Some(data_item_map) = value
                        .get("data")
                        .and_then(|v| v.get(0))
                        .and_then(|v| v.get("dataItemMap"))
                    {
                        dump.insert(type_id, data_item_map.to_owned());
                    } else {
                        log::warn!(
                            "No dataItemMap returned for device {}: {}",
                            type_id,
                            device.id
                        );
                    }
                    // .and_then(|v| v.get(0))
                    // .and_then(|v| v.get("dataItemMap"))
                    // .ok_or(Error::UnexpectedApiResponse)?;
                }

                // match response {
                //     Ok(s) => {
                //         dump.insert(device, s);
                //     }
                //     Err(e) => log::error!("{:?}", e),
                // };
            }
        }
    }

    Ok(dump)
}
