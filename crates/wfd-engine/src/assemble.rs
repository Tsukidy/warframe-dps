//! Weapon assembly (engine.md §2): kitgun and zaw parts applied to the base weapon record (06, 07 §9.9), the
//! progenitor bonus V_t of E-1 (01 §4.3, T-24), and the exalted Strength scaling factor m_str (01 E-1.1, T-31,
//! T-20/T-33). Runs once at L0. Owner: T13.

use smallvec::SmallVec;
use wfd_model::{Attack, Build, Bundle, DamageType, Toggles, Weapon, WeaponId};

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
    /// m_str per attack of `weapon.attacks` (index = `AttackIdx`): Str·R_r for the attacks of a model-1 exalted
    /// weapon (01 E-1.1, Table 4.3-A), with R_r from the attack's own table `Attack.exalted_s` when it has one
    /// (Noctua's active attack) and from `Weapon.exalted` otherwise; 1 for every other attack and weapon. Per attack
    /// since contract review N-3 (pass 3; was one `f64` per weapon, which `Attack.exalted_s` of M-1 contradicts). The
    /// model 2 and 3 terms (`A += m_r·Str − 1`, `A += p_r·Str`) are not here: T13 adds them to the A bucket
    /// (`ABuckets.a`, `base_scalar`).
    pub m_str: SmallVec<[f64; 4]>,
}

impl AssembledWeapon {
    /// m_str of `atk`, an attack of the assembled record (matched by `Attack.key`); 1 when the attack is not found
    /// or `m_str` has no entry for it.
    pub fn m_str_of(&self, atk: &Attack) -> f64 {
        self.weapon
            .attacks
            .iter()
            .position(|a| a.key == atk.key)
            .and_then(|i| self.m_str.get(i).copied())
            .unwrap_or(1.0)
    }
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
