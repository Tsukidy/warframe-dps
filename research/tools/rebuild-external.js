#!/usr/bin/env node
'use strict';
// Rebuild research/external/ (gitignored third-party reference material) from its public sources.
// See README.md in this directory for what the tree is, how long this takes and what can drift.
//
//   node research/tools/rebuild-external.js [TARGET_DIR] [options]
//
//   TARGET_DIR            default: <repo>/research/external
//   --only a,b            run only these steps: wiki, wfcd, inspiration, pe, scripts (default: all)
//   --refresh             re-download files that cannot be pinned (inspiration, Public Export) even if present
//   --write-manifest      after the run, write research/tools/manifest.tsv from TARGET_DIR
//   --verify-only         do not download; compare TARGET_DIR with research/tools/manifest.tsv
//
// Idempotent and resumable: every file is written atomically; pinned files (wiki, WFCD) are skipped when their
// hash already matches; an interrupted run is resumed by running the same command again.

const fs = require('fs');
const path = require('path');
const { spawnSync } = require('child_process');
const lib = require('./lib');

const { REPO_ROOT, TOOLS_DIR, WIKI_INDEX, WIKI_CUTOFF } = lib;
const MANIFEST = path.join(TOOLS_DIR, 'manifest.tsv');
const PINS = path.join(TOOLS_DIR, 'wiki-pins.tsv');

const WFCD = {
  name: '@wfcd/items',
  version: '1.1276.51', // the version research/findings/data-source-wfcd.md evaluated
  tarball: 'https://registry.npmjs.org/@wfcd/items/-/items-1.1276.51.tgz',
  integrity: 'sha512-Lq8vIKaHhuEB2HDA9+6tYmbliNbNCUYHID9xPMqlzRQrYTnFx0dHMRbUmwUTRDHLllPhxtd5QJ0HScEU495bpg==',
};
const INSPIRATION_BASE = 'https://warframe-damage.com';
const INSPIRATION_FILES = [
  // [path under inspiration/, URL path]
  ['index.html', '/'],
  ['min.js', '/min.js'],
  ['jquery-min.js', '/jquery-min.js'],
  ['style.css', '/style-min.css'],
  ['guide.html', '/en/guide/'],
  ['samples/enemies-min.json', '/data/json/enemies-min.json'],
];
const PE = {
  indexHosts: ['https://origin.warframe.com', 'https://content.warframe.com'], // WFCD's order: origin first
  indexPath: '/PublicExport/index_en.txt.lzma',
  manifestBase: 'https://content.warframe.com/PublicExport/Manifest/',
};

// ---------------------------------------------------------------------------------------------------------------
const args = process.argv.slice(2);
const opt = (name) => args.includes(name);
const optVal = (name) => {
  const i = args.indexOf(name);
  return i >= 0 ? args[i + 1] : undefined;
};
const positional = args.filter((a, i) => !a.startsWith('--') && !(i > 0 && args[i - 1] === '--only'));
const TARGET = path.resolve(positional[0] || path.join(REPO_ROOT, 'research', 'external'));
const ONLY = optVal('--only') ? optVal('--only').split(',') : ['wiki', 'wfcd', 'inspiration', 'pe', 'scripts'];
const REFRESH = opt('--refresh');

const log = (...a) => console.log(`[${new Date().toISOString().slice(11, 19)}]`, ...a);
const problems = [];
const report = { target: TARGET, started: new Date().toISOString(), steps: {} };
const T = (rel) => path.join(TARGET, rel);
const exists = (rel) => fs.existsSync(T(rel));
const readT = (rel) => fs.readFileSync(T(rel));

function ensureTools() {
  if (fs.existsSync(path.join(TOOLS_DIR, 'node_modules', 'prettier', 'package.json'))) return;
  log('installing pinned node tools (npm ci in research/tools)');
  const r = spawnSync('npm', ['ci', '--no-audit', '--no-fund'], { cwd: TOOLS_DIR, stdio: 'inherit' });
  if (r.status !== 0) throw new Error('npm ci failed in research/tools');
}

// --------------------------------------------------------------------------------------------------- wiki ----
function stepWiki() {
  const pins = lib.readTsv(PINS);
  const st = { fetched: 0, skipped: 0, mismatched: [], unavailable: [], redirects: [] };
  const ok = pins.filter((p) => p.status === 'ok');
  for (const p of pins) if (p.status !== 'ok') st.unavailable.push(`${p.kind} ${p.cited}: ${p.status}`);
  for (const p of pins) if (p.redirect) st.redirects.push(`${p.cited}: ${p.redirect}`);
  let i = 0;
  for (const p of ok) {
    i++;
    if (p.kind === 'parsed') {
      // Rendered HTML of the pinned revision (api.php?action=parse&oldid=...). The pin fixes the wikitext, but the
      // HTML also depends on the templates and modules as they are today, so it is not hash-checked and may drift.
      if (exists(p.file) && !REFRESH) {
        st.skipped++;
        continue;
      }
      const j = lib.wikiApi({ action: 'parse', oldid: p.revid, prop: 'text', disablelimitreport: '1' });
      lib.writeAtomic(T(p.file), j.parse.text);
      st.fetched++;
      continue;
    }
    if (exists(p.file) && lib.sha1(readT(p.file)) === p.sha1) {
      st.skipped++;
      continue;
    }
    const url = `${WIKI_INDEX}?oldid=${p.revid}&action=raw`;
    let body = null;
    for (let a = 0; a < 3 && !body; a++) {
      const r = lib.httpGetOk(url);
      if (lib.sha1(r.body) === p.sha1) body = r.body;
      else if (a === 2) {
        body = r.body;
        st.mismatched.push(`${p.file} (revid ${p.revid})`);
      }
    }
    if (/^#REDIRECT/i.test(body.toString('utf8', 0, 20))) st.redirects.push(`${p.cited}: revision ${p.revid} is a redirect`);
    lib.writeAtomic(T(p.file), body);
    st.fetched++;
    if (st.fetched % 50 === 0) log(`wiki: ${i}/${ok.length}`);
  }

  // Module revision list (as the earlier tree's revs.json) and the namespace listing.
  const mods = pins.filter((p) => p.kind === 'module');
  const revs = {};
  for (const m of mods) revs[m.title] = { revid: Number(m.revid) || null, timestamp: m.timestamp, sha1: m.sha1, file: m.file };
  lib.writeAtomic(T('cache/wiki-modules/revs.json'), JSON.stringify({ cutoff: WIKI_CUTOFF, modules: revs }, null, 2) + '\n');
  if (!exists('cache/wiki-modules/allpages.txt') || REFRESH) {
    // Current listing of the Module namespace (828). Not pinnable: it reflects the day of the rebuild.
    const titles = [];
    let cont = {};
    for (;;) {
      const j = lib.wikiApi({ action: 'query', list: 'allpages', apnamespace: '828', aplimit: '500', ...cont });
      for (const p of j.query.allpages) titles.push(p.title);
      if (!j.continue) break;
      cont = { apcontinue: j.continue.apcontinue, continue: j.continue.continue };
    }
    lib.writeAtomic(T('cache/wiki-modules/allpages.txt'), titles.join('\n') + '\n');
  }
  // Revision history of Module:Stances/data up to the cut-off (07-melee.md §1 `R:Stances`).
  const hist = pins.find((p) => p.kind === 'history');
  if (hist) {
    const dir = path.dirname(hist.file);
    const rel = `${dir}/revisions.json`;
    if (!exists(rel)) {
      const j = lib.wikiApi({
        action: 'query',
        prop: 'revisions',
        titles: hist.title,
        rvprop: 'ids|timestamp|user|comment|size|sha1',
        rvlimit: '500',
        rvstart: WIKI_CUTOFF,
        rvdir: 'older',
      });
      lib.writeAtomic(T(rel), JSON.stringify(j.query.pages[0].revisions, null, 1) + '\n');
    }
  }
  if (st.unavailable.length) lib.writeAtomic(T('cache/wiki/UNAVAILABLE.txt'), st.unavailable.join('\n') + '\n');
  if (st.mismatched.length) problems.push(`wiki sha1 mismatch: ${st.mismatched.join(', ')}`);
  log(`wiki: ${st.fetched} fetched, ${st.skipped} already present, ${st.unavailable.length} unavailable, ${st.mismatched.length} sha1 mismatches`);
  report.steps.wiki = st;
}

// --------------------------------------------------------------------------------------------------- wfcd ----
function stepWfcd() {
  const marker = 'cache/wfcd/.complete';
  if (exists(marker) && readT(marker).toString().trim() === WFCD.integrity) {
    log('wfcd: already complete');
    report.steps.wfcd = { skipped: true };
    return;
  }
  const meta = lib.httpGetOk(`https://registry.npmjs.org/${WFCD.name}/${WFCD.version}`);
  lib.writeAtomic(T(`cache/wfcd/npm-items-${WFCD.version}.json`), meta.body);
  const tgz = lib.httpGetOk(WFCD.tarball, { timeoutSec: 600 }).body;
  const integrity = `sha512-${lib.sha512b64(tgz)}`;
  if (integrity !== WFCD.integrity) throw new Error(`wfcd tarball integrity ${integrity} != ${WFCD.integrity}`);
  const dir = T('cache/wfcd');
  fs.rmSync(path.join(dir, 'package'), { recursive: true, force: true });
  fs.mkdirSync(dir, { recursive: true });
  const tmp = path.join(dir, `.items-${WFCD.version}.tgz`);
  fs.writeFileSync(tmp, tgz);
  const r = spawnSync('tar', ['-xzf', tmp, '-C', dir, '--no-same-owner'], { stdio: 'inherit' });
  fs.unlinkSync(tmp);
  if (r.status !== 0) throw new Error('tar failed for the wfcd tarball');
  // The findings cite category files as cache/wfcd/<Category>.json and cache/wfcd/i18n/<lang>.json: expose
  // package/data/json/ there too (hard links where the file system allows them, else copies).
  const src = path.join(dir, 'package', 'data', 'json');
  for (const f of lib.walk(src)) {
    const rel = path.relative(src, f);
    const dst = path.join(dir, rel);
    fs.mkdirSync(path.dirname(dst), { recursive: true });
    fs.rmSync(dst, { force: true });
    try {
      fs.linkSync(f, dst); // a hard link: same bytes, no second 98 MB on disk
    } catch (_) {
      fs.copyFileSync(f, dst);
    }
  }
  lib.writeAtomic(T(marker), WFCD.integrity + '\n');
  log(`wfcd: ${WFCD.name}@${WFCD.version} extracted`);
  report.steps.wfcd = { version: WFCD.version, integrity };
}

// ------------------------------------------------------------------------------------------- inspiration ----
function sampleWeaponNames() {
  const names = new Set();
  const { sourceFiles } = require('./citations');
  for (const f of sourceFiles()) {
    const t = fs.readFileSync(f, 'utf8');
    for (const m of t.matchAll(/weapon-([A-Za-z0-9_%'-]+)\.json/g)) names.add(decodeURIComponent(m[1]).replace(/_/g, ' '));
    for (const m of t.matchAll(/getWeapon\.php\?name=([A-Za-z0-9_%+'-]+)/g)) names.add(decodeURIComponent(m[1].replace(/\+/g, ' ')).replace(/_/g, ' '));
  }
  const tc = path.join(REPO_ROOT, 'research', 'test-cases');
  if (fs.existsSync(tc)) {
    for (const f of fs.readdirSync(tc).filter((x) => x.endsWith('.json'))) {
      try {
        const j = JSON.parse(fs.readFileSync(path.join(tc, f), 'utf8'));
        if (typeof j.w === 'string') names.add(j.w);
      } catch (_) {
        /* not a build */
      }
    }
  }
  for (const n of [...names]) if (n.length < 2 || /^(X|Name|name)$/.test(n)) names.delete(n); // placeholders
  return [...names].sort();
}

async function stepInspiration() {
  ensureTools();
  const st = { fetched: [], kept: [], headers: {} };
  const files = [...INSPIRATION_FILES];
  for (const n of sampleWeaponNames()) {
    files.push([`samples/weapon-${n.replace(/ /g, '_')}.json`, `/getWeapon.php?${new URLSearchParams({ name: n })}`]);
  }
  for (const [rel, urlPath] of files) {
    const out = `inspiration/${rel}`;
    if (exists(out) && !REFRESH) {
      st.kept.push(rel);
      continue;
    }
    const r = lib.httpGetOk(INSPIRATION_BASE + urlPath);
    if (rel.endsWith('.json')) JSON.parse(r.body.toString('utf8')); // fails on an error page
    lib.writeAtomic(T(out), r.body);
    st.fetched.push(rel);
    st.headers[rel] = { url: INSPIRATION_BASE + urlPath, 'last-modified': r.headers['last-modified'] || '', etag: r.headers.etag || '', fetched: new Date().toISOString(), sha256: lib.sha256(r.body) };
  }
  // Fetch log (merged with the previous one so that kept files keep their record).
  const logRel = 'inspiration/FETCHED.json';
  const prev = exists(logRel) ? JSON.parse(readT(logRel)) : {};
  lib.writeAtomic(T(logRel), JSON.stringify({ ...prev, ...st.headers }, null, 2) + '\n');

  // Derived files: always regenerated from the fetched ones (deterministic, a few seconds).
  const prettier = require(path.join(TOOLS_DIR, 'node_modules', 'prettier'));
  const minJs = readT('inspiration/min.js').toString('utf8');
  const pretty = await prettier.format(minJs, { parser: 'babel' });
  lib.writeAtomic(T('inspiration/min.pretty.js'), pretty);
  const { convert } = require(path.join(TOOLS_DIR, 'node_modules', 'html-to-text'));
  lib.writeAtomic(T('inspiration/index.txt'), convert(readT('inspiration/index.html').toString('utf8')));
  lib.writeAtomic(T('inspiration/guide.txt'), convert(readT('inspiration/guide.html').toString('utf8')));
  st.minJsSha256 = lib.sha256(readT('inspiration/min.js'));
  st.prettySha256 = lib.sha256(Buffer.from(pretty));
  st.prettyLines = pretty.split('\n').length - (pretty.endsWith('\n') ? 1 : 0);
  log(`inspiration: fetched ${st.fetched.length}, kept ${st.kept.length}; min.js ${st.minJsSha256}; min.pretty.js ${st.prettySha256} (${st.prettyLines} lines)`);
  delete st.headers;
  report.steps.inspiration = st;
}

// ------------------------------------------------------------------------------------------ public export ----
function stepPublicExport() {
  const st = { attempts: [] };
  const done = exists('cache/public-export/index_en.txt.lzma') && exists('cache/public-export/manifests') && !REFRESH;
  if (done) {
    log('public-export: present (use --refresh to re-download)');
    report.steps.pe = { skipped: true };
    return;
  }
  let idx = null;
  for (const host of PE.indexHosts) {
    const url = host + PE.indexPath;
    try {
      const r = lib.httpGet(url, { attempts: 2, accept404: true });
      st.attempts.push(`${url}: HTTP ${r.status}`);
      if (r.status === 200) {
        idx = { url, body: r.body, lastModified: r.headers['last-modified'] || '' };
        break;
      }
    } catch (e) {
      st.attempts.push(`${url}: ${e.message.split('\n')[0]}`);
    }
  }
  const notes = [`Public Export fetched ${new Date().toISOString()}`, ...st.attempts.map((a) => `index attempt: ${a}`)];
  if (!idx) {
    notes.push('No index host reachable: Public Export skipped.');
    lib.writeAtomic(T('cache/public-export/SOURCE.txt'), notes.join('\n') + '\n');
    problems.push('public-export: no index host reachable (skipped)');
    report.steps.pe = st;
    return;
  }
  const dec = spawnSync('python3', ['-I', '-c', 'import lzma,sys; sys.stdout.buffer.write(lzma.decompress(sys.stdin.buffer.read()))'], { input: idx.body, maxBuffer: 64 * 1024 * 1024 });
  if (dec.status !== 0) throw new Error(`lzma decompression failed: ${dec.stderr}`);
  const text = dec.stdout.toString('utf8');
  lib.writeAtomic(T('cache/public-export/index_en.txt.lzma'), idx.body);
  lib.writeAtomic(T('cache/public-export/index_en.txt'), dec.stdout);
  lib.writeAtomic(T('cache/public-export/index_en.lf.txt'), text.replace(/\r/g, ''));
  const lines = text.split(/\r?\n/).map((l) => l.trim()).filter(Boolean);
  for (const line of lines) {
    const r = lib.httpGetOk(PE.manifestBase + line); // the index line verbatim (DS:pe §1: no re-encoding)
    lib.writeAtomic(T(`cache/public-export/manifests/${line.split('!')[0]}`), r.body);
  }
  notes.push(`index used: ${idx.url} (Last-Modified ${idx.lastModified})`, 'manifests (index lines):', ...lines);
  lib.writeAtomic(T('cache/public-export/SOURCE.txt'), notes.join('\n') + '\n');
  st.index = idx.url;
  st.manifests = lines.length;
  log(`public-export: index from ${idx.url}, ${lines.length} manifests`);
  report.steps.pe = st;
}

// ----------------------------------------------------------------------------------------------- scripts ----
function stepScripts() {
  // Our own helper scripts, kept at the paths the findings cite.
  const copies = [
    ['probe.js', 'oracle/probe.js'],
    ['decode-inspiration-url.js', 'decode-inspiration-url.js'],
  ];
  for (const [src, dst] of copies) lib.writeAtomic(T(dst), fs.readFileSync(path.join(TOOLS_DIR, src)));
  log('scripts: copied probe.js and decode-inspiration-url.js');
  report.steps.scripts = { copied: copies.map((c) => c[1]) };
}

// ------------------------------------------------------------------------------------------------ manifest ----
// Not in the manifest: run logs and fetch logs (they carry the date of the run) and probe output.
const MANIFEST_EXCLUDE = [
  /^\.rebuild\//,
  /\.part-\d+$/,
  /^oracle\/out\.txt$/,
  /^inspiration\/FETCHED\.json$/,
  /^cache\/public-export\/SOURCE\.txt$/,
];
function scanTarget() {
  if (!fs.existsSync(TARGET)) return [];
  return lib
    .walk(TARGET)
    .map((f) => path.relative(TARGET, f).split(path.sep).join('/'))
    .filter((rel) => !MANIFEST_EXCLUDE.some((re) => re.test(rel)))
    .sort()
    .map((rel) => {
      const buf = fs.readFileSync(T(rel));
      return { path: rel, size: buf.length, sha256: lib.sha256(buf) };
    });
}

function compareManifest(actual) {
  if (!fs.existsSync(MANIFEST)) {
    log('no research/tools/manifest.tsv to compare with');
    return null;
  }
  const want = new Map(lib.readTsv(MANIFEST).map((r) => [r.path, r]));
  const have = new Map(actual.map((r) => [r.path, r]));
  const res = { match: 0, differ: [], missing: [], extra: [] };
  for (const [p, w] of want) {
    const h = have.get(p);
    if (!h) res.missing.push(p);
    else if (h.sha256 === w.sha256) res.match++;
    else res.differ.push(p);
  }
  for (const p of have.keys()) if (!want.has(p)) res.extra.push(p);
  log(`manifest check: ${res.match} match, ${res.differ.length} differ, ${res.missing.length} missing, ${res.extra.length} extra`);
  const show = (k) => res[k].length && log(`  ${k}: ${res[k].slice(0, 20).join(', ')}${res[k].length > 20 ? ` … (+${res[k].length - 20})` : ''}`);
  ['differ', 'missing', 'extra'].forEach(show);
  return res;
}

// ---------------------------------------------------------------------------------------------------- main ----
async function main() {
  log(`target: ${TARGET}`);
  if (opt('--verify-only')) {
    const res = compareManifest(scanTarget());
    process.exit(res && (res.differ.length || res.missing.length) ? 1 : 0);
  }
  fs.mkdirSync(TARGET, { recursive: true });
  const steps = { wiki: stepWiki, wfcd: stepWfcd, inspiration: stepInspiration, pe: stepPublicExport, scripts: stepScripts };
  for (const name of ONLY) {
    if (!steps[name]) throw new Error(`unknown step ${name}`);
    try {
      await steps[name]();
    } catch (e) {
      problems.push(`${name}: ${e.message}`);
      log(`ERROR in ${name}: ${e.message}`);
    }
  }
  const actual = scanTarget();
  report.files = actual.length;
  report.bytes = actual.reduce((s, r) => s + r.size, 0);
  if (opt('--write-manifest')) {
    lib.writeTsv(
      MANIFEST,
      ['path', 'size', 'sha256'],
      actual,
      `Files of research/external/ as built by rebuild-external.js on ${new Date().toISOString().slice(0, 10)} (paths relative to research/external).\nCheck a rebuild with: node research/tools/rebuild-external.js --verify-only`,
    );
    log(`wrote ${path.relative(REPO_ROOT, MANIFEST)} (${actual.length} files)`);
  } else report.manifest = compareManifest(actual);
  report.problems = problems;
  report.finished = new Date().toISOString();
  lib.writeAtomic(T('.rebuild/last-run.json'), JSON.stringify(report, null, 2) + '\n');
  log(`done: ${actual.length} files, ${(report.bytes / 1e6).toFixed(1)} MB; problems: ${problems.length ? problems.join(' | ') : 'none'}`);
  process.exit(problems.some((p) => !p.startsWith('public-export')) ? 1 : 0);
}

main().catch((e) => {
  console.error(e);
  process.exit(2);
});
