//! Server-sent-event framing of the internally tagged event enums ([`super::OptEvent`], [`super::DataEvent`];
//! server-api.md §8.2, §9): the `event:` field carries the tag, `data:` the JSON of the remaining fields.

use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::Value;

/// Name of the tag field.
pub(crate) const TAG: &str = "event";

/// Splits an event into its SSE name and the JSON of its fields.
pub(crate) fn to_sse<T: Serialize>(
    e: &T,
    name: &'static str,
) -> Result<(&'static str, String), serde_json::Error> {
    let mut v = serde_json::to_value(e)?;
    let obj = v.as_object_mut().ok_or_else(|| {
        <serde_json::Error as serde::de::Error>::custom("event is not a JSON object")
    })?;
    match obj.remove(TAG) {
        Some(Value::String(tag)) if tag == name => {}
        other => {
            return Err(<serde_json::Error as serde::de::Error>::custom(format!(
                "event tag {other:?} differs from the SSE name {name}"
            )));
        }
    }
    Ok((name, serde_json::to_string(&v)?))
}

/// Joins an SSE name and data JSON into an event. A `data:` object that already holds the tag must hold the same one.
pub(crate) fn from_sse<T: DeserializeOwned>(
    event: &str,
    data: &str,
) -> Result<T, serde_json::Error> {
    let mut v: Value = serde_json::from_str(data)?;
    let obj = v.as_object_mut().ok_or_else(|| {
        <serde_json::Error as serde::de::Error>::custom("SSE data is not a JSON object")
    })?;
    match obj.get(TAG) {
        None => {
            obj.insert(TAG.into(), Value::String(event.into()));
        }
        Some(Value::String(tag)) if tag == event => {}
        Some(other) => {
            return Err(<serde_json::Error as serde::de::Error>::custom(format!(
                "SSE data tag {other} differs from the event name {event}"
            )));
        }
    }
    serde_json::from_value(v)
}
