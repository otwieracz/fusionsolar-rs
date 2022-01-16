use crate::model::{DeviceTypeId, SupportedDeviceTypeId};
use serde_json::Value;

pub fn from_u64(v: u64) -> DeviceTypeId {
    match v {
        1 => DeviceTypeId::SupportedDeviceTypeId(SupportedDeviceTypeId::StringInverter),
        _ => DeviceTypeId::UnsupportedDeviceTypeId(v),
    }
}

impl<'de> serde::Deserialize<'de> for DeviceTypeId {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let value = Value::deserialize(d)?;

        Value::as_u64(&value)
            .ok_or_else(|| serde::de::Error::missing_field("deviceTypeId"))
            .map(from_u64)
    }
}
