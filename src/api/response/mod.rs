pub mod device_type;
mod get_device_list;
pub mod get_device_real_kpi;
mod get_station_real_kpi;
mod get_stations_list;

use serde::Deserialize;

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

/* Valid response types */
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
    fn get_dev_list() {
        let input = read_resource("getDevList.json");
        let output: super::get_device_list::GetDevicesList = serde_json::from_str(&input).unwrap();
        assert_eq!("devName1", output.data[0].dev_name);
        assert_eq!("devName2", output.data[1].dev_name);
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
        let unsupported_type = read_resource("getDeviceRealKpi_Unsupported.json");
        let unsupported_type_output: GetDeviceRealKpi =
            serde_json::from_str(&unsupported_type).unwrap();
        match unsupported_type_output {
            GetDeviceRealKpi::StringInverter(i) => {
                assert_eq!(2.053, i.data[0].data_item_map.active_power);
            }
        }
    }

    #[test]
    #[should_panic]
    fn get_device_real_kpi_valid_json() {
        let valid_json_input = read_resource("valid_json.json");
        let _valid_json_output: GetDeviceRealKpi = serde_json::from_str(&valid_json_input).unwrap();
    }

    #[test]
    #[should_panic]
    fn get_device_real_kpi_invalid_json() {
        let invalid_json_input = read_resource("invalid_json.json");
        let _invalid_json_output: GetDeviceRealKpi =
            serde_json::from_str(&invalid_json_input).unwrap();
    }
}
