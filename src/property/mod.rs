use log::warn;

use crate::{hostcalls, log_concern};

pub mod all;
pub mod envoy;

pub fn get_property(name: impl AsRef<str>) -> Option<Vec<u8>> {
    log_concern(
        "get-property",
        hostcalls::get_property(name.as_ref().split('.')),
    )
}

pub fn get_property_string(name: impl AsRef<str>) -> Option<String> {
    get_property(name).map(|x| String::from_utf8_lossy(&x).into_owned())
}

pub fn set_property(name: impl AsRef<str>, value: impl AsRef<[u8]>) {
    log_concern(
        "set-property",
        hostcalls::set_property(name.as_ref().split('.'), Some(value.as_ref())),
    );
}

pub fn get_property_int(name: &str) -> Option<i64> {
    let raw = get_property(name)?;
    if raw.len() != 8 {
        return None;
    }
    Some(i64::from_le_bytes(raw.try_into().unwrap()))
}

pub fn get_property_bool(name: &str) -> Option<bool> {
    let raw = get_property(name)?;
    if raw.is_empty() || raw.len() != 1 {
        return None;
    }
    Some(raw[0] != 0)
}

pub fn get_property_decode<P: prost::Message + Default>(name: &str) -> Option<P> {
    let raw = get_property(name)?;
    match P::decode(&raw[..]) {
        Ok(x) => Some(x),
        Err(e) => {
            warn!("failed to decode property '{name}': {e:?}");
            None
        }
    }
}
