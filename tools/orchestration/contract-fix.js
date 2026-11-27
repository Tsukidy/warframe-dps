export const meta = {
  name: 'contract-fix',
  description: 'Whole-contract review pass 2: area fixers apply the 133 accepted items (model first, then engine, crates, data), merge, and an xhigh recheck',
  phases: [
    { title: 'Model', detail: 'model-core and model-records fixers in parallel worktrees' },
    { title: 'Merge model' },
    { title: 'Areas', detail: 'engine, crates, data fixers in parallel worktrees' },
    { title: 'Merge' },
    { title: 'Recheck', detail: 'three xhigh reviewers verify every item' },
  ],
}

const REPO = '/home/user/warframe-dps'
const DEV = 'ccr-659c593d-av1db8'
const TRAILER = 'Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>\nClaude-Session: https://claude.ai/code/session_01CqN69RiyT52Hfw4yFJWxLL'
const INPUT = `${REPO}/docs/project/inputs/contract-review-pass1.json`

const MODEL_IDS = 'ENG-2 ENG-7 ENG-15 ENG-19 M-1 M-2 M-3 M-4 M-5 M-6 M-7 M-8 M-9 M-10 M-11 M-12 M-13 M-14 M-15 M-16 M-17 M-18 M-19 API-1 API-3 API-5 API-6 API-7 API-8 API-9 API-11 API-12 API-14 API-15 API-16 API-17 API-20 API-23 DC-1 DC-2 DC-3 DC-4 DC-5 DC-6 DC-7 DC-8 DC-9 DC-10 DC-11 DC-12 DC-13 DC-14 DC-15 DC-16 DC-17 DC-18 DC-19 DC-20 DC-21 DC-22 DC-24 DC-25 DC-26 DC-27 LINT-GLAIVE C-36 C-37 C-38 C-39 T02c-R5 T02c-R6 T03b-R2 T03b-R5 T03c-2 T03c-3 T03c-TriggerScaling-Wave'
const ENGINE_IDS = 'ENG-1 ENG-2 ENG-3 ENG-4 ENG-5 ENG-6 ENG-7 ENG-8 ENG-9 ENG-10 ENG-11 ENG-12 ENG-13 ENG-14 ENG-15 ENG-16 ENG-17 ENG-18 ENG-19 ENG-20 ENG-21 ENG-22 ENG-23 ENG-24 M-3 M-4 API-1 API-3 API-6 API-16 DC-4 DC-11 LINT-GLAIVE C-31 C-32 C-33 C-34 C-35 C-36 C-37 T02d-1 T02d-6 T03b-R2 T03c-TriggerScaling-Wave'
const CRATES_IDS = 'ENG-17 ENG-18 M-6 M-7 M-8 M-9 M-10 M-11 M-12 M-18 M-19 API-2 API-3 API-4 API-5 API-6 API-7 API-9 API-10 API-12 API-13 API-14 API-15 API-16 API-17 API-18 API-19 API-21 API-22 API-23 DC-8 DC-14 DC-18 DC-19 DC-21 DC-28 DC-29 LINT-GLAIVE C-31 C-34 C-35 C-40 T02b-1 T02b-3 T02b-4 T02c-R1 T02c-R2 T02c-R3 T02c-R4 T02c-R5 T02c-R6 T02c-R7 T02d-2 T02d-3 T02d-4 T02d-5 T02d-6 T02d-7 T03b-R3 T03b-R5'
const DATA_IDS = 'M-2 M-6 M-18 API-12 API-18 DC-1 DC-14 DC-16 DC-18 DC-19 DC-20 DC-22 DC-23 DC-25 DC-27 C-37 C-38 T02c-R2 T02c-R5 T03b-R3 T03b-R5 T03b-R6'

const CTX = `Project: warframe-dps (main tree ${REPO}, dev branch ${DEV}; its CLAUDE.md is in your context), a local Warframe DPS calculator and build optimizer: Rust workspace (wfd-model, wfd-engine, wfd-data, wfd-buildcode, wfd-datagen, wfd-opt, wfd-server, xtask) and a React frontend (frontend/).

Situation: wave 0 froze every shared interface. Before ~35 wave 1-3 tasks implement against them in parallel, five independent xhigh reviewers checked them against the spec (docs/mechanics/00-09, normative) and the architecture (docs/architecture/, docs/data/pipeline.md) and found 133 problems. All of them are in ${INPUT} (keys model, engine, crates, data, requests; each item has id, severity, location, problem, evidence, consumers, fix, breaking, owner). The manager accepted every item with the reviewer's fix and recorded rulings and specific choices in CONTRACTS.md, section "2026-11-27 · Manager · Rulings on the whole-contract review (pass 1)": read that section first, it overrides the reviewer's text where they differ. You are one of several area fixers applying these items.

How to work:
- Verify each item against the current code and documents before changing anything. Reviewers can be wrong. If an item does not hold, or you find a fix of the same scope that is clearly better, do not apply the reviewer's version; record the reason.
- Prefer the smallest change that fixes the item; additive where possible; the breaking changes listed in the rulings are approved.
- Spec-text questions (the rulings' item 4) are being settled by a separate spec round that edits docs/mechanics/ at the same time. Never edit docs/mechanics/. Make your interfaces able to carry either answer where the question is open.
- Keep the code honest: doc comments state the contract; every rule cites its spec source (E-x.y, D-xxx, § numbers); no item names or /Lotus/ paths in engine or optimizer code (I-2).
- Record every change in CONTRACTS.md, in one new section "2026-11-27 · Contract review fixes (<your area>)": per item id, what changed (types, signatures, files), or why it was rejected; plus every edit you made outside your area's files and why. Update the architecture documents your area names so they describe the code.
- Generated files (frontend/src/api/generated/**, data/curated/schema.json, crates/wfd-model/src/compat_generated.rs) are regenerated with \`cargo xtask gen-ts\`, \`cargo xtask gen-schema\`, \`cargo xtask spec-sync\`, never edited by hand.
- Definition of done before you commit: cargo fmt --all --check; cargo clippy --workspace --all-targets -- -D warnings; cargo test --workspace; cargo xtask spec-sync --check; cargo xtask gen-ts --check; cargo xtask gen-schema --check; cargo xtask doc-check. If you changed anything the frontend consumes (generated TS, message keys, toggle descriptors), also: npm --prefix frontend ci (once), npm --prefix frontend run lint, run typecheck, test -- --run; fix frontend breakages your change causes. \`cargo xtask lint-engine\` fails today on the fixture name "Glaive" until the crates fixer lands LINT-GLAIVE; report its result but do not block on it unless LINT-GLAIVE is yours. Do not weaken, skip or delete tests; a test that pinned the old (wrong) contract is updated to the new contract, and the change is listed.
- The disk is shared by five worktrees: build only in your own worktree, and delete your worktree's target/ directory after your final commit (\`rm -rf <worktree>/target\`).
- Stage files by path (never \`git add -A\` or \`git add .\`). Commit (do not push) with the message "Contract review fixes (<area>): <summary>", a blank line, then exactly these lines:
${TRAILER}
- Do all the work yourself; do not use the Agent or Workflow tools.
The gitignored research/external/ (inspiration bundle, pinned wiki pages, Lua modules) exists only in the main tree: ${REPO}/research/external/.`

const WT = (area, base) => `Create your worktree first: \`git -C ${REPO} worktree add /home/user/wt/cr-${area} -b wip/cr-${area} ${base}\` (if it exists from an earlier attempt, reuse it and continue from its state). Work, build and commit only inside /home/user/wt/cr-${area}.`

const MODEL_CORE = `${CTX}

Your area: model-core. ${WT('model-core', DEV)}
Your files: crates/wfd-model/src/ effect DSL, mechanic kinds, stat registry, curated authoring types, rule-table types and item types (effect*.rs, mechanic*.rs, stats.rs, curated.rs, rules.rs, item.rs and the modules they use), data/curated/schema.json (regenerated), domain-model.md §10-§17. Another fixer (model-records) works in parallel in /home/user/wt/cr-model-records on weapon.rs, enemy.rs, enums.rs, scenario.rs, build.rs, bundle.rs, pick.rs, api/**, message/**, domain-model.md §1-§9 and server-api.md. Do not edit those files; if an item needs a change there, describe the exact change in your CONTRACTS.md section under "For model-records" (the merge reconciles them).
Your items: the parts that fall in your files of these ids: ${MODEL_IDS}. Several items have parts for other areas (engine, crates, data); do only the model-core part.`

const MODEL_RECORDS = `${CTX}

Your area: model-records. ${WT('model-records', DEV)}
Your files: crates/wfd-model/src/{weapon.rs, enemy.rs, enums.rs, scenario.rs, build.rs, bundle.rs, pick.rs, api/**, message/**} and the modules they use, frontend/src/api/generated/** (regenerated) and any frontend code your DTO changes break, domain-model.md §1-§9, server-api.md. Another fixer (model-core) works in parallel in /home/user/wt/cr-model-core on the effect DSL, mechanic kinds, stat registry, curated, rules and item types and domain-model.md §10-§17. Do not edit those files; if an item needs a change there, describe the exact change in your CONTRACTS.md section under "For model-core" (the merge reconciles them).
Your items: the parts that fall in your files of these ids: ${MODEL_IDS}. Several items have parts for other areas (engine, crates, data); do only the model-records part.`

const AREA = (area, ids, files) => `${CTX}

Your area: ${area}. ${WT(area, DEV)} The model fixes of this pass are already merged into ${DEV} (CONTRACTS.md sections "Contract review fixes (model-core)" and "(model-records)"): build on them, and use their new types instead of inventing parallel ones.
Your files: ${files}. Two other fixers work in parallel on the other areas (engine, crates, data). If an item needs a change in their files, describe it exactly in your CONTRACTS.md section under "For <area>" and implement your side so it compiles alone.
Your items: the parts that fall in your files of these ids: ${ids}.`

const ENGINE = AREA('engine', ENGINE_IDS, 'crates/wfd-engine/** (all of it for this pass, including Cargo.toml, error.rs, cancel.rs) and docs/architecture/engine.md (bring §1-§6 and §14 fully in line with the code, and §7-§8, §11, §15 as far as these items touch them; ruling C-34)')
const CRATES = AREA('crates', CRATES_IDS, 'crates/wfd-data/**, crates/wfd-buildcode/**, crates/wfd-datagen/**, crates/wfd-opt/**, crates/wfd-server/**, xtask/** (LINT-GLAIVE is yours), and the documents docs/architecture/{overview,optimizer,server-api (non-DTO parts),testing,frontend}.md, docs/data/pipeline.md and docs/architecture/work-breakdown.md (task sections and the §8 ownership map, exactly as the rulings and the items say; this pass only)')
const DATA = AREA('data', DATA_IDS, 'data/curated/rules/** (T03b, T03c tables), data/fixtures/** and crates/wfd-model/tests/{fixtures,rules_tables,melee_tables}.rs; fill the new model fields of this pass where the fixture or the rule tables need them (e.g. ability-source marks, F_inh custom item, doughty_rank, override files) and align the fixture with the real rule tables as T03b-R6 and the rulings say')

const FIX_SCHEMA = {
  type: 'object',
  properties: {
    area: { type: 'string' }, branch: { type: 'string' }, commit: { type: 'string' },
    applied: { type: 'array', items: { type: 'string' }, description: 'one line per item id: what changed' },
    rejected: { type: 'array', items: { type: 'string' }, description: 'item id: why' },
    for_other_areas: { type: 'array', items: { type: 'string' } },
    checks: { type: 'array', items: { type: 'object', properties: { command: { type: 'string' }, passed: { type: 'boolean' }, note: { type: 'string' } }, required: ['command', 'passed'] } },
    left_undone: { type: 'array', items: { type: 'string' } },
  },
  required: ['area', 'branch', 'commit', 'applied', 'rejected', 'checks', 'left_undone'],
}
const MERGE_SCHEMA = {
  type: 'object',
  properties: {
    merged: { type: 'array', items: { type: 'string' } }, head_commit: { type: 'string' },
    integration_fixes: { type: 'array', items: { type: 'string' } },
    checks: { type: 'array', items: { type: 'object', properties: { command: { type: 'string' }, passed: { type: 'boolean' }, note: { type: 'string' } }, required: ['command', 'passed'] } },
    problems: { type: 'array', items: { type: 'string' } },
  },
  required: ['merged', 'head_commit', 'checks', 'problems'],
}

const MERGE = (branches, reports) => `You are the integrator for the warframe-dps project in ${REPO} (branch ${DEV} checked out; its CLAUDE.md is in your context). Merge these branches into ${DEV}, in this order: ${branches.join(', ')}. They apply the accepted items of the whole-contract review (CONTRACTS.md "Manager · Rulings on the whole-contract review (pass 1)"). The fixers' reports: ${JSON.stringify(reports).slice(0, 14000)}

Steps:
1. Check \`git -C ${REPO} status\`. Another agent may be editing docs/mechanics/*.md in this tree right now (a spec round); leave those files alone, never stage them, and do not let a merge touch them (none of these branches should change docs/mechanics).
2. Merge each branch with \`git merge --no-ff <branch>\` and the message "Merge <branch>", a blank line, then exactly these lines:
${TRAILER}
   Resolve conflicts so that both sides' accepted changes survive: CONTRACTS.md and documents are a union (keep every section; merge edits to the same paragraph by keeping both changes); in code, combine both changes. Where a fixer described a change for another area ("For <area>" in its CONTRACTS.md section) and the other side did not make it, make it now if it is small and clearly specified; otherwise list it under problems. Generated files are regenerated (\`cargo xtask gen-ts\`, \`gen-schema\`, \`spec-sync\`), never hand-merged; Cargo.lock is re-locked.
3. Run the full Definition of done on the merged tree: cargo fmt --all --check; cargo clippy --workspace --all-targets -- -D warnings; cargo test --workspace; cargo xtask spec-sync --check; gen-ts --check; gen-schema --check; doc-check; lint-engine; and the frontend checks (npm --prefix frontend ci, run lint, run typecheck, test -- --run, run build). Fix integration breakages with the smallest correct change, committed as "Integrate contract review fixes: <summary>" with the same trailer lines. Do not weaken or ignore tests; if a break needs a design decision, leave it failing and report it.
4. Remove the merged worktrees (\`git -C ${REPO} worktree remove --force /home/user/wt/<name>\`) and delete the merged branches (\`git branch -d\`). Do not push.
Do all the work yourself; do not use the Agent or Workflow tools.`

const RECHECK = (area, keys) => `Project: warframe-dps (${REPO}, branch ${DEV}; its CLAUDE.md is in your context). A whole-contract review found 133 problems in the frozen interfaces (${INPUT}); the manager accepted them (CONTRACTS.md "Manager · Rulings on the whole-contract review (pass 1)") and area fixers have just applied them and merged (CONTRACTS.md sections "2026-11-27 · Contract review fixes (…)", one per area). Your job: verify the items of ${keys} in that file, adversarially. For each item: is it resolved in the current code and documents (the reviewer's problem is gone, the fix matches the ruling, the documents describe the code), rejected with a reason that holds up, or still open? Also report NEW problems the fixes introduced in your area (a contradiction between two frozen interfaces, a consumer task that now cannot implement its spec section, a doc that now misdescribes the code). Read the code; run tests where useful (CARGO_TARGET_DIR=/tmp/claude-0/recheck-${area}, delete it afterwards). Do not edit any file. Do the work yourself; do not use the Agent or Workflow tools.`

const RECHECK_SCHEMA = {
  type: 'object',
  properties: {
    resolved: { type: 'integer' },
    open: { type: 'array', items: { type: 'object', properties: { id: { type: 'string' }, severity: { type: 'string', enum: ['critical', 'major', 'minor'] }, problem: { type: 'string' }, evidence: { type: 'string' }, fix: { type: 'string' }, owner: { type: 'string' } }, required: ['id', 'severity', 'problem', 'evidence', 'fix'] } },
    rejected_ok: { type: 'array', items: { type: 'string' } },
    new_issues: { type: 'array', items: { type: 'object', properties: { id: { type: 'string' }, severity: { type: 'string', enum: ['critical', 'major', 'minor'] }, problem: { type: 'string' }, evidence: { type: 'string' }, fix: { type: 'string' }, owner: { type: 'string' } }, required: ['id', 'severity', 'problem', 'evidence', 'fix'] } },
    overall: { type: 'string' },
  },
  required: ['resolved', 'open', 'new_issues', 'overall'],
}

phase('Model')
const [core, records] = await parallel([
  () => agent(MODEL_CORE, { label: 'fix:model-core', phase: 'Model', effort: 'xhigh', schema: FIX_SCHEMA }),
  () => agent(MODEL_RECORDS, { label: 'fix:model-records', phase: 'Model', effort: 'xhigh', schema: FIX_SCHEMA }),
])
if (!core || !records) { log('a model fixer returned nothing; stopping before the merge'); return { core, records } }

phase('Merge model')
const mergeModel = await agent(MERGE(['wip/cr-model-core', 'wip/cr-model-records'], [core, records]), { label: 'merge:model', phase: 'Merge model', effort: 'high', schema: MERGE_SCHEMA })
if (!mergeModel || mergeModel.problems.some((p) => /fail/i.test(p))) log(`model merge problems: ${JSON.stringify(mergeModel && mergeModel.problems).slice(0, 500)}`)

phase('Areas')
const areas = await parallel([
  () => agent(ENGINE, { label: 'fix:engine', phase: 'Areas', effort: 'xhigh', schema: FIX_SCHEMA }),
  () => agent(CRATES, { label: 'fix:crates', phase: 'Areas', effort: 'xhigh', schema: FIX_SCHEMA }),
  () => agent(DATA, { label: 'fix:data', phase: 'Areas', effort: 'xhigh', schema: FIX_SCHEMA }),
])
const done = areas.filter(Boolean)
log(`area fixers done: ${done.map((a) => a.area).join(', ')}`)

phase('Merge')
const merge = await agent(MERGE(done.map((a) => a.branch || `wip/cr-${a.area}`), done), { label: 'merge:areas', phase: 'Merge', effort: 'high', schema: MERGE_SCHEMA })

phase('Recheck')
const rechecks = await parallel([
  () => agent(RECHECK('model', 'the keys "model" and "data" (M-*, DC-*)'), { label: 'recheck:model-data', phase: 'Recheck', effort: 'xhigh', schema: RECHECK_SCHEMA }),
  () => agent(RECHECK('engine', 'the key "engine" (ENG-*)'), { label: 'recheck:engine', phase: 'Recheck', effort: 'xhigh', schema: RECHECK_SCHEMA }),
  () => agent(RECHECK('crates', 'the keys "crates" and "requests" (API-*, C-*, T02*, T03*, LINT-GLAIVE)'), { label: 'recheck:crates-requests', phase: 'Recheck', effort: 'xhigh', schema: RECHECK_SCHEMA }),
])
return { core, records, mergeModel, areas: done, merge, rechecks }
