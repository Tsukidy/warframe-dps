'use strict';
// Shared helpers for the research/external rebuild kit (our own code; see README.md).
// HTTP goes through curl so that the container's HTTPS proxy and CA settings apply unchanged.

const { spawnSync } = require('child_process');
const crypto = require('crypto');
const fs = require('fs');
const path = require('path');

const REPO_ROOT = path.resolve(__dirname, '..', '..');
const TOOLS_DIR = __dirname;
const USER_AGENT = 'warframe-dps-research-rebuild/1.0 (local research tool; polite, cached)';
const WIKI_API = 'https://wiki.warframe.com/api.php';
const WIKI_INDEX = 'https://wiki.warframe.com/w/index.php';
// Spec written 2026-11-20 .. 2026-11-24: every wiki title is pinned to its latest revision at or before this instant.
const WIKI_CUTOFF = '2026-11-24T23:59:59Z';

// Minimum gap between two requests to the same host (milliseconds). The wiki asks for about 2 requests per second.
const HOST_GAP_MS = {
  'wiki.warframe.com': 600,
  'warframe-damage.com': 1000,
  'content.warframe.com': 1000,
  'registry.npmjs.org': 200,
};
const lastRequestAt = new Map();

function sleepMs(ms) {
  if (ms > 0) Atomics.wait(new Int32Array(new SharedArrayBuffer(4)), 0, 0, ms);
}

function politeWait(url) {
  const host = new URL(url).host;
  const gap = HOST_GAP_MS[host] ?? 1000;
  const last = lastRequestAt.get(host) ?? 0;
  const wait = last + gap - Date.now();
  sleepMs(wait);
  lastRequestAt.set(host, Date.now());
}

// Fetch `url` into a Buffer. Retries on network errors, 429 and 5xx with exponential back-off.
// Returns { status, body, headers } for the final attempt; throws only when every attempt failed at the network level.
function httpGet(url, { attempts = 6, timeoutSec = 180, accept404 = false } = {}) {
  let lastErr = null;
  for (let i = 0; i < attempts; i++) {
    if (i > 0) sleepMs(Math.min(30000, 1500 * 2 ** (i - 1)));
    politeWait(url);
    const hdrFile = path.join(require('os').tmpdir(), `wfd-rebuild-hdr-${process.pid}-${Date.now()}.txt`);
    const r = spawnSync(
      'curl',
      ['-sS', '-L', '--compressed', '-m', String(timeoutSec), '-A', USER_AGENT, '-D', hdrFile, '-o', '-', url],
      { maxBuffer: 512 * 1024 * 1024 },
    );
    let headersRaw = '';
    try {
      headersRaw = fs.readFileSync(hdrFile, 'latin1');
      fs.unlinkSync(hdrFile);
    } catch (_) {
      /* no headers written */
    }
    if (r.status !== 0) {
      lastErr = new Error(`curl exit ${r.status} for ${url}: ${String(r.stderr).trim()}`);
      continue;
    }
    // The last header block belongs to the final response (after redirects / proxy CONNECT).
    const blocks = headersRaw.split(/\r?\n\r?\n/).filter((b) => /^HTTP\//.test(b));
    const last = blocks[blocks.length - 1] || '';
    const status = Number((last.match(/^HTTP\/[\d.]+ (\d{3})/) || [])[1] || 0);
    const headers = {};
    for (const line of last.split(/\r?\n/).slice(1)) {
      const m = line.match(/^([^:]+):\s*(.*)$/);
      if (m) headers[m[1].toLowerCase()] = m[2];
    }
    if (status === 200) return { status, body: r.stdout, headers };
    if (status === 404 && accept404) return { status, body: r.stdout, headers };
    if (status === 429 || status >= 500 || status === 0) {
      const ra = Number(headers['retry-after']);
      if (ra > 0) sleepMs(Math.min(ra, 60) * 1000);
      lastErr = new Error(`HTTP ${status} for ${url}`);
      continue;
    }
    return { status, body: r.stdout, headers }; // other 4xx: final, caller decides
  }
  throw lastErr || new Error(`failed: ${url}`);
}

function httpGetOk(url, opts) {
  const r = httpGet(url, opts);
  if (r.status !== 200) throw new Error(`HTTP ${r.status} for ${url}`);
  return r;
}

function wikiApi(params) {
  const q = new URLSearchParams({ format: 'json', formatversion: '2', maxlag: '5', ...params });
  const url = `${WIKI_API}?${q.toString()}`;
  for (let i = 0; i < 5; i++) {
    const r = httpGetOk(url);
    const j = JSON.parse(r.body.toString('utf8'));
    if (j.error && j.error.code === 'maxlag') {
      sleepMs(5000);
      continue;
    }
    if (j.error) throw new Error(`wiki API error ${j.error.code}: ${j.error.info}`);
    return j;
  }
  throw new Error('wiki API: maxlag persisted');
}

const sha256 = (buf) => crypto.createHash('sha256').update(buf).digest('hex');
const sha1 = (buf) => crypto.createHash('sha1').update(buf).digest('hex');
const sha512b64 = (buf) => crypto.createHash('sha512').update(buf).digest('base64');

// Write atomically: a partly written file never carries the final name, so an interrupted run resumes cleanly.
function writeAtomic(file, data) {
  fs.mkdirSync(path.dirname(file), { recursive: true });
  const tmp = `${file}.part-${process.pid}`;
  fs.writeFileSync(tmp, data);
  fs.renameSync(tmp, file);
}

// Wiki page title (cited form, underscores or spaces) -> cache file name, as the earlier tree named them:
// spaces as "_", "/" as "__", "'" as %27, "&" as %26.
function wikiPageFile(title) {
  return `${title.replace(/ /g, '_').replace(/\//g, '__').replace(/'/g, '%27').replace(/&/g, '%26')}.wiki`;
}

// Module title -> cache/wiki-modules/raw/ file name ("Module:Enemies/data/grineer" -> "Enemies_data_grineer.lua").
function wikiModuleFile(title) {
  return `${title.replace(/^Module:/, '').replace(/ /g, '_').replace(/\//g, '_')}.lua`;
}

function readTsv(file) {
  const lines = fs.readFileSync(file, 'utf8').split('\n').filter((l) => l && !l.startsWith('#'));
  const head = lines.shift().split('\t');
  return lines.map((l) => {
    const cells = l.split('\t');
    const o = {};
    head.forEach((h, i) => (o[h] = cells[i] ?? ''));
    return o;
  });
}

function writeTsv(file, columns, rows, comment) {
  const out = [];
  if (comment) for (const c of comment.split('\n')) out.push(`# ${c}`);
  out.push(columns.join('\t'));
  for (const r of rows) out.push(columns.map((c) => String(r[c] ?? '').replace(/[\t\n]/g, ' ')).join('\t'));
  writeAtomic(file, out.join('\n') + '\n');
}

function walk(dir, out = []) {
  for (const e of fs.readdirSync(dir, { withFileTypes: true })) {
    const p = path.join(dir, e.name);
    if (e.isDirectory()) walk(p, out);
    else if (e.isFile()) out.push(p);
  }
  return out;
}

module.exports = {
  REPO_ROOT,
  TOOLS_DIR,
  USER_AGENT,
  WIKI_API,
  WIKI_INDEX,
  WIKI_CUTOFF,
  sleepMs,
  httpGet,
  httpGetOk,
  wikiApi,
  sha256,
  sha1,
  sha512b64,
  writeAtomic,
  wikiPageFile,
  wikiModuleFile,
  readTsv,
  writeTsv,
  walk,
};
