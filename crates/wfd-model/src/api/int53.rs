//! Field codec for `u64` values of API types (domain-model.md §1 "Integers in TypeScript"): the JSON carries a plain
//! number, which a browser holds exactly only up to 2^53 − 1, so values above [`MAX`] are rejected when written and
//! when read. Use `#[serde(with = "crate::api::int53")]` with `#[ts(type = "number")]` (and
//! `#[schemars(with = "u64")]`); `int53::option` is the same for `Option<u64>`.

use serde::de::Error as _;
use serde::ser::Error as _;
use serde::{Deserialize, Deserializer, Serializer};

/// Largest integer a JSON number carries exactly in a browser: 2^53 − 1 (= [`crate::MAX_SEED`]).
pub const MAX: u64 = crate::scenario::MAX_SEED;

/// Serializes `x`, failing above [`MAX`].
pub fn serialize<S: Serializer>(x: &u64, s: S) -> Result<S::Ok, S::Error> {
    if *x > MAX {
        return Err(S::Error::custom(format!("{x} exceeds 2^53 − 1")));
    }
    s.serialize_u64(*x)
}

/// Deserializes an integer 0 ..= [`MAX`].
pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<u64, D::Error> {
    let x = u64::deserialize(d)?;
    if x > MAX {
        return Err(D::Error::custom(format!("{x} exceeds 2^53 − 1")));
    }
    Ok(x)
}

/// The same codec for `Option<u64>` (`None` = `null`, or absent with `skip_serializing_if`).
pub mod option {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    #[derive(Serialize, Deserialize)]
    #[serde(transparent)]
    struct Wrap(#[serde(with = "super")] u64);

    /// Serializes `x`.
    pub fn serialize<S: Serializer>(x: &Option<u64>, s: S) -> Result<S::Ok, S::Error> {
        x.map(Wrap).serialize(s)
    }

    /// Deserializes a value written by [`serialize`].
    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<Option<u64>, D::Error> {
        Ok(Option::<Wrap>::deserialize(d)?.map(|w| w.0))
    }
}

#[cfg(test)]
mod tests {
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    struct T {
        #[serde(with = "super")]
        a: u64,
        #[serde(default, with = "super::option")]
        b: Option<u64>,
    }

    #[test]
    fn range_is_checked_both_ways() {
        let t: T = serde_json::from_str(r#"{"a":9007199254740991,"b":null}"#).unwrap();
        assert_eq!(t.a, super::MAX);
        assert_eq!(t.b, None);
        assert!(serde_json::from_str::<T>(r#"{"a":9007199254740992}"#).is_err());
        assert!(serde_json::from_str::<T>(r#"{"a":1,"b":9007199254740992}"#).is_err());
        assert!(serde_json::from_str::<T>(r#"{"a":-1}"#).is_err());
        let t: T = serde_json::from_str(r#"{"a":1}"#).unwrap();
        assert_eq!(serde_json::to_string(&t).unwrap(), r#"{"a":1,"b":null}"#);
        assert!(
            serde_json::to_string(&T {
                a: super::MAX + 1,
                b: None
            })
            .is_err()
        );
    }
}
