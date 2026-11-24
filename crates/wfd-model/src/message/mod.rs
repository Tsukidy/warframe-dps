//! Messages and message keys (server-api.md §2, §4; frontend.md §8; overview.md I-8).
//!
//! Server-produced text is never English: errors, warnings, optimizer logs, legality reasons and import reports are
//! [`Message`]s, `{ key, args, severity? }`, rendered by the frontend through its ICU catalogue. The key type
//! [`MessageKey`] and the argument type [`MsgArg`] live in the T01a module [`crate::i18n`] (domain types carry them,
//! e.g. `Violation.key`); this module re-exports them and adds:
//!
//! - [`Message`] and [`Severity`];
//! - one key enum per area (work-breakdown.md §2.2: `message/<area>.rs`, one file per area): [`ApiKey`] (HTTP errors),
//!   [`DocKey`] (malformed documents), [`EngineKey`] (engine warnings and errors), [`OptKey`] (optimizer),
//!   [`DataKey`] (data status, refresh and validation), [`ImportKey`] (share codes and inspiration links),
//!   [`LegalityKey`] (the rule keys of spec 09);
//! - [`KnownKey`], the union of the area enums, and [`all_keys`], the list `cargo xtask gen-ts` exports to the
//!   frontend (`messageKeys.ts`), so a test can check that every key has an `en.json` entry (I-8).
//!
//! Every area enum converts into [`MessageKey`] (`MessageKey::from(OptKey::JobRunning)`), and declares the argument
//! names its message carries (`OptKey::JobRunning.args() == ["job"]`). Keys are dotted lowercase strings; the area is
//! the owner of the key, not necessarily its first segment (spec 09 rule keys such as `cap.exceeded` belong to the
//! legality area). Later tasks add keys to their area's file with a CONTRACTS.md entry (work-breakdown.md §2.1).

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fmt;

pub use crate::i18n::{MessageKey, MsgArg};

mod api;
mod data;
mod doc;
mod engine;
mod import;
mod legality;
mod opt;

pub use api::ApiKey;
pub use data::DataKey;
pub use doc::DocKey;
pub use engine::EngineKey;
pub use import::ImportKey;
pub use legality::LegalityKey;
pub use opt::OptKey;

/// A server-produced message (server-api.md §4): `{ "key": "opt.job_running", "args": { "job": "opt-7f3a9c" } }`.
/// `args` is always written (possibly `{}`) and defaults to empty when read; `severity` is omitted when absent.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Message {
    /// The i18n key (`"api.not_found"`, `"cap.exceeded"`).
    pub key: MessageKey,
    /// Named arguments of the ICU message: numbers in model units or strings (names, slot ids, keys).
    #[serde(default)]
    pub args: BTreeMap<Box<str>, MsgArg>,
    /// Display severity; absent = the context decides (an error body is an error, a warning list holds warnings).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "ts", ts(optional))]
    pub severity: Option<Severity>,
}

impl Message {
    /// A message without arguments and without severity.
    pub fn new(key: impl Into<MessageKey>) -> Self {
        Message {
            key: key.into(),
            args: BTreeMap::new(),
            severity: None,
        }
    }

    /// Adds (or replaces) a named argument.
    pub fn arg(mut self, name: &str, value: impl Into<MsgArg>) -> Self {
        self.args.insert(name.into(), value.into());
        self
    }

    /// Sets the severity.
    pub fn with_severity(mut self, severity: Severity) -> Self {
        self.severity = Some(severity);
        self
    }

    /// The argument `name`, if present.
    pub fn get(&self, name: &str) -> Option<&MsgArg> {
        self.args.get(name)
    }
}

impl From<f64> for MsgArg {
    fn from(x: f64) -> Self {
        MsgArg::Num(x)
    }
}

impl From<u32> for MsgArg {
    fn from(x: u32) -> Self {
        MsgArg::Num(f64::from(x))
    }
}

impl From<i32> for MsgArg {
    fn from(x: i32) -> Self {
        MsgArg::Num(f64::from(x))
    }
}

impl From<&str> for MsgArg {
    fn from(s: &str) -> Self {
        MsgArg::Str(s.into())
    }
}

impl From<String> for MsgArg {
    fn from(s: String) -> Self {
        MsgArg::Str(s.into())
    }
}

impl From<Box<str>> for MsgArg {
    fn from(s: Box<str>) -> Self {
        MsgArg::Str(s)
    }
}

/// Display severity of a [`Message`] (server-api.md §4: `"info" | "warning" | "error"`).
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum Severity {
    Info,
    Warning,
    Error,
}

/// Declares the key enum of one area: variants with their dotted key and argument names, `ALL`, `AREA`, `as_str`,
/// `args`, `parse`, `message`, `Display` and `From<Enum> for MessageKey`.
macro_rules! message_keys {
    (
        $(#[$m:meta])*
        pub enum $name:ident, area = $area:literal {
            $( $(#[$vm:meta])* $variant:ident = $key:literal [$($arg:ident),*] ),+ $(,)?
        }
    ) => {
        $(#[$m])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub enum $name {
            $( $(#[$vm])* $variant ),+
        }

        impl $name {
            /// Every key of the area, in declaration order.
            pub const ALL: &'static [$name] = &[$($name::$variant),+];
            /// The area name (the `message/<area>.rs` file).
            pub const AREA: &'static str = $area;

            /// The dotted key string.
            pub const fn as_str(self) -> &'static str {
                match self { $($name::$variant => $key),+ }
            }

            /// Names of the arguments the message carries.
            pub const fn args(self) -> &'static [&'static str] {
                match self { $($name::$variant => &[$(stringify!($arg)),*]),+ }
            }

            /// The key of a dotted string, if it belongs to this area.
            pub fn parse(s: &str) -> Option<Self> {
                Self::ALL.iter().copied().find(|k| k.as_str() == s)
            }

            /// A message with this key and no arguments.
            pub fn message(self) -> $crate::message::Message {
                $crate::message::Message::new(self)
            }
        }

        impl ::std::fmt::Display for $name {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                f.write_str(self.as_str())
            }
        }

        impl From<$name> for $crate::i18n::MessageKey {
            fn from(k: $name) -> Self {
                $crate::i18n::MessageKey::new(k.as_str())
            }
        }

        impl PartialEq<$name> for $crate::i18n::MessageKey {
            fn eq(&self, other: &$name) -> bool {
                self.as_str() == other.as_str()
            }
        }
    };
}
pub(crate) use message_keys;

/// A key of any area: the assembled key set of the server.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum KnownKey {
    Api(ApiKey),
    Doc(DocKey),
    Engine(EngineKey),
    Opt(OptKey),
    Data(DataKey),
    Import(ImportKey),
    Legality(LegalityKey),
}

impl KnownKey {
    /// Every key of every area, area by area in the order of the variants above.
    pub fn all() -> Vec<KnownKey> {
        let mut out = Vec::new();
        out.extend(ApiKey::ALL.iter().copied().map(KnownKey::Api));
        out.extend(DocKey::ALL.iter().copied().map(KnownKey::Doc));
        out.extend(EngineKey::ALL.iter().copied().map(KnownKey::Engine));
        out.extend(OptKey::ALL.iter().copied().map(KnownKey::Opt));
        out.extend(DataKey::ALL.iter().copied().map(KnownKey::Data));
        out.extend(ImportKey::ALL.iter().copied().map(KnownKey::Import));
        out.extend(LegalityKey::ALL.iter().copied().map(KnownKey::Legality));
        out
    }

    /// The key of a dotted string (keys are unique across areas).
    pub fn parse(s: &str) -> Option<KnownKey> {
        ApiKey::parse(s)
            .map(KnownKey::Api)
            .or_else(|| DocKey::parse(s).map(KnownKey::Doc))
            .or_else(|| EngineKey::parse(s).map(KnownKey::Engine))
            .or_else(|| OptKey::parse(s).map(KnownKey::Opt))
            .or_else(|| DataKey::parse(s).map(KnownKey::Data))
            .or_else(|| ImportKey::parse(s).map(KnownKey::Import))
            .or_else(|| LegalityKey::parse(s).map(KnownKey::Legality))
    }

    /// The dotted key string.
    pub fn as_str(self) -> &'static str {
        match self {
            KnownKey::Api(k) => k.as_str(),
            KnownKey::Doc(k) => k.as_str(),
            KnownKey::Engine(k) => k.as_str(),
            KnownKey::Opt(k) => k.as_str(),
            KnownKey::Data(k) => k.as_str(),
            KnownKey::Import(k) => k.as_str(),
            KnownKey::Legality(k) => k.as_str(),
        }
    }

    /// Names of the arguments the message carries.
    pub fn args(self) -> &'static [&'static str] {
        match self {
            KnownKey::Api(k) => k.args(),
            KnownKey::Doc(k) => k.args(),
            KnownKey::Engine(k) => k.args(),
            KnownKey::Opt(k) => k.args(),
            KnownKey::Data(k) => k.args(),
            KnownKey::Import(k) => k.args(),
            KnownKey::Legality(k) => k.args(),
        }
    }

    /// The area name (`"api"`, `"legality"` …).
    pub fn area(self) -> &'static str {
        match self {
            KnownKey::Api(_) => ApiKey::AREA,
            KnownKey::Doc(_) => DocKey::AREA,
            KnownKey::Engine(_) => EngineKey::AREA,
            KnownKey::Opt(_) => OptKey::AREA,
            KnownKey::Data(_) => DataKey::AREA,
            KnownKey::Import(_) => ImportKey::AREA,
            KnownKey::Legality(_) => LegalityKey::AREA,
        }
    }
}

impl fmt::Display for KnownKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl From<KnownKey> for MessageKey {
    fn from(k: KnownKey) -> Self {
        MessageKey::new(k.as_str())
    }
}

/// One exported key: area, dotted key, argument names.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KeyInfo {
    /// Area name (`message/<area>.rs`).
    pub area: &'static str,
    /// Dotted key.
    pub key: &'static str,
    /// Argument names.
    pub args: &'static [&'static str],
}

/// Every key the server code emits, in [`KnownKey::all`] order. Keys that curated data defines (toggle labels, item
/// parameter labels, riven trait labels) are not listed: they travel in the bundle and the schema route.
pub fn all_keys() -> Vec<KeyInfo> {
    KnownKey::all()
        .into_iter()
        .map(|k| KeyInfo {
            area: k.area(),
            key: k.as_str(),
            args: k.args(),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    #[test]
    fn keys_are_unique_and_well_formed() {
        let mut seen = BTreeSet::new();
        for k in KnownKey::all() {
            let s = k.as_str();
            assert!(seen.insert(s), "duplicate key {s}");
            assert!(
                !s.is_empty()
                    && s.split('.').all(|seg| {
                        !seg.is_empty()
                            && seg
                                .chars()
                                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
                    }),
                "malformed key {s}"
            );
            assert_eq!(KnownKey::parse(s), Some(k), "{s}");
            let mut args = BTreeSet::new();
            for a in k.args() {
                assert!(args.insert(*a), "{s}: duplicate argument {a}");
            }
        }
        assert_eq!(all_keys().len(), seen.len());
        assert_eq!(KnownKey::parse("no.such.key"), None);
    }

    #[test]
    fn message_json_form() {
        let m = OptKey::JobRunning.message().arg("job", "opt-7f3a9c");
        assert_eq!(m.key, OptKey::JobRunning);
        let json = serde_json::to_string(&m).unwrap();
        assert_eq!(
            json,
            r#"{"key":"opt.job_running","args":{"job":"opt-7f3a9c"}}"#
        );
        let back: Message = serde_json::from_str(&json).unwrap();
        assert_eq!(back, m);
        let w = Message::new(EngineKey::TtkExtrapolated).with_severity(Severity::Warning);
        let json = serde_json::to_string(&w).unwrap();
        assert_eq!(
            json,
            r#"{"key":"ttk.extrapolated","args":{},"severity":"warning"}"#
        );
        // `args` may be omitted on input; unknown fields are rejected.
        let m: Message = serde_json::from_str(r#"{"key":"api.not_found"}"#).unwrap();
        assert!(m.args.is_empty());
        assert!(serde_json::from_str::<Message>(r#"{"key":"x","text":"English"}"#).is_err());
        let n = Message::new(LegalityKey::CapExceeded)
            .arg("used", 72.0)
            .arg("capacity", 60u32);
        assert_eq!(n.get("used"), Some(&MsgArg::Num(72.0)));
        assert_eq!(
            serde_json::to_string(&n).unwrap(),
            r#"{"key":"cap.exceeded","args":{"capacity":60.0,"used":72.0}}"#
        );
    }

    #[test]
    fn spec_09_rule_keys_are_legality_keys() {
        // Every rule key named in spec 09 §1.5 and the E-9 rules is a legality key (09 §1.5: "a stable key … used as
        // the i18n message key").
        for key in [
            "slot.count",
            "slot.kind",
            "compat",
            "compat.type",
            "compat.tags",
            "compat.semi",
            "compat.magazine",
            "arcane.family",
            "arcane.gate",
            "excl.pair",
            "dup.mod",
            "dup.arcane",
            "dup.riven",
            "rank.range",
            "cap.rank",
            "cap.exceeded",
            "shard.count",
            "shard.bonus",
            "helminth.count",
            "helminth.replaced",
            "helminth.host",
            "set.companion",
            "riven.binding",
            "riven.layout",
            "riven.trait",
            "riven.range",
            "riven.physical",
            "incarnon.line",
            "incarnon.tier",
            "incarnon.form",
            "valence.weapon",
            "valence.element",
            "valence.value",
            "valence.unset",
        ] {
            assert!(
                matches!(KnownKey::parse(key), Some(KnownKey::Legality(_))),
                "{key}"
            );
        }
        assert_eq!(LegalityKey::ALL.len(), 34);
    }
}
