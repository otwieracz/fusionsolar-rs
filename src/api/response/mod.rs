use num_derive::FromPrimitive;

pub mod get_device_list;
pub mod get_device_real_kpi;
pub mod get_station_real_kpi;
pub mod get_stations_list;

#[derive(FromPrimitive)]
pub enum FailCode {
    AccessFrequencyIsTooHigh = 407,
}

#[cfg(test)]
mod test {
    use super::get_device_real_kpi::StringInverter;
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
        let output: StringInverter = serde_json::from_str(&input).unwrap();
        assert_eq!(2.053, output.data[0].data_item_map.active_power);
    }

    #[test]
    #[should_panic]
    fn get_device_real_kpi_unsupported() {
        let unsupported_type = read_resource("getDeviceRealKpi_Unsupported.json");
        serde_json::from_str::<StringInverter>(&unsupported_type).unwrap();
    }

    #[test]
    #[should_panic]
    fn get_device_real_kpi_valid_json() {
        let valid_json_input = read_resource("valid_json.json");
        serde_json::from_str::<StringInverter>(&valid_json_input).unwrap();
    }

    #[test]
    #[should_panic]
    fn get_device_real_kpi_invalid_json() {
        let invalid_json_input = read_resource("invalid_json.json");
        serde_json::from_str::<StringInverter>(&invalid_json_input).unwrap();
    }
}
