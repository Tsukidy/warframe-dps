//! The rule keys of spec 09 (09 §1.5: every rule has a stable key, used as the i18n message key). They are the `key`
//! of `Violation`s; `wfd-data::legality` (T04) emits them, the optimizer treats `illegal` and `illegal_unverified`
//! ones as constraints (09 §1.5 rule 2). Arguments are the rule's inputs; `Violation.slots` names the slots.

use super::message_keys;

message_keys! {
    /// Rule keys of spec 09 (area `legality`).
    pub enum LegalityKey, area = "legality" {
        /// 09 E-9.2 (and E-9.21 for the exilus slot).
        SlotCount = "slot.count" [slot],
        /// 09 E-9.3.
        SlotKind = "slot.kind" [slot, kind],
        /// 09 E-9.8 (mod compatibility, the conjunction of E-9.4 to E-9.7).
        Compat = "compat" [item],
        /// 09 E-9.4 (compatibility tree).
        CompatType = "compat.type" [item],
        /// 09 E-9.5 (tags, X-EXALT).
        CompatTags = "compat.tags" [item, tag],
        /// 09 E-9.6 (Cannonade mods need a semi-automatic trigger).
        CompatSemi = "compat.semi" [item],
        /// 09 E-9.7 (Synth Charge: base magazine requirement).
        CompatMagazine = "compat.magazine" [item],
        /// 09 E-9.9 (exclusion graph).
        ExclPair = "excl.pair" [a, b],
        /// 09 E-9.10 D1.
        DupMod = "dup.mod" [item],
        /// 09 E-9.10 D2.
        DupArcane = "dup.arcane" [item],
        /// 09 E-9.10 D4, E-9.52 (illegal_unverified).
        DupRiven = "dup.riven" [],
        /// 09 E-9.11.
        RankRange = "rank.range" [item, rank, max],
        /// 09 E-9.16 (item capacity rank).
        CapRank = "cap.rank" [rank],
        /// 09 E-9.18 (capacity rule under the capacity mode).
        CapExceeded = "cap.exceeded" [used, capacity],
        /// 09 E-9.30.
        ArcaneFamily = "arcane.family" [item],
        /// 09 E-9.30a.
        ArcaneGate = "arcane.gate" [item, tag],
        /// 09 E-9.31: at most 5 shards.
        ShardCount = "shard.count" [count],
        /// 09 E-9.31: one bonus of the shard's colour.
        ShardBonus = "shard.bonus" [item],
        /// 09 E-9.32 H1.
        HelminthCount = "helminth.count" [],
        /// 09 E-9.32 H2.
        HelminthReplaced = "helminth.replaced" [ability],
        /// 09 E-9.32 H3.
        HelminthHost = "helminth.host" [ability, host],
        /// 09 E-9.41.
        SetCompanion = "set.companion" [item],
        /// 09 E-9.51.
        RivenBinding = "riven.binding" [class],
        /// 09 E-9.54.
        RivenLayout = "riven.layout" [positive, negative],
        /// 09 E-9.55.
        RivenTrait = "riven.trait" [trait_id],
        /// 09 E-9.56.
        RivenRange = "riven.range" [trait_id, value, min, max],
        /// 09 E-9.57 (warning).
        RivenPhysical = "riven.physical" [trait_id],
        /// 09 E-9.60.
        IncarnonLine = "incarnon.line" [],
        /// 09 E-9.60.
        IncarnonTier = "incarnon.tier" [tier],
        /// 09 E-9.60.
        IncarnonForm = "incarnon.form" [],
        /// 09 E-9.70 V1.
        ValenceWeapon = "valence.weapon" [],
        /// 09 E-9.70 V2.
        ValenceElement = "valence.element" [element],
        /// 09 E-9.70 V3.
        ValenceValue = "valence.value" [v10],
        /// 09 E-9.70 V1 (warning: an adversary weapon without a valence uses T-24's default).
        ValenceUnset = "valence.unset" [],
    }
}
