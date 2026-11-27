# research/tools: rebuild kit for `research/external/`

`research/external/` holds the third-party material that the spec (`docs/mechanics/`) and the findings
(`research/findings/`) cite by exact line: the warframe-damage.com client bundle ("the inspiration"), cached
wiki.warframe.com pages and Lua data modules, the WFCD `@wfcd/items` package and DE's Public Export, plus two small
scripts of our own. This directory holds the code and the pins that recreate it. It contains no third-party content.

## Why `research/external/` is not committed

It is third-party content: the inspiration is someone else's proprietary site, wiki text and modules are
CC BY-NC-SA 3.0, the Public Export is © Digital Extremes, and WFCD is a separate MIT project. Redistributing it in this
repository is not ours to decide (see `docs/architecture/overview.md` AD-11). `.gitignore` excludes
`/research/external/`, and `**/node_modules/` covers `research/tools/node_modules/`. Never stage either.

## Files here

| File | What it is |
|---|---|
| `rebuild-external.js` | The rebuild script. Idempotent and resumable (atomic writes; pinned files are skipped when their hash matches; rerun the same command after an interruption). Retries every download (exponential back-off, honours `Retry-After`). |
| `pin-wiki.js` | Finds every wiki page and module cited in `docs/` and `research/` (via `citations.js`) and pins each to its latest revision at or before **2026-11-24T23:59:59Z** with the MediaWiki API (`https://wiki.warframe.com/api.php`, 50 titles per query). Writes `wiki-pins.tsv`. `--check` lists citations that are not pinned yet (no network). |
| `citations.js` | Citation extraction (`W:<Page>` with optional `L37` / `§Section` suffixes, `wiki.warframe.com/w/<Page>`, `cache/wiki/<File>.wiki`, `Module:...` including brace lists, the spec's `M:` keys, and the module inventories of `research/findings/data-source-wiki-modules.md` §2 and `docs/data/pipeline.md` §11). `node citations.js` prints the lists. |
| `wiki-pins.tsv` | Committed pins: kind, cited title, file (relative to `research/external`), fetched title, redirect, revid, timestamp, sha1 (hex, from the API), size, status, note. |
| `manifest.tsv` | Path (relative to `research/external`), size and sha256 of every file the rebuild produced on 2026-11-27. |
| `decode-inspiration-url.js` | Decodes a warframe-damage.com build link (`?p=…&n=1` LZString, or legacy base64 without `n`) to JSON; `--compare build.json` diffs it with a stored build; `--encode build.json` makes a link. |
| `probe.js` | The oracle harness (`research/findings/oracle-feasibility.md`): runs a build link offline in headless Chromium against the cached inspiration and writes the page text. |
| `lib.js` | Shared helpers (curl-based HTTP with per-host politeness, hashing, file naming). |
| `package.json`, `package-lock.json` | Pin prettier 3.3.3, lz-string 1.5.0, html-to-text 9.0.5. |

## Rebuilding in a fresh container

Prerequisites: Node 22 with npm, `python3` (its `lzma` module decompresses the Public Export index), `curl`, `tar`.
Only `probe.js` needs Playwright with Chromium (preinstalled in Claude Code cloud containers; `require('playwright')`
works from any directory; do not run `playwright install`). Hosts used: `wiki.warframe.com`, `warframe-damage.com`,
`registry.npmjs.org`, `content.warframe.com` (and `origin.warframe.com`, which is tried first and was blocked here).

```bash
cd <repo>/research/tools
npm ci --no-audit --no-fund                  # prettier 3.3.3, lz-string 1.5.0, html-to-text 9.0.5 (the script runs this itself if missing)
node rebuild-external.js                     # builds <repo>/research/external
node rebuild-external.js /some/other/dir     # or any other target directory
node rebuild-external.js --only wiki,scripts # a subset: wiki, wfcd, inspiration, pe, scripts
```

Expected time: about 6 minutes on a 4-CPU cloud container, almost all of it the wiki step (483 files fetched one
at a time, at most about 2 requests per second). The tree is about 125 MB on disk (628 files, 227 MB of content: the WFCD category files under
`cache/wfcd/` are hard links into `cache/wfcd/package/`). The run
ends with a manifest comparison and writes `research/external/.rebuild/last-run.json` (counts, problems, hashes).
Exit code 0 means every step succeeded (a skipped Public Export is reported but does not fail the run).

What each step writes (paths relative to `research/external/`):

| Step | Output | Source and pin |
|---|---|---|
| `wiki` | `cache/wiki/<Page>.wiki` (page name with spaces as `_`, `/` as `__`, `'` as `%27`, `&` as `%26`), `cache/wiki-modules/raw/<Module_name>.lua` (`Module:Enemies/data/grineer` → `Enemies_data_grineer.lua`), `cache/wiki-modules/Furis_Incarnon_Genesis.wiki`, `cache/wiki-gadgets/enemyinfoboxslider.js`, `cache/web/stances-history/` (three old revisions of `Module:Stances/data` and its revision list up to the cut-off), `cache/wiki-modules/revs.json`, `cache/wiki-modules/allpages.txt` | `https://wiki.warframe.com/w/index.php?oldid=<revid>&action=raw` for every row of `wiki-pins.tsv`; each file's sha1 is checked against the pin |
| `wfcd` | `cache/wfcd/package/` (the npm package as published), `cache/wfcd/<Category>.json` and `cache/wfcd/i18n/<lang>.json` (hard links to, or copies of, `package/data/json/`: the paths the findings cite), `cache/wfcd/npm-items-1.1276.51.json` (registry metadata of that version) | `@wfcd/items@1.1276.51` tarball from registry.npmjs.org, checked against its npm `sha512` integrity |
| `inspiration` | `inspiration/index.html` (`/`), `min.js`, `jquery-min.js`, `style.css` (served as `/style-min.css`), `guide.html` (`/en/guide/`), `samples/enemies-min.json` (`/data/json/enemies-min.json`), `samples/weapon-<Name>.json` (`/getWeapon.php?name=<Name>`, for every weapon the docs, findings and test cases use: Catabolyst, Furis, Nikana Prime, Phenmor), derived `min.pretty.js` (prettier 3.3.3, `parser: "babel"`, default options), `index.txt` and `guide.txt` (html-to-text 9.0.5 defaults), `FETCHED.json` (URL, Last-Modified, ETag, sha256 per file) | live site, **not pinnable** |
| `pe` | `cache/public-export/index_en.txt.lzma`, `index_en.txt`, `index_en.lf.txt`, `manifests/<Export…>.json`, `SOURCE.txt` (which index host answered) | `origin.warframe.com` index (blocked here), then `content.warframe.com`; manifests by the index line verbatim; **not pinnable** |
| `scripts` | `oracle/probe.js`, `decode-inspiration-url.js` | copies of the scripts in this directory, at the paths the findings cite |

Existing inspiration and Public Export files are kept on a rerun; pass `--refresh` to download them again.

## Checking a rebuild against the manifest

```bash
node research/tools/rebuild-external.js --verify-only    # compare research/external with manifest.tsv
node research/tools/rebuild-external.js /dir --verify-only
```

It prints how many files match by sha256, differ, are missing or are extra (exit code 1 if any differ or are
missing). Expected after a rebuild on another day: wiki and WFCD files match exactly (they are pinned);
`cache/wiki-modules/allpages.txt` may differ (it is today's listing of the Module namespace, used only for discovery);
inspiration and Public Export files differ whenever upstream changed. Not in the manifest: `oracle/out.txt` (probe
output), `inspiration/FETCHED.json`, `cache/public-export/SOURCE.txt` and `.rebuild/` (they carry the date of the run).
To record a new baseline on purpose: `node research/tools/rebuild-external.js --write-manifest`.

## What can drift, and what drift means for the docs

**Wiki: pinned.** Every cited page and module is fetched at its latest revision at or before 2026-11-24T23:59:59Z, the
end of the period in which the spec was written (2026-11-20 to 2026-11-24). Line references such as
`W:Enemy_Body_Parts L37` therefore hold for these files, provided the earlier cache was itself current when cited (9
pages were edited after the cut-off; `wiki-pins.tsv` notes them and they are pinned to the older revision). On
2026-11-27 all 483 pins resolved (433 pages, 45 modules, 1 gadget, 3 old revisions of `Module:Stances/data`, and a
second copy of one page): no redirects, no missing pages, no title without a revision before the cut-off. A title
that is a redirect would be stored under its cited name with the target's content and the redirect recorded in the
`redirect` column; a missing one would be listed in `cache/wiki/UNAVAILABLE.txt` and never invented. A page fetched earlier than the cut-off and edited between its fetch and the cut-off would show a newer
revision here than the one a citation was written against; the pins cannot detect that case.

**WFCD: pinned** to `@wfcd/items@1.1276.51` with its npm integrity hash.

**Inspiration: drifts, and has drifted.** warframe-damage.com changed on 2026-11-27 (Last-Modified 11:15:48 GMT). The
spec and findings cite the **earlier** bundle, which is no longer obtainable (web.archive.org is blocked here):

| | min.js sha256 | min.pretty.js sha256 | min.pretty.js lines |
|---|---|---|---|
| cited by the docs (fetched 2026-11-20 … 24) | `5265a03779891aef1b8a2902c0671440fbd53a5d928a898c9bf81e474d363de4` | `7b9e4711068651646fe95c791fe5f7371e5c37ac1daa5bf4abf43ae9d71b3eff` | 22,038 |
| rebuilt 2026-11-27 | `7bb618341d34c02a1fc7400b29aef9e2410fa7dd5797615f1cb532a4285393e5` | `ba6cf810bf83d3bdad0507410176691505c3ab00ed4be3caf72149fe366d95c0` | 22,300 |

**Line references in the docs (`JS L…`, `min.pretty.js L…`, `HTML L…`) point to the earlier version, not to the file
this kit builds.** With the current bundle they are off by a varying amount (262 more lines overall) and some cited
code may have changed. Find the cited code by its identifiers (function names, string literals, property keys quoted
next to the line numbers) rather than by line. The other inspiration files (index.html, style.css, guide, samples)
can change at any time too; `FETCHED.json` records what was fetched and when, and `--verify-only` shows which files
changed since the manifest was written. Oracle numbers from `probe.js` (for example the table in
`research/findings/oracle-feasibility.md`) describe the bundle that was live when they were taken.

**Public Export: drifts** with every game update. The docs cite manifests of build label 2026.10.28 (`DS:pe` §1.7). On
2026-11-27 `origin.warframe.com` was blocked by the proxy and `content.warframe.com` served the index with
Last-Modified 2026-10-28 17:14 GMT, i.e. the same build; `SOURCE.txt` records the host, date and the manifest lines.

**Not rebuilt** (they existed in the earlier tree but are not sources of the docs' line references, or cannot be
fetched from here): `cache/poc/` (the local mlua proof of concept and its JSON outputs `cache/poc/out/*.json`, cited as
`D:<weapon>`, `D:Stances` and the cached JSON of `M:Arcane` / `M:Mods`; regenerate them from the pinned modules with
the data pipeline when it exists), `cache/wfcd/repo/src/` (a sparse git clone of WFCD's build scripts; GitHub was not
reachable), the full npm packument `cache/wfcd/npm-items.json` (replaced by the version document), and the
`wiki-modules/raw/` scratch files the earlier sessions fetched outside the cited set.

## Updating the pins

When a document starts citing a new wiki page or module, run `node pin-wiki.js --check` to see it, then decide
deliberately. A citation written after the cut-off was written against the revision its author read; if `wiki-pins.tsv`
notes "edited after cut-off" for it, check its line references against the pinned file. Re-pinning is otherwise
stable: existing rows do not change while the cut-off stays the same. `node pin-wiki.js` re-pins **every** title to the cut-off in `lib.js` (`WIKI_CUTOFF`). Moving the cut-off
changes the content of already-cited pages and so can break existing line references; change it only together with a
recheck of those references. After re-pinning, rebuild and `--write-manifest`, and commit `wiki-pins.tsv` and
`manifest.tsv`.

## Using the two scripts

```bash
node research/tools/decode-inspiration-url.js "https://warframe-damage.com/?p=…&n=1"
node research/tools/decode-inspiration-url.js "<link>" --compare research/test-cases/user-example-furis.json
node research/tools/decode-inspiration-url.js --encode research/test-cases/user-example-furis.json
node research/tools/probe.js "<link>" --out /tmp/out.txt --seed 1 [--wait 8000] [--fetch-missing]
```

`probe.js` routes `/`, `/min.js`, `/jquery-min.js`, `/style-min.css` and `/data/json/enemies-min.json` to the cached
files, `/getWeapon.php?name=X` to `samples/weapon-X.json` (spaces as underscores; `--fetch-missing` fetches an uncached
weapon once and stores it), `/getImg.php` to a 1x1 PNG and everything else to 204. `--seed` replaces `Math.random`
with a seeded generator, because the inspiration's firing queue is random. Both scripts find `research/tools/node_modules`
and `research/external/inspiration` from either of their locations (`research/tools/` or the copies under
`research/external/`).

## Checks run when the kit was built (2026-11-27)

- Rebuilt from empty into `research/external/` (5 min 22 s), rerun on the full tree (2 s, nothing fetched), and
  rebuilt into a scratch directory after interrupting the first attempt (resumed; 626 of 626 files matched the
  manifest of the time, inspiration and Public Export included because upstream had not changed in between).
- The `?p=` link the project started from is not stored in the repository; `research/test-cases/user-example-furis.json`
  holds only its decoded content. So the decoder was checked by round trip: `--encode` of that file, then decoding the
  link with `--compare` gives `IDENTICAL (structure and key order)`, and the decoded text equals the file byte for byte.
- `probe.js` on that link against the 2026-11-27 bundle, `--seed 1` (two runs, identical output):

  | Layer | Attack | Random queue DPS | TTK |
  |---|---|---|---|
  | Armor | Normal Attack | 19,944 | not killed |
  | Armor | Incarnon Form | 2,967,778 | 2.143 s |
  | Health | Normal Attack | 54,497 | not killed |
  | Health | Incarnon Form | 8,162,118 | 1.000 s |

  Unseeded, one run: 19,274 / 3,224,342 (2.202 s) / 48,820 / 8,867,733 (1.488 s). The values recorded in
  `research/findings/oracle-feasibility.md` came from the earlier bundle and one unseeded run.
