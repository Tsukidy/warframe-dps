export const meta = {
  name: 'external-rebuild-kit',
  description: 'Write and test a reproducible rebuild script for the gitignored research/external tree (pinned wiki revisions, hash manifest)',
  phases: [
    { title: 'Build', detail: 'write research/tools/ rebuild kit and manifest' },
    { title: 'Verify', detail: 'rebuild into an empty directory and compare hashes' },
  ],
}

const CTX = `Project: warframe-dps (repository /home/user/warframe-dps; its CLAUDE.md is in your context). The directory research/external/ (about 73 MB) is gitignored on purpose: it holds third-party material that must never be committed (the prettified warframe-damage.com bundle "the inspiration", cached wiki.warframe.com pages and Lua data modules, WFCD warframe-items files, DE Public Export files) plus two small scripts of our own. It exists only in this container. A future session will run in a fresh container without it, yet the spec in docs/mechanics/ and the findings in research/findings/ cite exact line numbers in research/external/inspiration/min.pretty.js and in cached wiki pages (for example "W:Enemy_Body_Parts L37", "min.pretty.js L10467-10469"). So the tree must be rebuildable byte-for-byte where possible, and drift must be detectable where not.

What is known about how it was built:
- research/external/inspiration/: index.html, min.js, jquery-min.js, style.css (served as /style-min.css), guide.html, samples/*.json (weapon-<Name>.json from https://warframe-damage.com/getWeapon.php?name=<Name>, enemies-min.json from /data/json/enemies-min.json), and the .txt renderings of index/guide made with html-to-text 9.0.5. min.pretty.js was made with prettier 3.3.3 (default options) from min.js; 22,038 lines; sha256 of min.js 5265a03779891aef1b8a2902c0671440fbd53a5d928a898c9bf81e474d363de4 and of min.pretty.js 7b9e4711068651646fe95c791fe5f7371e5c37ac1daa5bf4abf43ae9d71b3eff.
- research/external/oracle/probe.js (ours: a Playwright harness that serves the cached inspiration files and runs a build link) and research/external/decode-inspiration-url.js (ours: lz-string 1.5.0 decode of a ?p= build link).
- research/external/cache/wiki/*.wiki: pages fetched as https://wiki.warframe.com/w/<Page>?action=raw; the file name is the page name with spaces as "_", "/" as "__", "'" as %27 and "&" as %26. cache/wiki-modules/ holds Lua data modules (raw/), allpages listings and revs.json. cache/wfcd/ is the WFCD warframe-items npm package content (see its package.json for the version). cache/public-export/ is DE's Public Export (index_en.txt.lzma from origin.warframe.com, manifests from content.warframe.com). cache/poc/ is a local mlua proof of concept (ours, rebuildable from source in it, or skippable). cache/web/stances-history and cache/wiki-gadgets/ are small extras. More wiki pages were fetched into /tmp/claude-0/-home-user/fba0a26e-e0d2-55c2-9445-fb2cf89f09fb/scratchpad/wiki/ during spec review; include them in the page list and copy them into research/external/cache/wiki/ if missing there (that copy is allowed).
Outbound HTTPS goes through a proxy; warframe-damage.com sometimes drops connections, so retry. Be polite to the wiki: at most about 2 requests per second, and batch API queries (up to 50 titles per query).`

const BUILD = `${CTX}

Your task: create research/tools/ (committable, our own code only, no third-party content) containing:
1. README.md: what research/external/ is, why it is not committed, and exact steps to rebuild it in a fresh container (tool versions, commands, expected time), how to check the result against the manifest, and what to do on drift (which documents' line references may then be off).
2. A rebuild script (bash or node, your choice; node is available, as is python3) that recreates research/external/ into a target directory given as an argument (default research/external), idempotently and resumably, with retries. For wiki pages and wiki Lua modules, pin each file to the revision it was fetched at: use the MediaWiki API to find, for each title, the revision id that matches the cached content (compare sha1 of the cached bytes with the API's rvprop=sha1, walking back through revisions from the file's mtime if needed), record title -> revid in a committed list, and fetch by oldid (https://wiki.warframe.com/w/index.php?oldid=<revid>&action=raw). Report the pages whose cached content matches no revision. For WFCD pin the npm package version. For Public Export and the inspiration, record what is fetched and accept drift (they cannot be pinned), but check hashes.
3. A manifest of every file now in research/external/ (path relative to research/external, size, sha256), excluding cache/poc build outputs (target directories, binaries). Hashes and sizes are fine to commit; content is not.
4. Copies of our own two scripts (probe.js, decode-inspiration-url.js) under research/tools/, with paths made relative to the repository instead of hard-coded /home/user paths, and a package.json pinning prettier 3.3.3, lz-string 1.5.0, html-to-text 9.0.5 (do not commit node_modules; add research/tools/node_modules to .gitignore only if it is not already covered by the existing **/node_modules/ rule).
Do not modify anything under research/external/ except adding the missing scratchpad wiki pages. Do not commit. Do not use the Agent or Workflow tools. Final response: a plain-text report under 250 words: files created, how many wiki titles were pinned and how many could not be, anything that cannot be rebuilt.`

const VERIFY = (rep) => `${CTX}

Another agent just wrote a rebuild kit in research/tools/ (its report: ${JSON.stringify(rep || '').slice(0, 2500)}). Your task: test it as a fresh container would. Do not modify research/external/ and do not commit. Do not use the Agent or Workflow tools.
1. Read research/tools/README.md and follow it literally, rebuilding into an empty directory under /tmp/claude-0/-home-user/fba0a26e-e0d2-55c2-9445-fb2cf89f09fb/scratchpad/rebuild-test/ (install the node tools into a scratch directory, not into the repository).
2. Compare the rebuilt tree with the manifest and with the live research/external/: count files that match by sha256, that differ, and that are missing; for differences in min.pretty.js or wiki pages, say whether line numbers would shift.
3. Fix any defect you find in research/tools/ (script bugs, README steps that do not work, missing pins) and rerun until it works or the remaining differences are inherent (upstream drift). You may edit only files in research/tools/ and .gitignore.
Final response: a plain-text report under 250 words: what was rebuilt identically, what drifted and why, what you fixed.`

phase('Build')
const b = await agent(BUILD, { label: 'build-kit', phase: 'Build', effort: 'high' })
phase('Verify')
const v = await agent(VERIFY(b), { label: 'verify-kit', phase: 'Verify', effort: 'high' })
return { build: b, verify: v }
