//! `CalcReport` assembly (engine.md §9): the weapon and enemy panels, per attack and view `{ ev, mc? }`, groups,
//! conditions, touched rows, warnings; the mapping of spec 00 §4.5 onto the report fields and the E-8.24 deviation
//! flag (08 §8.11). The breakdown sources both evaluators accumulate are [`SourceKind`]s, mapped here to the source
//! ids of `wfd_model::api::SourceDps`. Owner: T17.

use wfd_model::{AttackIdx, DamageType, StatusType};

/// A breakdown source (08 §8.2). Its id in the report: `direct.<DamageType>`, `dot.<StatusType>`, `heat`, `blast`,
/// `xh.<EffectKey>`, `duplicate`, `child.<attack slug>`, `magnetic_break` (`wfd_model::api::SourceDps`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SourceKind {
    /// Direct damage of a final type.
    Direct(DamageType),
    /// DoT ticks of a status.
    Dot(StatusType),
    /// Heat pulses.
    Heat,
    /// Blast hits.
    Blast,
    /// Extra hits of one mechanic (index into the attack's `MechSet`), Blast-detonation extra hits of that
    /// mechanic included (04 E-4.13a: the same 04 source, keyed `"xd/"` only for the draws of 08 E-8.16).
    ExtraHit(u16),
    /// Flat damage on proc of one `flat_damage_on_proc` mechanic (Cascadia Empowered, 04 §5.4; 08 E-8.22 counts it
    /// in `direct_h`), index into the attack's `MechSet`. 08 §8.2 lists no source of its own for it; the report maps
    /// it to the `xh.<EffectKey>` id of its effect (a damage instance per 04 source) until the spec names one.
    /// Addition, contract review ENG-23.
    FlatOnProc(u16),
    /// Melee Duplicate copies.
    Duplicate,
    /// Lingering child ticks of an attack.
    Child(AttackIdx),
    /// The Magnetic break (TTK breakdowns only).
    MagneticBreak,
}
