//! Serde helpers shared by the model types.
//!
//! Two encodings exist for every model type (domain-model.md §1, pipeline.md §8):
//!
//! - the **human-readable** form (JSON documents and API, curated TOML, `data/generated/*.json`), which uses compact
//!   authoring shapes: flag sets as name lists, the flattened effect target (`stat = "cc_rel"`), untagged values
//!   (`value = 1.65`);
//! - the **compact** form (postcard, the embedded `bundle.bin`), which is positional and not self-describing, so it
//!   must not use untagged, internally tagged or flattened enums, nor skipped fields.
//!
//! Types whose two forms differ derive the plain (compact) form with `#[serde(remote = "Self")]` and get their trait
//! impls from `dual_serde!`, which dispatches on `is_human_readable()` to a `…Repr` type for the readable form.

/// Letter case of flag names in the readable form.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FlagCase {
    /// `HEAVY_SLAM` → `"heavy_slam"` (attack roles, weapon traits, enemy flags, CO flags).
    Lower,
    /// Names as declared (`"SLA"`, `"OG"`, compat flag names).
    Upper,
}

/// Readable name of a flag constant.
pub fn flag_name(name: &str, case: FlagCase) -> String {
    match case {
        FlagCase::Lower => name.to_ascii_lowercase(),
        FlagCase::Upper => name.to_owned(),
    }
}

/// Constant name of a readable flag name.
pub fn flag_const(name: &str, case: FlagCase) -> String {
    match case {
        FlagCase::Lower => name.to_ascii_uppercase(),
        FlagCase::Upper => name.to_owned(),
    }
}

/// Serde, TypeScript and JSON-schema impls for a `bitflags` type: readable form = list of flag names in declaration
/// order (`["heavy", "slam"]`), compact form = the bits. Unknown names are rejected.
/// `opaque` makes the TypeScript form `Array<string>` (used for `CompatFlags`, domain-model.md §9.5).
macro_rules! flags_serde {
    ($ty:ident, $bits:ty, $case:ident) => {
        $crate::serde_util::flags_serde!(@impl $ty, $bits, $case, false);
    };
    ($ty:ident, $bits:ty, $case:ident, opaque) => {
        $crate::serde_util::flags_serde!(@impl $ty, $bits, $case, true);
    };
    (@impl $ty:ident, $bits:ty, $case:ident, $opaque:expr) => {
        impl $ty {
            /// Readable names of the set flags, in declaration order.
            pub fn names(&self) -> ::std::vec::Vec<::std::string::String> {
                self.iter_names()
                    .map(|(n, _)| $crate::serde_util::flag_name(n, $crate::serde_util::FlagCase::$case))
                    .collect()
            }
            /// Every readable flag name of the type, in declaration order.
            pub fn all_names() -> ::std::vec::Vec<::std::string::String> {
                <Self as ::bitflags::Flags>::FLAGS
                    .iter()
                    .map(|f| $crate::serde_util::flag_name(f.name(), $crate::serde_util::FlagCase::$case))
                    .collect()
            }
            /// Parses one readable flag name.
            pub fn from_flag_name(name: &str) -> Option<Self> {
                Self::from_name(&$crate::serde_util::flag_const(name, $crate::serde_util::FlagCase::$case))
            }
        }
        impl ::serde::Serialize for $ty {
            fn serialize<S: ::serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
                if s.is_human_readable() {
                    ::serde::Serialize::serialize(&self.names(), s)
                } else {
                    ::serde::Serialize::serialize(&self.bits(), s)
                }
            }
        }
        impl<'de> ::serde::Deserialize<'de> for $ty {
            fn deserialize<D: ::serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
                if d.is_human_readable() {
                    let names: ::std::vec::Vec<::std::string::String> = ::serde::Deserialize::deserialize(d)?;
                    let mut out = Self::empty();
                    for n in &names {
                        match Self::from_flag_name(n) {
                            Some(f) => out |= f,
                            None => {
                                return Err(<D::Error as ::serde::de::Error>::custom(format!(
                                    "unknown {} flag `{}` (expected one of {:?})",
                                    stringify!($ty),
                                    n,
                                    Self::all_names()
                                )));
                            }
                        }
                    }
                    Ok(out)
                } else {
                    let bits: $bits = ::serde::Deserialize::deserialize(d)?;
                    Ok(Self::from_bits_retain(bits))
                }
            }
        }
        #[cfg(feature = "ts")]
        impl ::ts_rs::TS for $ty {
            type WithoutGenerics = Self;
            type OptionInnerType = Self;
            fn name(_: &::ts_rs::Config) -> ::std::string::String {
                let names = Self::all_names();
                if $opaque || names.is_empty() {
                    "Array<string>".to_owned()
                } else {
                    let alts: ::std::vec::Vec<::std::string::String> =
                        names.iter().map(|n| format!("\"{n}\"")).collect();
                    format!("Array<{}>", alts.join(" | "))
                }
            }
            fn inline(cfg: &::ts_rs::Config) -> ::std::string::String {
                <Self as ::ts_rs::TS>::name(cfg)
            }
        }
        #[cfg(feature = "schema")]
        impl ::schemars::JsonSchema for $ty {
            fn schema_name() -> ::std::borrow::Cow<'static, str> {
                stringify!($ty).into()
            }
            fn json_schema(_: &mut ::schemars::SchemaGenerator) -> ::schemars::Schema {
                let names = Self::all_names();
                if $opaque || names.is_empty() {
                    ::schemars::json_schema!({ "type": "array", "items": { "type": "string" }, "uniqueItems": true })
                } else {
                    ::schemars::json_schema!({
                        "type": "array",
                        "items": { "type": "string", "enum": names },
                        "uniqueItems": true
                    })
                }
            }
        }
    };
}
pub(crate) use flags_serde;

/// Serde and JSON-schema impls for a type with a plain derived form (`#[serde(remote = "Self")]`) and a readable
/// `Repr` form. Requires `impl From<&T> for Repr` and `impl TryFrom<Repr> for T` with a `Display` error.
/// TypeScript: derive `TS` on the type with `#[ts(as = "Repr")]`.
macro_rules! dual_serde {
    ($ty:ty, $repr:ty) => {
        impl ::serde::Serialize for $ty {
            fn serialize<S: ::serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
                if s.is_human_readable() {
                    ::serde::Serialize::serialize(&<$repr>::from(self), s)
                } else {
                    <$ty>::serialize(self, s)
                }
            }
        }
        impl<'de> ::serde::Deserialize<'de> for $ty {
            fn deserialize<D: ::serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
                if d.is_human_readable() {
                    let r = <$repr as ::serde::Deserialize>::deserialize(d)?;
                    <$ty>::try_from(r).map_err(<D::Error as ::serde::de::Error>::custom)
                } else {
                    <$ty>::deserialize(d)
                }
            }
        }
        #[cfg(feature = "schema")]
        impl ::schemars::JsonSchema for $ty {
            fn schema_name() -> ::std::borrow::Cow<'static, str> {
                <$repr as ::schemars::JsonSchema>::schema_name()
            }
            fn schema_id() -> ::std::borrow::Cow<'static, str> {
                <$repr as ::schemars::JsonSchema>::schema_id()
            }
            fn json_schema(g: &mut ::schemars::SchemaGenerator) -> ::schemars::Schema {
                <$repr as ::schemars::JsonSchema>::json_schema(g)
            }
            fn inline_schema() -> bool {
                <$repr as ::schemars::JsonSchema>::inline_schema()
            }
        }
    };
}
pub(crate) use dual_serde;

/// Serde, TypeScript and JSON-schema impls for a type that is a string in every encoding, through `Display` and
/// `FromStr` (stat keys, resolved-stat names). `$schema_values` lists the legal strings for the JSON schema.
macro_rules! string_serde {
    ($ty:ty, $schema_values:expr) => {
        impl ::serde::Serialize for $ty {
            fn serialize<S: ::serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
                s.collect_str(self)
            }
        }
        impl<'de> ::serde::Deserialize<'de> for $ty {
            fn deserialize<D: ::serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
                let s: ::std::borrow::Cow<'de, str> = ::serde::Deserialize::deserialize(d)?;
                s.parse::<$ty>().map_err(<D::Error as ::serde::de::Error>::custom)
            }
        }
        #[cfg(feature = "ts")]
        impl ::ts_rs::TS for $ty {
            type WithoutGenerics = Self;
            type OptionInnerType = Self;
            fn name(_: &::ts_rs::Config) -> ::std::string::String {
                "string".to_owned()
            }
            fn inline(cfg: &::ts_rs::Config) -> ::std::string::String {
                <Self as ::ts_rs::TS>::name(cfg)
            }
        }
        #[cfg(feature = "schema")]
        impl ::schemars::JsonSchema for $ty {
            fn schema_name() -> ::std::borrow::Cow<'static, str> {
                stringify!($ty).into()
            }
            fn json_schema(_: &mut ::schemars::SchemaGenerator) -> ::schemars::Schema {
                let values: ::std::vec::Vec<::std::string::String> = $schema_values;
                ::schemars::json_schema!({ "type": "string", "enum": values })
            }
        }
    };
}
pub(crate) use string_serde;

/// `skip_serializing_if` helper for documents (never used on bundle types, which postcard encodes positionally).
pub fn is_false(b: &bool) -> bool {
    !*b
}

/// `skip_serializing_if` helper for documents.
pub fn is_true(b: &bool) -> bool {
    *b
}

/// Serde default helper.
pub fn default_true() -> bool {
    true
}

/// Serde default helper.
pub fn default_one_f64() -> f64 {
    1.0
}

/// Serde default helper.
pub fn default_one_u16() -> u16 {
    1
}

/// Field codec for an `f64` that may be infinite (Lanka's `FireRate = math.huge`, 06 §4.1 and EC-06-06; robotic
/// `AmmoMax`, 06 §11.4): the readable form (JSON, TOML) writes +∞ and −∞ as the strings `"Infinity"` and `"-Infinity"`
/// (the spelling of pipeline.md §4.1) and finite values as numbers, and reads both (a TOML `inf` float too); NaN is
/// rejected. The compact form is the plain `f64`. Use `#[serde(with = "crate::serde_util::ext_f64")]` together with
/// `#[ts(type = "number | \"Infinity\" | \"-Infinity\"")]` and `#[schemars(with = "crate::serde_util::ExtF64")]`.
pub mod ext_f64 {
    use serde::de::Error as _;
    use serde::ser::Error as _;
    use serde::{Deserialize, Deserializer, Serializer};

    /// Readable spelling of +∞.
    pub const INFINITY: &str = "Infinity";
    /// Readable spelling of −∞.
    pub const NEG_INFINITY: &str = "-Infinity";

    /// Serializes `x` (see the module documentation).
    pub fn serialize<S: Serializer>(x: &f64, s: S) -> Result<S::Ok, S::Error> {
        if !s.is_human_readable() || x.is_finite() {
            s.serialize_f64(*x)
        } else if x.is_nan() {
            Err(S::Error::custom("NaN is not a valid model value"))
        } else if *x > 0.0 {
            s.serialize_str(INFINITY)
        } else {
            s.serialize_str(NEG_INFINITY)
        }
    }

    #[derive(Deserialize)]
    #[serde(untagged)]
    enum Readable {
        Num(f64),
        Str(String),
    }

    /// Deserializes a value written by [`serialize`].
    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<f64, D::Error> {
        if !d.is_human_readable() {
            return f64::deserialize(d);
        }
        let x = match Readable::deserialize(d)? {
            Readable::Num(x) => x,
            Readable::Str(s) if s == INFINITY => f64::INFINITY,
            Readable::Str(s) if s == NEG_INFINITY => f64::NEG_INFINITY,
            Readable::Str(s) => {
                return Err(D::Error::custom(format!(
                    "expected a number, \"{INFINITY}\" or \"{NEG_INFINITY}\", found \"{s}\""
                )));
            }
        };
        if x.is_nan() {
            return Err(D::Error::custom("NaN is not a valid model value"));
        }
        Ok(x)
    }

    /// The same codec for `Option<f64>` (`None` is `null` in JSON and absent in TOML).
    pub mod option {
        use serde::{Deserialize, Deserializer, Serialize, Serializer};

        #[derive(Serialize, Deserialize)]
        #[serde(transparent)]
        struct Wrap(#[serde(with = "super")] f64);

        /// Serializes `x`.
        pub fn serialize<S: Serializer>(x: &Option<f64>, s: S) -> Result<S::Ok, S::Error> {
            x.map(Wrap).serialize(s)
        }

        /// Deserializes a value written by [`serialize`].
        pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<Option<f64>, D::Error> {
            Ok(Option::<Wrap>::deserialize(d)?.map(|w| w.0))
        }
    }
}

/// JSON schema of [`ext_f64`]: a number or one of the strings `"Infinity"`, `"-Infinity"`.
#[cfg(feature = "schema")]
pub struct ExtF64;

#[cfg(feature = "schema")]
impl schemars::JsonSchema for ExtF64 {
    fn schema_name() -> std::borrow::Cow<'static, str> {
        "ExtF64".into()
    }
    fn json_schema(_: &mut schemars::SchemaGenerator) -> schemars::Schema {
        schemars::json_schema!({
            "oneOf": [
                { "type": "number" },
                { "enum": [ext_f64::INFINITY, ext_f64::NEG_INFINITY] }
            ]
        })
    }
}
