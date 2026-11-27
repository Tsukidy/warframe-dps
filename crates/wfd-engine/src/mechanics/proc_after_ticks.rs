//! The `proc_after_ticks` mechanic kind: 04 §6.5 (Primary Debilitate; draws `deb`, `deb_type`, `deb_comp`, 02 E-2.17b).
//! Owner: T14.

/// The `proc_after_ticks` kind (domain-model.md §11.2).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ProcAfterTicks;

super::mechanic_skeleton!(ProcAfterTicks, "T14");
