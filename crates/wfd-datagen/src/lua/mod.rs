//! Wiki Lua modules in an mlua sandbox, converted to JSON (pipeline.md §4.1). Owner: T05.
//!
//! T05 ports the proof of concept of `DS:wiki` §9: `mlua` 0.12 (`lua51`, `vendored`, `send`) with only the `math`,
//! `string` and `table` libraries (no `io`, `os`, `package`, `debug`, `load`, `dofile`, `require` of the standard
//! library; also no `loadstring` and `loadfile`, which the Lua 5.1 base library that mlua always opens provides:
//! `loadfile` reads files from disk; CONTRACTS.md T02c request T02c-R7); stubs `mw.loadData(title)` and
//! `require(title)` that return an already-evaluated module of the batch (`DamageTypes/data` and `Stances/data` run
//! code at load; `Stances/data` with `addSharedCombos`), minimal `mw.site` and `mw.text`; the instruction and memory
//! limits of [`LuaLimits`]; and the table → JSON rules (keys exactly 1..n → array, other tables → object with string
//! keys, empty table → `{}`, integers stay integers). `math.huge` and `-math.huge` become the JSON strings
//! `"Infinity"` and `"-Infinity"` in the [`serde_json::Value`] (which cannot hold ±∞; pipeline.md §4.1 "written as the
//! string `"Infinity"`"); the typed wire structs read them as `f64::INFINITY` through `wfd_model::serde_util::ext_f64`.

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
    use crate::tests::pipeline_md;
    use serde_json::json;

    fn module(title: &str, text: &str) -> WikiSource {
        WikiSource {
            title: title.into(),
            revid: 1,
            text: text.into(),
        }
    }

    /// The limit of the pipeline.md §4.1 bullet "Modules are untrusted code: instruction limit via `set_hook` every
    /// 10^6 instructions, abort after 10^9; memory limit `Lua::set_memory_limit(256 MiB)`".
    #[test]
    fn default_limits_are_those_of_pipeline_md() {
        let md = pipeline_md();
        // The whole bullet: its first line and the indented continuation lines.
        let (_, from) = md
            .split_once("- Modules are untrusted code")
            .expect("the §4.1 limits bullet");
        let line: String = from
            .lines()
            .take_while(|l| !l.trim_start().starts_with("- ") && !l.trim().is_empty())
            .map(str::trim)
            .collect::<Vec<_>>()
            .join(" ");
        let line = line.as_str();
        let num = |re: &str| -> u64 {
            let caps = regex::Regex::new(re)
                .unwrap()
                .captures(line)
                .unwrap_or_else(|| panic!("`{re}` in: {line}"));
            caps[1].parse().unwrap()
        };
        let hook = 10u64.pow(num(r"every 10\^(\d+) instructions") as u32);
        let max = 10u64.pow(num(r"abort after 10\^(\d+)") as u32);
        let mem = num(r"set_memory_limit\((\d+) MiB\)") << 20;
        let l = LuaLimits::default();
        assert_eq!(
            (
                u64::from(l.hook_every),
                l.max_instructions,
                l.max_memory as u64
            ),
            (hook, max, mem)
        );
    }

    #[test]
    #[ignore = "T05"]
    fn tables_convert_to_json() {
        let src = module(
            "Module:T",
            "return { List = { 'a', 'b' }, Map = { x = 1, y = 2.5 }, Empty = {}, Inf = math.huge, \
             NegInf = -math.huge, Holes = { [1] = 1, [3] = 3 } }",
        );
        let out = eval_wiki_modules(&[src], &LuaLimits::default()).unwrap();
        // `serde_json::Value` cannot hold ±∞: `math.huge` is the string "Infinity", its JSON output form (§4.1) and
        // the readable form of `serde_util::ext_f64`.
        assert_eq!(
            out["Module:T"],
            json!({ "List": ["a", "b"], "Map": { "x": 1, "y": 2.5 }, "Empty": {}, "Inf": "Infinity",
                    "NegInf": "-Infinity", "Holes": { "1": 1, "3": 3 } })
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

    /// The standard-library entry points of pipeline.md §4.1 are absent, and so are `loadstring` and `loadfile` of the
    /// Lua 5.1 base library (`loadfile` reads a file from disk, and its parse error would echo the file's text). The
    /// check is by `type()`, so a present function cannot pass by failing for another reason (`dofile('x')` raises
    /// for a missing file too).
    #[test]
    #[ignore = "T05"]
    fn sandbox_has_no_io_os_package_or_load() {
        let absent = [
            "io",
            "os",
            "package",
            "debug",
            "load",
            "loadstring",
            "loadfile",
            "dofile",
        ];
        let code = format!(
            "return {{ {} }}",
            absent
                .iter()
                .map(|g| format!("{g} = type({g})"))
                .collect::<Vec<_>>()
                .join(", ")
        );
        let out =
            eval_wiki_modules(&[module("Module:Probe", &code)], &LuaLimits::default()).unwrap();
        let expected: serde_json::Map<String, serde_json::Value> = absent
            .iter()
            .map(|g| (g.to_string(), json!("nil")))
            .collect();
        assert_eq!(out["Module:Probe"], serde_json::Value::Object(expected));
        // `require` is the batch stub: an unknown title is an error with the module's title, never a file search.
        for code in [
            "return require('Module:Missing')",
            "return mw.loadData('Module:Missing')",
        ] {
            let err = eval_wiki_modules(&[module("Module:Evil", code)], &LuaLimits::default())
                .unwrap_err();
            assert!(
                matches!(&err, DatagenError::Lua { title, .. } if &**title == "Module:Evil"),
                "{code}: {err}"
            );
        }
    }

    /// Each limit trips on its own (the other one set high) and the error names the module and the limit.
    #[test]
    #[ignore = "T05"]
    fn limits_trip_with_the_module_title() {
        let instructions = LuaLimits {
            hook_every: 1_000,
            max_instructions: 100_000,
            ..LuaLimits::default()
        };
        let memory = LuaLimits {
            max_memory: 1 << 20,
            ..LuaLimits::default()
        };
        for (code, limits, name) in [
            ("while true do end", instructions, "instruction"),
            // One 4 MiB string, built inside `string.rep`: no VM instructions to count, only memory.
            ("return ('x'):rep(2^22)", memory, "memory"),
        ] {
            let err = eval_wiki_modules(&[module("Module:Hog", code)], &limits).unwrap_err();
            let DatagenError::Lua { title, detail } = &err else {
                panic!("{code}: {err}");
            };
            assert_eq!(&**title, "Module:Hog");
            assert!(detail.to_lowercase().contains(name), "{code}: {detail}");
        }
    }
}
