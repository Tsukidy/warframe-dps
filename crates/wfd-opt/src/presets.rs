//! Search presets and defaults (optimizer.md §10.2, §11).
//!
//! Owner: T33. `Preset::Default` keeps the request as sent (defaults of §11 are `SearchParams::default()` and the
//! other request types' defaults in `wfd_model::api`). `Preset::InspirationCompatible` sets the inspiration's defaults
//! (`I:feat` §17.1, `I:opt` §2): metric DPS, view `auto`, `NO_FACTION` on, conditions `all_max`, the inspiration's
//! prohibited defaults, status-duration-only items excluded, search window `clamp(round(100 / attack speed), 5, 20)` s,
//! frame mods limited to one slot, capacity mode `ignore` (D-092). The window and T-02 `all_max` a preset sets are
//! scenario inputs: they are written into the scenario of every `RankedBuild.input`, so the share code of a ranked
//! build reproduces its score (08 §10; contract review M-10, API-9). The capacity mode is not (it is
//! `constraints.capacity`, server-api.md §8.1).
