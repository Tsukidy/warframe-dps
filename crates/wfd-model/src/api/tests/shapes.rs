//! The generated TypeScript of the API types matches server-api.md §5-§9 (and `Message` of §4): every documented
//! field exists with the same optionality, nullability and type, inline objects of the document match the named types
//! that implement them (recursively, union arms by their tag), and fields the document does not list are optional
//! additions. Scalar types are compared after resolving names on both sides (`StableKey` → `number`, `View` → its
//! literal union): base types, literal unions, arrays, tuples and maps must agree. Differences required by the spec
//! or recorded in CONTRACTS.md (T01b) are listed in `KNOWN_DIFFERENCES` (optionality, nullability) and
//! `TYPE_DIFFERENCES` (types); an entry that is no longer needed fails the test.

use super::ts::{
    Def, Field, array_elem, mapped, named, object_fields, parse_defs, split_top, strip_comments,
    substitute,
};
use std::collections::{BTreeMap, BTreeSet};

/// (generated type, field, reason): deliberate optionality and nullability differences from server-api.md.
const KNOWN_DIFFERENCES: &[(&str, &str, &str)] = &[
    // The spec-driven differences of T01b (C-23 … C-27) were written into server-api.md by T01d (rulings of
    // 2026-11-27); these two remain T01b's own design (CONTRACTS.md T01b review fixes).
    (
        "SearchParams",
        "time_limit_s",
        "absent = the optimizer.md §11 default, null = no limit",
    ),
    (
        "SearchParams",
        "mc_confirm",
        "absent = the optimizer.md §11 default, null = no MC confirmation",
    ),
];

/// (generated type, field, reason): deliberate type differences from server-api.md. A field of an inline object is
/// keyed by the path the comparison gives it (`CalcReport` for `CalcReport.capacity`, `OptRequest.search` …).
const TYPE_DIFFERENCES: &[(&str, &str, &str)] = &[
    (
        "ItemSummary",
        "kind",
        "ItemKindName, the kind slug: the bundle's ItemKind carries dense ids (CONTRACTS.md T01b)",
    ),
    (
        "ItemDetail",
        "kind",
        "ItemKindName, the kind slug: the bundle's ItemKind carries dense ids (CONTRACTS.md T01b)",
    ),
    (
        "Meta",
        "api_version",
        "a u16 holding API_VERSION (= 1); a literal TypeScript type would have to change with the constant",
    ),
    (
        "OptRequest.constraints.riven|search.search",
        "traits",
        "a u8; the range 2 ..= 4 is documented on `RivenPolicy::Search::traits`",
    ),
    (
        "MagazineView",
        "ammo_max",
        "06 §11.4: ∞ on robotic weapons, written \"Infinity\" (serde_util::ext_f64)",
    ),
    (
        "FireView",
        "rate",
        "06 §4.1: Lanka's fire rate is math.huge, written \"Infinity\"",
    ),
    (
        "BaseModded",
        "base",
        "06 §11.4: `ammo_max` is ∞ on robotic weapons, written \"Infinity\" (serde_util::ext_f64)",
    ),
    (
        "BaseModded",
        "modded",
        "06 §11.4: `ammo_max` is ∞ on robotic weapons, written \"Infinity\" (serde_util::ext_f64)",
    ),
    (
        "StatBlock",
        "fire_rate",
        "06 §4.1: Lanka's fire rate is math.huge, written \"Infinity\"",
    ),
    (
        "ViewEv",
        "crit_tiers",
        "08 §8.4: always 8 bins, a fixed-length tuple",
    ),
    (
        "ViewEv",
        "crit_tiers_steady",
        "08 §8.4: always 8 bins, a fixed-length tuple",
    ),
    (
        "ViewMc",
        "crit_hist",
        "08 §8.4: always 8 bins, a fixed-length tuple",
    ),
    (
        "CapacityReport",
        "mode",
        "CapacityMode instead of string (CONTRACTS.md T01b)",
    ),
    (
        "ConditionView",
        "override",
        "CondOverride instead of unknown (CONTRACTS.md T01b)",
    ),
];

/// server-api.md types not compared, with the reason.
const SKIPPED: &[(&str, &str)] = &[];

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
    /// Every declaration of server-api.md (§4 on), to resolve the names the document uses.
    api_all: &'a BTreeMap<String, Def>,
    problems: Vec<String>,
    /// Fields compared (server-api field ↔ generated field).
    fields: usize,
    /// Scalar type comparisons made.
    scalars: usize,
    /// `KNOWN_DIFFERENCES` entries that were needed.
    known_hit: BTreeSet<(String, String)>,
    /// `TYPE_DIFFERENCES` entries that were needed.
    type_hit: BTreeSet<(String, String)>,
}

/// (type, field) a comparison reports a type difference under.
type Ctx = (String, String);

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

    /// The expansion of a name: on the server-api side its own declaration first, then the generated one (the
    /// document uses enum names such as `View` without declaring them).
    fn expand(&self, n: &str, args: &[String], api_side: bool) -> Option<String> {
        let d = if api_side {
            self.api_all.get(n).or_else(|| self.generated.get(n))
        } else {
            self.generated.get(n)
        }?;
        Some(substitute(&d.expr, &d.params, args))
    }

    /// The scalar form of a type: its non-null union members after resolving names (see [`Atom`]).
    fn atoms(&self, ty: &str, api_side: bool, depth: usize) -> BTreeSet<Atom> {
        let mut out = BTreeSet::new();
        for alt in split_top(ty, &['|']) {
            let a = strip_parens(&alt);
            if split_top(a, &['|']).len() > 1 {
                out.extend(self.atoms(a, api_side, depth + 1));
                continue;
            }
            match a {
                "null" => {}
                "number" | "string" | "boolean" | "unknown" => {
                    out.insert(Atom::Base(a.to_owned()));
                }
                _ if a.starts_with('"')
                    || a == "true"
                    || a == "false"
                    || a.parse::<f64>().is_ok() =>
                {
                    out.insert(Atom::Lit(a.to_owned()));
                }
                _ if a.starts_with('`') => match template_literals(a) {
                    Some(lits) => out.extend(lits.into_iter().map(Atom::Lit)),
                    None => {
                        out.insert(Atom::Other(a.to_owned()));
                    }
                },
                _ => {
                    if let Some(e) = array_elem(a) {
                        out.insert(Atom::Array(self.atoms(&e, api_side, depth + 1)));
                    } else if let Some(inner) =
                        a.strip_prefix('[').and_then(|t| t.strip_suffix(']'))
                    {
                        out.insert(Atom::Tuple(
                            split_top(inner, &[','])
                                .iter()
                                .map(|e| self.atoms(e, api_side, depth + 1))
                                .collect(),
                        ));
                    } else if let Some(v) = record_value(a).or_else(|| mapped(a).map(|(_, v)| v)) {
                        out.insert(Atom::Map(self.atoms(&v, api_side, depth + 1)));
                    } else if object_fields(a).is_some() {
                        out.insert(Atom::Base("object".to_owned()));
                    } else if let Some((n, field)) = indexed_access(a) {
                        // `ItemDetail["params"]`: the field's type.
                        let ty = self
                            .expand(&n, &[], api_side)
                            .and_then(|e| object_fields(&e))
                            .and_then(|f| f.into_iter().find(|f| f.name == field))
                            .map(|f| f.ty);
                        match ty {
                            Some(t) if depth < 16 => {
                                out.extend(self.atoms(&t, api_side, depth + 1))
                            }
                            _ => {
                                out.insert(Atom::Other(a.to_owned()));
                            }
                        }
                    } else if let Some((n, args)) = named(a) {
                        match self.expand(&n, &args, api_side) {
                            Some(e) if depth < 16 => {
                                out.extend(self.atoms(&e, api_side, depth + 1))
                            }
                            _ => {
                                out.insert(Atom::Name(n));
                            }
                        }
                    } else {
                        out.insert(Atom::Other(a.to_owned()));
                    }
                }
            }
        }
        out
    }

    /// Compares the scalar forms of a server-api type and a generated type. The generated type may narrow a `string`
    /// of the document to string literals (the model's enum of slugs, also inside arrays and maps); every other
    /// difference needs a `TYPE_DIFFERENCES` entry. A total `Record<K, V>` of the document matches an object with
    /// exactly the members of K as required fields of type V (`PerDamageType`).
    fn compare_scalar(&mut self, what: &str, api: &str, generated: &str, ctx: &Ctx, objects: bool) {
        self.scalars += 1;
        if let Some((k, v)) = total_record(api) {
            let (g, _) = self.resolve(generated.to_owned(), None);
            if mapped(&g).is_none()
                && let Some(fields) = object_fields(&g)
            {
                let keys = self.atoms(&k, true, 0);
                let names: BTreeSet<Atom> = fields
                    .iter()
                    .map(|f| Atom::Lit(format!("\"{}\"", f.name)))
                    .collect();
                if keys != names || fields.iter().any(|f| f.optional) {
                    self.problems.push(format!(
                        "{what}: record keys {} in server-api, required fields {} generated",
                        show(&keys),
                        show(&names)
                    ));
                }
                for f in &fields {
                    self.compare_scalar(&format!("{what}.{}", f.name), &v, &f.ty, ctx, objects);
                }
                return;
            }
        }
        let keep = |s: BTreeSet<Atom>| -> BTreeSet<Atom> {
            s.into_iter()
                .filter(|a| objects || *a != Atom::Base("object".to_owned()))
                .collect()
        };
        let a = keep(self.atoms(api, true, 0));
        let g = keep(self.atoms(generated, false, 0));
        if a == g || narrows_strings(&g, &a) {
            return;
        }
        if TYPE_DIFFERENCES
            .iter()
            .any(|(t, f, _)| *t == ctx.0 && *f == ctx.1)
        {
            self.type_hit.insert(ctx.clone());
            return;
        }
        self.problems.push(format!(
            "{what}: type `{}` in server-api, `{}` generated (key {:?})",
            show(&a),
            show(&g),
            ctx
        ));
    }

    fn non_null(ty: &str) -> Vec<String> {
        split_top(ty, &['|'])
            .into_iter()
            .filter(|a| a != "null")
            .collect()
    }

    /// Compares an api type with a generated type: structurally where the api type has inline objects, by scalar form
    /// otherwise. `ctx` names the field for `TYPE_DIFFERENCES`.
    fn compare(
        &mut self,
        what: &str,
        api: &str,
        gen_ty: &str,
        gen_name: Option<String>,
        ctx: &Ctx,
    ) {
        if !api.contains('{') {
            self.compare_scalar(what, api, gen_ty, ctx, true);
            return;
        }
        let (gen_ty, mut gen_name) = self.resolve(gen_ty.to_owned(), gen_name);
        let api_alts = Self::non_null(api);
        let mut gen_alts = Self::non_null(&gen_ty);
        if gen_alts.len() == 1 {
            // `McConfirm | null`: resolve the one non-null member.
            let (g, n) = self.resolve(gen_alts[0].clone(), gen_name.clone());
            gen_alts = vec![g];
            gen_name = n;
        }
        if api_alts.len() == 1 && gen_alts.len() == 1 {
            let (a, g) = (&api_alts[0], &gen_alts[0]);
            let a = a.trim_start_matches('(').trim_end_matches(')');
            if let Some(ae) = array_elem(a) {
                match array_elem(g) {
                    Some(ge) => self.compare(&format!("{what}[]"), &ae, &ge, None, ctx),
                    None => self
                        .problems
                        .push(format!("{what}: array in server-api, `{g}` generated")),
                }
                return;
            }
            if let Some(v) = record_value(a) {
                match mapped(g) {
                    Some((_, gv)) => self.compare(&format!("{what}{{}}"), &v, &gv, None, ctx),
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
        // Unions: the members that are not objects compare by scalar form; object arms match by their tag (a
        // literal field) or their field names.
        self.compare_scalar(what, api, &gen_ty, ctx, false);
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
            let ctx = (gen_name.to_owned(), a.name.clone());
            self.compare(&format!("{what}.{}", a.name), &a.ty, &g.ty, None, &ctx);
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

/// The scalar form of one union member (`Cmp::atoms`).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum Atom {
    /// `number`, `string`, `boolean`, `unknown`, or `object` for an object literal (compared field by field).
    Base(String),
    /// A literal: `"armor"`, `1`, `true`.
    Lit(String),
    /// `T[]` / `Array<T>`.
    Array(BTreeSet<Atom>),
    /// `[A, B]`.
    Tuple(Vec<BTreeSet<Atom>>),
    /// `Record<K, V>`, `Partial<Record<K, V>>`, `{ [key in K]?: V }`: the value type (keys are not compared).
    Map(BTreeSet<Atom>),
    /// A name neither side declares.
    Name(String),
    /// A form the reader does not know.
    Other(String),
}

impl std::fmt::Display for Atom {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Atom::Base(s) | Atom::Lit(s) | Atom::Name(s) => f.write_str(s),
            Atom::Other(s) => write!(f, "?{s}"),
            Atom::Array(x) => write!(f, "Array<{}>", show(x)),
            Atom::Map(x) => write!(f, "Map<{}>", show(x)),
            Atom::Tuple(xs) => {
                let parts: Vec<String> = xs.iter().map(show).collect();
                write!(f, "[{}]", parts.join(", "))
            }
        }
    }
}

fn show(s: &BTreeSet<Atom>) -> String {
    s.iter()
        .map(|a| a.to_string())
        .collect::<Vec<_>>()
        .join(" | ")
}

/// `true` when `g` differs from `a` only by string literals where `a` has `string` (recursively in arrays, maps and
/// tuples): the model's enum of slugs where server-api.md writes `string`.
fn narrows_strings(g: &BTreeSet<Atom>, a: &BTreeSet<Atom>) -> bool {
    let string = Atom::Base("string".to_owned());
    g.iter().all(|x| {
        a.contains(x)
            || (matches!(x, Atom::Lit(l) if l.starts_with('"')) && a.contains(&string))
            || match x {
                Atom::Array(gx) => a
                    .iter()
                    .any(|y| matches!(y, Atom::Array(ay) if narrows_strings(gx, ay))),
                Atom::Map(gx) => a
                    .iter()
                    .any(|y| matches!(y, Atom::Map(ay) if narrows_strings(gx, ay))),
                Atom::Tuple(gx) => a.iter().any(|y| {
                    matches!(y, Atom::Tuple(ay) if ay.len() == gx.len()
                        && gx.iter().zip(ay).all(|(p, q)| narrows_strings(p, q)))
                }),
                _ => false,
            }
    }) && a.iter().all(|y| {
        // Nothing of the document may be missing: every api member is matched by a generated one.
        g.contains(y)
            || (*y == string
                && g.iter()
                    .any(|x| matches!(x, Atom::Lit(l) if l.starts_with('"'))))
            || match y {
                Atom::Array(ay) => g
                    .iter()
                    .any(|x| matches!(x, Atom::Array(gx) if narrows_strings(gx, ay))),
                Atom::Map(ay) => g
                    .iter()
                    .any(|x| matches!(x, Atom::Map(gx) if narrows_strings(gx, ay))),
                Atom::Tuple(ay) => g.iter().any(|x| {
                    matches!(x, Atom::Tuple(gx) if ay.len() == gx.len()
                        && gx.iter().zip(ay).all(|(p, q)| narrows_strings(p, q)))
                }),
                _ => false,
            }
    })
}

/// The string literals of a template literal type whose holes are literal unions: `` `inc${2|3}` `` →
/// `"inc2"`, `"inc3"`.
fn template_literals(t: &str) -> Option<Vec<String>> {
    let body = t.strip_prefix('`')?.strip_suffix('`')?;
    let mut out = vec![String::new()];
    let mut rest = body;
    while !rest.is_empty() {
        match rest.find("${") {
            None => {
                out.iter_mut().for_each(|s| s.push_str(rest));
                break;
            }
            Some(i) => {
                let (lit, tail) = rest.split_at(i);
                out.iter_mut().for_each(|s| s.push_str(lit));
                let close = tail.find('}')?;
                let opts: Vec<String> = tail[2..close]
                    .split('|')
                    .map(|o| o.trim().trim_matches('"').to_owned())
                    .collect();
                out = out
                    .iter()
                    .flat_map(|s| opts.iter().map(move |o| format!("{s}{o}")))
                    .collect();
                rest = &tail[close + 1..];
            }
        }
    }
    Some(out.into_iter().map(|s| format!("\"{s}\"")).collect())
}

/// `Name["field"]` → (Name, field).
fn indexed_access(t: &str) -> Option<(String, String)> {
    let (n, rest) = t.split_once("[\"")?;
    let field = rest.strip_suffix("\"]")?;
    named(n)
        .filter(|(_, args)| args.is_empty())
        .map(|(n, _)| (n, field.to_owned()))
}

/// A total `Record<K, V>` (not `Partial<…>`) → (K, V).
fn total_record(ty: &str) -> Option<(String, String)> {
    let inner = ty.trim().strip_prefix("Record<")?.strip_suffix('>')?;
    let parts = split_top(inner, &[',']);
    (parts.len() == 2).then(|| (parts[0].clone(), parts[1].clone()))
}

/// The type without enclosing parentheses (`("a" | "b")` → `"a" | "b"`).
fn strip_parens(ty: &str) -> &str {
    let t = ty.trim();
    let Some(inner) = t.strip_prefix('(').and_then(|x| x.strip_suffix(')')) else {
        return t;
    };
    // Only when the outer pair matches: `(A) | (B)` keeps them (the first `(` closes before the end).
    let mut depth = 0;
    for c in inner.chars() {
        match c {
            '(' => depth += 1,
            ')' if depth == 0 => return t,
            ')' => depth -= 1,
            _ => {}
        }
    }
    strip_parens(inner)
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
    let api_all = api_defs(&doc, "## 4. Shared document shapes");
    api.insert("Message".into(), api_all["Message"].clone());
    let mut cmp = Cmp {
        generated: &generated,
        api_all: &api_all,
        problems: Vec::new(),
        fields: 0,
        scalars: 0,
        known_hit: Default::default(),
        type_hit: Default::default(),
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
        let ctx = (name.clone(), String::new());
        cmp.compare(name, &def.expr, name, Some(name.clone()), &ctx);
        compared += 1;
    }
    for (shape, ty) in ROUTE_SHAPES {
        let doc_form = shape.replace(", ", "; ");
        assert!(
            doc.contains(shape) || doc.contains(&doc_form),
            "route shape `{shape}` no longer in server-api.md"
        );
        let ctx = ((*ty).to_owned(), String::new());
        cmp.compare(ty, shape, ty, Some((*ty).to_owned()), &ctx);
    }
    assert!(compared >= 36, "only {compared} server-api types found");
    assert!(cmp.problems.is_empty(), "{}", cmp.problems.join("\n"));
    assert!(cmp.fields > 400, "only {} fields compared", cmp.fields);
    assert!(
        cmp.scalars > 300,
        "only {} scalar types compared",
        cmp.scalars
    );
    for (t, f, why) in KNOWN_DIFFERENCES {
        assert!(
            cmp.known_hit.contains(&((*t).to_owned(), (*f).to_owned())),
            "KNOWN_DIFFERENCES entry {t}.{f} ({why}) is no longer needed"
        );
    }
    for (t, f, why) in TYPE_DIFFERENCES {
        assert!(
            cmp.type_hit.contains(&((*t).to_owned(), (*f).to_owned())),
            "TYPE_DIFFERENCES entry {t}.{f} ({why}) is no longer needed"
        );
    }
}

#[test]
fn scalar_comparison_catches_type_mismatches() {
    // The comparison is not vacuous: a scalar or literal mismatch is a problem unless listed.
    let generated: BTreeMap<String, Def> = parse_defs(
        "type G = { n: number, s: string, l: \"a\" | \"b\", k: K, t: [number, number], m: { [key in K]?: number } };\n\
         type K = \"x\" | \"y\";",
    );
    let api_all = BTreeMap::new();
    let mut cmp = Cmp {
        generated: &generated,
        api_all: &api_all,
        problems: Vec::new(),
        fields: 0,
        scalars: 0,
        known_hit: Default::default(),
        type_hit: Default::default(),
    };
    let same = "{ n: number; s: string; l: \"b\" | \"a\"; k: \"y\" | \"x\"; t: [number, number]; \
                m: Partial<Record<K, number>> }";
    cmp.compare(
        "G",
        same,
        "G",
        Some("G".into()),
        &("G".into(), String::new()),
    );
    assert!(cmp.problems.is_empty(), "{:?}", cmp.problems);
    // `k: string` is matched by the enum of slugs `K` (a narrowing); the other five differ.
    let differs =
        "{ n: string; s: number; l: \"a\"; k: string; t: number[]; m: Record<string, string> }";
    cmp.compare(
        "G",
        differs,
        "G",
        Some("G".into()),
        &("G".into(), String::new()),
    );
    assert_eq!(cmp.problems.len(), 5, "{:#?}", cmp.problems);
    // Template literals and literal widening.
    assert_eq!(
        template_literals("`inc${2|3}`").unwrap(),
        ["\"inc2\"", "\"inc3\""]
    );
    let lit = |s: &str| Atom::Lit(s.into());
    let base = |s: &str| Atom::Base(s.into());
    assert!(narrows_strings(
        &[lit("\"a\"")].into(),
        &[base("string")].into()
    ));
    assert!(!narrows_strings(
        &[base("number")].into(),
        &[lit("1")].into()
    ));
    assert!(!narrows_strings(
        &[base("string")].into(),
        &[lit("\"a\"")].into()
    ));
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
