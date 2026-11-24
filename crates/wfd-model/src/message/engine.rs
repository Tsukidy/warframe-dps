//! Engine keys: the warnings of `CalcReport.warnings` and the `EngineError` messages (engine.md §15), the enemy notes
//! of the enemy panel (server-api.md §5.5 `special_dr.note`).

use super::message_keys;

message_keys! {
    /// Keys of engine warnings and errors (area `engine`).
    pub enum EngineKey, area = "engine" {
        /// An installed item whose effects are not encoded: it contributes nothing (I-6; engine.md §15).
        ItemUnmodelled = "item.unmodelled" [item],
        /// An item whose upstream text changed since review (pipeline.md §7, fingerprint).
        ItemStale = "item.stale" [item],
        /// An item whose effects are a draft.
        ItemDraft = "item.draft" [item],
        /// An item whose numbers were auto-patched from new game text and are not reviewed (pipeline.md §7).
        ItemAutoPatched = "item.auto_patched" [item],
        /// A register row of severity UNVERIFIED was exercised (`row` = `"D-023"`).
        RuleUnverified = "rule.unverified" [row],
        /// The EV gate-splitting limit was reached; the EV result is approximate (engine.md §7.4).
        GateSplitLimit = "ev.approx.gate_split_limit" [],
        /// The steady fixed point stopped at n = 200 iterations without converging (08 E-8.13 steady state, the
        /// warning `ev.steady.no_convergence`).
        SteadyNoConvergence = "ev.steady.no_convergence" [attack, view],
        /// EV and MC disagree beyond 08 E-8.24 (`gap_classes` comma-separated, 08 §12).
        McDeviation = "ev.mc_deviation" [attack, view, ev, ci, gap_classes],
        /// The EV TTK was extrapolated beyond the horizon (08 E-8.21).
        TtkExtrapolated = "ttk.extrapolated" [],
        /// Shields may regenerate during the fight (05 §10.5).
        TtkShieldRegen = "ttk.shield_regen_possible" [],
        /// A draft effect uses a DSL construct without a spec vector (overview.md R-2).
        NoSpecVector = "effect.no_spec_vector" [effect],
        /// The enemy has attenuation (05 §15.1): unique damage reduction.
        EnemyAttenuation = "enemy.special_dr.attenuation" [],
        /// The enemy has a flat or DPS-dependent damage reduction (05 §15.3).
        EnemyFlatDr = "enemy.special_dr.flat" [],
        /// The enemy adapts to damage types (05 §15.4).
        EnemyAdaptation = "enemy.special_dr.adaptation" [],
        /// `EngineError::InvalidBuild`; the reasons follow as further messages.
        InvalidBuild = "engine.invalid_build" [],
        /// `EngineError::MissingData`.
        MissingData = "engine.missing_data" [what],
        /// `EngineError::UnsupportedMechanic`.
        UnsupportedMechanic = "engine.unsupported_mechanic" [effect],
        /// `EngineError::ScheduleTooLong`: horizon × rate over 10^7 events.
        ScheduleTooLong = "engine.schedule_too_long" [shots],
        /// `EngineError::MechanicIdCollision`: two mechanic id strings with one FNV-1a hash (engine.md §8.2).
        MechanicIdCollision = "engine.mechanic_id_collision" [a, b],
        /// `EngineError::RngIndexRange`: an anchor index outside the E-8.16 ranges.
        RngIndexRange = "engine.rng_index_range" [what, value],
        /// The calculation was cancelled (a superseded request, server-api.md §11).
        Cancelled = "engine.cancelled" [],
        /// A probe path the probe registry does not know (`/api/explain`, engine.md §11.2).
        ProbeUnknown = "engine.probe_unknown" [probe],
    }
}
