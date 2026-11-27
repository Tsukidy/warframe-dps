#!/usr/bin/env node
'use strict';
// Oracle probe: run a warframe-damage.com build link fully offline in headless Chromium (Playwright) against the
// cached inspiration files, and save the page's visible text (research/findings/oracle-feasibility.md).
//
//   node probe.js "<https://warframe-damage.com/?p=...&n=1>" [options]
//
//   --out FILE           where to write document.body.innerText (default: out.txt in the current directory)
//   --wait MS            how long to let the page calculate before reading it (default 8000)
//   --seed N             replace Math.random with a seeded generator (the inspiration's queue is random)
//   --inspiration DIR    cached inspiration files (default: <repo>/research/external/inspiration)
//   --fetch-missing      fetch a weapon that has no samples/weapon-<Name>.json once from the live site and cache it
//   --quiet              do not print the DPS/TTK summary
//
// Request routing: /, /min.js, /jquery-min.js, /style-min.css, /data/json/enemies-min.json -> local files;
// /getWeapon.php?name=X -> samples/weapon-X.json (spaces -> underscores); /getImg.php -> a 1x1 PNG;
// any other request (other paths, other hosts) -> 204 No Content.

const fs = require('fs');
const path = require('path');
const { spawnSync } = require('child_process');
const { chromium } = require('playwright');

const PNG_1x1 = Buffer.from(
  'iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mNkYAAAAAYAAjCB0C8AAAAASUVORK5CYII=',
  'base64',
);

function parseArgs(argv) {
  const o = { wait: 8000, out: 'out.txt', seed: null, inspiration: null, fetchMissing: false, quiet: false, link: null };
  for (let i = 0; i < argv.length; i++) {
    const a = argv[i];
    if (a === '--out') o.out = argv[++i];
    else if (a === '--wait') o.wait = Number(argv[++i]);
    else if (a === '--seed') o.seed = Number(argv[++i]);
    else if (a === '--inspiration') o.inspiration = argv[++i];
    else if (a === '--fetch-missing') o.fetchMissing = true;
    else if (a === '--quiet') o.quiet = true;
    else if (!o.link) o.link = a;
  }
  return o;
}

// Find <repo>/research/external/inspiration from this script's location (research/tools/ or research/external/oracle/).
function defaultInspirationDir() {
  // The copy at <external>/oracle/probe.js: its own tree first, also when <external> is not research/external.
  const sibling = path.join(__dirname, '..', 'inspiration');
  if (path.basename(__dirname) === 'oracle' && fs.existsSync(path.join(sibling, 'min.js'))) return sibling;
  let dir = __dirname;
  for (let i = 0; i < 6; i++) {
    const cand = path.join(dir, 'research', 'external', 'inspiration');
    if (fs.existsSync(cand)) return cand;
    dir = path.dirname(dir);
  }
  throw new Error('research/external/inspiration not found: rebuild it (research/tools/README.md) or pass --inspiration');
}

async function main() {
  const o = parseArgs(process.argv.slice(2));
  if (!o.link) {
    console.error('usage: node probe.js "<warframe-damage.com build link>" [--out FILE] [--wait MS] [--seed N]');
    process.exit(2);
  }
  const insp = path.resolve(o.inspiration || defaultInspirationDir());
  const local = {
    '/': 'index.html',
    '/index.html': 'index.html',
    '/min.js': 'min.js',
    '/jquery-min.js': 'jquery-min.js',
    '/style-min.css': 'style.css',
    '/data/json/enemies-min.json': 'samples/enemies-min.json',
  };
  const types = { html: 'text/html; charset=utf-8', js: 'application/javascript', css: 'text/css', json: 'application/json' };
  const typeOf = (f) => types[f.split('.').pop()] || 'application/octet-stream';
  const missing = [];

  const browser = await chromium.launch();
  try {
    const page = await (await browser.newContext()).newPage();
    if (o.seed !== null) {
      await page.addInitScript((seed) => {
        let s = seed >>> 0;
        Math.random = () => {
          s = (s + 0x6d2b79f5) >>> 0;
          let t = s;
          t = Math.imul(t ^ (t >>> 15), t | 1);
          t ^= t + Math.imul(t ^ (t >>> 7), t | 61);
          return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
        };
      }, o.seed);
    }
    await page.route('**/*', async (route) => {
      const u = new URL(route.request().url());
      if (u.hostname !== 'warframe-damage.com') return route.fulfill({ status: 204, body: '' });
      if (local[u.pathname]) {
        const f = path.join(insp, local[u.pathname]);
        return route.fulfill({ status: 200, contentType: typeOf(f), body: fs.readFileSync(f) });
      }
      if (u.pathname === '/getWeapon.php') {
        const name = u.searchParams.get('name') || '';
        const f = path.join(insp, 'samples', `weapon-${name.replace(/ /g, '_')}.json`);
        if (!fs.existsSync(f) && o.fetchMissing) {
          // One request through curl (honours the HTTPS proxy); cached for every later run.
          const r = spawnSync('curl', ['-sS', '--fail', '--retry', '4', '--retry-all-errors', '-m', '60', u.toString()]);
          try {
            if (r.status === 0) JSON.parse(r.stdout.toString('utf8'));
            if (r.status === 0) fs.writeFileSync(f, r.stdout);
          } catch (_) {
            /* not JSON: leave it uncached */
          }
        }
        if (!fs.existsSync(f)) {
          missing.push(name);
          return route.fulfill({ status: 404, contentType: 'text/plain', body: 'not cached' });
        }
        return route.fulfill({ status: 200, contentType: 'application/json', body: fs.readFileSync(f) });
      }
      if (u.pathname === '/getImg.php') return route.fulfill({ status: 200, contentType: 'image/png', body: PNG_1x1 });
      return route.fulfill({ status: 204, body: '' });
    });
    const errors = [];
    page.on('pageerror', (e) => errors.push(String(e.message || e)));
    const target = new URL(o.link);
    await page.goto(`https://warframe-damage.com/${target.search}`, { waitUntil: 'load' });
    await page.waitForTimeout(o.wait);
    const text = await page.evaluate(() => document.body.innerText);
    fs.writeFileSync(o.out, text);
    console.log(`wrote ${o.out} (${text.length} chars)`);
    if (missing.length) console.log(`weapons without a cached sample: ${[...new Set(missing)].join(', ')} (use --fetch-missing)`);
    if (errors.length) console.log(`page errors: ${errors.slice(0, 5).join(' | ')}`);
    if (!o.quiet) {
      // Summary table: layer (the heading before "Expand"), attack, random-queue DPS and TTK.
      const lines = text.split('\n').map((l) => l.trim()).filter(Boolean);
      let layer = '';
      const rows = [];
      lines.forEach((l, i) => {
        if (lines[i + 1] === 'Expand') layer = l;
        if (l === 'Random queue dps:') {
          const ttk = lines.slice(i + 1, i + 12).find((x) => x.startsWith('TTK:')) || '';
          rows.push(`${layer} | ${lines[i - 1]} | ${lines[i + 1]} | ${ttk.replace(/^TTK:\s*/, '')}`);
        }
      });
      console.log(['Layer | Attack | Random queue DPS | TTK', ...rows].join('\n'));
    }
  } finally {
    await browser.close();
  }
}

main().catch((e) => {
  console.error(e);
  process.exit(1);
});
