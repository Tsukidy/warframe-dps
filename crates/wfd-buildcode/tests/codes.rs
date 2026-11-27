//! Share codes (AD-18; server-api.md §7) on the scenario fixtures: encode → decode identity in every accepted text
//! form, determinism, migration of the current version, and rejection of what is not ours. Every test needs a T10
//! body and is ignored until T10 implements it (work-breakdown.md T02b, T10). T10 adds its acceptance tests next to
//! these: the proptest round trip (documents with local keys decoded against a bundle with different local
//! numbering), the sample codes of `tests/codes/v1`, the import of `research/test-cases/user-example-furis.json`
//! with zero unmapped entries, and export → import identity.

use std::path::{Path, PathBuf};
use wfd_buildcode::{CODE_PREFIX, CodeError, decode, encode, migrate};
use wfd_model::{CalcInput, DOC_VERSION};

/// `data/fixtures/scenarios/*.json` (global keys) and `real/*.json` (session-local keys with `x`).
fn scenario_documents() -> Vec<(PathBuf, CalcInput)> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data/fixtures/scenarios");
    let mut out = Vec::new();
    for dir in [root.clone(), root.join("real")] {
        let mut paths: Vec<PathBuf> = std::fs::read_dir(&dir)
            .unwrap_or_else(|e| panic!("{}: {e}", dir.display()))
            .map(|e| e.unwrap().path())
            .filter(|p| p.extension().is_some_and(|x| x == "json"))
            .collect();
        paths.sort();
        for p in paths {
            let text = std::fs::read_to_string(&p).unwrap();
            let input: CalcInput =
                serde_json::from_str(&text).unwrap_or_else(|e| panic!("{}: {e}", p.display()));
            out.push((p, input));
        }
    }
    assert!(!out.is_empty());
    out
}

/// The fixtures themselves are documents (this part runs today: the inputs of the ignored tests below are valid).
#[test]
fn scenario_fixtures_are_documents() {
    let docs = scenario_documents();
    assert!(
        docs.iter().any(|(_, d)| !d.x.is_empty()),
        "real/ documents carry local keys in x (AD-18)"
    );
}

#[test]
#[ignore = "T10: encode, decode"]
fn codes_round_trip_in_every_text_form() {
    for (path, input) in scenario_documents() {
        let code = encode(&input);
        assert!(code.starts_with(CODE_PREFIX), "{}: {code}", path.display());
        assert!(
            code[CODE_PREFIX.len()..]
                .bytes()
                .all(|c| c.is_ascii_alphanumeric() || c == b'-' || c == b'_'),
            "base64url without padding: {code}"
        );
        assert_eq!(encode(&input), code, "deterministic");
        for text in [
            code.clone(),
            format!("/?b={code}"),
            format!("http://127.0.0.1:8765/?b={code}"),
        ] {
            let (back, report) =
                decode(&text).unwrap_or_else(|e| panic!("{}: {text}: {e}", path.display()));
            assert_eq!(back, input, "{}: {text}", path.display());
            assert!(
                report.unmapped.is_empty() && report.warnings.is_empty(),
                "{report:?}"
            );
        }
    }
}

#[test]
#[ignore = "T10: migrate"]
fn current_version_migrates_unchanged_and_newer_is_rejected() {
    for (path, input) in scenario_documents() {
        let raw = serde_json::to_value(&input).unwrap();
        assert_eq!(
            migrate(DOC_VERSION, raw.clone()).unwrap(),
            input,
            "{}",
            path.display()
        );
        assert!(matches!(
            migrate(DOC_VERSION + 1, raw),
            Err(CodeError::VersionUnsupported { v, max }) if v == DOC_VERSION + 1 && max == DOC_VERSION
        ));
    }
}

#[test]
#[ignore = "T10: decode"]
fn text_that_is_not_ours_is_rejected() {
    assert!(matches!(decode("hello"), Err(CodeError::Unrecognized)));
    assert!(matches!(
        decode("https://warframe-damage.com/?p=N4Ig&n=1"),
        Err(CodeError::Unrecognized)
    ));
    assert!(matches!(decode("wfd1-!!!"), Err(CodeError::Corrupt { .. })));
}
