//! Compile levels L0-L2 (engine.md §3; domain-model.md §12, §18). L0 compiles a scenario ([`CompiledScenario`]), L1 one
//! candidate pick ([`CompiledItem`]), L2 a whole build ([`CompiledBuild`]) with the finalizers (sets, shards, resolved
//! scalars, element order). Owner: T06; `melee.rs` T08b.

pub mod buffs;
pub mod build;
pub mod conditions;
pub mod custom;
pub mod melee;
pub mod monotone;
pub mod passives;
pub mod pick;
pub mod riven;
pub mod scenario;
pub mod scope;
pub mod sets;
pub mod shards;

pub use build::CompiledBuild;
pub use conditions::{CompiledCondition, CondKind, StackCond};
pub use melee::{
    ComboRef, CompiledMelee, MeleePlan, MeleeTablesOwned, PlanKind, melee_plan, plan_selection,
    resolve_combo_refs, resolve_combo_tables,
};
pub use monotone::{ReadQuantity, Reader, monotone_classes};
pub use pick::CompiledItem;
pub use scenario::{
    CompiledOverrides, CompiledScenario, MechSignature, MechSlotTable, MultSlotTable,
};
