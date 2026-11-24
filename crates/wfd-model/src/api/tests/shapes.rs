//! The generated TypeScript of the API types matches server-api.md §5-§9 (and `Message` of §4): every documented
//! field exists with the same optionality and nullability, inline objects of the document match the named types
//! that implement them (recursively, union arms by their tag), and fields the document does not list are optional
//! additions. Differences required by the spec are listed in `KNOWN_DIFFERENCES` (CONTRACTS.md, T01b).

use super::ts::{
    Def, Field, array_elem, mapped, named, object_fields, parse_defs, split_top, strip_comments,
    substitute,
};
use std::collections::BTreeMap;

/// (generated type, field, reason): deliberate differences from server-api.md.
const KNOWN_DIFFERENCES: &[(&str, &str, &str)] = &[
    (
        "ViewEv",
        "per_shot",
        "spec 08 §8.5: not available for gauge-gated attacks",
    ),
    (
        "ViewEv",
        "per_magazine",
        "spec 08 §8.5: none for N_mag = ∞ and gauge-gated attacks",
    ),
    ("Summary", "sd", "spec 08 E-8.19: R = 1 gives no sd"),
    ("Summary", "ci95", "spec 08 E-8.19: R = 1 gives no interval"),
];

/// server-api.md types not compared, with the reason.
const SKIPPED: &[(&str, &str)] = &[(
    "Candidate",
    "server-api's union is discriminated by a boolean (`legal: true | false`), which serde and ts-rs cannot express; \
     the struct `{ key, legal, reason? }` carries the same JSON",
)];

/// Inline shapes of the route table (server-api.md §2-§3) and the type that implements each.
const ROUTE_SHAPES: &[(&str, &str)] = &[
    ("{ \"error\": Message }", "ErrorResponse"),
    ("{ code: string, path: string }", "EncodeResponse"),
    ("{ text: string }", "DecodeRequest"),
    ("{ url: string }", "ExportInspirationResponse"),
    ("{ job: string }", "JobAccepted"),
    ("{ curated?: \"embedded\" }", "RefreshRequest"),
    ("{ bundle: string }", "ActivateRequest"),
];

fn server_api() -> String {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../docs/architecture/server-api.md");
    std::fs::read_to_string(path).expect("docs/architecture/server-api.md")
}

/// Declarations of every ```ts block from `from` on.
fn api_defs(doc: &str, from: &str) -> BTreeMap<String, Def> {
    let start = doc
        .find(from)
        .unwrap_or_else(|| panic!("no section `{from}`"));
    let mut out = BTreeMap::new();
    let mut rest = &doc[start..];
    while let Some(i) = rest.find("```ts\n") {
        let body = &rest[i + 6..];
        let end = body.find("```").expect("closed block");
        out.extend(parse_defs(&strip_comments(&body[..end])));
        rest = &body[end + 3..];
    }
    out
}

struct Cmp<'a> {
    generated: &'a BTreeMap<String, Def>,
    problems: Vec<String>,
    /// Fields compared (server-api field ↔ generated field).
    fields: usize,
    /// `KNOWN_DIFFERENCES` entries that were needed.
    known_hit: std::collections::BTreeSet<(String, String)>,
}

impl Cmp<'_> {
    /// Resolves names (and generics) of the generated side to their expression, remembering the last name.
    fn resolve(&self, mut ty: String, mut name: Option<String>) -> (String, Option<String>) {
        for _ in 0..16 {
            match named(&ty) {
                Some((n, args)) if self.generated.contains_key(&n) => {
                    let d = &self.generated[&n];
                    ty = substitute(&d.expr, &d.params, &args);
                    name = Some(n);
                }
                _ => break,
            }
        }
        (ty, name)
    }

    fn non_null(ty: &str) -> Vec<String> {
        split_top(ty, &['|'])
            .into_iter()
            .filter(|a| a != "null")
            .collect()
    }

    /// Compares an api type with a generated type structurally where the api type has inline objects.
    fn compare(&mut self, what: &str, api: &str, gen_ty: &str, gen_name: Option<String>) {
        if !api.contains('{') {
            return;
        }
        let (gen_ty, gen_name) = self.resolve(gen_ty.to_owned(), gen_name);
        let api_alts = Self::non_null(api);
        let gen_alts = Self::non_null(&gen_ty);
        if api_alts.len() == 1 && gen_alts.len() == 1 {
            let (a, g) = (&api_alts[0], &gen_alts[0]);
            let a = a.trim_start_matches('(').trim_end_matches(')');
            if let Some(ae) = array_elem(a) {
                match array_elem(g) {
                    Some(ge) => self.compare(&format!("{what}[]"), &ae, &ge, None),
                    None => self
                        .problems
                        .push(format!("{what}: array in server-api, `{g}` generated")),
                }
                return;
            }
            if let Some(v) = record_value(a) {
                match mapped(g) {
                    Some((_, gv)) => self.compare(&format!("{what}{{}}"), &v, &gv, None),
                    None if object_fields(g).is_some() => {}
                    None => self
                        .problems
                        .push(format!("{what}: record in server-api, `{g}` generated")),
                }
                return;
            }
            if let Some(af) = object_fields(a) {
                match object_fields(g) {
                    Some(gf) => {
                        self.compare_objects(what, &af, &gf, gen_name.as_deref().unwrap_or(what))
                    }
                    None => self
                        .problems
                        .push(format!("{what}: object in server-api, `{g}` generated")),
                }
            }
            return;
        }
        // Unions: match object arms by their tag (a literal field) or their field names.
        let mut gen_arms: BTreeMap<String, (Vec<Field>, Option<String>)> = BTreeMap::new();
        for g in &gen_alts {
            let (g, n) = self.resolve(g.clone(), None);
            if let Some(f) = object_fields(&g) {
                gen_arms.insert(arm_key(&f), (f, n));
            }
        }
        for a in &api_alts {
            let Some(af) = object_fields(a) else { continue };
            let key = arm_key(&af);
            match gen_arms.get(&key) {
                Some((gf, n)) => {
                    let gname = n.clone().unwrap_or_else(|| format!("{what}|{key}"));
                    self.compare_objects(&format!("{what}|{key}"), &af, gf, &gname)
                }
                None => self.problems.push(format!(
                    "{what}: union arm `{key}` of server-api has no generated counterpart in `{gen_ty}`"
                )),
            }
        }
    }

    fn compare_objects(&mut self, what: &str, api: &[Field], generated: &[Field], gen_name: &str) {
        let known = |f: &str| {
            KNOWN_DIFFERENCES
                .iter()
                .any(|(t, k, _)| *t == gen_name && *k == f)
        };
        for a in api {
            self.fields += 1;
            let Some(g) = generated.iter().find(|g| g.name == a.name) else {
                self.problems.push(format!(
                    "{what}.{}: missing in generated `{gen_name}`",
                    a.name
                ));
                continue;
            };
            let differs = a.optional != g.optional || a.nullable() != g.nullable();
            if differs && known(&a.name) {
                self.known_hit.insert((gen_name.to_owned(), a.name.clone()));
            }
            if !known(&a.name) {
                if a.optional != g.optional {
                    self.problems.push(format!(
                        "{what}.{}: optional {} in server-api, {} in `{gen_name}` (`{}` vs `{}`)",
                        a.name, a.optional, g.optional, a.ty, g.ty
                    ));
                }
                if a.nullable() != g.nullable() {
                    self.problems.push(format!(
                        "{what}.{}: nullable {} in server-api, {} in `{gen_name}` (`{}` vs `{}`)",
                        a.name,
                        a.nullable(),
                        g.nullable(),
                        a.ty,
                        g.ty
                    ));
                }
            }
            self.compare(&format!("{what}.{}", a.name), &a.ty, &g.ty, None);
        }
        for g in generated {
            if !api.iter().any(|a| a.name == g.name) && !g.optional && !known(&g.name) {
                self.problems.push(format!(
                    "{what}.{}: in `{gen_name}` but not in server-api, and not optional",
                    g.name
                ));
            }
        }
    }
}

/// `Record<K, V>` or `Partial<Record<K, V>>` → V.
fn record_value(ty: &str) -> Option<String> {
    let t = ty.trim();
    let t = t
        .strip_prefix("Partial<")
        .and_then(|x| x.strip_suffix('>'))
        .unwrap_or(t);
    let inner = t.strip_prefix("Record<")?.strip_suffix('>')?;
    let parts = split_top(inner, &[',']);
    (parts.len() == 2).then(|| parts[1].clone())
}

/// Key of a union arm: its literal tag field (`event=“step”`), else its sorted field names.
fn arm_key(fields: &[Field]) -> String {
    for f in fields {
        if !f.optional && f.ty.starts_with('"') && split_top(&f.ty, &['|']).len() == 1 {
            return format!("{}={}", f.name, f.ty);
        }
    }
    let mut names: Vec<&str> = fields.iter().map(|f| f.name.as_str()).collect();
    names.sort_unstable();
    names.join(",")
}

#[test]
fn api_types_match_server_api() {
    let doc = server_api();
    let generated = super::ts::generated_defs();
    let mut api = api_defs(&doc, "## 5. Catalogue and schema routes");
    let section4 = api_defs(&doc, "## 4. Shared document shapes");
    api.insert("Message".into(), section4["Message"].clone());
    let mut cmp = Cmp {
        generated: &generated,
        problems: Vec::new(),
        fields: 0,
        known_hit: Default::default(),
    };
    let mut compared = 0;
    for (name, def) in &api {
        if SKIPPED.iter().any(|(n, _)| n == name) {
            continue;
        }
        assert!(
            generated.contains_key(name),
            "server-api.md type `{name}` has no generated declaration"
        );
        cmp.compare(name, &def.expr, name, Some(name.clone()));
        compared += 1;
    }
    for (shape, ty) in ROUTE_SHAPES {
        let doc_form = shape.replace(", ", "; ");
        assert!(
            doc.contains(shape) || doc.contains(&doc_form),
            "route shape `{shape}` no longer in server-api.md"
        );
        cmp.compare(ty, shape, ty, Some((*ty).to_owned()));
    }
    assert!(compared >= 35, "only {compared} server-api types found");
    assert!(cmp.problems.is_empty(), "{}", cmp.problems.join("\n"));
    assert!(cmp.fields > 400, "only {} fields compared", cmp.fields);
    for (t, f, why) in KNOWN_DIFFERENCES {
        assert!(
            cmp.known_hit.contains(&((*t).to_owned(), (*f).to_owned())),
            "KNOWN_DIFFERENCES entry {t}.{f} ({why}) is no longer needed"
        );
    }
}

#[test]
fn generated_typescript_has_no_bigint() {
    // domain-model.md §1: every u64 of an API or document type carries #[ts(type = "number")].
    let files = crate::api::codegen::typescript_files().unwrap();
    assert!(files.len() > 150, "only {} files", files.len());
    for (path, text) in &files {
        assert!(!text.contains("bigint"), "{path} uses bigint:\n{text}");
    }
    // CompatFlags is never exported (it is `Array<string>` inside `Profile`).
    assert!(!files.contains_key("CompatFlags.ts"));
    assert!(files["Profile.ts"].contains("Array<string>"));
    // The message keys are exported.
    let keys = &files["messageKeys.ts"];
    for k in crate::message::all_keys() {
        assert!(keys.contains(&format!("\"{}\":", k.key)), "{}", k.key);
    }
    assert!(files["index.ts"].contains("export * from \"./CalcReport\";"));
}
