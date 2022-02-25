use fusionsolar_rs::api;
use fusionsolar_rs::model::{Api, DeviceRealKpi, DeviceTypeId, LoggedInApi, Station};
use prometheus::{Encoder, GaugeVec, TextEncoder};

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

/// Process DeviceRealKpi `device_real_kpi` of `device` installed in `station` and feed them to
/// Prometheus metrics. Based on device type, different KPIs can be presented.
fn process_device_real_kpi(
    dev_real_kpi: &DeviceRealKpi,
    station: &Station,
    device: fusionsolar_rs::model::Device,
) {
    if (num::FromPrimitive::from_u64(device.type_id) as Option<DeviceTypeId>).is_some() {
        if let Some(active_power) = dev_real_kpi.active_power {
            DEVICE_ACTIVE_POWER_GAUGE
                .with_label_values(&[
                    &station.code,
                    &dev_real_kpi.id.to_string(),
                    &(device.type_id).to_string(),
                ])
                .set(active_power);
        }

        if let Some(temperature) = dev_real_kpi.temperature {
            DEVICE_TEMPERAURE_GAUGE
                .with_label_values(&[
                    &station.code,
                    &dev_real_kpi.id.to_string(),
                    &(device.type_id).to_string(),
                ])
                .set(temperature);
        }
    }
}

/// Iterate through all devices within station and collect KPI for supported ones.
async fn collect_station_devices(api: &LoggedInApi, station: &Station) -> Result<(), api::Error> {
    let devices = api::devices(api, station).await?;

    for device in devices {
        if let Ok(dev_kpi_vec) = api::device_real_kpi(api, &device).await {
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

/// Collect `day_power` metric for every station.
async fn collect_day_power(api: &LoggedInApi) -> Result<(), api::Error> {
    let stations = api::stations(api).await?;

    for station in stations {
        let kpi = api::station_real_kpi(api, &station).await?;

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

/// Collect all supported metrics from `api`, updating Prometheus exporter registry.
pub async fn collect(api: &Api) -> Result<(), api::Error> {
    let logged_in_api = api::login(api).await?;
    collect_day_power(&logged_in_api).await?;

    Ok(())
}

/// Read metrics from Prometheus exporter registry.
pub async fn read() -> Result<String, api::Error> {
    // Gather the metrics.
    let mut buffer = Vec::new();
    let encoder = TextEncoder::new();
    let metric_families = prometheus::gather();

    encoder.encode(&metric_families, &mut buffer).unwrap();
    String::from_utf8(buffer).or(Err(api::Error::FormatError))
}
