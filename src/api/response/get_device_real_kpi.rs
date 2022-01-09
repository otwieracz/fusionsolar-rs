use crate::api::response::device_type::DeviceTypeId;
use serde::Deserialize;
use serde_json::Value;

/* Device Type 1: String Inverter */
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
        let data = Value::deserialize(d)?;

        let device_type_id = data
            .get("params")
            .and_then(|v| v.get("devTypeId"))
            .and_then(Value::as_u64)
            .ok_or_else(|| serde::de::Error::missing_field("devTypeId"))?;

        /* Deserialize into variant of `GetDeviceRealKpi` depending on `.params.devTypeId` */
        match num::FromPrimitive::from_u64(device_type_id)
            .ok_or_else(|| serde::de::Error::custom("unexpected error"))?
        {
            /* TODO: unwraps here */
            DeviceTypeId::StringInverter => Ok(GetDeviceRealKpi::StringInverter(
                StringInverter::deserialize(data).unwrap(),
            )),
            DeviceTypeId::UnsupportedDeviceType => Err(serde::de::Error::custom(format!(
                "Unsupported GetDeviceRealKpi device type: {}",
                device_type_id
            ))),
        }
    }
}
