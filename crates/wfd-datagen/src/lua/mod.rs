//! Wiki Lua modules in an mlua sandbox, converted to JSON (pipeline.md §4.1). Owner: T05.
//!
//! T05 ports the proof of concept of `DS:wiki` §9: `mlua` 0.12 (`lua51`, `vendored`, `send`) with only the `math`,
//! `string` and `table` libraries (no `io`, `os`, `package`, `debug`, `load`, `dofile`, `require` of the standard
//! library); stubs `mw.loadData(title)` and `require(title)` that return an already-evaluated module of the batch
//! (`DamageTypes/data` and `Stances/data` run code at load; `Stances/data` with `addSharedCombos`), minimal `mw.site`
//! and `mw.text`; the instruction and memory limits of [`LuaLimits`]; and the table → JSON rules (keys exactly 1..n →
//! array, other tables → object with string keys, empty table → `{}`, `math.huge` → `f64::INFINITY`, integers stay
//! integers).

use crate::{DatagenError, WikiSource};
use std::collections::BTreeMap;

/// Sandbox limits (pipeline.md §4.1): modules are untrusted code, and one that trips a limit fails the build with its
/// title.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LuaLimits {
    /// The instruction hook runs every this many VM instructions (`set_hook`).
    pub hook_every: u32,
    /// Evaluation aborts after this many instructions in one module.
    pub max_instructions: u64,
    /// `Lua::set_memory_limit`, bytes.
    pub max_memory: usize,
}

impl Default for LuaLimits {
    /// The limits of pipeline.md §4.1: hook every 10^6 instructions, abort after 10^9, 256 MiB.
    fn default() -> Self {
        LuaLimits {
            hook_every: 1_000_000,
            max_instructions: 1_000_000_000,
            max_memory: 256 << 20,
        }
    }
}

/// Evaluates a batch of wiki modules (in an order where every `mw.loadData`/`require` target precedes its user, or
/// resolving them on demand within the batch) and converts each module's return value to JSON. Returns title → value
/// for every source. A Lua error or a tripped limit is [`DatagenError::Lua`] with the module's title.
pub fn eval_wiki_modules(
    sources: &[WikiSource],
    limits: &LuaLimits,
) -> Result<BTreeMap<Box<str>, serde_json::Value>, DatagenError> {
    let _ = (sources, limits);
    todo!("T05: mlua sandbox and Lua → JSON (pipeline.md §4.1)")
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn module(title: &str, text: &str) -> WikiSource {
        WikiSource {
            title: title.into(),
            revid: 1,
            text: text.into(),
        }
    }

    #[test]
    fn default_limits_are_those_of_pipeline_md() {
        let l = LuaLimits::default();
        assert_eq!(
            (l.hook_every, l.max_instructions, l.max_memory),
            (1_000_000, 1_000_000_000, 268_435_456)
        );
    }

    #[test]
    #[ignore = "T05"]
    fn tables_convert_to_json() {
        let src = module(
            "Module:T",
            "return { List = { 'a', 'b' }, Map = { x = 1, y = 2.5 }, Empty = {}, Inf = math.huge, Holes = { [1] = 1, [3] = 3 } }",
        );
        let out = eval_wiki_modules(&[src], &LuaLimits::default()).unwrap();
        // `serde_json::Value` cannot hold ±∞: `math.huge` is the string "Infinity", its JSON output form (§4.1) and
        // the readable form of `serde_util::ext_f64`.
        assert_eq!(
            out["Module:T"],
            json!({ "List": ["a", "b"], "Map": { "x": 1, "y": 2.5 }, "Empty": {}, "Inf": "Infinity",
                    "Holes": { "1": 1, "3": 3 } })
        );
    }

    #[test]
    #[ignore = "T05"]
    fn load_data_and_require_resolve_within_the_batch() {
        let a = module("Module:A", "return { n = 2 }");
        let b = module(
            "Module:B",
            "local a = mw.loadData('Module:A') local r = require('Module:A') return { m = a.n + r.n }",
        );
        let out = eval_wiki_modules(&[b, a], &LuaLimits::default()).unwrap();
        assert_eq!(out["Module:B"], json!({ "m": 4 }));
    }

    #[test]
    #[ignore = "T05"]
    fn sandbox_has_no_io_os_package_or_load() {
        for code in [
            "return io.open('x')",
            "return os.time()",
            "return package.path",
            "return load('return 1')()",
            "return dofile('x')",
        ] {
            let err = eval_wiki_modules(&[module("Module:Evil", code)], &LuaLimits::default())
                .unwrap_err();
            assert!(
                matches!(&err, DatagenError::Lua { title, .. } if &**title == "Module:Evil"),
                "{code}: {err}"
            );
        }
    }

    #[test]
    #[ignore = "T05"]
    fn limits_trip_with_the_module_title() {
        let small = LuaLimits {
            hook_every: 1_000,
            max_instructions: 100_000,
            max_memory: 1 << 20,
        };
        for code in [
            "while true do end",
            "local t = {} for i = 1, 1e9 do t[i] = ('x'):rep(100) end return t",
        ] {
            let err = eval_wiki_modules(&[module("Module:Loop", code)], &small).unwrap_err();
            assert!(
                matches!(&err, DatagenError::Lua { title, .. } if &**title == "Module:Loop"),
                "{code}: {err}"
            );
        }
    }
}
