//! The public API of `wfd-buildcode` as work-breakdown.md T02b fixes it, and the parts the skeleton already implements
//! (the i18n form of the errors). `signatures_of_work_breakdown_t02b` pins every signature: a change fails to compile
//! here and needs a CONTRACTS.md entry (work-breakdown.md §2.1). Behaviour tests of the T10 bodies live in
//! `tests/codes.rs` (ignored until T10).

use wfd_buildcode::{
    CODE_PREFIX, CodeError, DecodeReport, DecodeStage, ImportReport, decode, encode,
    export_inspiration, import_inspiration, migrate,
};
use wfd_data::GameData;
use wfd_model::message::{DocKey, ImportKey, Message};
use wfd_model::{CalcInput, MessageKey, MsgArg};

#[test]
#[expect(
    clippy::type_complexity,
    reason = "the pinned signatures are spelled out as work-breakdown.md T02b writes them"
)]
fn signatures_of_work_breakdown_t02b() {
    let _: fn(&CalcInput) -> String = encode;
    let _: fn(&str) -> Result<(CalcInput, DecodeReport), CodeError> = decode;
    let _: fn(u16, serde_json::Value) -> Result<CalcInput, CodeError> = migrate;
    let _: fn(&str, &GameData) -> Result<(CalcInput, ImportReport), CodeError> = import_inspiration;
    let _: fn(&CalcInput, &GameData) -> Result<String, CodeError> = export_inspiration;
    assert_eq!(CODE_PREFIX, "wfd1-", "AD-18");
}

fn key_and_args(m: &Message) -> (MessageKey, Vec<(&str, MsgArg)>) {
    (
        m.key.clone(),
        m.args.iter().map(|(k, v)| (&**k, v.clone())).collect(),
    )
}

/// Each error becomes the `import.*` message of its kind with the arguments `message/import.rs` declares; document
/// issues pass through unchanged.
#[test]
fn errors_become_import_messages() {
    let m = CodeError::Unrecognized.messages();
    assert_eq!(m, vec![Message::new(ImportKey::Unrecognized)]);

    let m = CodeError::Corrupt {
        stage: DecodeStage::Deflate,
        detail: "invalid block type".into(),
    }
    .messages();
    assert_eq!(m.len(), 1);
    assert_eq!(
        key_and_args(&m[0]),
        (
            ImportKey::Corrupt.into(),
            vec![("stage", MsgArg::Str("deflate".into()))]
        )
    );

    let m = CodeError::VersionUnsupported { v: 7, max: 1 }.messages();
    assert_eq!(
        key_and_args(&m[0]),
        (
            ImportKey::VersionUnsupported.into(),
            vec![("max", MsgArg::Num(1.0)), ("v", MsgArg::Num(7.0))]
        )
    );

    let m = CodeError::ExportUnsupported {
        feature: "toggles.melee_plans".into(),
    }
    .messages();
    assert_eq!(
        key_and_args(&m[0]),
        (
            ImportKey::ExportUnsupported.into(),
            vec![("feature", MsgArg::Str("toggles.melee_plans".into()))]
        )
    );

    let issues = vec![
        Message::new(DocKey::KeyUnknown)
            .arg("what", "weapon")
            .arg("key", 99u32),
    ];
    assert_eq!(CodeError::Invalid(issues.clone()).messages(), issues);

    // Every argument a message carries is one its key declares.
    for e in [
        CodeError::Unrecognized,
        CodeError::Corrupt {
            stage: DecodeStage::Json,
            detail: "".into(),
        },
        CodeError::VersionUnsupported { v: 2, max: 1 },
        CodeError::ExportUnsupported {
            feature: "x".into(),
        },
    ] {
        for m in e.messages() {
            let declared = ImportKey::ALL
                .iter()
                .find(|k| m.key == **k)
                .unwrap_or_else(|| panic!("{:?} is not an import key", m.key))
                .args();
            let mut names: Vec<&str> = m.args.keys().map(|k| &**k).collect();
            let mut want: Vec<&str> = declared.to_vec();
            names.sort_unstable();
            want.sort_unstable();
            assert_eq!(names, want, "{e:?}");
        }
    }
}

/// The stage names are the `stage` values `import.corrupt` documents.
#[test]
fn decode_stage_names() {
    let names: Vec<String> = [
        DecodeStage::Base64,
        DecodeStage::Deflate,
        DecodeStage::Lz,
        DecodeStage::Json,
    ]
    .iter()
    .map(|s| s.to_string())
    .collect();
    assert_eq!(names, ["base64", "deflate", "lz", "json"]);
}
