//! Buff portfolio (optimizer.md §6.3).
//!
//! Owner: T33. With `BuffPolicy { portfolio: true }` the scenarios "none", each allowed buff alone and all allowed
//! buffs are solved as separate searches; `OptResult.buff_table` gives per scenario the best build, its score, the value
//! of the buff on the no-buff best build and the gain of rebuilding for it (`BuffScenarioRow`). With `portfolio: false`
//! every allowed buff is on for the whole search (inspiration behaviour).
