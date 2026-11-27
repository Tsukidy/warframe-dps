//! A candidate for one slot or position of a build (engine.md §1 `Pick`): the unit the engine compiles at L1
//! (`Engine::compile_pick`, T02a) and the unit `wfd-data::legality::check_slot` checks against a build (spec 09;
//! work-breakdown.md T02b, T04), so the optimizer's pools (optimizer.md §2 step 2) hand the same value to both.
//!
//! The type lives in `wfd-model`, not in `wfd-engine` as engine.md §1 writes it, because `wfd-data` and `wfd-engine`
//! both depend on `wfd-model` and not on each other (overview.md §3.1): a `Pick` defined in the engine could not be
//! named by `check_slot`. The shape is exactly engine.md §1's. Request T02b-2 (CONTRACTS.md T02b) asks the manager to
//! confirm this placement before T02a and T02b merge: `wfd-engine` then re-exports this type instead of defining an
//! enum of its own, so the optimizer hands one value to both functions.
//!
//! Which slot each variant goes into is the table of `wfd_data::legality::check_slot`. There is no variant for the
//! Helminth slot (`SlotId::Subsumed`; engine.md §1 has none), and the variant parts (`KitgunParts`, `ZawParts`,
//! `Progenitor`) fill `Build.variant`, which no `SlotId` names.

use crate::enums::DamageType;
use crate::ids::{ComboKey, ItemId, ParamKey};
use crate::item::{ParamValue, Riven};
use serde::{Deserialize, Serialize};
use smallvec::SmallVec;

/// One candidate content of a build position, with dense ids (resolved form, domain-model.md §9.2). Never persisted
/// and never sent over HTTP (dense ids stay in one loaded bundle); the serde derives follow the model convention
/// (domain-model.md §1). JSON form: `snake_case` variant names, e.g. `{ "arcane": { "item": 3, "rank": 5 } }`.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Pick {
    /// A weapon mod in a general or exilus slot (09 §2.1, E-9.3).
    Mod {
        /// The mod.
        item: ItemId,
        /// Its rank (09 §5.1).
        rank: u8,
        /// Item parameter values (an element choice, a user value).
        params: SmallVec<[(ParamKey, ParamValue); 1]>,
    },
    /// A riven in a general slot (09 §12; a riven is not a slot, 09 §2.1).
    Riven(Riven),
    /// An arcane: a weapon arcane in `arcane0` / `arcane1` or a warframe arcane in `farcane0` / `farcane1` (09 §8,
    /// E-9.30). The slot the caller names tells which; the arcane's family must fit that slot (E-9.30, `warframe`
    /// for `farcane`).
    Arcane {
        /// The arcane.
        item: ItemId,
        /// Its rank.
        rank: u8,
    },
    /// An Incarnon evolution perk (09 §13): `index` into `IncarnonDef.tiers[tier - 1]`, slot `inc<tier>`.
    Perk {
        /// Evolution 2..=5.
        tier: u8,
        /// Perk index within the evolution.
        index: u8,
    },
    /// An Archon shard on the warframe (09 §9).
    Shard {
        /// Colour and bonus.
        item: ItemId,
        /// Tauforged shard.
        tauforged: bool,
    },
    /// A stance with the combo type its light loop uses (09 §2.3, 07 E-7.0b, §8.1).
    Stance {
        /// The stance mod.
        item: ItemId,
        /// The selected combo (07 E-7.0b slug).
        combo: ComboKey,
    },
    /// A warframe mod in `fgen0` … `fgen7` or `fexilus` (09 §2.1).
    FrameMod {
        /// The mod.
        item: ItemId,
        /// Its rank.
        rank: u8,
    },
    /// An aura in `aura0` / `aura1` (09 §2.2, E-9.15).
    Aura {
        /// The aura mod.
        item: ItemId,
        /// Its rank.
        rank: u8,
    },
    /// Kitgun grip and loader (T-25; indices into `ModularTables.grips` / `.loaders`, as `Variant.kitgun`).
    KitgunParts {
        /// Grip index.
        grip: u16,
        /// Loader index.
        loader: u16,
    },
    /// Zaw grip and link (T-25; indices into `ModularTables.grips` / `.links`, as `Variant.zaw`).
    ZawParts {
        /// Grip index.
        grip: u16,
        /// Link index.
        link: u16,
    },
    /// Progenitor element (T-24, 09 §14); the value stays the user's (09 E-9.70).
    Progenitor {
        /// The element.
        element: DamageType,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::enums::{Polarity, RivenClass};
    use crate::item::RivenTrait;
    use arrayvec::ArrayVec;

    fn every_variant() -> Vec<Pick> {
        let mut traits = ArrayVec::new();
        traits.push(RivenTrait {
            trait_id: "critical_chance".into(),
            value: 1.5,
        });
        traits.push(RivenTrait {
            trait_id: "multishot".into(),
            value: 0.9,
        });
        vec![
            Pick::Mod {
                item: ItemId(3),
                rank: 10,
                params: smallvec::smallvec![(ParamKey::new("value"), ParamValue::Num(0.5))],
            },
            Pick::Riven(Riven {
                class: RivenClass::Rifle,
                traits,
                rank: None,
                polarity: Some(Polarity::Madurai),
                drain: None,
                family: None,
            }),
            Pick::Arcane {
                item: ItemId(4),
                rank: 5,
            },
            Pick::Perk { tier: 2, index: 1 },
            Pick::Shard {
                item: ItemId(5),
                tauforged: true,
            },
            Pick::Stance {
                item: ItemId(6),
                combo: ComboKey::new("neutral"),
            },
            Pick::FrameMod {
                item: ItemId(7),
                rank: 3,
            },
            Pick::Aura {
                item: ItemId(8),
                rank: 5,
            },
            Pick::KitgunParts { grip: 0, loader: 1 },
            Pick::ZawParts { grip: 2, link: 0 },
            Pick::Progenitor {
                element: DamageType::TOX,
            },
        ]
    }

    /// Both encodings round-trip every variant (the riven's readable and compact forms included).
    #[test]
    fn pick_round_trips() {
        for p in every_variant() {
            let json = serde_json::to_string(&p).unwrap();
            assert_eq!(serde_json::from_str::<Pick>(&json).unwrap(), p, "{json}");
            let bytes = postcard::to_allocvec(&p).unwrap();
            assert_eq!(postcard::from_bytes::<Pick>(&bytes).unwrap(), p, "{json}");
        }
        let json = serde_json::to_value(Pick::Arcane {
            item: ItemId(3),
            rank: 5,
        })
        .unwrap();
        assert_eq!(
            json,
            serde_json::json!({ "arcane": { "item": 3, "rank": 5 } })
        );
    }
}
