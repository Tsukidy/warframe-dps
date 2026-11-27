//! Set counting of spec 09 E-9.40, implemented once (AD-24): `wfd-data::legality` (T04) delegates to it and the
//! engine's set-bonus compilation (`compile/sets.rs`, T06) calls it, so TV-09-09 checks the count that drives damage.
//! Addition, contract review API-6 (CONTRACTS.md, "Contract review fixes (model-core)" and "(model-records)").

use crate::build::{Build, Slot};
use crate::bundle::Bundle;
use crate::ids::{ItemId, SetId};
use crate::item::{Item, SetDef};
use smallvec::SmallVec;

/// N_S of 09 E-9.40 for every set with at least one counting piece, in `SetId` order.
///
/// The counted slots are the warframe's `fgen0..fgen7`, `fexilus`, `aura0`, `aura1`, the active weapon's `gen0..gen7`
/// and `exilus` (the stance slot holds no set mods), and the declared companion and robotic-weapon set mods
/// (`Build.companion_set_mods`). The active weapon is the build's weapon: a build evaluates the attacks of its one
/// weapon, which is the exalted weapon when those are exalted attacks, and set mods on holstered weapons never count
/// (U38.5). Pieces are distinct item identities (`pieces(S)`: a mod on both the weapon and the robotic weapon counts
/// once), members listed in `SetDef.noncounting` (`set_noncounting`: Mecha Empowered) never count, ranks do not
/// matter, and `N_S ≤ |SetDef.members|` (NumUpgradesInSet). Riven slots and empty slots contribute nothing; an item
/// id outside the bundle is ignored (legality reports it elsewhere). Declared companion pieces count whatever
/// `Build.companion_kind` says: their legality (E-9.41, key `set.companion`) is a violation that `wfd-data::legality`
/// reports, and validation never blocks evaluation (09 §1.5 rule 1).
pub fn set_counts(bundle: &Bundle, build: &Build) -> SmallVec<[(SetId, u8); 4]> {
    count_pieces(&bundle.items, &bundle.sets, set_slot_items(build))
}

/// The items in the slots that count for sets (09 E-9.40 `slots(S-count)`), in slot order, duplicates included.
pub fn set_slot_items(build: &Build) -> impl Iterator<Item = ItemId> + '_ {
    let frame = &build.frame;
    frame
        .r#gen
        .iter()
        .chain(std::iter::once(&frame.exilus))
        .chain(frame.auras.iter())
        .chain(build.r#gen.iter())
        .chain(std::iter::once(&build.exilus))
        .filter_map(|slot| match slot {
            Slot::Mod { item, .. } => Some(*item),
            Slot::Empty | Slot::Riven(_) => None,
        })
        .chain(build.companion_set_mods.iter().copied())
}

/// The counting rule of 09 E-9.40 over a list of equipped items: distinct identities per set, without the set's
/// `noncounting` members, capped at the set's member count; sets without a counting piece are omitted.
pub fn count_pieces(
    items: &[Item],
    sets: &[SetDef],
    equipped: impl IntoIterator<Item = ItemId>,
) -> SmallVec<[(SetId, u8); 4]> {
    let mut pieces: SmallVec<[(SetId, ItemId); 8]> = SmallVec::new();
    for id in equipped {
        let Some(set) = items.get(id.idx()).and_then(|it| it.set) else {
            continue;
        };
        let Some(def) = sets.get(set.idx()) else {
            continue;
        };
        if def.noncounting.contains(&id) || pieces.contains(&(set, id)) {
            continue;
        }
        pieces.push((set, id));
    }
    pieces.sort();
    let mut out: SmallVec<[(SetId, u8); 4]> = SmallVec::new();
    for (set, _) in pieces {
        match out.last_mut() {
            Some((s, n)) if *s == set => *n = n.saturating_add(1),
            _ => out.push((set, 1)),
        }
    }
    for (set, n) in &mut out {
        let members = sets[set.idx()].members.len();
        *n = (*n).min(u8::try_from(members).unwrap_or(u8::MAX));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ids::{NameKey, StableKey, UniqueName};
    use crate::item::{CompatRule, EffectStatus, ItemKind};
    use crate::{Polarity, Rarity};

    fn item(i: u32, set: Option<u16>) -> Item {
        Item {
            id: ItemId(i),
            key: StableKey(i),
            unique_name: UniqueName::new(format!("wfd:test/{i}")),
            name: NameKey(format!("wfd:test/{i}").into()),
            kind: ItemKind::WeaponMod {
                exilus: false,
                augment_of: None,
            },
            compat: CompatRule::default(),
            incompat: SmallVec::new(),
            polarity: Some(Polarity::Madurai),
            base_drain: 4,
            max_rank: 5,
            rarity: Rarity::Common,
            set: set.map(SetId),
            params: Vec::new(),
            effects: Vec::new(),
            status: EffectStatus::Verified,
            effect_tier: None,
            auto_patched: false,
            sources: Vec::new(),
            upstream_text: None,
            inspiration_name: None,
            image: None,
        }
    }

    fn set(i: u16, members: &[u32], noncounting: &[u32]) -> SetDef {
        SetDef {
            id: SetId(i),
            unique_name: UniqueName::new(format!("wfd:set/{i}")),
            name: NameKey(format!("wfd:set/{i}").into()),
            members: members.iter().map(|m| ItemId(*m)).collect(),
            noncounting: noncounting.iter().map(|m| ItemId(*m)).collect(),
            bonus: Vec::new(),
            member_scale: None,
        }
    }

    /// 09 E-9.40: distinct identities, `set_noncounting`, the member cap, items outside any set.
    #[test]
    fn counts_distinct_counting_pieces() {
        // Set 0: items 0, 1, 2 (item 2 never counts, as Mecha Empowered); set 1: item 3; item 4 has no set.
        let items = vec![
            item(0, Some(0)),
            item(1, Some(0)),
            item(2, Some(0)),
            item(3, Some(1)),
            item(4, None),
        ];
        let sets = vec![set(0, &[0, 1, 2], &[2]), set(1, &[3], &[])];
        let ids = |v: &[u32]| v.iter().map(|i| ItemId(*i)).collect::<Vec<_>>();
        // The same mod on the warframe's weapon and the robotic weapon counts once.
        assert_eq!(
            count_pieces(&items, &sets, ids(&[0, 1, 0, 2, 4, 3])).as_slice(),
            &[(SetId(0), 2), (SetId(1), 1)]
        );
        assert!(count_pieces(&items, &sets, ids(&[2, 4])).is_empty());
        // An id outside the bundle is ignored.
        assert_eq!(
            count_pieces(&items, &sets, ids(&[99, 3])).as_slice(),
            &[(SetId(1), 1)]
        );
    }
}
