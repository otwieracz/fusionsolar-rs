#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate prometheus;
#[macro_use]
extern crate rocket;

use config::Config;
use fusionsolar_rs::api;
use fusionsolar_rs::model::Api;
use rocket::{Build, Rocket, State};
use std::sync::Mutex;
use std::time::Instant;

mod metrics;

const API_URL: &str = "https://eu5.fusionsolar.huawei.com/thirdData";

#[derive(Clone, serde::Deserialize)]
pub struct FusionsolarConfig {
    api_url: String,
    username: String,
    password: String,
    interval: u64,
}

/// Structure containing state for API handlers.
pub struct StateData {
    api: Api,
    interval: u64,
    /// Timestamp of last successful metric collection via `metrics::collect()`
    timestamp: Mutex<Option<Instant>>,
}

impl StateData {
    /// Updates `timestamp` to `now()`.
    fn touch(&self) {
        if let Ok(mut ts) = self.timestamp.lock() {
            *ts = Some(Instant::now());
        } else {
            log::trace!("Unable to lock timestamp mutex, will refresh again")
        }
    }

    /// Checks whether `interval_seconds` elapsed since last `touch()`
    fn interval_elapsed(&self, interval_secs: u64) -> bool {
        let elapsed_opt = self
            .timestamp
            .lock()
            .ok()
            .and_then(|a| a.map(|b| b.elapsed().as_secs()));

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
        .unwrap()
        .set_default("api_url", API_URL)
        .unwrap();

    settings.try_into().expect("Configuration error")
}

#[get("/metrics")]
async fn metrics_route(state: &State<StateData>) -> Result<String, api::Error> {
    if state.interval_elapsed(state.interval) {
        metrics::collect(&state.api).await?;
        state.touch();
    } else {
        log::info!("interval time not yet elapsed since last run; returning cached result")
    }
    metrics::read().await
}

#[get("/dump-devices")]
async fn dump_devices_route(state: &State<StateData>) -> Result<String, api::Error> {
    let logged_in_api = api::login(&state.api).await?;
    let dump = api::dump_devices(&logged_in_api).await?;

    Ok(format!("{:#?}", dump))
}

#[launch]
fn rocket() -> Rocket<Build> {
    env_logger::init();

    let settings = read_settings();
    let api = api::api(settings.api_url, settings.username, settings.password);
    let state = StateData {
        api,
        interval: settings.interval,
        timestamp: Mutex::new(None),
    };

    rocket::build()
        .manage(state)
        .mount("/", routes![metrics_route, dump_devices_route])
}
