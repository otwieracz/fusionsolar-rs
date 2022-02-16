#[macro_use]
extern crate rocket;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate prometheus;

use config::Config;
use fusionsolar_rs::model::{Api, DeviceRealKpi, DeviceTypeId, LoggedInApi, Station};
use prometheus::{Encoder, GaugeVec, TextEncoder};
use rocket::response::Debug;
use rocket::{Build, Rocket, State};
use std::sync::Mutex;
use std::time::Instant;

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

// Process DeviceRealKpi `device_real_kpi` of `device` installed in `station` and feed them to
// Prometheus metrics. Based on device type, different KPIs can be presented.
fn process_device_real_kpi(
    dev_real_kpi: &DeviceRealKpi,
    station: &Station,
    device: fusionsolar_rs::model::Device,
) {
    if let DeviceTypeId::SupportedDeviceTypeId(type_id) = device.type_id {
        if let Some(active_power) = dev_real_kpi.active_power {
            DEVICE_ACTIVE_POWER_GAUGE
                .with_label_values(&[
                    &station.code,
                    &dev_real_kpi.id.to_string(),
                    &(type_id as u64).to_string(),
                ])
                .set(active_power);
        }

        if let Some(temperature) = dev_real_kpi.temperature {
            DEVICE_TEMPERAURE_GAUGE
                .with_label_values(&[
                    &station.code,
                    &dev_real_kpi.id.to_string(),
                    &(type_id as u64).to_string(),
                ])
                .set(temperature);
        }
    }
}

async fn collect_station_devices(
    api: &LoggedInApi,
    station: &Station,
) -> Result<(), fusionsolar_rs::Error> {
    let devices = fusionsolar_rs::devices(api, station).await?;

    for device in devices {
        if let Ok(dev_kpi_vec) = fusionsolar_rs::device_real_kpi(api, &device).await {
            if let Some(dev_real_kpi) = dev_kpi_vec.get(0) {
                process_device_real_kpi(dev_real_kpi, station, device);
            } else {
                log::error!(
                    "No KPI returned for device {} of station {}",
                    device.id,
                    station.code
                );
            }
        }
    }
    Ok(())
}

async fn collect_day_power(api: &LoggedInApi) -> Result<(), fusionsolar_rs::Error> {
    let stations = fusionsolar_rs::stations(api).await?;

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

#[derive(Clone, serde::Deserialize)]
pub struct FusionsolarConfig {
    api_url: String,
    username: String,
    password: String,
    interval: u64,
}

pub struct StateData {
    api: Api,
    interval: u64,
    timestamp: Mutex<Option<Instant>>,
}

impl StateData {
    fn touch(&self) {
        if let Ok(mut ts) = self.timestamp.lock() {
            *ts = Some(Instant::now());
        } else {
            log::trace!("Unable to lock timestamp mutex, will refresh again")
        }
    }

    fn interval_elapsed(&self, interval_secs: u64) -> bool {
        let elapsed_opt = self
            .timestamp
            .lock()
            .ok()
            .map(|a| a.map(|b| b.elapsed().as_secs()))
            .flatten();

        if let Some(elapsed) = elapsed_opt {
            elapsed > interval_secs
        } else {
            /* If there is None timestamp/elapsed, always return true to trigger action */
            true
        }
    }
}

pub fn read_settings() -> FusionsolarConfig {
    let mut settings = Config::default();
    settings
        .merge(config::Environment::with_prefix("FS"))
        .unwrap()
        .set_default("api_url", API_URL)
        .unwrap();

    settings.try_into().expect("Configuration error")
}

#[get("/metrics")]
async fn metrics(state: &State<StateData>) -> Result<String, Debug<fusionsolar_rs::Error>> {
    if state.interval_elapsed(state.interval) {
        collect_metrics(&state.api).await?;
        state.touch();
    } else {
        log::info!("interval time not yet elapsed since last run; returning cached result")
    }
    read_metrics().await.map_err(Debug)
}

#[get("/dump-devices")]
async fn dump_devices(state: &State<StateData>) -> Result<String, Debug<fusionsolar_rs::Error>> {
    let logged_in_api = fusionsolar_rs::login(&state.api).await?;
    let dump = fusionsolar_rs::dump_devices(&logged_in_api).await?;

    Ok(format!("{:#?}", dump))
}

#[launch]
fn rocket() -> Rocket<Build> {
    env_logger::init();

    let settings = read_settings();
    let api = fusionsolar_rs::api(settings.api_url, settings.username, settings.password);
    let state = StateData {
        api,
        interval: settings.interval,
        timestamp: Mutex::new(None),
    };

    rocket::build()
        .manage(state)
        .mount("/", routes![metrics, dump_devices])
}
