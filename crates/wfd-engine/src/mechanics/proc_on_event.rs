//! The `proc_on_event` mechanic kind: 02 §4.7.3-§4.7.5 (Hunter Munitions, Internal Bleeding, Hemorrhage, Magnetic Welt,
//! Secondary Encumber, exclusive groups). Owner: T14.

/// The `proc_on_event` kind (domain-model.md §11.2).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ProcOnEvent;

super::mechanic_skeleton!(ProcOnEvent, "T14");
