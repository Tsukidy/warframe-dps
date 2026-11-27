//! Weapon assembly (engine.md §2): kitgun and zaw parts applied to the base weapon record (06, 07 §9.9), the
//! progenitor bonus V_t of E-1 (01 §4.3, T-24), and the exalted Strength scaling factor m_str (01 E-1.1, T-31,
//! T-20/T-33). Runs once at L0. Owner: T13.

use wfd_model::{Build, Bundle, DamageType, Toggles, Weapon, WeaponId};

use crate::error::EngineError;

/// The weapon a scenario evaluates, with its modular parts and scenario-level base modifiers applied.
#[derive(Debug, Clone, PartialEq)]
pub struct AssembledWeapon {
    /// The bundle record the weapon came from.
    pub id: WeaponId,
    /// The assembled record: attacks with kitgun chamber + grip + loader or zaw strike + grip + link stats applied.
    pub weapon: Weapon,
    /// Progenitor element and its value v of E-1 (`V_t = v·Σ B0` for t = e_v); `None` without a progenitor.
    pub progenitor: Option<ProgenitorBonus>,
    /// m_str = Str·R_r for model-1 exalted weapons (01 E-1.1), else 1.
    pub m_str: f64,
}

/// The progenitor bonus of E-1 (01 §4.3).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ProgenitorBonus {
    /// The element e_v.
    pub element: DamageType,
    /// The fraction v.
    pub v: f64,
}

/// Assembles the build's weapon for a scenario (parts of `build.variant`, progenitor T-24, exalted rank T-31 and
/// Strength T-20/T-33). T13.
pub fn assemble(
    b: &Bundle,
    build: &Build,
    toggles: &Toggles,
) -> Result<AssembledWeapon, EngineError> {
    let _ = (b, build, toggles);
    todo!("T13")
}
