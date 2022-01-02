#[macro_use]
extern crate rocket;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate prometheus;

use config::Config;
use fusionsolar_rs::model::{Api, DeviceRealKpi, LoggedInApi, Station};
use prometheus::{Encoder, GaugeVec, TextEncoder};
use rocket::response::Debug;
use rocket::State;

const API_URL: &str = "https://eu5.fusionsolar.huawei.com/thirdData";

lazy_static! {
    static ref DAY_POWER_GAUGE: GaugeVec = register_gauge_vec!(
        opts!(
            "day_power",
            "total amount of power generated in current day (in kWh)",
        ),
        &["station_code"],
    )
    .unwrap();
    static ref DEVICE_ACTIVE_POWER_GAUGE: GaugeVec = register_gauge_vec!(
        opts!(
            "device_active_power",
            "active power production reported by inverter",
        ),
        &["station_code", "device_id", "device_type_id",],
    )
    .unwrap();
    static ref DEVICE_TEMPERAURE_GAUGE: GaugeVec = register_gauge_vec!(
        opts!("device_temperature", "device reported temperature",),
        &["station_code", "device_id", "device_type_id",],
    )
    .unwrap();
}

async fn collect_station_devices(
    api: &LoggedInApi,
    station: &Station,
) -> Result<(), fusionsolar_rs::Error> {
    let devices = fusionsolar_rs::devices(api, station).await?;
    log::debug!("devices: {:?}", devices);

    for device in devices {
        match fusionsolar_rs::device_real_kpi(api, &device).await {
            Ok(dev_kpi_vec) => match dev_kpi_vec.get(0) {
                None => {
                    log::warn!(
                        "No KPI returned for device {} of station {}",
                        device.id,
                        station.code
                    );
                }
                Some(dev_kpi) => match dev_kpi {
                    DeviceRealKpi::StringInverterRealKpi(string_inverter_kpi) => {
                        DEVICE_ACTIVE_POWER_GAUGE
                            .with_label_values(&[
                                &station.code,
                                &string_inverter_kpi.id.to_string(),
                                &(device.type_id as u64).to_string(),
                            ])
                            .set(string_inverter_kpi.active_power);
                        DEVICE_TEMPERAURE_GAUGE
                            .with_label_values(&[
                                &station.code,
                                &string_inverter_kpi.id.to_string(),
                                &(device.type_id as u64).to_string(),
                            ])
                            .set(string_inverter_kpi.temperature);
                    }
                },
            },
            Err(e) => log::debug!("{:?}: {:?}", e, device),
        }
    }

    Ok(())
}

async fn collect_day_power(api: &LoggedInApi) -> Result<(), fusionsolar_rs::Error> {
    let stations = fusionsolar_rs::stations(api).await?;
    log::debug!("stations: {:?}", stations);

    for station in stations {
        let kpi = fusionsolar_rs::station_real_kpi(api, &station).await?;

        match kpi.get(0) {
            None => {
                log::warn!("No KPI returned for station: {}", &station.code);
            }
            Some(kpi) => {
                DAY_POWER_GAUGE
                    .with_label_values(&[&station.code])
                    .set(kpi.day_power);
            }
        }

        collect_station_devices(api, &station).await?;
    }

    Ok(())
}

async fn collect_metrics(api: &Api) -> Result<(), fusionsolar_rs::Error> {
    let logged_in_api = fusionsolar_rs::login(api).await?;
    collect_day_power(&logged_in_api).await?;
    fusionsolar_rs::logout(&logged_in_api).await.or_else(|e| {
        log::warn!("Error while logging out: {:#?}", e);
        Ok(())
    })?;

    Ok(())
}

async fn read_metrics() -> Result<String, fusionsolar_rs::Error> {
    // Gather the metrics.
    let mut buffer = Vec::new();
    let encoder = TextEncoder::new();
    let metric_families = prometheus::gather();

    encoder.encode(&metric_families, &mut buffer).unwrap();
    String::from_utf8(buffer).or(Err(fusionsolar_rs::Error::FormatError))
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct Fusionsolar {
    pub api_url: String,
    pub username: String,
    pub password: String,
}

pub fn read_settings() -> Fusionsolar {
    let mut settings = Config::default();
    settings
        .merge(config::Environment::with_prefix("FS"))
        .unwrap()
        .set_default("api_url", API_URL)
        .unwrap();

    settings.try_into().expect("Configuration error")
}

#[get("/metrics")]
async fn metrics(api: &State<Api>) -> Result<String, Debug<fusionsolar_rs::Error>> {
    collect_metrics(api).await?;
    read_metrics().await.map_err(Debug)
}

#[launch]
fn rocket() -> _ {
    env_logger::init();

    let settings = read_settings();
    let api = fusionsolar_rs::api(settings.api_url, settings.username, settings.password);
    rocket::build().manage(api).mount("/", routes![metrics])
}

// #[tokio::main]
// async fn main() {
//     let settings = read_settings();
//     let api = fusionsolar_rs::api(settings.api_url, settings.username, settings.password);
//     collect_metrics(&api).await.unwrap();
// }