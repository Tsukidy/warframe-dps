export const meta = {
  name: 'impl-phase',
  description: 'Implement a set of independent work-breakdown tasks (worktree per task), review and fix each, then merge into the dev branch',
  phases: [
    { title: 'Implement', detail: 'one agent per task, in its own git worktree (or the main tree for a single task)' },
    { title: 'Review', detail: 'adversarial review per task (where enabled)' },
    { title: 'Fix', detail: 'apply confirmed review issues' },
    { title: 'Merge', detail: 'merge task branches, run the full definition of done, fix integration breaks' },
  ],
}

// args: { label, trailer, mode: 'main' | 'worktree', merge: bool, tasks: [{ id, effort, review: bool, note? }] }
// trailer: the exact commit trailer lines for this session, e.g. 'Co-Authored-By: ...\nClaude-Session: https://claude.ai/code/session_...'
const A = args
const REPO = '/home/user/warframe-dps'
// args.dev: the session's working branch (each session is given its own; see docs/project/continuation.md §4)
const DEV = A.dev || 'claude/tender-planck-bpibcq'
if (!A || !A.trailer) throw new Error('args.trailer is required: the commit trailer lines of the current session (see docs/project/continuation.md)')
const TRAILER = A.trailer

const loc = (t) => (A.mode === 'main' ? REPO : `/home/user/wt/${t.id}`)
const branch = (t) => (A.mode === 'main' ? DEV : `wip/${t.id}`)

const WHERE = (t) => A.mode === 'main'
  ? `Work directly in ${REPO} on branch ${DEV} (already checked out). Other agents may be editing files under docs/ at the same time; never touch or stage them unless your task owns them.`
  : `Create your git worktree first: \`git -C ${REPO} worktree add /home/user/wt/${t.id} -b wip/${t.id} ${DEV}\` (if /home/user/wt/${t.id} already exists from an earlier attempt, reuse it: check \`git -C /home/user/wt/${t.id} status\` and continue from its state). Work, build and commit only inside /home/user/wt/${t.id}. Other agents work in sibling worktrees at the same time.`

const RULES = `Rules:
- Do all the work yourself. Do not use the Agent or Workflow tools to start other agents.
- Edit only the files your task owns (work-breakdown.md §8 ownership map and your task's "Owns" line). If you must change a frozen interface, follow work-breakdown.md §2.1 (CONTRACTS.md entry, smallest backward-compatible change).
- Stage files explicitly by path. Never run \`git add -A\` or \`git add .\`.
- Do not weaken, skip, ignore or delete tests to get a green run. If an acceptance item cannot be met honestly (it needs a later task, data that does not exist yet, or a blocked network host), leave it visibly failing or \`#[ignore = "reason"]\` with the reason, and report it.
- This machine has 4 cores shared with other agents and builds are slow: iterate with \`cargo build -p <crate>\` / \`cargo test -p <crate>\`, and run the full workspace checks once near the end. sccache may be configured as the rustc wrapper; that is expected.
- When the spec (docs/mechanics/) and the architecture disagree, the spec wins; record the conflict as work-breakdown.md §2.4 says instead of silently choosing.
- Commit your work (do not push) with the message "<task id>: <one-line summary>", a blank line, then exactly these trailer lines:
${TRAILER}`

const IMPL = (t) => `You are implementing task ${t.id} of the warframe-dps project: a local Warframe DPS calculator and build optimizer (Rust workspace, React frontend served by an Axum binary). The project's CLAUDE.md is in your context; its conventions and standing corrections apply.

The plan is fixed in documents. Read, in this order: docs/architecture/work-breakdown.md §2 (rules for every task) and your task's section "### ${t.id} …" (its Depends, Owns, Provides and Acceptance are your contract); the architecture documents and spec sections your task section cites (docs/architecture/*.md, docs/data/pipeline.md, docs/mechanics/00-09); CONTRACTS.md if it exists (interface changes made by earlier tasks). Then inspect the existing code your task builds on.
${t.note ? '\nManager note for this task: ' + t.note + '\n' : ''}
Where to work: ${WHERE(t)}

${RULES}

Implement everything in your task's "Provides" exactly as specified (names, signatures, fields), with tests, and make its Acceptance commands and the Definition of done (work-breakdown.md §2.3) pass. Your final response is the structured output.`

const REVIEW = (t, impl) => `Adversarially review task ${t.id} of the warframe-dps project, as implemented in ${loc(t)} on branch ${branch(t)}${impl && impl.commit ? ' (latest commit ' + impl.commit + ')' : ''}. The project's CLAUDE.md is in your context. Find problems; do not praise.

Read the task's section "### ${t.id} …" in docs/architecture/work-breakdown.md, its §2 rules and §8 ownership map, and the architecture and spec sections the task cites (docs/architecture/*.md, docs/data/pipeline.md, docs/mechanics/00-09). Then check the implementation:
1. Every "Provides" item exists with exactly the specified names, signatures, fields and semantics.
2. Run every Acceptance command and the Definition of done yourself in ${loc(t)} and report the real results.
3. Spec conformance: formulas, constants, units, order of operations, rounding and edge cases match docs/mechanics (cite the section or equation id, e.g. E-12, D-026, 03 §10.4). Every rule implemented cites its spec source.
4. Tests assert meaningful behaviour, cover the edge cases the spec lists for this task, and none are ignored or weakened without a stated reason.
5. Ownership: no files edited outside the task's ownership (check \`git -C ${loc(t)} show --stat\` over the task's commits); no unrelated files committed.
6. Problems that will hurt later tasks: panics or unwrap on external input in library code, non-determinism where the architecture requires determinism (AD-19), item names or /Lotus/ paths in engine/optimizer code, public items without docs, interfaces that later tasks cannot use as described.
The implementer reported: ${JSON.stringify(impl || {}).slice(0, 4000)}
Report only problems you can support with evidence (file:line, command output, spec reference). Severity: critical (wrong results or broken contract), major (missing piece, wrong edge case, failing acceptance), minor. Do not edit any file.`

const FIX = (t, issues) => `You are finishing task ${t.id} of the warframe-dps project in ${loc(t)} (branch ${branch(t)}). The project's CLAUDE.md is in your context. An independent reviewer reported the issues below. Reviewers can be wrong: verify each issue against the code, docs/architecture/work-breakdown.md (task section "### ${t.id} …") and docs/mechanics before changing anything. Fix every issue that holds up; for each rejected issue, say why in your final output. Re-run the task's Acceptance commands and the Definition of done.

${RULES}

Issues:
${JSON.stringify(issues, null, 1)}

Your final response is the structured output (summarise both the original work and your fixes).`

const MERGE = (done) => `You are the integrator for the warframe-dps project in ${REPO} (branch ${DEV} checked out). The project's CLAUDE.md is in your context. These task branches were just completed in sibling worktrees and must be merged into ${DEV}, in this order: ${done.map((d) => d.branch).join(', ')}.
Task reports: ${JSON.stringify(done.map((d) => ({ task: d.task, branch: d.branch, summary: d.summary, left_undone: d.left_undone, contract_changes: d.contract_changes })), null, 1).slice(0, 12000)}

Steps:
1. Check \`git -C ${REPO} status\`. Other agents may have uncommitted edits under docs/; leave those alone and never stage them.
2. Merge each branch with \`git merge --no-ff <branch>\`, using a merge message "Merge <task id>" followed by a blank line and these trailer lines:
${TRAILER}
   Resolve conflicts by the ownership map in docs/architecture/work-breakdown.md §8: the owning task's version wins; generated files (TypeScript types, JSON schema, compat_generated.rs, data/generated) are regenerated with their xtask/datagen command, never hand-merged; Cargo.toml workspace members and dependencies are unioned.
3. Run the full Definition of done (work-breakdown.md §2.3, including the frontend checks if frontend/ exists) on the merged tree. Fix integration breakages with the smallest correct change and commit them as "Integrate <task ids>: <summary>" with the same trailer lines. Do not weaken or ignore tests; if a break needs a real design change, leave it failing and report it.
4. After every branch is merged and the checks have run, remove the worktrees (\`git -C ${REPO} worktree remove --force /home/user/wt/<id>\`) and delete the merged branches (\`git branch -d\`). Do not push.
Do all the work yourself; do not use the Agent or Workflow tools. Your final response is the structured output.`

const IMPL_SCHEMA = {
  type: 'object',
  properties: {
    task: { type: 'string' },
    location: { type: 'string' },
    branch: { type: 'string' },
    commit: { type: 'string' },
    acceptance: { type: 'array', items: { type: 'object', properties: { command: { type: 'string' }, passed: { type: 'boolean' }, note: { type: 'string' } }, required: ['command', 'passed'] } },
    summary: { type: 'string' },
    contract_changes: { type: 'array', items: { type: 'string' } },
    spec_conflicts: { type: 'array', items: { type: 'string' } },
    left_undone: { type: 'array', items: { type: 'string' } },
  },
  required: ['task', 'branch', 'commit', 'acceptance', 'summary', 'left_undone'],
}
const ISSUES_SCHEMA = {
  type: 'object',
  properties: {
    issues: { type: 'array', items: { type: 'object', properties: {
      severity: { type: 'string', enum: ['critical', 'major', 'minor'] },
      location: { type: 'string' }, problem: { type: 'string' }, evidence: { type: 'string' }, fix: { type: 'string' },
    }, required: ['severity', 'location', 'problem', 'evidence', 'fix'] } },
    acceptance_results: { type: 'string' },
    overall: { type: 'string' },
  },
  required: ['issues', 'overall'],
}
const MERGE_SCHEMA = {
  type: 'object',
  properties: {
    merged: { type: 'array', items: { type: 'string' } },
    not_merged: { type: 'array', items: { type: 'string' } },
    head_commit: { type: 'string' },
    checks: { type: 'array', items: { type: 'object', properties: { command: { type: 'string' }, passed: { type: 'boolean' }, note: { type: 'string' } }, required: ['command', 'passed'] } },
    integration_fixes: { type: 'array', items: { type: 'string' } },
    problems: { type: 'array', items: { type: 'string' } },
  },
  required: ['merged', 'not_merged', 'head_commit', 'checks', 'problems'],
}

// args.merge_only: true merges existing branches wip/<id> of A.tasks (each { id, summary?, left_undone?, contract_changes? })
// without implementing anything; use it after several runs with merge: false, so one integrator merges them all.
const results = A.merge_only ? A.tasks.map((t) => ({ task: t.id, branch: `wip/${t.id}`, summary: t.summary || '', left_undone: t.left_undone || [], contract_changes: t.contract_changes || [] })) : await pipeline(
  A.tasks,
  (t) => agent(IMPL(t), { label: `impl:${t.id}`, phase: 'Implement', effort: t.effort, schema: IMPL_SCHEMA }),
  async (impl, t) => {
    if (!impl) { log(`${t.id}: implementer returned nothing`); return null }
    if (!t.review) return { impl, issues: [] }
    const rev = await agent(REVIEW(t, impl), { label: `review:${t.id}`, phase: 'Review', effort: t.effort, schema: ISSUES_SCHEMA })
    const issues = (rev && rev.issues) || []
    log(`${t.id}: ${issues.length} review issues (${issues.filter((i) => i.severity === 'critical').length} critical, ${issues.filter((i) => i.severity === 'major').length} major)`)
    return { impl, issues, reviewOverall: rev && rev.overall }
  },
  async (r, t) => {
    if (!r) return null
    const serious = r.issues.filter((i) => i.severity !== 'minor' || r.issues.length <= 12)
    if (!serious.length) return { task: t.id, ...r.impl, issueCount: 0 }
    const fixed = await agent(FIX(t, r.issues), { label: `fix:${t.id}`, phase: 'Fix', effort: t.effort, schema: IMPL_SCHEMA })
    return { task: t.id, ...(fixed || r.impl), issueCount: r.issues.length, reviewOverall: r.reviewOverall }
  },
)

const done = results.filter(Boolean)
const missing = A.tasks.filter((t) => !done.find((d) => d.task === t.id)).map((t) => t.id)
if (missing.length) log(`No result from: ${missing.join(', ')}`)

let merge = null
if ((A.merge || A.merge_only) && A.mode === 'worktree' && done.length) {
  phase('Merge')
  const ordered = A.tasks.map((t) => done.find((d) => d.task === t.id)).filter(Boolean).map((d) => ({ ...d, branch: d.branch || `wip/${d.task}` }))
  merge = await agent(MERGE(ordered), { label: 'merge', phase: 'Merge', effort: 'high', schema: MERGE_SCHEMA })
}

return {
  tasks: done.map((d) => ({ task: d.task, commit: d.commit, issueCount: d.issueCount, summary: d.summary, acceptance: d.acceptance, left_undone: d.left_undone, contract_changes: d.contract_changes, spec_conflicts: d.spec_conflicts })),
  missing,
  merge,
}
