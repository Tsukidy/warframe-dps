//! Keys of document issues: a `CalcInput` that cannot be resolved (`wfd-data::GameData::resolve` returns them as
//! `Err(Vec<Message>)`, the server answers 422 `api.invalid` with them as `issues`; domain-model.md §9.2,
//! server-api.md §6.1) and the per-item warnings of decoding (pipeline.md §9.2 `DocIssue::ItemUnknown`,
//! `DocIssue::ItemRetired`). Rule violations of spec 09 are not document issues (legality keys, never blocking).

use super::message_keys;

message_keys! {
    /// Keys of document issues (area `doc`).
    pub enum DocKey, area = "doc" {
        /// `CalcInput.v` is newer than this binary understands (server-api.md §2 "Versioning").
        VersionUnsupported = "doc.version_unsupported" [v, max],
        /// A StableKey that names nothing in the active bundle (`what` = weapon, item, enemy, frame, buff, part).
        KeyUnknown = "doc.key_unknown" [what, key],
        /// A key that names the wrong kind of thing (an item key in `weapon`, a mod in a buff list).
        KeyWrongKind = "doc.key_wrong_kind" [key, expected],
        /// A session-local key (≥ 2^31) whose uniqueName is unknown to the active bundle (pipeline.md §9.2,
        /// `DocIssue::ItemUnknown`; AD-18).
        ItemUnknown = "doc.item_unknown" [key, unique_name],
        /// A session-local key without an entry in `CalcInput.x` (AD-18).
        LocalKeyUnmapped = "doc.local_key_unmapped" [key],
        /// An item removed from the game data; dropped from the build with this warning where it can be
        /// (pipeline.md §9.2 `DocIssue::ItemRetired`), an issue where it cannot (server-api.md §6.1).
        ItemRetired = "doc.item_retired" [key],
        /// Content of a kind that is impossible in its slot, such as a riven in `aura0` (domain-model.md §9.2).
        SlotContent = "doc.slot_content" [slot, content],
        /// Two entries for one slot.
        SlotDuplicate = "doc.slot_duplicate" [slot],
        /// A slot id that does not exist for the weapon or frame at all (`gen8`, `inc6`).
        SlotUnknown = "doc.slot_unknown" [slot],
        /// An Incarnon perk index outside the evolution's perk list.
        PerkIndex = "doc.perk_index" [slot, index],
        /// A parameter key the item does not declare.
        ParamUnknown = "doc.param_unknown" [key, param],
        /// A parameter value of the wrong kind or outside its range.
        ParamInvalid = "doc.param_invalid" [key, param],
        /// A riven trait id that does not exist.
        RivenTraitUnknown = "doc.riven_trait_unknown" [trait_id],
        /// A custom-stat id that does not exist (T-26).
        CustomUnknown = "doc.custom_unknown" [id],
        /// A condition override for an effect key that names no effect (dropped with this warning).
        EffectUnknown = "doc.effect_unknown" [effect],
        /// A scenario value outside its range (`field` = document path, e.g. `toggles.armor_strip`).
        ValueRange = "doc.value_range" [field, min, max],
        /// A combo key the weapon's combo tables do not have (07 E-7.0b).
        ComboUnknown = "doc.combo_unknown" [combo],
        /// A kitgun or zaw part that does not fit the weapon, or a variant on a non-modular weapon.
        VariantInvalid = "doc.variant_invalid" [field],
        /// An attack key of `toggles.melee_plans` or `toggles.pellet_hit` (T-30, T-41) that names no attack of the
        /// weapon (dropped with this warning; CONTRACTS.md T01d).
        AttackUnknown = "doc.attack_unknown" [field, attack],
    }
}

/// The document issues of pipeline.md §9.2 (`DocIssue::ItemUnknown`, `DocIssue::ItemRetired`) are these keys: an
/// issue travels as a [`Message`](super::Message) whose key is a [`DocKey`] (decode reports, `CalcReport.warnings`,
/// the `issues` of a 422 `api.invalid`), so `DocIssue` is the key enum itself (ruling on C-18, CONTRACTS.md T01d).
/// `wfd-buildcode` (T10) may wrap it in richer decode types of its own.
pub type DocIssue = DocKey;
