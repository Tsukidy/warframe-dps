#!/usr/bin/env node
'use strict';
// Pin every wiki page and Lua module cited in docs/ and research/ to its latest revision at or before WIKI_CUTOFF,
// using the MediaWiki API of wiki.warframe.com, and write research/tools/wiki-pins.tsv (committed).
//
//   node pin-wiki.js            rebuild wiki-pins.tsv from the current citations (keeps nothing from the old file)
//   node pin-wiki.js --check    report citations that are not in wiki-pins.tsv (no network)
//
// rebuild-external.js only reads wiki-pins.tsv; it never re-pins, so a rebuild is reproducible.

const fs = require('fs');
const path = require('path');
const { TOOLS_DIR, WIKI_CUTOFF, wikiApi, wikiPageFile, wikiModuleFile, readTsv, writeTsv } = require('./lib');
const { collect } = require('./citations');

const PINS = path.join(TOOLS_DIR, 'wiki-pins.tsv');
const COLUMNS = ['kind', 'cited', 'file', 'title', 'redirect', 'revid', 'timestamp', 'sha1', 'size', 'status', 'note'];

// Extras cited by path rather than as W:/Module: (docs/mechanics/05 §1 `G:slider`; 07 §1 `R:Stances`).
const GADGETS = [{ title: 'MediaWiki:Gadget-enemyinfoboxslider.js', file: 'cache/wiki-gadgets/enemyinfoboxslider.js' }];
const HISTORY = { title: 'Module:Stances/data', revids: [2594921, 2765569, 2788398], dir: 'cache/web/stances-history' };
// Pages the findings cite under a second path (research/findings/data-source-wiki-modules.md §3).
const PAGE_COPIES = [{ title: 'Furis Incarnon Genesis', file: 'cache/wiki-modules/Furis_Incarnon_Genesis.wiki' }];

function chunk(a, n) {
  const out = [];
  for (let i = 0; i < a.length; i += n) out.push(a.slice(i, i + n));
  return out;
}

// Latest revision of `title` at or before the cut-off (single-title query), or null.
function revAtCutoff(title) {
  const j = wikiApi({
    action: 'query',
    prop: 'revisions',
    titles: title,
    rvprop: 'ids|timestamp|sha1|size',
    rvlimit: '1',
    rvstart: WIKI_CUTOFF,
    rvdir: 'older',
  });
  const p = j.query.pages[0];
  if (!p || p.missing || !p.revisions || !p.revisions.length) return null;
  return p.revisions[0];
}

// Resolve many titles at once: normalisation, redirects, existence and latest revision (≤ 50 titles per request).
function resolveBatch(titles) {
  const info = new Map();
  for (const batch of chunk(titles, 50)) {
    const j = wikiApi({
      action: 'query',
      prop: 'info|revisions',
      rvprop: 'ids|timestamp|sha1|size',
      redirects: '1',
      titles: batch.join('|'),
    });
    const norm = new Map((j.query.normalized || []).map((n) => [n.from, n.to]));
    const redir = new Map((j.query.redirects || []).map((r) => [r.from, r]));
    const pages = new Map((j.query.pages || []).map((p) => [p.title, p]));
    for (const t of batch) {
      let cur = norm.get(t) || t;
      let redirect = null;
      const seen = new Set();
      while (redir.has(cur) && !seen.has(cur)) {
        seen.add(cur);
        const r = redir.get(cur);
        redirect = { from: redirect ? redirect.from : cur, to: r.to, fragment: r.tofragment || '' };
        cur = r.to;
      }
      info.set(t, { normalized: norm.get(t) || t, redirect, page: pages.get(cur) || null, finalTitle: cur });
    }
  }
  return info;
}

function pinTitles(entries) {
  // entries: [{ kind, key (title to query), cited, file }]
  const info = resolveBatch([...new Set(entries.map((e) => e.key))]);
  const rows = [];
  for (const e of entries) {
    const r = info.get(e.key);
    const row = { kind: e.kind, cited: e.cited, file: e.file, title: r.finalTitle, redirect: '', note: '' };
    if (r.redirect) {
      row.redirect = `${r.redirect.from} -> ${r.redirect.to}${r.redirect.fragment ? '#' + r.redirect.fragment : ''}`;
      // Was the redirect already there at the cut-off, or was the cited page moved later (history moves with it)?
      const own = revAtCutoff(r.redirect.from);
      row.note = own
        ? `redirect page existed at cut-off (its revid ${own.revid}, ${own.timestamp})`
        : 'redirect created after the cut-off (page moved; the target carries the cited history)';
    }
    const p = r.page;
    if (!p || p.missing || p.invalid) {
      Object.assign(row, { status: p && p.invalid ? 'invalid-title' : 'missing', revid: '', timestamp: '', sha1: '', size: '' });
      rows.push(row);
      continue;
    }
    let rev = p.revisions && p.revisions[0];
    if (!rev || rev.timestamp > WIKI_CUTOFF) {
      rev = revAtCutoff(r.finalTitle);
      if (rev) row.note = [row.note, `edited after cut-off (latest revid ${p.lastrevid})`].filter(Boolean).join('; ');
    }
    if (!rev) {
      Object.assign(row, { status: 'no-revision-before-cutoff', revid: '', timestamp: '', sha1: '', size: '' });
      rows.push(row);
      continue;
    }
    Object.assign(row, { revid: rev.revid, timestamp: rev.timestamp, sha1: rev.sha1, size: rev.size, status: 'ok' });
    rows.push(row);
  }
  return rows;
}

function historyRows() {
  const j = wikiApi({
    action: 'query',
    prop: 'revisions',
    titles: HISTORY.title,
    rvprop: 'ids|timestamp|sha1|size',
    rvlimit: '500',
    rvstart: WIKI_CUTOFF,
    rvdir: 'older',
  });
  const revs = j.query.pages[0].revisions || [];
  return HISTORY.revids.map((id) => {
    const rev = revs.find((r) => r.revid === id);
    const base = {
      kind: 'history',
      cited: `R:Stances ${id}`,
      file: `${HISTORY.dir}/${wikiModuleFile(HISTORY.title).replace(/\.lua$/, '')}@${id}.lua`,
      title: HISTORY.title,
      redirect: '',
      note: '',
    };
    if (!rev) return { ...base, status: 'missing', revid: id, timestamp: '', sha1: '', size: '' };
    return { ...base, status: 'ok', revid: id, timestamp: rev.timestamp, sha1: rev.sha1, size: rev.size };
  });
}

function main() {
  const { pages, modules, parsed } = collect();
  if (process.argv.includes('--check')) {
    // Compare by normalised title (spaces), so that a page cited in a second spelling counts as pinned.
    const norm = (t) => t.replace(/_/g, ' ');
    const rows = readTsv(PINS);
    const pinned = new Set(rows.map((r) => `${r.kind}\t${norm(r.cited)}`));
    let missing = 0;
    const report = (kind, key, files) => {
      if (pinned.has(`${kind}\t${key}`)) return;
      console.log(`not pinned: ${kind} ${key}${files ? ` (${[...files].join(', ')})` : ''}`);
      missing++;
    };
    for (const [k, v] of pages) report('page', k, v.files);
    for (const [k, v] of parsed) report('parsed', k, v.files);
    for (const [k] of modules) report('module', k);
    console.log(missing ? `${missing} citation(s) not pinned: run node pin-wiki.js` : 'all citations pinned');
    process.exit(missing ? 1 : 0);
  }

  const entries = [];
  for (const [key, v] of [...pages].sort()) {
    const cited = [...v.cited].sort()[0];
    entries.push({ kind: 'page', key, cited, file: `cache/wiki/${wikiPageFile(cited)}` });
  }
  for (const [key, v] of [...parsed].sort()) {
    const cited = [...v.cited].sort()[0];
    entries.push({ kind: 'parsed', key, cited, file: `cache/wiki/${wikiPageFile(cited).replace(/\.wiki$/, '.parsed.html')}` });
  }
  for (const c of PAGE_COPIES) entries.push({ kind: 'page-copy', key: c.title, cited: c.title, file: c.file });
  for (const [key] of [...modules].sort()) {
    entries.push({ kind: 'module', key, cited: key, file: `cache/wiki-modules/raw/${wikiModuleFile(key)}` });
  }
  for (const g of GADGETS) entries.push({ kind: 'gadget', key: g.title, cited: g.title, file: g.file });

  const rows = [...pinTitles(entries), ...historyRows()];
  writeTsv(
    PINS,
    COLUMNS,
    rows,
    [
      `Wiki pins: latest revision at or before ${WIKI_CUTOFF} of every wiki title cited in docs/ and research/.`,
      `Written by research/tools/pin-wiki.js on ${new Date().toISOString()}; read by rebuild-external.js.`,
      'file is relative to research/external; sha1 is the revision sha1 reported by the MediaWiki API (hex).',
    ].join('\n'),
  );
  const by = {};
  for (const r of rows) by[`${r.kind}/${r.status}`] = (by[`${r.kind}/${r.status}`] || 0) + 1;
  console.log(JSON.stringify(by));
  for (const r of rows) if (r.status !== 'ok') console.log(`NOT PINNED: ${r.kind} ${r.cited}: ${r.status}`);
  for (const r of rows) if (r.redirect) console.log(`redirect: ${r.cited}: ${r.redirect} (${r.note})`);
}

main();
