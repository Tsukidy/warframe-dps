//! Keys of share-code decoding and inspiration-link import (server-api.md §7 `DecodeResponse.report`; frontend.md §9;
//! work-breakdown.md T10).

use super::message_keys;

message_keys! {
    /// Keys of decoding and import (area `import`).
    pub enum ImportKey, area = "import" {
        /// The text is neither a `wfd1-` code, our URL, nor a warframe-damage.com URL.
        Unrecognized = "import.unrecognized" [],
        /// The payload could not be decoded (`stage` = base64, deflate, lz, json).
        Corrupt = "import.corrupt" [stage],
        /// The code's document version is newer than this binary understands.
        VersionUnsupported = "import.version_unsupported" [v, max],
        /// The code was migrated from an older document version (`wfd-buildcode::migrate`).
        Migrated = "import.migrated" [from, to],
        /// An inspiration field or name that has no mapping (also listed in `report.unmapped`).
        Unmapped = "import.unmapped" [field, value],
        /// An inspiration value converted to model units (`"25" %` → 0.25; riven displayed values, 04 §8.2).
        Converted = "import.converted" [field, from, to],
        /// A default applied for a field the inspiration never sets (`o.v.stucks = 0`, `I:feat` §15).
        DefaultApplied = "import.default_applied" [field, value],
        /// An item removed from the game data was dropped (also listed in `report.retired_items`).
        RetiredItem = "import.retired_item" [key],
        /// An attack matched only by its index, not by its slug (domain-model.md §2 `AttackKey`).
        AttackIndexOnly = "import.attack_index_only" [slug, idx],
        /// Export to an inspiration link is impossible: the scenario uses something the inspiration cannot express.
        ExportUnsupported = "import.export_unsupported" [feature],
    }
}
