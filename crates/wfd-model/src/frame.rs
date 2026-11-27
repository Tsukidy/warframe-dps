//! Warframes (domain-model.md §5, AD-26): `Module:Warframes/data` joined to `ExportWarframes` on uniqueName.
//! Frames carry identity, slot layout, Helminth facts and the frame passives that act on the build's weapons; base
//! health, shield and energy are not damage inputs.

use crate::effect::EffectDef;
use crate::enums::Polarity;
use crate::ids::{FrameId, ItemId, NameKey, StableKey, UniqueName};
use serde::{Deserialize, Serialize};
use smallvec::SmallVec;

/// A warframe record.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Frame {
    pub id: FrameId,
    pub key: StableKey,
    pub unique_name: UniqueName,
    pub name: NameKey,
    /// Prime and Umbra variants are the same frame for Helminth (09 E-9.32 H6).
    pub variant_of: Option<FrameId>,
    #[cfg_attr(feature = "ts", ts(as = "Vec<Polarity>"))]
    pub polarities: SmallVec<[Polarity; 8]>,
    #[cfg_attr(feature = "ts", ts(as = "Vec<Polarity>"))]
    pub aura_polarities: SmallVec<[Polarity; 2]>,
    pub exilus_polarity: Option<Polarity>,
    /// 1; 2 for Jade (09 §2.2).
    pub aura_slots: u8,
    #[cfg_attr(feature = "ts", ts(as = "Vec<FrameAbility>"))]
    pub abilities: SmallVec<[FrameAbility; 4]>,
    /// Host(frame) of 09 E-9.32 H3 (`rules/helminth.toml`).
    pub helminth_host: Option<ItemId>,
    pub image: Option<Box<str>>,
    /// The frame's passive effects on the build's weapons in the DSL (04 §5.11: Ash SD +0.50 on Slash, Lavos
    /// +1.00 × Duration, Saryn SD +0.25 on every proc of the build; 04 §5.6 Dante's Chronicler's Mark, on by default
    /// when Dante is the frame), applied when the build's frame (`BuildDoc.warframe.frame`) is this frame; T-16
    /// implies Saryn (04 §4.4, D-037), so Toxic Lash procs get Saryn's +0.25 once, through SD (04 E-4.14), never a
    /// second time on the extra hit. Variants (`variant_of`) carry their own list. Filled by the pipeline from
    /// curated data. Addition, CONTRACTS.md "Contract review fixes (model-records)", DC-14.
    #[serde(default)]
    pub passives: Vec<EffectDef>,
}

/// An ability of a frame (H2: a self-cast buff cannot be replaced by the Helminth ability).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct FrameAbility {
    pub name: NameKey,
    pub self_buff: Option<ItemId>,
}
