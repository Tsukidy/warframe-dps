//! Riven search (optimizer.md §6.2 step 6).
//!
//! Owner: T33. With `RivenPolicy::Search`, an outer loop over the trait combinations legal for the weapon class
//! (spec 04 §8, spec 09 §12) with values at the chosen quality of the value formula (disposition 1.2); each riven is one
//! main candidate, pruned by the same bound; the best riven per trait set is reported.
