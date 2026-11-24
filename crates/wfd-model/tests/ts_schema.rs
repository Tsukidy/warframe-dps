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

// ---------------------------------------------------------------------------------------------------------------
// Document shapes: generated TypeScript vs server-api.md §4 and vs the JSON serde writes
// ---------------------------------------------------------------------------------------------------------------

/// Removes `//` line comments and `/* */` block comments.
fn strip_comments(text: &str) -> String {
    let mut out = String::new();
    let mut rest = text;
    while !rest.is_empty() {
        if let Some(r) = rest.strip_prefix("/*") {
            rest = r.find("*/").map_or("", |i| &r[i + 2..]);
        } else if let Some(r) = rest.strip_prefix("//") {
            rest = r.find('\n').map_or("", |i| &r[i..]);
        } else {
            let c = rest.chars().next().unwrap();
            out.push(c);
            rest = &rest[c.len_utf8()..];
        }
    }
    out
}

fn depth_delta(c: char) -> i32 {
    match c {
        '{' | '[' | '(' | '<' => 1,
        '}' | ']' | ')' | '>' => -1,
        _ => 0,
    }
}

/// Splits at `sep` characters on bracket depth 0.
fn split_top(text: &str, seps: &[char]) -> Vec<String> {
    let mut out = vec![String::new()];
    let mut depth = 0;
    for c in text.chars() {
        if depth == 0 && seps.contains(&c) {
            out.push(String::new());
            continue;
        }
        depth += depth_delta(c);
        out.last_mut().unwrap().push(c);
    }
    out.into_iter()
        .map(|s| s.trim().to_owned())
        .filter(|s| !s.is_empty())
        .collect()
}

/// The type expression of `type <name> = …;` in a TypeScript text (comments already removed).
fn type_expr(text: &str, name: &str) -> Option<String> {
    for pat in [format!("export type {name} ="), format!("type {name} =")] {
        if let Some(i) = text.find(&pat) {
            let body = &text[i + pat.len()..];
            let mut depth = 0;
            for (j, c) in body.char_indices() {
                if depth == 0 && c == ';' {
                    return Some(body[..j].trim().to_owned());
                }
                depth += depth_delta(c);
            }
        }
    }
    None
}

/// One field of an object type.
#[derive(Debug, Clone)]
struct TsField {
    optional: bool,
    nullable: bool,
    ty: String,
}

/// Top-level fields of an object type expression `{ a: T; b?: U }` (`;` or `,` separated).
fn object_fields(expr: &str) -> std::collections::BTreeMap<String, TsField> {
    let expr = expr.trim();
    let inner = expr
        .strip_prefix('{')
        .and_then(|e| e.strip_suffix('}'))
        .unwrap_or_else(|| panic!("not an object type: {expr}"));
    let mut out = std::collections::BTreeMap::new();
    for field in split_top(inner, &[';', ',']) {
        let (name, ty) = field
            .split_once(':')
            .unwrap_or_else(|| panic!("bad field `{field}`"));
        let name = name.trim();
        let (name, optional) = match name.strip_suffix('?') {
            Some(n) => (n.trim(), true),
            None => (name, false),
        };
        let ty = ty.trim().to_owned();
        let nullable = split_top(&ty, &['|']).iter().any(|a| a == "null");
        out.insert(
            name.trim_matches('"').to_owned(),
            TsField {
                optional,
                nullable,
                ty,
            },
        );
    }
    out
}

/// The object type of a field type such as `{ … }`, `{ … }[]` or `{ … } | null`.
fn inline_object(ty: &str) -> String {
    let alts = split_top(ty, &['|']);
    let obj = alts
        .iter()
        .find(|a| a.starts_with('{'))
        .unwrap_or_else(|| panic!("no inline object in `{ty}`"));
    obj.strip_suffix("[]").unwrap_or(obj).trim().to_owned()
}

fn server_api_section_4() -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/architecture/server-api.md");
    let text = std::fs::read_to_string(path).unwrap();
    let start = text
        .find("## 4. Shared document shapes")
        .expect("server-api §4");
    let rest = &text[start..];
    let open = rest.find("```ts").expect("ts block") + "```ts".len();
    let close = rest[open..].find("```").expect("end of block");
    strip_comments(&rest[open..open + close])
}

fn generated(dir: &Path, name: &str) -> String {
    let text = std::fs::read_to_string(dir.join(format!("{name}.ts")))
        .unwrap_or_else(|e| panic!("{name}.ts: {e}"));
    let text = strip_comments(&text);
    type_expr(&text, name).unwrap_or_else(|| panic!("no type {name} in {name}.ts"))
}

/// Exports `CalcInput` and its dependencies once per test binary (the tests run in parallel).
fn export_documents() -> &'static Path {
    static DIR: std::sync::OnceLock<std::path::PathBuf> = std::sync::OnceLock::new();
    DIR.get_or_init(|| {
        let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("ts_export_documents");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        export::<CalcInput>(&dir);
        dir
    })
}

/// Fields that differ from server-api.md §4 on purpose (CONTRACTS.md, T01a): (generated type, field).
const KNOWN_DIFFERENCES: &[(&str, &str)] = &[
    // T-08 default "the maximum" rank (spec 00 §6.1): the rank may be omitted.
    ("Stealth", "rank"),
];

/// Compares a generated object type with the server-api.md §4 shape: every documented field exists with the same
/// optionality and nullability; extra generated fields (additions recorded in CONTRACTS.md) are optional.
fn compare(what: &str, api: &str, gen_name: &str, gen_expr: &str) -> Vec<String> {
    let api = object_fields(api);
    let gen_fields = object_fields(gen_expr);
    let mut problems = Vec::new();
    for (name, a) in &api {
        if KNOWN_DIFFERENCES.contains(&(gen_name, name.as_str())) {
            continue;
        }
        match gen_fields.get(name) {
            None => problems.push(format!("{what}.{name}: missing in {gen_name}.ts")),
            Some(g) => {
                if a.optional != g.optional {
                    problems.push(format!(
                        "{what}.{name}: server-api optional = {}, {gen_name}.ts optional = {} (`{}` vs `{}`)",
                        a.optional, g.optional, a.ty, g.ty
                    ));
                }
                if a.nullable != g.nullable {
                    problems.push(format!(
                        "{what}.{name}: server-api nullable = {}, {gen_name}.ts nullable = {} (`{}` vs `{}`)",
                        a.nullable, g.nullable, a.ty, g.ty
                    ));
                }
            }
        }
    }
    for (name, g) in &gen_fields {
        if !api.contains_key(name) && !g.optional {
            problems.push(format!(
                "{gen_name}.{name}: not in server-api.md §4 and not optional"
            ));
        }
    }
    problems
}

#[test]
fn document_types_match_server_api_section_4() {
    let api = server_api_section_4();
    let dir = export_documents();
    let api_type =
        |n: &str| type_expr(&api, n).unwrap_or_else(|| panic!("server-api §4 has no type {n}"));
    let api_field = |t: &str, f: &str| {
        let fields = object_fields(&api_type(t));
        inline_object(&fields.get(f).unwrap_or_else(|| panic!("{t}.{f}")).ty)
    };
    let mut problems = Vec::new();
    for (what, api_expr, gen_name) in [
        ("CalcInput", api_type("CalcInput"), "CalcInput"),
        ("BuildDoc", api_type("BuildDoc"), "BuildDoc"),
        (
            "BuildDoc.weapon_item",
            api_field("BuildDoc", "weapon_item"),
            "WeaponItemDoc",
        ),
        (
            "BuildDoc.variant",
            api_field("BuildDoc", "variant"),
            "VariantDoc",
        ),
        (
            "BuildDoc.warframe",
            api_field("BuildDoc", "warframe"),
            "WarframeDoc",
        ),
        ("ScenarioDoc", api_type("ScenarioDoc"), "ScenarioDoc"),
        (
            "ScenarioDoc.enemy",
            api_field("ScenarioDoc", "enemy"),
            "EnemySetup",
        ),
        (
            "ScenarioDoc.buffs[]",
            api_field("ScenarioDoc", "buffs"),
            "BuffPick",
        ),
        (
            "ScenarioDoc.conditions",
            api_field("ScenarioDoc", "conditions"),
            "ConditionPolicy",
        ),
        (
            "ScenarioDoc.eval",
            api_field("ScenarioDoc", "eval"),
            "EvalSettings",
        ),
        ("Toggles", api_type("Toggles"), "Toggles"),
        (
            "Toggles.co_status_count",
            api_field("Toggles", "co_status_count"),
            "CoStatusCount",
        ),
        (
            "Toggles.stealth",
            api_field("Toggles", "stealth"),
            "Stealth",
        ),
        ("Toggles.heavy", api_field("Toggles", "heavy"), "Heavy"),
        ("SlotEntry", api_type("SlotEntry"), "SlotEntry"),
    ] {
        problems.extend(compare(
            what,
            &api_expr,
            gen_name,
            &generated(dir, gen_name),
        ));
    }
    // Polarity maps are partial (server-api: `Partial<Record<SlotId, Polarity>>`).
    for gen_name in ["WeaponItemDoc", "WarframeDoc"] {
        let f = object_fields(&generated(dir, gen_name));
        let ty = &f["polarities"].ty;
        if !ty.contains("]?:") {
            problems.push(format!("{gen_name}.polarities is not partial: `{ty}`"));
        }
    }
    assert!(problems.is_empty(), "{}", problems.join("\n"));
}

/// Checks a JSON object written by serde against a generated object type: every required field is present, every
/// present field is declared, `null` only where the type allows it.
fn check_json(dir: &Path, gen_name: &str, json: &serde_json::Value) -> Vec<String> {
    let fields = object_fields(&generated(dir, gen_name));
    let obj = json
        .as_object()
        .unwrap_or_else(|| panic!("{gen_name}: not an object: {json}"));
    let mut problems = Vec::new();
    for (name, f) in &fields {
        match obj.get(name) {
            None if !f.optional => problems.push(format!(
                "{gen_name}.{name}: required in TypeScript but omitted by serde ({json})"
            )),
            Some(serde_json::Value::Null) if !f.nullable => problems.push(format!(
                "{gen_name}.{name}: serde writes null, TypeScript `{}`",
                f.ty
            )),
            _ => {}
        }
    }
    for name in obj.keys() {
        if !fields.contains_key(name) {
            problems.push(format!("{gen_name}.{name}: written by serde, not declared"));
        }
    }
    problems
}

#[test]
fn generated_document_types_accept_what_serde_writes() {
    let dir = export_documents();
    let minimal: CalcInput = serde_json::from_str(
        r#"{"v":1,"build":{"weapon":7},"scenario":{"enemy":{"enemy":9,"level":30}}}"#,
    )
    .unwrap();
    let full: CalcInput = serde_json::from_str(
        r#"{"v":1,
            "build":{"weapon":7,
              "weapon_item":{"rank":3,"catalyst":false,"capacity_override":60,"polarities":{"gen0":"madurai"},
                             "genesis_installed":false},
              "variant":{"kitgun":{"grip":1,"loader":2},"progenitor":{"element":"TOX","v10":600}},
              "slots":[{"slot":"gen0","content":{"item":{"item":3301,"rank":5,"params":{"value":0.3}}}}],
              "stance_combo":"neutral",
              "warframe":{"frame":5,"reactor":false,"polarities":{"aura0":"any"},"frame_complete":true},
              "companion_set_mods":[11],"companion_kind":"kubrow"},
            "scenario":{"enemy":{"enemy":9,"level":30,"steel_path":true,"eximus":true,"party":2,
                                 "mission":"archon_hunt","context_overguard":true},
              "toggles":{"external_viral":10,"stealth":{"rank":40},"heavy":{},"light_combo":3,"ability_combo":12,
                         "sniper_combo":1.5,"zoom_level":1,"capacity":{"budget":60}},
              "buffs":[{"buff":40,"params":{"value":0.5},"caster":{"ally":{"strength":2.0}}}],
              "custom":{"base":0.5},
              "conditions":{"mode":"simulated","overrides":{}},
              "eval":{"seed":"auto"},
              "profile":{"compat":[]}},
            "x":{"2147483651":"/Lotus/Weapons/Tenno/LongGuns/NewRifle/NewRifleWeapon"}}"#,
    )
    .unwrap();
    let mut problems = Vec::new();
    for input in [&minimal, &full] {
        let v = serde_json::to_value(input).unwrap();
        problems.extend(check_json(dir, "CalcInput", &v));
        let b = &v["build"];
        problems.extend(check_json(dir, "BuildDoc", b));
        for (field, ty) in [
            ("weapon_item", "WeaponItemDoc"),
            ("variant", "VariantDoc"),
            ("warframe", "WarframeDoc"),
        ] {
            if let Some(x) = b.get(field) {
                problems.extend(check_json(dir, ty, x));
            }
        }
        let s = &v["scenario"];
        problems.extend(check_json(dir, "ScenarioDoc", s));
        problems.extend(check_json(dir, "EnemySetup", &s["enemy"]));
        problems.extend(check_json(dir, "Toggles", &s["toggles"]));
        problems.extend(check_json(dir, "EvalSettings", &s["eval"]));
        problems.extend(check_json(dir, "ConditionPolicy", &s["conditions"]));
        for b in s["buffs"].as_array().unwrap() {
            problems.extend(check_json(dir, "BuffPick", b));
        }
        for (field, ty) in [("stealth", "Stealth"), ("heavy", "Heavy")] {
            if !s["toggles"][field].is_null() {
                problems.extend(check_json(dir, ty, &s["toggles"][field]));
            }
        }
    }
    assert!(problems.is_empty(), "{}", problems.join("\n"));
}
