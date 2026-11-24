//! Every JSON example of server-api.md parses into its type (work-breakdown.md T01b acceptance).
//!
//! The examples are read from the document, not copied. Where the document elides parts of an example (`{}` or
//! `"…": "…"` placeholders, "keys illustrative"), the test fills exactly those parts with complete values and says so;
//! everything the document does show is parsed as written.

use crate::api::*;
use crate::scenario::CalcInput;
use serde_json::{Value, json};

fn server_api() -> String {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../docs/architecture/server-api.md");
    std::fs::read_to_string(path).expect("docs/architecture/server-api.md")
}

/// The contents of every fenced block with the given info string.
fn blocks(text: &str, info: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut rest = text;
    let open = format!("```{info}\n");
    while let Some(i) = rest.find(&open) {
        let body = &rest[i + open.len()..];
        let end = body.find("```").expect("closed block");
        out.push(body[..end].to_owned());
        rest = &body[end + 3..];
    }
    out
}

/// Inline code spans that hold a JSON object.
fn inline_json(text: &str) -> Vec<Value> {
    let mut out = Vec::new();
    let mut in_block = false;
    for line in text.lines() {
        if line.starts_with("```") {
            in_block = !in_block;
            continue;
        }
        if in_block {
            continue;
        }
        for (i, span) in line.split('`').enumerate() {
            if i % 2 == 1
                && span.trim_start().starts_with('{')
                && let Ok(v) = serde_json::from_str::<Value>(span)
            {
                out.push(v);
            }
        }
    }
    out
}

fn parse<T: serde::de::DeserializeOwned + serde::Serialize>(what: &str, v: Value) -> T {
    let t: T = serde_json::from_value(v.clone()).unwrap_or_else(|e| panic!("{what}: {e}\n{v:#}"));
    // What parsed also serializes (no non-finite numbers, no int53 overflow).
    serde_json::to_value(&t).unwrap_or_else(|e| panic!("{what}: {e}"));
    t
}

/// A complete RankedBuild for the elided `best` of the `done` example.
fn ranked(input: &Value) -> Value {
    json!({ "input": input, "code": "wfd1-x", "score": 1.0,
            "key": { "band": 0, "primary": 1.0, "tie": 0.0, "tie2": 0.0 },
            "ev_mc_deviation": false, "mix": [], "draft_items": [] })
}

#[test]
fn every_json_example_parses() {
    let doc = server_api();

    // §4: the two CalcInput examples.
    let json_blocks = blocks(&doc, "json");
    assert_eq!(
        json_blocks.len(),
        2,
        "server-api.md has a new ```json example: add it to this test"
    );
    let braton: Value = serde_json::from_str(&json_blocks[0]).expect("example 1 is JSON");
    let input: CalcInput = parse("§4 Braton Prime example", braton.clone());
    assert_eq!(input.v, 1);
    assert_eq!(input.build.slots.len(), 8);
    let mut local: Value = serde_json::from_str(&json_blocks[1]).expect("example 2 is JSON");
    // Elided: `"scenario": { "…": "…" }` stands for any scenario; use the first example's.
    assert_eq!(local["scenario"], json!({ "…": "…" }), "elision changed");
    local["scenario"] = braton["scenario"].clone();
    let local: CalcInput = parse("§4 local-key example", local);
    assert_eq!(local.x.len(), 1);
    assert!(local.build.weapon.is_local());

    // §8.1: the 202 response.
    let inline = inline_json(&doc);
    assert_eq!(
        inline.len(),
        1,
        "server-api.md has a new inline JSON example: add it to this test"
    );
    let job: JobAccepted = parse("§8.1 202 response", inline[0].clone());
    assert_eq!(&*job.job, "opt-7f3a9c");

    // §8.2: the SSE stream.
    let sse = blocks(&doc, "text")
        .into_iter()
        .find(|b| b.contains("event: started"))
        .expect("§8.2 SSE example");
    let mut events = Vec::new();
    let (mut name, mut id) = (None::<String>, None::<u64>);
    for line in sse.lines() {
        if let Some(n) = line.strip_prefix("event: ") {
            name = Some(n.trim().to_owned());
        } else if let Some(i) = line.strip_prefix("id: ") {
            id = Some(i.trim().parse().expect("numeric id"));
        } else if let Some(d) = line.strip_prefix("data: ") {
            events.push((
                name.take().expect("event before data"),
                id.take(),
                d.to_owned(),
            ));
        }
    }
    let names: Vec<&str> = events.iter().map(|(n, _, _)| n.as_str()).collect();
    assert_eq!(names, ["started", "progress", "incumbent", "done"]);
    assert!(events.iter().all(|(_, id, _)| id.is_some()));
    for (name, _, data) in &events {
        let mut v: Value = serde_json::from_str(data).expect("SSE data is JSON");
        match name.as_str() {
            "incumbent" => {
                // Elided: `"build": {}` and `"scenario": {}` inside the build's input.
                let input = &mut v["build"]["input"];
                assert_eq!(input["build"], json!({}), "elision changed");
                assert_eq!(input["scenario"], json!({}), "elision changed");
                *input = braton.clone();
            }
            "done" => {
                // Elided: `"best": {}` and `"stats": {}`.
                let r = &mut v["result"];
                assert_eq!(r["best"], json!({}), "elision changed");
                assert_eq!(r["stats"], json!({}), "elision changed");
                r["best"] = ranked(&braton);
                r["stats"] = json!({ "evaluated": 0, "bounded": 0, "pruned": 0, "leaves": 0, "phases": [],
                                     "threads": 1, "reproducible": true, "elapsed_ms": 0 });
            }
            _ => {}
        }
        let e = OptEvent::from_sse(name, &v.to_string())
            .unwrap_or_else(|err| panic!("SSE `{name}`: {err}\n{v:#}"));
        assert_eq!(e.name(), name);
        // The SSE framing round-trips; events without documents write back exactly the example's fields (documents
        // expand their defaults).
        let (n, data) = e.to_sse().unwrap();
        assert_eq!(n, name);
        assert_eq!(OptEvent::from_sse(n, &data).unwrap(), e, "SSE `{name}`");
        if matches!(name.as_str(), "started" | "progress") {
            let back: Value = serde_json::from_str(&data).unwrap();
            assert_eq!(
                super::normalize_numbers(&back),
                super::normalize_numbers(&v),
                "SSE `{name}`"
            );
        }
        match (&e, name.as_str()) {
            (OptEvent::Started { plan, .. }, _) => {
                assert_eq!(plan.positions.len(), 2);
                assert_eq!(
                    plan.excluded[0].reason.key.as_str(),
                    "opt.excluded.unmodelled"
                );
            }
            (OptEvent::Progress { gap, fraction, .. }, _) => {
                assert_eq!(*gap, None);
                assert_eq!(*fraction, Some(0.42));
            }
            (OptEvent::Incumbent { rank, build }, _) => {
                assert_eq!(*rank, 1);
                assert_eq!(build.key.tie, -412.0);
                assert_eq!(build.mix, [crate::DamageType::COR, crate::DamageType::HEA]);
            }
            (OptEvent::Done { result }, _) => {
                assert_eq!(
                    result.proof,
                    Proof::ProvenOptimal {
                        scope: ProofScope::MainGivenContext,
                        assumes_fast_error: Some(0.005)
                    }
                );
                assert_eq!(result.condition_sensitivity, Some(31002.7));
                assert_eq!(result.buff_table, None);
            }
            (e, n) => panic!("unexpected event {n}: {e:?}"),
        }
    }
}

#[test]
fn route_table_bodies_parse() {
    // The inline shapes of the route table (server-api.md §3) and their example values.
    let _: EncodeResponse = parse(
        "encode",
        json!({ "code": "wfd1-abc", "path": "/?b=wfd1-abc" }),
    );
    let _: DecodeRequest = parse("decode", json!({ "text": "wfd1-abc" }));
    let _: ExportInspirationResponse = parse(
        "export",
        json!({ "url": "https://example.invalid/?p=x&n=1" }),
    );
    let r: RefreshRequest = parse("refresh", json!({}));
    assert_eq!(r.curated, None);
    let r: RefreshRequest = parse("refresh", json!({ "curated": "embedded" }));
    assert_eq!(r.curated, Some(CuratedSource::Embedded));
    let _: ActivateRequest = parse("activate", json!({ "bundle": "embedded" }));
    // 409 `opt.job_running` with `args.job` (§3, §8.1).
    let e: ErrorResponse = parse(
        "409",
        json!({ "error": { "key": "opt.job_running", "args": { "job": "opt-7f3a9c" } } }),
    );
    assert_eq!(e.error.key, crate::message::OptKey::JobRunning);
    // Unknown fields in requests are rejected (domain-model.md §1).
    assert!(serde_json::from_value::<DecodeRequest>(json!({ "text": "x", "extra": 1 })).is_err());
}
