#!/usr/bin/env node
'use strict';
// Decode a warframe-damage.com build link (?p=<data>[&n=1]) into its JSON build, as the inspiration does
// (getSettingsFromUrl: n present -> LZString.decompressFromEncodedURIComponent, else legacy atob base64;
// research/findings/inspiration-feature-inventory.md, "Build link").
//
//   node decode-inspiration-url.js "<link | ?p=... | p value>"     print the build as JSON (4-space indent)
//   node decode-inspiration-url.js - < link.txt                     read the link from stdin
//   node decode-inspiration-url.js <link> --compare build.json      also compare with a stored decoded build
//   node decode-inspiration-url.js --encode build.json              make a link from a build JSON (n=1 form)
//
// Needs lz-string 1.5.0: `npm ci` in research/tools (this script finds research/tools/node_modules from either of
// its locations, research/tools/ or research/external/), or put lz-string on NODE_PATH.

const fs = require('fs');
const path = require('path');

function loadLzString() {
  try {
    return require('lz-string');
  } catch (_) {
    /* fall through */
  }
  // Walk up from this file to the repository root and use research/tools/node_modules.
  let dir = __dirname;
  for (let i = 0; i < 6; i++) {
    const cand = path.join(dir, 'research', 'tools', 'node_modules', 'lz-string');
    if (fs.existsSync(cand)) return require(cand);
    const local = path.join(dir, 'node_modules', 'lz-string');
    if (fs.existsSync(local)) return require(local);
    dir = path.dirname(dir);
  }
  throw new Error('lz-string not found: run `npm ci` in research/tools');
}

function decode(input) {
  const s = input.trim();
  let p;
  let n;
  if (/^https?:\/\//.test(s) || s.startsWith('?') || s.includes('p=')) {
    const u = new URL(s.startsWith('http') ? s : `https://warframe-damage.com/${s.startsWith('?') ? s : '?' + s}`);
    p = u.searchParams.get('p');
    n = u.searchParams.get('n');
    if (p === null) throw new Error('no p= parameter in the link');
  } else {
    p = s;
    n = '1';
  }
  let text;
  if (n) {
    text = loadLzString().decompressFromEncodedURIComponent(p);
    if (text === null || text === '') throw new Error('LZString decompression failed');
  } else {
    text = Buffer.from(p, 'base64').toString('latin1'); // atob semantics
  }
  return JSON.parse(text);
}

function encode(build) {
  const p = loadLzString().compressToEncodedURIComponent(JSON.stringify(build));
  const u = new URL('https://warframe-damage.com/');
  u.searchParams.set('p', p);
  u.searchParams.set('n', '1');
  return u.toString();
}

// Structural comparison; returns a list of "path: a != b" strings.
function diff(a, b, at = '$', out = []) {
  if (typeof a !== typeof b || Array.isArray(a) !== Array.isArray(b) || (a === null) !== (b === null)) {
    out.push(`${at}: ${JSON.stringify(a)} != ${JSON.stringify(b)}`);
  } else if (a && typeof a === 'object') {
    const keys = new Set([...Object.keys(a), ...Object.keys(b)]);
    for (const k of keys) {
      if (!(k in a)) out.push(`${at}.${k}: missing in decoded link`);
      else if (!(k in b)) out.push(`${at}.${k}: missing in stored file`);
      else diff(a[k], b[k], `${at}.${k}`, out);
    }
  } else if (a !== b) out.push(`${at}: ${JSON.stringify(a)} != ${JSON.stringify(b)}`);
  return out;
}

function main() {
  const args = process.argv.slice(2);
  const ei = args.indexOf('--encode');
  if (ei >= 0) {
    console.log(encode(JSON.parse(fs.readFileSync(args[ei + 1], 'utf8'))));
    return;
  }
  const ci = args.indexOf('--compare');
  const cmp = ci >= 0 ? args[ci + 1] : null;
  const pos = args.filter((a, i) => !(ci >= 0 && (i === ci || i === ci + 1)));
  if (!pos.length) {
    console.error('usage: decode-inspiration-url.js <link | -> [--compare build.json] | --encode build.json');
    process.exit(2);
  }
  const input = pos[0] === '-' ? fs.readFileSync(0, 'utf8') : pos[0];
  const build = decode(input);
  if (!cmp) {
    console.log(JSON.stringify(build, null, 4));
    return;
  }
  const stored = JSON.parse(fs.readFileSync(cmp, 'utf8'));
  const d = diff(build, stored);
  const sameText = JSON.stringify(build, null, 4) === JSON.stringify(stored, null, 4);
  console.log(d.length ? `DIFFERENT (${d.length}):\n${d.join('\n')}` : `IDENTICAL (structure${sameText ? ' and key order' : ''})`);
  process.exit(d.length ? 1 : 0);
}

module.exports = { decode, encode, diff };
if (require.main === module) main();
