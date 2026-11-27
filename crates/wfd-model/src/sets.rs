//! Set counting (spec 09 E-9.40): the one implementation of the count that drives the set bonuses of 04 §9.
//!
//! `wfd-data::legality::set_counts` (T04, legality and the UI) and the engine's set-bonus compilation (`wfd-engine`
//! `compile/sets.rs`, T06, which depends on `wfd-model` only) both call [`set_counts`], so the count that drives
//! damage is the one TV-09-09 checks (AD-24 "implemented once"; CONTRACTS.md "Contract review fixes
//! (model-records)", API-6). A pure function of the bundle and the resolved build: the one piece of game logic in
//! this crate besides construction and validation helpers, placed here because both consumers can reach it.

use crate::build::{Build, Slot};
use crate::bundle::Bundle;
use crate::ids::{ItemId, SetId};
use smallvec::SmallVec;

/// Pieces of each set that count (09 E-9.40), sorted by set id; a set without a counting piece is absent.
///
/// - Counted slots: the warframe's general slots `fgen0`..`fgen7`, `fexilus`, `aura0`, `aura1`; the build's weapon
///   (the active weapon: the weapon of the evaluated attack, exalted weapons included) `gen0`..`gen7`, `exilus`;
///   and the declared companion and robotic-weapon pieces `Build.companion_set_mods`. Arcanes, the stance, Incarnon
///   perks, shards and rivens hold no set mods. A holstered weapon is never part of a build, so its set mods never
///   count (U38.5).
/// - A piece is an item identity (`ItemId`, the PE uniqueName): the same mod in two places counts once (`W:Set_Mods`
///   L217-224: the warframe's weapon and the robotic weapon), and a set item's set is `Item.set` (PE `modSet`).
/// - Members of `SetDef.noncounting` (`set_noncounting` = Mecha Empowered, OQ-439) never count; ranks do not matter.
/// - Declared companion pieces count whatever `Build.companion_kind` says: their legality (E-9.41, key
///   `set.companion`) is a violation that `wfd-data::legality` reports, and validation never blocks evaluation
///   (09 §1.5 rule 1).
///
/// An item id outside `bundle.items` or a set id outside `bundle.sets` adds nothing (`GameData::resolve` produces
/// neither).
pub fn set_counts(bundle: &Bundle, build: &Build) -> SmallVec<[(SetId, u8); 4]> {
    let mut pieces: SmallVec<[(SetId, ItemId); 8]> = SmallVec::new();
    let mut add = |item: ItemId| {
        let Some(set) = bundle.items.get(item.idx()).and_then(|it| it.set) else {
            return;
        };
        let Some(def) = bundle.sets.get(set.idx()) else {
            return;
        };
        if !def.noncounting.contains(&item) {
            pieces.push((set, item));
        }
    };
    let weapon = build.r#gen.iter().chain(std::iter::once(&build.exilus));
    let frame = build
        .frame
        .r#gen
        .iter()
        .chain(std::iter::once(&build.frame.exilus))
        .chain(build.frame.auras.iter());
    for slot in weapon.chain(frame) {
        if let Slot::Mod { item, .. } = slot {
            add(*item);
        }
    }
    for item in &build.companion_set_mods {
        add(*item);
    }
    pieces.sort_unstable();
    pieces.dedup();
    let mut out: SmallVec<[(SetId, u8); 4]> = SmallVec::new();
    for (set, _) in pieces {
        match out.last_mut() {
            Some((s, n)) if *s == set => *n = n.saturating_add(1),
            _ => out.push((set, 1)),
        }
    }
    out
}
