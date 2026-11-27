//! Riven search (optimizer.md §6.2 step 6).
//!
//! Owner: T33. With `RivenPolicy::Search`, an outer loop over the trait combinations legal for the weapon class
//! (spec 04 §8, spec 09 §12) with values at the chosen quality of the value formula (disposition 1.2); each riven is one
//! main candidate, pruned by the same bound; the best riven per trait set is reported (`OptResult.riven_sets`).
//! The class and the traits come from `wfd-data::legality` (contract review API-12): `riven_class` (09 E-9.50, the
//! generated `Riven.class`) and `riven_traits_for` (the class column's traits, T2 and T5 of E-9.55, never a vintage
//! companion trait on a companion riven, §12.3). `quality` is u of 04 E-4.18, applied to every trait (u = 1 gives e_1
//! of 09 E-9.56: v_hi for a bonus, v_lo for a curse), and `legality::riven_range` checks the generated values.
