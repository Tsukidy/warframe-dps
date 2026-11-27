export const meta = {
  name: 'external-rebuild',
  description: 'Rebuild the gitignored research/external tree from scratch with a committed rebuild kit, test the kit, and analyse the 2026-11-27 inspiration update',
  phases: [
    { title: 'Rebuild', detail: 'write research/tools/ kit and use it to build research/external/' },
    { title: 'Check', detail: 'test the kit from an empty directory; map old inspiration citations onto the new bundle' },
  ],
}

const SCRATCH = '/tmp/claude-0/-home-user-warframe-dps/1058c195-3e21-5e86-9591-c42773f99ef3/scratchpad'

const CTX = `Project: warframe-dps, a local Warframe DPS calculator (repository /home/user/warframe-dps; its CLAUDE.md is in your context). Its damage spec (docs/mechanics/) and research findings (research/findings/) cite third-party sources by exact line: the prettified warframe-damage.com bundle ("the inspiration", cited as "JS L9154-9204" or "min.pretty.js L10467-10469"), cached wiki.warframe.com pages ("W:Enemy_Body_Parts L37") and wiki Lua data modules ("Module:Weapons/data/..."). That material lives in research/external/, which is gitignored on purpose: it is third-party content and must never be committed. This container is fresh and research/external/ does not exist. Agents that check the spec against its sources need it now, and future sessions must be able to rebuild it the same way, so the project needs both the tree and a committed, tested rebuild kit.

What is known about the earlier tree (from docs/project/continuation.md §4 and the earlier manager's notes):
- research/external/inspiration/: index.html, min.js, jquery-min.js, style.css (served as /style-min.css), guide.html, samples/*.json (weapon-<Name>.json from https://warframe-damage.com/getWeapon.php?name=<Name>; enemies-min.json from /data/json/enemies-min.json), .txt renderings of index and guide made with html-to-text 9.0.5, and min.pretty.js made with prettier 3.3.3 (default options) from min.js.
- research/external/oracle/probe.js and research/external/decode-inspiration-url.js were our own scripts (a Playwright harness that serves the cached inspiration files and runs a build link; an lz-string 1.5.0 decoder of a ?p= build link). Their behaviour is described in research/findings/oracle-feasibility.md. They were never committed and are lost.
- research/external/cache/wiki/*.wiki: pages fetched as https://wiki.warframe.com/w/<Page>?action=raw; file name = page name with spaces as "_", "/" as "__", "'" as %27 and "&" as %26. cache/wiki-modules/ held Lua data modules (raw/), allpages listings and revs.json. cache/wfcd/ held the WFCD warframe-items npm package. cache/public-export/ held DE's Public Export (index_en.txt.lzma from origin.warframe.com, manifests from content.warframe.com).
- The inspiration changed TODAY (2026-11-27, Last-Modified 11:15 GMT): the current min.js has sha256 7bb618341d34c02a1fc7400b29aef9e2410fa7dd5797615f1cb532a4285393e5 and prettifies to 22,300 lines. The spec cites the earlier version: min.js 5265a03779891aef1b8a2902c0671440fbd53a5d928a898c9bf81e474d363de4, min.pretty.js 7b9e4711068651646fe95c791fe5f7371e5c37ac1daa5bf4abf43ae9d71b3eff, 22,038 lines. The earlier version cannot be obtained here (web.archive.org is blocked).
- The spec was written between 2026-11-20 and 2026-11-24, so wiki line references hold for the revisions current on 2026-11-24.
- Network from this container: wiki.warframe.com, warframe-damage.com (sometimes drops connections: retry), registry.npmjs.org and content.warframe.com are reachable; origin.warframe.com, web.archive.org and the GitHub API are not. Be polite to the wiki: at most about 2 requests per second, and batch API queries (up to 50 titles per query).
- Node 22, python3 and Playwright with Chromium are installed. require('playwright') works from any directory and chromium.launch() needs no options; do not run "playwright install".

Do all the work yourself; do not use the Agent or Workflow tools. Do not commit or push (the manager commits).`

const BUILD = `${CTX}

Your task: write a rebuild kit in research/tools/ and build research/external/ with it.

1. research/tools/ (committed later; our own code only, no third-party content):
   - A rebuild script (node or bash) that recreates research/external/ into a target directory given as an argument (default research/external), idempotently and resumably, with retries.
   - Wiki pages: the page list is every wiki page cited in docs/ and research/ (citation forms "W:<Page>", "W:<Page> L37", "W:<Page> §Section", "wiki.warframe.com/w/<Page>"; there are roughly 430 distinct strings, some carrying section or line suffixes: normalise them to page titles). Lua modules: every "Module:..." named in docs/ and research/ (docs/data/pipeline.md and research/findings/data-source-wiki-modules.md list the data modules). Pin each title to the latest revision at or before 2026-11-24T23:59:59Z using the MediaWiki API (find the API endpoint of wiki.warframe.com), record title, revid, timestamp and sha1 in a committed research/tools/wiki-pins.tsv, and fetch by oldid. Titles that are redirects: store the target's content under the cited name and record the redirect. Titles that do not exist or have no revision before the cut-off: list them, do not invent content.
   - WFCD: the npm package @wfcd/items at version 1.1276.51 (the version research/findings/data-source-wfcd.md evaluated) into cache/wfcd/.
   - Inspiration: the files listed above, plus samples/weapon-<Name>.json for every weapon name the findings, test cases and docs use with getWeapon.php or weapon-<Name>.json; min.pretty.js via prettier 3.3.3 (parser babel, default options); record the hashes and line count in the README and say plainly that line references in the docs point to the earlier version.
   - Public Export: try it; if the index host is blocked, record that and skip.
   - Our two scripts, recreated from research/findings/oracle-feasibility.md: research/tools/probe.js and research/tools/decode-inspiration-url.js, with paths relative to the repository (no hard-coded /home/user). The rebuild script also copies them to their old locations (research/external/oracle/probe.js, research/external/decode-inspiration-url.js), because findings refer to those paths.
   - research/tools/package.json pinning prettier 3.3.3, lz-string 1.5.0 and html-to-text 9.0.5. node_modules must not be committed: check that .gitignore covers research/tools/node_modules and add a rule only if it does not.
   - research/tools/manifest.tsv: path (relative to research/external), size and sha256 of every file the rebuild produced.
   - research/tools/README.md: what research/external/ is, why it is not committed, the exact rebuild steps for a fresh container (tool versions, commands, expected time), how to check the result against the manifest, which parts can drift (inspiration, Public Export) and what drift means for the line references in the docs.
2. Build research/external/ by running your rebuild script from empty. The script must be what built the tree; hand-fetched files do not count.
3. Check the result: decode the build link in research/test-cases/user-example-furis.json with decode-inspiration-url.js and compare with the decoded content stored in that file; run probe.js on that link if you can and say what it printed (the inspiration is now a newer version, so numbers may differ from those recorded; report them, do not judge them).
Edit only research/tools/, research/external/ and (if needed) .gitignore.`

const BUILD_SCHEMA = {
  type: 'object',
  properties: {
    files_created: { type: 'array', items: { type: 'string' } },
    rebuild_command: { type: 'string' },
    wiki_pages_pinned: { type: 'integer' },
    wiki_titles_failed: { type: 'array', items: { type: 'string' } },
    modules_pinned: { type: 'integer' },
    inspiration: { type: 'string', description: 'files fetched, min.js and min.pretty.js sha256, line count' },
    not_rebuilt: { type: 'array', items: { type: 'string' } },
    decoder_check: { type: 'string' },
    probe_check: { type: 'string' },
    notes: { type: 'string' },
  },
  required: ['files_created', 'rebuild_command', 'wiki_pages_pinned', 'wiki_titles_failed', 'modules_pinned', 'inspiration', 'not_rebuilt', 'decoder_check', 'probe_check'],
}

const VERIFY = (rep) => `${CTX}

Another agent just wrote a rebuild kit in research/tools/ and used it to build research/external/. Its report: ${JSON.stringify(rep || {}).slice(0, 3000)}

Your task: test the kit as a fresh container would use it.
1. Read research/tools/README.md and follow it literally, rebuilding into the empty directory ${SCRATCH}/rebuild-test/external (install node tools into ${SCRATCH}/rebuild-test/, not into the repository).
2. Compare the rebuilt tree with research/tools/manifest.tsv and with research/external/: count files that match by sha256, differ, or are missing. For each difference say whether it is inherent (upstream drift since the first build) or a kit defect.
3. Fix every kit defect you find (script bugs, README steps that do not work, missing pins, pages the page-list extraction misses: compare the page list with the W: citations in docs/ and research/ yourself) and rerun until the rebuild works or the remaining differences are inherent. If you change the page list, also run the fixed script against research/external/ so the live tree has the added pages, and regenerate manifest.tsv.
Edit only research/tools/, research/external/ and (if needed) .gitignore; never delete files from research/external/ that the manifest lists.`

const VERIFY_SCHEMA = {
  type: 'object',
  properties: {
    identical: { type: 'integer' }, differing: { type: 'integer' }, missing: { type: 'integer' },
    differences: { type: 'array', items: { type: 'string' } },
    fixes: { type: 'array', items: { type: 'string' } },
    remaining_problems: { type: 'array', items: { type: 'string' } },
  },
  required: ['identical', 'differing', 'missing', 'differences', 'fixes', 'remaining_problems'],
}

const DIFF = `${CTX}

The inspiration bundle is now in research/external/inspiration/min.pretty.js (the 2026-11-27 version, 22,300 lines). The spec (docs/mechanics/*.md) and the findings research/findings/inspiration-*.md and oracle-feasibility.md cite the earlier version (22,038 lines) by line. Those citations are how implementers and the accuracy audit find the inspiration's code, and the inspiration is the project's test oracle, so we need to know (a) where each cited piece of code is now and (b) whether any behaviour the spec attributes to the inspiration changed.

Your task:
1. Collect the citations: every reference to an inspiration line or range in docs/mechanics/*.md and research/findings/*.md ("JS L…", "min.pretty.js L…", "L…" in an inspiration context), each with what the citing text says is there (function name, identifiers, quoted expression, constant, behaviour).
2. Relocate each in the new file by content. From the relocated anchors derive a piecewise map from old line to new line. Between two edits the offset is constant, so a change of offset between neighbouring anchors brackets a block that was inserted, removed or rewritten.
3. Read every bracketed region in the new file and classify the change: line shift only, data change (items, numbers, tables), or logic change. For each data or logic change that touches something the spec relies on, name the spec document and id (D-xxx, E-x.y, TV-xx-yy, section) and quote both the spec's statement and the new code. Say what you cannot relocate or cannot decide; do not guess.
4. Look in the new index.html and guide.html (research/external/inspiration/) for a changelog, news or version note near 2026-11-27, and report what it lists.
5. Write research/findings/inspiration-update-2026-11-27.md starting with "Status: draft", containing: a summary; the old→new line map as a table of old line ranges and offsets; the changed regions with your classification and evidence; the affected spec items; and the citations you could not relocate. Cite new-file lines as "min.pretty.js@7bb6 L…". Edit no other file.`

const DIFF_SCHEMA = {
  type: 'object',
  properties: {
    citations_found: { type: 'integer' },
    citations_relocated: { type: 'integer' },
    changed_regions: { type: 'array', items: { type: 'object', properties: { new_lines: { type: 'string' }, kind: { type: 'string', enum: ['shift', 'data', 'logic', 'unknown'] }, summary: { type: 'string' }, spec_items: { type: 'array', items: { type: 'string' } } }, required: ['new_lines', 'kind', 'summary'] } },
    changelog: { type: 'string' },
    unrelocated: { type: 'array', items: { type: 'string' } },
    file: { type: 'string' },
  },
  required: ['citations_found', 'citations_relocated', 'changed_regions', 'changelog', 'unrelocated', 'file'],
}

phase('Rebuild')
const built = await agent(BUILD, { label: 'rebuild', phase: 'Rebuild', effort: 'high', schema: BUILD_SCHEMA })
if (!built) { log('rebuild agent returned nothing; stopping'); return { built: null } }
log(`rebuild: ${built.wiki_pages_pinned} wiki pages, ${built.modules_pinned} modules, ${built.wiki_titles_failed.length} titles failed`)
phase('Check')
const [verified, diff] = await parallel([
  () => agent(VERIFY(built), { label: 'verify-kit', phase: 'Check', effort: 'high', schema: VERIFY_SCHEMA }),
  () => agent(DIFF, { label: 'inspiration-diff', phase: 'Check', effort: 'high', schema: DIFF_SCHEMA }),
])
return { built, verified, diff }
