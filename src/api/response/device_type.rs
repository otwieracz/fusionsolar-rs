use num_derive::FromPrimitive;
use serde_json::Value;

#[derive(Debug, Clone, Copy, FromPrimitive)]
pub enum DeviceTypeId {
    UnsupportedDeviceType,
    StringInverter = 1,
}

impl<'de> serde::Deserialize<'de> for DeviceTypeId {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let value = Value::deserialize(d)?;

        Value::as_u64(&value)
            .ok_or_else(|| serde::de::Error::missing_field("deviceTypeId"))
            .map(|v| match num::FromPrimitive::from_u64(v) {
                Some(device_type_id) => device_type_id,
                None => DeviceTypeId::UnsupportedDeviceType,
            })
    }
}
