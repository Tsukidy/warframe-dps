//! The `ts` and `schema` features produce TypeScript declarations and JSON schemas for every root type without
//! panicking, and no generated TypeScript uses `bigint` (domain-model.md §1). T01b owns the generators
//! (`cargo xtask gen-ts`, `gen-schema`); this test only proves the model types are exportable.

use std::path::Path;
use ts_rs::TS;
use wfd_model::curated::*;
use wfd_model::*;

fn export<T: TS + 'static>(dir: &Path) {
    let cfg = ts_rs::Config::new().with_out_dir(dir);
    T::export_all(&cfg).unwrap_or_else(|e| panic!("{}: {e}", std::any::type_name::<T>()));
}

#[test]
fn typescript_exports_without_bigint() {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("ts_export");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    export::<Bundle>(&dir);
    export::<CalcInput>(&dir);
    export::<Resolved>(&dir);
    export::<CuratedItemFile>(&dir);
    export::<IncarnonFamilyFile>(&dir);
    export::<MultGroupsFile>(&dir);
    export::<SlotLayoutFile>(&dir);
    export::<RivenTraitsFile>(&dir);
    export::<RivenFormulaFile>(&dir);
    export::<BodyPartsFile>(&dir);
    export::<SpecialDrFile>(&dir);
    export::<TogglesFile>(&dir);
    export::<FactionsFile>(&dir);
    export::<CategoriesFile>(&dir);
    export::<HelminthFile>(&dir);
    export::<FramesFile>(&dir);
    export::<AttackRolesFile>(&dir);
    export::<StatLinesFile>(&dir);
    export::<HeavyClassFile>(&dir);
    export::<HeavyKappaFile>(&dir);
    export::<AttackTriggersFile>(&dir);
    export::<StanceCoFile>(&dir);
    export::<FinishersFile>(&dir);
    export::<ComboSpecialsFile>(&dir);
    let mut files = 0;
    let mut stack = vec![dir.clone()];
    while let Some(d) = stack.pop() {
        for e in std::fs::read_dir(&d).unwrap() {
            let p = e.unwrap().path();
            if p.is_dir() {
                stack.push(p);
            } else if p.extension().is_some_and(|x| x == "ts") {
                files += 1;
                let text = std::fs::read_to_string(&p).unwrap();
                assert!(
                    !text.contains("bigint"),
                    "{} uses bigint:\n{text}",
                    p.display()
                );
            }
        }
    }
    assert!(files > 100, "only {files} declarations exported");
    let seed = std::fs::read_to_string(dir.join("SeedSel.ts")).unwrap();
    assert!(seed.contains("\"auto\" | number"), "{seed}");
}

#[test]
fn json_schemas_generate() {
    for (name, schema) in [
        ("CuratedItemFile", schemars::schema_for!(CuratedItemFile)),
        (
            "IncarnonFamilyFile",
            schemars::schema_for!(IncarnonFamilyFile),
        ),
        ("TogglesFile", schemars::schema_for!(TogglesFile)),
        ("StatLinesFile", schemars::schema_for!(StatLinesFile)),
        ("CalcInput", schemars::schema_for!(CalcInput)),
        ("Bundle", schemars::schema_for!(Bundle)),
    ] {
        let text = serde_json::to_string(&schema).unwrap();
        assert!(text.len() > 200, "{name}: {text}");
    }
    let s = serde_json::to_string(&schemars::schema_for!(StatKey)).unwrap();
    assert!(
        s.contains("\"vuln_add.HP.HEA\""),
        "stat keys are enumerated: {s}"
    );
}
