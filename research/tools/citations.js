'use strict';
// Collect every wiki page and wiki Lua module that docs/ and research/ cite.
// Used by pin-wiki.js (to build wiki-pins.tsv) and by `node citations.js` (to print the lists for review).

const fs = require('fs');
const path = require('path');
const { REPO_ROOT, walk, readTsv } = require('./lib');

const SCAN_DIRS = ['docs', 'research'];
const SKIP_DIRS = [path.join('research', 'external'), path.join('research', 'tools')];

function sourceFiles() {
  const files = [];
  for (const d of SCAN_DIRS) {
    const abs = path.join(REPO_ROOT, d);
    if (!fs.existsSync(abs)) continue;
    for (const f of walk(abs)) {
      const rel = path.relative(REPO_ROOT, f);
      if (SKIP_DIRS.some((s) => rel === s || rel.startsWith(s + path.sep))) continue;
      if (/\.(md|json|txt|tsv|toml)$/.test(f)) files.push(f);
    }
  }
  return files.sort();
}

const isPlaceholder = (s) => /[<>*{}]/.test(s) || s === '';

// "<stem>.wiki" not followed by more name characters (so "wiki.warframe.com" never matches). The stem may hold one
// balanced "(...)" group, as in Condition_Overload_(Mechanic).wiki; a "(" that opens a parenthesis is not part of it.
const STEM = String.raw`[^\s\`"|,;\]/()]+(?:\([^\s\`"|,;\]/()]*\)[^\s\`"|,;\]/()]*)?`;
const WIKI_FILE_RE = new RegExp(String.raw`(?<![^\s\`"'(/[])(${STEM})\.wiki(?![A-Za-z0-9_-])(?!\.[A-Za-z0-9])`, 'g');
const PARSED_FILE_RE = new RegExp(String.raw`(?<![^\s\`"'(/[])(${STEM})\.parsed\.html\b`, 'g');

// A backticked string that can be a wiki page title in the findings' bare-title form: starts with a capital or a
// digit followed by a letter, no citation-key colon, no file extension, no module path, no camelCase field name.
function looksLikeTitle(s) {
  const t = s.replace(/_/g, ' ');
  if (isPlaceholder(t) || /[:=|#[\]`+,]/.test(t) || /\bL\d/.test(t)) return false;
  if (!/^[A-Z]/.test(t) && !/^\d+[A-Za-z]/.test(t)) return false;
  if (/\.(md|js|json|lua|rs|toml|ts|txt|wiki|html)$/i.test(t)) return false;
  if (/(^|[/ ])data\b/.test(t)) return false; // module paths and fields: Mods/data, Arcane_data, Mods/data.Polarity
  if (/[a-z][A-Z]/.test(t)) return false; // FireRate, AttackName
  return true;
}

// "`Galvanized Shot`/`Aptitude`/`Savvy`" -> Galvanized Shot, Galvanized Aptitude, Galvanized Savvy;
// "`Kuva Bramma`, `Evensong`, `Cernos`" -> the three titles.
function backtickSeries(s) {
  const out = [];
  const re = /`([^`\n]+)`(\/)?/g;
  let prev = null;
  let slash = false;
  for (const m of s.matchAll(re)) {
    let t = m[1].replace(/_/g, ' ');
    if (slash && prev && !t.includes(' ') && prev.includes(' ')) t = `${prev.slice(0, prev.lastIndexOf(' '))} ${t}`;
    out.push(t);
    prev = t;
    slash = Boolean(m[2]);
  }
  return out;
}

// Trim punctuation that ends a sentence or closes a surrounding bracket, keeping balanced "(...)" in titles
// such as "Condition_Overload_(Mechanic)".
function trimTitle(s) {
  let t = s;
  for (;;) {
    const before = t;
    t = t.replace(/[.,:;'!?]+$/, '');
    if (t.endsWith(')')) {
      const open = (t.match(/\(/g) || []).length;
      const close = (t.match(/\)/g) || []).length;
      if (close > open) t = t.slice(0, -1);
    }
    if (t === before) return t;
  }
}

// "Damage__Heat_Damage" (cache file stem) -> "Damage/Heat_Damage".
function fileStemToTitle(stem) {
  return stem.replace(/__/g, '/').replace(/%27/g, "'").replace(/%26/g, '&');
}

function expandBraces(s) {
  const m = s.match(/^(.*?)\{([^{}]*)\}(.*)$/);
  if (!m) return [s];
  return m[2].split(',').flatMap((alt) => expandBraces(m[1] + alt.trim() + m[3]));
}

// Spec citation keys that stand for modules (docs/mechanics/04 §1.2, 05 §1, 06 §1, 07 §1).
const M_KEYS = {
  'M:Weapons.lua': ['Module:Weapons'],
  'M:StatObject.lua': ['Module:StatObject'],
  'M:Stances': ['Module:Stances'],
  'M:Modular': ['Module:Modular'],
  'M:infobox': ['Module:Enemies/infobox'],
  'M:dt': ['Module:DamageTypes/data'],
  'M:Mods': ['Module:Mods/data'],
  'M:Arcane': ['Module:Arcane/data'],
  'M:Weapons': ['Module:Weapons', 'Module:Weapons/data'],
};

function collect() {
  const pages = new Map(); // normalised title (spaces) -> { cited: Set, files: Set }
  const modules = new Map();
  const parsed = new Map(); // rendered pages: normalised title -> { cited, files }
  const addPage = (raw, file) => {
    const t = trimTitle(raw);
    if (isPlaceholder(t) || /^Module:/i.test(t)) return;
    const key = t.replace(/_/g, ' ').trim();
    if (!key) return;
    if (!pages.has(key)) pages.set(key, { cited: new Set(), files: new Set() });
    pages.get(key).cited.add(t);
    pages.get(key).files.add(path.relative(REPO_ROOT, file));
  };
  const addParsed = (raw, file) => {
    const t = trimTitle(raw);
    if (isPlaceholder(t)) return;
    const key = t.replace(/_/g, ' ').trim();
    if (!parsed.has(key)) parsed.set(key, { cited: new Set(), files: new Set() });
    parsed.get(key).cited.add(t);
    parsed.get(key).files.add(path.relative(REPO_ROOT, file));
  };
  const addModule = (raw, file) => {
    for (const x of expandBraces(raw)) {
      let t = x.replace(/[./]+$/, '').replace(/\.lua$/, '');
      if (isPlaceholder(t)) continue;
      if (!t.startsWith('Module:')) t = `Module:${t}`;
      if (t.slice('Module:'.length).includes(':')) continue; // e.g. a citation key such as F:buffs in a table cell
      t = t.replace(/_/g, ' ');
      if (!modules.has(t)) modules.set(t, { cited: new Set(), files: new Set() });
      modules.get(t).cited.add(x);
      modules.get(t).files.add(path.relative(REPO_ROOT, file));
    }
  };

  for (const f of sourceFiles()) {
    const text = fs.readFileSync(f, 'utf8');
    // W:<Page> (the spec's citation key), with optional " L37", " §Section", " lines 3-5" after it.
    for (const m of text.matchAll(/(?<![A-Za-z0-9])W:([^\s`"|,;\]]+)/g)) addPage(m[1], f);
    // wiki.warframe.com/w/<Page>
    for (const m of text.matchAll(/wiki\.warframe\.com\/w\/([^\s`"|,;\]?#]+)/g)) {
      const p = decodeURIComponent(m[1]);
      if (/^Module:/.test(p)) addModule(p, f);
      else if (!/^index\.php/.test(p)) addPage(p, f);
    }
    // File names of cached pages, with or without a directory: cache/wiki/<File>.wiki, cache/wiki-modules/<File>.wiki,
    // and bare `Stun.wiki`, (Fire_Rate.wiki L39-45), `Condition_Overload_(Mechanic).wiki`.
    for (const m of text.matchAll(WIKI_FILE_RE)) addPage(fileStemToTitle(m[1]), f);
    // Rendered pages (api.php?action=parse), cited as `Tenet_Detron.parsed.html`.
    for (const m of text.matchAll(PARSED_FILE_RE)) addParsed(fileStemToTitle(m[1]), f);
    // The findings' bare-title form: `Title` § Section, `Title` page(s), `A`, `B`, `C` pages,
    // `Galvanized Shot`/`Aptitude`/`Savvy` § Notes, and wiki `Title` (only multi-word or sub-page titles there,
    // because single backticked words after "wiki" are mostly module field names such as `Traits` or `Name`).
    for (const m of text.matchAll(/((?:`[^`\n]+`\/)*`[^`\n]+`)(?=\s*§|\s+page\b)/g)) {
      for (const t of backtickSeries(m[1])) if (looksLikeTitle(t)) addPage(t.replace(/ /g, '_'), f);
    }
    for (const m of text.matchAll(/((?:`[^`\n]+`(?:,\s*|,?\s+and\s+))+`[^`\n]+`)(?=\s+pages\b)/g)) {
      for (const t of backtickSeries(m[1])) if (looksLikeTitle(t)) addPage(t.replace(/ /g, '_'), f);
    }
    for (const m of text.matchAll(/\bwiki:? `([^`\n]+)`/gi)) {
      if (looksLikeTitle(m[1]) && /[ /]/.test(m[1].replace(/_/g, ' '))) addPage(m[1].replace(/ /g, '_'), f);
    }
    // Module:<Name>, including brace lists such as Module:Weapons/data/{primary,secondary}.
    for (const m of text.matchAll(/Module:([A-Za-z0-9_./-]*(?:\{[^}\s]*\}[A-Za-z0-9_./-]*)?)/g)) addModule(m[1], f);
    // Spec module keys (M:...).
    for (const m of text.matchAll(/`(M:[A-Za-z0-9_./]+)`/g)) {
      for (const t of M_KEYS[m[1]] || []) addModule(t, f);
      const em = m[1].match(/^M:enemies\/([a-z0-9-]+)$/);
      if (em) addModule(`Enemies/data/${em[1]}`, f);
    }
  }

  // Module inventories written as tables of bare names: research/findings/data-source-wiki-modules.md §2 and
  // docs/data/pipeline.md §11 (row "wiki modules").
  const inv1 = path.join(REPO_ROOT, 'research/findings/data-source-wiki-modules.md');
  if (fs.existsSync(inv1)) {
    const t = fs.readFileSync(inv1, 'utf8');
    const sec = t.split(/^## /m).find((s) => s.startsWith('2. Module inventory')) || '';
    for (const line of sec.split('\n')) {
      if (!line.startsWith('| `')) continue;
      const first = line.split('|')[1];
      for (const m of first.matchAll(/`([^`]+)`/g)) addModule(m[1], inv1);
    }
  }
  const inv2 = path.join(REPO_ROOT, 'docs/data/pipeline.md');
  if (fs.existsSync(inv2)) {
    const t = fs.readFileSync(inv2, 'utf8');
    for (const line of t.split('\n')) {
      if (!/^\| wiki modules \|/.test(line)) continue;
      for (const m of line.split('|')[2].matchAll(/`([^`]+)`/g)) addModule(m[1], inv2);
    }
  }
  // Wildcard / placeholder module citations ("Module:Weapons/data/*", "Module:Enemies/data/<faction>") are covered
  // by the inventory expansion above.
  // Pages named only in the findings' prose lists of pages read ("Sources used", "Pages used", ...), which no
  // pattern can parse reliably: listed by hand in listed-pages.tsv (title, the file and line that names it).
  const listed = path.join(__dirname, 'listed-pages.tsv');
  if (fs.existsSync(listed)) {
    for (const r of readTsv(listed)) {
      const add = r.kind === 'parsed' ? addParsed : addPage;
      add(r.title.replace(/ /g, '_'), path.join(REPO_ROOT, r.source.split(':')[0]));
    }
  }
  return { pages, modules, parsed };
}

module.exports = { collect, trimTitle, fileStemToTitle, expandBraces, sourceFiles, looksLikeTitle, backtickSeries, WIKI_FILE_RE, PARSED_FILE_RE };

if (require.main === module) {
  const { pages, modules, parsed } = collect();
  console.log(`# ${pages.size} pages`);
  for (const [k, v] of [...pages].sort()) console.log(`P\t${k}\t${[...v.cited].join(' | ')}`);
  console.log(`# ${parsed.size} rendered pages`);
  for (const [k, v] of [...parsed].sort()) console.log(`R\t${k}\t${[...v.cited].join(' | ')}`);
  console.log(`# ${modules.size} modules`);
  for (const [k, v] of [...modules].sort()) console.log(`M\t${k}\t${[...v.cited].join(' | ')}`);
}
