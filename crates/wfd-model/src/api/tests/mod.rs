//! Tests of the API DTOs (work-breakdown.md T01b acceptance): the JSON examples of server-api.md parse, every API type
//! round-trips, the generated TypeScript matches server-api.md and accepts what serde writes, no `bigint`.

mod examples;
mod samples;
mod shapes;
mod ts;

use serde_json::Value;

/// Every number as an f64 (serde_json distinguishes `1` from `1.0`; the API does not).
pub(super) fn normalize_numbers(v: &Value) -> Value {
    match v {
        Value::Number(n) => serde_json::Number::from_f64(n.as_f64().unwrap_or(f64::NAN))
            .map(Value::Number)
            .unwrap_or(Value::Null),
        Value::Array(a) => Value::Array(a.iter().map(normalize_numbers).collect()),
        Value::Object(m) => Value::Object(
            m.iter()
                .map(|(k, x)| (k.clone(), normalize_numbers(x)))
                .collect(),
        ),
        x => x.clone(),
    }
}
