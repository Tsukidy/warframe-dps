//! Data keys: data routes (server-api.md §9), the refresh job, and the validation checks of the pipeline
//! (pipeline.md §7), which reach the UI's data page through `DataEvent::Report`. A `detail` argument carries the
//! untranslated diagnostic of a parser (TOML, Lua, JSON) for the data page; every other argument is a name, key or
//! number.

use super::message_keys;

message_keys! {
    /// Keys of data status, refresh and validation (area `data`).
    pub enum DataKey, area = "data" {
        /// 409: a refresh job is already running (server-api.md §3).
        RefreshRunning = "data.refresh_running" [job],
        /// The binary was built without the `refresh` feature.
        RefreshUnavailable = "data.refresh_unavailable" [],
        /// A user bundle could not be decoded at startup and was skipped (server-api.md §1.2).
        BundleCorrupt = "data.bundle_corrupt" [id],
        /// A user bundle has another `schema_version` than the binary.
        BundleSchema = "data.bundle_schema" [id, version, expected],
        /// A request to a source host failed (pipeline.md §3).
        FetchFailed = "data.fetch_failed" [host, status],
        /// A source host is not on the allow-list (pipeline.md §3.1).
        HostNotAllowed = "data.host_not_allowed" [host],
        /// Log: a source was fetched (`source` = wiki, pe, wfcd; `count` files).
        LogFetched = "data.log.fetched" [source, count],
        /// Log: a pipeline step finished.
        LogStep = "data.log.step" [step, ms],
        /// Log: the new bundle was written.
        LogWritten = "data.log.written" [bundle_id, size],
        /// Validation check "Schema" (pipeline.md §7).
        CheckSchema = "data.check.schema" [file, detail],
        /// Validation check "References".
        CheckReferences = "data.check.references" [file, id],
        /// Validation check "Class": the derived T-02 class differs from the declared one.
        CheckClass = "data.check.class" [effect, derived, declared],
        /// Validation check "Element order".
        CheckElementOrder = "data.check.element_order" [effect],
        /// Validation check "Applicability".
        CheckApplicability = "data.check.applicability" [effect, stat],
        /// Validation check "Binding".
        CheckBinding = "data.check.binding" [effect, token],
        /// Validation check "Unbound numbers".
        CheckUnboundNumbers = "data.check.unbound_numbers" [item, number],
        /// Validation check "Rank scaling".
        CheckRankScaling = "data.check.rank_scaling" [item, rank],
        /// Validation check "Fingerprint": the upstream text changed; the item is stale.
        CheckFingerprint = "data.check.fingerprint" [item],
        /// The item's numbers were auto-patched from the new upstream text (pipeline.md §7).
        AutoPatched = "data.auto_patched" [item, field, old, new],
        /// Validation check "Upgrade types".
        CheckUpgradeTypes = "data.check.upgrade_types" [item, upgrade_type],
        /// Validation check "Ranges and units".
        CheckRanges = "data.check.ranges" [effect, value],
        /// Validation check "Rule invariants".
        CheckRuleInvariants = "data.check.rule_invariants" [what],
        /// Validation check "Incarnon completeness".
        CheckIncarnon = "data.check.incarnon_completeness" [family],
        /// Validation check "Cross-source".
        CheckCrossSource = "data.check.cross_source" [weapon, field],
        /// Validation check "Coverage".
        CheckCoverage = "data.check.coverage" [kind, count, minimum],
        /// Validation check "Spec anchors".
        CheckSpecAnchors = "data.check.spec_anchors" [item],
        /// A conflict between sources that the join resolved by precedence (pipeline.md §5.1).
        Conflict = "data.conflict" [path, chosen, other],
        /// The refresh failed (the step and the reason are arguments).
        RefreshFailed = "data.refresh_failed" [step, detail],
    }
}
