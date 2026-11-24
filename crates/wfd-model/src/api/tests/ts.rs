//! A small reader of the TypeScript subset that ts-rs emits and server-api.md writes: type declarations, unions,
//! object literals, arrays, tuples, mapped types, generics. Used to compare the generated declarations with
//! server-api.md and to check that the declarations accept exactly the JSON serde writes.

use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

/// Removes `//` and `/* */` comments outside string literals.
pub fn strip_comments(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let b: Vec<char> = text.chars().collect();
    let mut i = 0;
    let mut in_str: Option<char> = None;
    while i < b.len() {
        let c = b[i];
        if let Some(q) = in_str {
            out.push(c);
            if c == '\\' && i + 1 < b.len() {
                out.push(b[i + 1]);
                i += 2;
                continue;
            }
            if c == q {
                in_str = None;
            }
            i += 1;
            continue;
        }
        if c == '"' {
            in_str = Some(c);
            out.push(c);
            i += 1;
        } else if c == '/' && b.get(i + 1) == Some(&'*') {
            i += 2;
            while i + 1 < b.len() && !(b[i] == '*' && b[i + 1] == '/') {
                i += 1;
            }
            i += 2;
        } else if c == '/' && b.get(i + 1) == Some(&'/') {
            while i < b.len() && b[i] != '\n' {
                i += 1;
            }
        } else {
            out.push(c);
            i += 1;
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

/// Splits at any of `seps` at bracket depth 0 and outside string literals; empty parts are dropped.
pub fn split_top(text: &str, seps: &[char]) -> Vec<String> {
    let mut out = vec![String::new()];
    let mut depth = 0;
    let mut in_str = false;
    for c in text.chars() {
        if in_str {
            out.last_mut().unwrap().push(c);
            if c == '"' {
                in_str = false;
            }
            continue;
        }
        if c == '"' {
            in_str = true;
        } else if depth == 0 && seps.contains(&c) {
            out.push(String::new());
            continue;
        } else {
            depth += depth_delta(c);
        }
        out.last_mut().unwrap().push(c);
    }
    out.into_iter()
        .map(|s| s.trim().to_owned())
        .filter(|s| !s.is_empty())
        .collect()
}

/// Position of the first `c` at depth 0 outside strings.
fn find_top(text: &str, c0: char) -> Option<usize> {
    let mut depth = 0;
    let mut in_str = false;
    for (i, c) in text.char_indices() {
        if in_str {
            if c == '"' {
                in_str = false;
            }
            continue;
        }
        if c == '"' {
            in_str = true;
        } else if depth == 0 && c == c0 {
            return Some(i);
        } else {
            depth += depth_delta(c);
        }
    }
    None
}

/// A type declaration: generic parameters and the type expression.
#[derive(Debug, Clone)]
pub struct Def {
    pub params: Vec<String>,
    pub expr: String,
}

/// Every `type Name<P> = expr;` of a text (comments already removed).
pub fn parse_defs(text: &str) -> BTreeMap<String, Def> {
    let mut out = BTreeMap::new();
    let mut rest = text;
    while let Some(i) = rest.find("type ") {
        let before_ok = i == 0
            || !rest[..i]
                .chars()
                .next_back()
                .is_some_and(|c| c.is_alphanumeric() || c == '_');
        let after = &rest[i + 5..];
        rest = after;
        if !before_ok {
            continue;
        }
        let name: String = after
            .chars()
            .take_while(|c| c.is_alphanumeric() || *c == '_')
            .collect();
        if name.is_empty() {
            continue;
        }
        let mut tail = after[name.len()..].trim_start();
        let mut params = Vec::new();
        if let Some(t) = tail.strip_prefix('<') {
            let Some(close) = t.find('>') else { continue };
            params = t[..close].split(',').map(|p| p.trim().to_owned()).collect();
            tail = t[close + 1..].trim_start();
        }
        let Some(body) = tail.strip_prefix('=') else {
            continue;
        };
        let Some(end) = find_top(body, ';') else {
            continue;
        };
        out.insert(
            name,
            Def {
                params,
                expr: body[..end].trim().to_owned(),
            },
        );
        rest = &body[end..];
    }
    out
}

/// One field of an object literal.
#[derive(Debug, Clone)]
pub struct Field {
    pub name: String,
    pub optional: bool,
    pub ty: String,
}

impl Field {
    /// `true` when the type admits `null`.
    pub fn nullable(&self) -> bool {
        split_top(&self.ty, &['|']).iter().any(|a| a == "null")
    }
}

/// `{ a: T; b?: U }` → its fields, or `None` when `expr` is not an object literal. Mapped types
/// (`{ [key in K]?: V }`) are objects with one field whose name starts with `[`.
pub fn object_fields(expr: &str) -> Option<Vec<Field>> {
    let inner = expr.trim().strip_prefix('{')?.strip_suffix('}')?;
    let mut out = Vec::new();
    for f in split_top(inner, &[',', ';']) {
        let colon = find_top(&f, ':')?;
        let (name, ty) = (f[..colon].trim(), f[colon + 1..].trim());
        let (name, optional) = match name.strip_suffix('?') {
            Some(n) => (n.trim(), true),
            None => (name, false),
        };
        out.push(Field {
            name: name.trim_matches('"').to_owned(),
            optional,
            ty: ty.to_owned(),
        });
    }
    Some(out)
}

/// A mapped type `{ [key in K]?: V }` → (K, V).
pub fn mapped(expr: &str) -> Option<(String, String)> {
    let fields = object_fields(expr)?;
    if fields.len() != 1 {
        return None;
    }
    let f = &fields[0];
    let k = f.name.strip_prefix('[')?.strip_suffix(']')?;
    let k = k
        .strip_prefix("key in ")
        .or_else(|| k.strip_prefix("key: "))?;
    Some((k.trim().to_owned(), f.ty.clone()))
}

/// `X[]` or `Array<X>` → X.
pub fn array_elem(expr: &str) -> Option<String> {
    let e = expr.trim();
    if let Some(x) = e.strip_suffix("[]") {
        return Some(x.trim().to_owned());
    }
    let x = e.strip_prefix("Array<")?.strip_suffix('>')?;
    // `Array<A> | Array<B>` would not reach here (split by the union first).
    Some(x.trim().to_owned())
}

/// `Name<A, B>` → (Name, [A, B]); a plain name → (Name, []).
pub fn named(expr: &str) -> Option<(String, Vec<String>)> {
    let e = expr.trim();
    let name: String = e
        .chars()
        .take_while(|c| c.is_alphanumeric() || *c == '_')
        .collect();
    if name.is_empty() || !name.chars().next().unwrap().is_alphabetic() {
        return None;
    }
    let rest = &e[name.len()..];
    if rest.is_empty() {
        return Some((name, Vec::new()));
    }
    let args = rest.strip_prefix('<')?.strip_suffix('>')?;
    Some((name, split_top(args, &[','])))
}

/// Replaces whole-word occurrences of generic parameters.
pub fn substitute(expr: &str, params: &[String], args: &[String]) -> String {
    let mut out = String::new();
    let mut word = String::new();
    let flush = |word: &mut String, out: &mut String| {
        match params
            .iter()
            .position(|p| p == word)
            .and_then(|i| args.get(i))
        {
            Some(a) => out.push_str(a),
            None => out.push_str(word),
        }
        word.clear();
    };
    for c in expr.chars() {
        if c.is_alphanumeric() || c == '_' {
            word.push(c);
        } else {
            flush(&mut word, &mut out);
            out.push(c);
        }
    }
    flush(&mut word, &mut out);
    out
}

/// Checks JSON values against the generated declarations and records which object literals and fields were seen.
pub struct Checker<'a> {
    pub defs: &'a BTreeMap<String, Def>,
    /// Names whose subtrees are not recorded for coverage (documents tested by T01a).
    pub no_cover: BTreeSet<String>,
    /// Object literal (normalized text) → fields seen present.
    pub seen: BTreeMap<String, BTreeSet<String>>,
}

impl<'a> Checker<'a> {
    pub fn new(defs: &'a BTreeMap<String, Def>) -> Self {
        Checker {
            defs,
            no_cover: BTreeSet::new(),
            seen: BTreeMap::new(),
        }
    }

    fn resolve(&self, name: &str, args: &[String]) -> Option<String> {
        let d = self.defs.get(name)?;
        Some(substitute(&d.expr, &d.params, args))
    }

    /// Problems of `v` against `ty`; records coverage when `record`.
    pub fn check(&mut self, ty: &str, v: &Value, path: &str, record: bool) -> Vec<String> {
        let ty = ty.trim();
        let alts = split_top(ty, &['|']);
        if alts.len() > 1 {
            let mut best: Option<Vec<String>> = None;
            for a in &alts {
                let p = self.check(a, v, path, false);
                if p.is_empty() {
                    return self.check(a, v, path, record);
                }
                if best.as_ref().is_none_or(|b| p.len() < b.len()) {
                    best = Some(p);
                }
            }
            let mut p = vec![format!("{path}: no alternative of `{ty}` matches {v}")];
            p.extend(best.unwrap_or_default());
            return p;
        }
        let ok = |b: bool| {
            if b {
                Vec::new()
            } else {
                vec![format!("{path}: expected `{ty}`, found {v}")]
            }
        };
        match ty {
            "null" => return ok(v.is_null()),
            "number" => return ok(v.is_number()),
            "string" => return ok(v.is_string()),
            "boolean" => return ok(v.is_boolean()),
            "true" => return ok(v.as_bool() == Some(true)),
            "false" => return ok(v.as_bool() == Some(false)),
            "unknown" => return Vec::new(),
            _ => {}
        }
        if let Some(lit) = ty.strip_prefix('"').and_then(|t| t.strip_suffix('"')) {
            return ok(v.as_str() == Some(lit));
        }
        if let Some(elem) = array_elem(ty) {
            let Some(items) = v.as_array() else {
                return ok(false);
            };
            let mut p = Vec::new();
            for (i, x) in items.iter().enumerate() {
                p.extend(self.check(&elem, x, &format!("{path}[{i}]"), record));
            }
            return p;
        }
        if let Some(inner) = ty.strip_prefix('[').and_then(|t| t.strip_suffix(']')) {
            let elems = split_top(inner, &[',']);
            let Some(items) = v.as_array() else {
                return ok(false);
            };
            if items.len() != elems.len() {
                return ok(false);
            }
            let mut p = Vec::new();
            for (i, (e, x)) in elems.iter().zip(items).enumerate() {
                p.extend(self.check(e, x, &format!("{path}[{i}]"), record));
            }
            return p;
        }
        if let Some((k, val)) = mapped(ty) {
            let Some(obj) = v.as_object() else {
                return ok(false);
            };
            let mut p = Vec::new();
            for (key, x) in obj {
                if !self.key_ok(&k, key) {
                    p.push(format!("{path}: key `{key}` is not a `{k}`"));
                }
                p.extend(self.check(&val, x, &format!("{path}.{key}"), record));
            }
            return p;
        }
        if let Some(fields) = object_fields(ty) {
            let Some(obj) = v.as_object() else {
                return ok(false);
            };
            let mut p = Vec::new();
            for f in &fields {
                match obj.get(&f.name) {
                    None if !f.optional => p.push(format!("{path}.{}: missing", f.name)),
                    None => {}
                    Some(x) => {
                        p.extend(self.check(&f.ty, x, &format!("{path}.{}", f.name), record))
                    }
                }
            }
            for key in obj.keys() {
                if !fields.iter().any(|f| &f.name == key) {
                    p.push(format!("{path}.{key}: not declared in `{ty}`"));
                }
            }
            if record && p.is_empty() {
                let entry = self.seen.entry(normalize(ty)).or_default();
                entry.extend(obj.keys().cloned());
            }
            return p;
        }
        if let Some((name, args)) = named(ty) {
            let Some(expr) = self.resolve(&name, &args) else {
                return vec![format!("{path}: unknown type `{name}`")];
            };
            let record = record && !self.no_cover.contains(&name);
            return self.check(&expr, v, path, record);
        }
        vec![format!("{path}: unsupported type `{ty}`")]
    }

    /// The smallest value of the same shape: optional fields dropped, lists (`Array<T>`) and maps emptied, union arms
    /// kept (minimized inside). Subtrees of `no_cover` names are kept as they are.
    pub fn minimize(&mut self, ty: &str, v: &Value) -> Value {
        let ty = ty.trim();
        let alts = split_top(ty, &['|']);
        if alts.len() > 1 {
            for a in &alts {
                if self.check(a, v, "", false).is_empty() {
                    return self.minimize(a, v);
                }
            }
            return v.clone();
        }
        if array_elem(ty).is_some() {
            return Value::Array(Vec::new());
        }
        if let Some(inner) = ty.strip_prefix('[').and_then(|t| t.strip_suffix(']')) {
            let elems = split_top(inner, &[',']);
            let items = v.as_array().cloned().unwrap_or_default();
            return Value::Array(
                elems
                    .iter()
                    .zip(&items)
                    .map(|(e, x)| self.minimize(e, x))
                    .collect(),
            );
        }
        if mapped(ty).is_some() {
            return Value::Object(Default::default());
        }
        if let Some(fields) = object_fields(ty) {
            let Some(obj) = v.as_object() else {
                return v.clone();
            };
            let mut out = serde_json::Map::new();
            for f in fields.iter().filter(|f| !f.optional) {
                if let Some(x) = obj.get(&f.name) {
                    out.insert(f.name.clone(), self.minimize(&f.ty, x));
                }
            }
            return Value::Object(out);
        }
        if let Some((name, args)) = named(ty)
            && !self.no_cover.contains(&name)
            && let Some(expr) = self.resolve(&name, &args)
        {
            return self.minimize(&expr, v);
        }
        v.clone()
    }

    fn key_ok(&self, k: &str, key: &str) -> bool {
        match k {
            "string" => true,
            "number" => key.parse::<f64>().is_ok(),
            _ => {
                if let Some((name, args)) = named(k)
                    && let Some(expr) = self.resolve(&name, &args)
                {
                    return self.key_ok(&expr, key);
                }
                split_top(k, &['|'])
                    .iter()
                    .any(|a| a.trim_matches('"') == key || (a == "string") || (a == "number"))
            }
        }
    }

    /// Every object literal reachable from `ty` (outside `no_cover`), with its fields.
    pub fn reachable_objects(&self, ty: &str, out: &mut BTreeMap<String, Vec<Field>>) {
        let mut visited = BTreeSet::new();
        self.walk(ty, out, &mut visited);
    }

    fn walk(
        &self,
        ty: &str,
        out: &mut BTreeMap<String, Vec<Field>>,
        visited: &mut BTreeSet<String>,
    ) {
        let ty = ty.trim();
        let alts = split_top(ty, &['|']);
        if alts.len() > 1 {
            for a in alts {
                self.walk(&a, out, visited);
            }
            return;
        }
        if let Some(elem) = array_elem(ty) {
            return self.walk(&elem, out, visited);
        }
        if let Some(inner) = ty.strip_prefix('[').and_then(|t| t.strip_suffix(']')) {
            for e in split_top(inner, &[',']) {
                self.walk(&e, out, visited);
            }
            return;
        }
        if let Some((_, val)) = mapped(ty) {
            return self.walk(&val, out, visited);
        }
        if let Some(fields) = object_fields(ty) {
            for f in &fields {
                self.walk(&f.ty, out, visited);
            }
            out.insert(normalize(ty), fields);
            return;
        }
        if let Some((name, args)) = named(ty) {
            if self.no_cover.contains(&name) || !visited.insert(ty.to_owned()) {
                return;
            }
            for a in &args {
                self.walk(a, out, visited);
            }
            if let Some(expr) = self.resolve(&name, &args) {
                self.walk(&expr, out, visited);
            }
        }
    }
}

/// Whitespace-normalized type text (the coverage key of an object literal).
pub fn normalize(ty: &str) -> String {
    ty.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Every generated declaration (file contents of `typescript_files`), comments removed.
pub fn generated_defs() -> BTreeMap<String, Def> {
    let files = crate::api::codegen::typescript_files().expect("typescript export");
    let mut defs = BTreeMap::new();
    for (path, text) in &files {
        if !path.ends_with(".ts") || path == "index.ts" || path == "messageKeys.ts" {
            continue;
        }
        defs.extend(parse_defs(&strip_comments(text)));
    }
    defs
}

#[test]
fn reader_handles_the_generated_forms() {
    let text = strip_comments(
        "// note\nimport type { A } from \"./A\";\n/** doc */\nexport type J<R> = { a: A, /** x */ b?: R | null, \
         m: { [key in \"x\" | \"y\"]?: number }, t: [number, string], };\n",
    );
    let defs = parse_defs(&text);
    let j = &defs["J"];
    assert_eq!(j.params, ["R"]);
    let f = object_fields(&j.expr).unwrap();
    assert_eq!(f.len(), 4);
    assert!(f[1].optional && f[1].nullable());
    assert_eq!(mapped(&f[2].ty).unwrap().0, "\"x\" | \"y\"");
    assert_eq!(
        substitute(&j.expr, &j.params, &["Q".into()])
            .matches("Q")
            .count(),
        1
    );
    assert_eq!(
        named("JobStatus<OptResult>").unwrap(),
        ("JobStatus".into(), vec!["OptResult".into()])
    );
    let mut defs = defs;
    defs.insert(
        "A".into(),
        Def {
            params: vec![],
            expr: "{ \"k\": number } | \"z\"".into(),
        },
    );
    let mut c = Checker::new(&defs);
    let ok = serde_json::json!({ "a": "z", "b": null, "m": { "x": 1 }, "t": [1, "s"] });
    assert!(c.check("J<number>", &ok, "$", true).is_empty());
    // Boolean literal types (a union discriminated by `legal`, server-api.md §5.7).
    let mut c2 = Checker::new(&defs);
    let u = "{ k: number, legal: true } | { k: number, legal: false, r: string }";
    assert!(
        c2.check(u, &serde_json::json!({ "k": 1, "legal": true }), "$", false)
            .is_empty()
    );
    assert!(
        c2.check(
            u,
            &serde_json::json!({ "k": 1, "legal": false, "r": "x" }),
            "$",
            false
        )
        .is_empty()
    );
    assert!(
        !c2.check(
            u,
            &serde_json::json!({ "k": 1, "legal": false }),
            "$",
            false
        )
        .is_empty()
    );
    assert!(
        !c2.check(
            u,
            &serde_json::json!({ "k": 1, "legal": true, "r": "x" }),
            "$",
            false
        )
        .is_empty()
    );
    let bad = serde_json::json!({ "a": { "k": "1" }, "m": { "w": 1 }, "t": [1], "c": 0 });
    assert_eq!(
        c.check("J<number>", &bad, "$", true).len(),
        5,
        "{:?}",
        c.check("J<number>", &bad, "$", false)
    );
}
