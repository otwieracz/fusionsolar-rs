use serde::Deserialize;
use serde_json::Value;
use std::convert::From;

#[derive(Debug, Clone, Copy)]
pub enum DeviceTypeId {
    UnsupportedDeviceType,
    StringInverter = 1,
}

impl<'de> serde::Deserialize<'de> for DeviceTypeId {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let value = Value::deserialize(d)?;

        match Value::as_u64(&value) {
            Some(v) => {
                let type_id: DeviceTypeId = v.into();
                Ok(type_id)
            }
            None => todo!(),
        }
    }
}

impl From<u64> for DeviceTypeId {
    fn from(value: u64) -> Self {
        match value {
            type_id if type_id == DeviceTypeId::StringInverter as u64 => {
                DeviceTypeId::StringInverter
            }
            _ => DeviceTypeId::UnsupportedDeviceType,
        }
    }
}

mod get_stations_list {
    use serde::Deserialize;

    #[derive(Deserialize)]
    pub struct Data {
        #[serde(rename = "stationCode")]
        pub station_code: String,
        #[serde(rename = "stationName")]
        pub station_name: String,
        pub capacity: f64,
    }

    #[derive(Deserialize)]
    pub struct GetStationsList {
        pub data: Vec<Data>,
    }
}

mod get_station_real_kpi {
    #[derive(serde::Deserialize)]
    pub struct DataItemMap {
        pub day_power: f64,
    }
    #[derive(serde::Deserialize)]
    pub struct Data {
        #[serde(rename = "dataItemMap")]
        pub data_item_map: DataItemMap,
        #[serde(rename = "stationCode")]
        pub station_code: String,
    }
    #[derive(serde::Deserialize)]
    pub struct GetStationRealKpi {
        pub data: Vec<Data>,
    }
}

mod get_device_list {
    use super::DeviceTypeId;
    use serde::Deserialize;

    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Data {
        pub dev_name: String,
        pub id: u64,
        pub dev_type_id: DeviceTypeId,
    }

    #[derive(Deserialize)]
    pub struct GetDevicesList {
        pub data: Vec<Data>,
    }
}

pub mod get_device_real_kpi {
    use super::DeviceTypeId;
    use serde::Deserialize;
    use serde_json::Value;
    use std::convert::Into;

    pub mod string_inverter {
        use serde::Deserialize;

        #[derive(Deserialize)]
        pub struct DataItemMap {
            pub temperature: f64,
            pub active_power: f64,
            pub mppt_power: f64,
        }

        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        pub struct Data {
            pub dev_id: u64,
            pub data_item_map: DataItemMap,
        }
    }

    #[derive(Deserialize)]
    pub struct StringInverter {
        pub data: Vec<string_inverter::Data>,
    }

    pub enum GetDeviceRealKpi {
        StringInverter(StringInverter),
    }

    impl<'de> serde::Deserialize<'de> for GetDeviceRealKpi {
        fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
            let value = Value::deserialize(d)?;

            match value
                .get("params")
                .and_then(|v| v.get("devTypeId"))
                .and_then(Value::as_u64)
            {
                Some(tag) => match tag.into() {
                    DeviceTypeId::StringInverter => Ok(GetDeviceRealKpi::StringInverter(
                        StringInverter::deserialize(value).unwrap(),
                    )),
                    DeviceTypeId::UnsupportedDeviceType => Err(serde::de::Error::custom(format!(
                        "Unsupported GetDeviceRealKpi device type: {}",
                        tag
                    ))),
                },
                None => todo!(),
            }
        }
    }
}

/* Generic success */
#[derive(Deserialize)]
pub struct SuccessResponse {
    pub success: bool,
}

/* Generic error */
#[derive(Deserialize)]
pub struct ErrorResponse {
    #[serde(rename = "failCode")]
    pub fail_code: u32,
    pub message: Option<String>,
}

#[derive(Deserialize)]
#[serde(untagged)]
pub enum FusionsolarApiResponse {
    GetStationsList(get_stations_list::GetStationsList),
    GetStationRealKpi(get_station_real_kpi::GetStationRealKpi),
    GetDevicesList(get_device_list::GetDevicesList),
    GetDeviceRealKpi(get_device_real_kpi::GetDeviceRealKpi),
    Success(SuccessResponse),
    Error(ErrorResponse),
}

#[cfg(test)]
mod test {
    use super::get_device_real_kpi::GetDeviceRealKpi;
    use std::fs;
    use std::path::PathBuf;

    fn read_resource(filename: &str) -> String {
        let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        d.push(format!("resources/test/{}", filename));
        fs::read_to_string(d.as_path()).unwrap()
    }

    #[test]
    fn get_stations_list() {
        let input = read_resource("getStationList.json");
        let output: super::get_stations_list::GetStationsList =
            serde_json::from_str(&input).unwrap();
        assert_eq!("StationCode", output.data[0].station_code);
        assert_eq!("StationName", output.data[0].station_name);
        assert_eq!(0.005, output.data[0].capacity);
    }

    #[test]
    fn get_device_real_kpi() {
        let input = read_resource("getDeviceRealKpi.json");
        let output: GetDeviceRealKpi = serde_json::from_str(&input).unwrap();
        match output {
            GetDeviceRealKpi::StringInverter(i) => {
                assert_eq!(2.053, i.data[0].data_item_map.active_power);
            }
        }
    }

    #[test]
    #[should_panic]
    fn get_device_real_kpi_unsupported() {
        let input = read_resource("getDeviceRealKpi_Unsupported.json");
        let output: GetDeviceRealKpi = serde_json::from_str(&input).unwrap();
        match output {
            GetDeviceRealKpi::StringInverter(i) => {
                assert_eq!(2.053, i.data[0].data_item_map.active_power);
            }
        }
    }
}