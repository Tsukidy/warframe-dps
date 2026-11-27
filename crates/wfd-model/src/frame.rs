//! Warframes (domain-model.md §5, AD-26): `Module:Warframes/data` joined to `ExportWarframes` on uniqueName.
//! Frames carry identity, slot layout, Helminth facts and their passives' effects; base health, shield and energy are
//! not damage inputs.

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
    /// Effects of the frame's passive that apply because the frame is the build's frame (04 §5.11 class "–": Saryn
    /// SD +0.25 on every proc of the build, Ash SD +0.50 on Slash, Lavos SD +1.00 × Duration; 04 §5.6 Dante's passive,
    /// on by default when Dante is the frame). The engine applies them when `FrameLoadout.frame` (or T-16, which
    /// implies Saryn, 00 §6.1) is this frame; they are never also authored as buff or mechanic parameters (04 E-4.14,
    /// D-037). Source: `rules/frames.toml` `passives` (T18). Addition, contract review DC-14.
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
