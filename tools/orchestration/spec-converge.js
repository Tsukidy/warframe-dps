export const meta = {
  name: 'spec-converge',
  description: 'Settle the residual cross-document items of the spec and loop recheck/fix until no issues remain (xhigh)',
  phases: [
    { title: 'Triage', detail: 'check every residual item against the current documents' },
    { title: 'Fix', detail: 'one xhigh fixer per document with confirmed items' },
    { title: 'Recheck', detail: 'verify the fixes landed and introduced no new contradiction; loop' },
  ],
}

// args: { input: absolute path of a file listing the residual items (JSON or Markdown), wiki_scratch?: extra directory of cached wiki pages, note?: manager note appended to every prompt }
if (!args || !args.input) throw new Error('args.input is required')
const ROUND2 = args.input
const WIKI_SCRATCH = args.wiki_scratch ? ` and ${args.wiki_scratch}` : ''
const NOTE = args.note ? `\n\nManager note: ${args.note}` : ''
const DOCS = ['00-overview', '01-damage-composition', '02-crit-status-multishot', '03-status-effects', '04-conditional-bonuses-and-buffs', '05-enemies', '06-ranged-firing-model', '07-melee', '08-evaluation-and-outputs', '09-build-rules']
const MAX_ROUNDS = 3

const COMMON = `The damage-calculation spec of the warframe-dps project (a local Warframe DPS calculator and build optimizer; repository /home/user/warframe-dps, all relative paths below are inside it) is in docs/mechanics/: 00-overview.md is the master (notation, master formula, evaluators, discrepancy register D-xxx, toggles T-xx, ownership map in §7) and 01-09 are topic documents. The project's CLAUDE.md is in your context. A Rust engine will be implemented directly from these documents, so two documents that disagree become a bug.

The spec has been through several review rounds in which agents edited documents in parallel. Each round left a tail of items of the form "document X must still change because document Y changed". This workflow settles that tail. Sources for game behaviour: research/findings/, research/external/inspiration/min.pretty.js, wiki pages via https://wiki.warframe.com/w/PAGE?action=raw (check research/external/cache/${WIKI_SCRATCH} first). Each document's "Review notes" section records earlier decisions and why; read the relevant notes before re-deciding anything.
Do all work yourself; do not use the Agent or Workflow tools. Do not commit.${NOTE}`

const ISSUES_SCHEMA = {
  type: 'object',
  properties: {
    issues: { type: 'array', items: { type: 'object', properties: {
      doc: { type: 'string', enum: DOCS },
      severity: { type: 'string', enum: ['critical', 'major', 'minor'] },
      location: { type: 'string' }, problem: { type: 'string' }, evidence: { type: 'string' }, fix: { type: 'string' },
    }, required: ['doc', 'severity', 'location', 'problem', 'evidence', 'fix'] } },
    already_resolved: { type: 'array', items: { type: 'string' } },
    overall: { type: 'string' },
  },
  required: ['issues', 'overall'],
}

const RULES_FOR_ISSUES = `Report an issue only when you have checked the current text of every document involved and the problem is still there. For each issue: name the single document that must change ("doc"), quote the current text, give the evidence (document and section or id, or the game source), and write the fix precisely enough that the fixer can apply it without changing any other document: state the final wording or formula, not "align with X". Where two documents disagree, decide which one is right from the evidence and 00 §7 ownership (the owner of a mechanic wins unless the evidence says otherwise); where the evidence is balanced, follow the owner and mark the point UNVERIFIED. Severity: critical (wrong damage in common builds, or two documents prescribe incompatible behaviour), major (wrong in specific builds or edge cases, or blocks implementation), minor (stale cross-reference, wording, citation). List items you found already resolved in "already_resolved" (one line each).`

const TRIAGE = `${COMMON}

Your task: triage the residual items. Read ${ROUND2}, the result of the last review round. Collect every item it contains that says a document must still change ("other documents must still change", "still needed elsewhere", open issues, unfixed minor issues). Many of them were handled by another agent in the same round, so check each one against the current text of the documents. Do not edit any file.

${RULES_FOR_ISSUES}`

const RECHECK = (fixReports) => `${COMMON}

Your task: recheck after a fix round. The fixers' reports are below. For every change a fixer reports, check that it landed and that it does not contradict any other document (read the sections of the other documents that touch the same quantity). Also handle every "other document must change" item in the reports. Recompute with python3 any test vector a fixer added or changed. Do not edit any file.

${RULES_FOR_ISSUES}

Fixer reports:
${JSON.stringify(fixReports, null, 1)}`

const FIX = (doc, issues) => `${COMMON}

Revise docs/mechanics/${doc}.md. A checker confirmed the issues below against the current documents. Checkers can be wrong: verify each issue's evidence before changing anything. Apply every issue that holds up; for a rejected issue add a one-line note to the "Review notes" section saying why. Make each fix complete within this document: do not create new work for other documents. If you find that a correct fix needs another document to change too, still apply your side and name the other document's exact required change in your report. Recompute with python3 any test vector you add or change. Keep the first line "Status: reviewed (spec)" and keep IDs stable (new ids go at the end of their series). Edit only docs/mechanics/${doc}.md; other agents are revising other documents at the same time.

Issues:
${JSON.stringify(issues, null, 1)}

Final response: a plain-text report under 150 words: applied, rejected (why), and the exact change any other document still needs.`

const history = []
// args.issues: an array of already-confirmed issues (ISSUES_SCHEMA items, e.g. the open list of a previous run); skips the triage
phase('Triage')
let check = Array.isArray(args.issues)
  ? { issues: args.issues, overall: 'issues passed in by the manager (confirmed by the previous recheck)' }
  : await agent(TRIAGE, { label: 'triage', phase: 'Triage', effort: 'xhigh', schema: ISSUES_SCHEMA })
for (let round = 1; round <= MAX_ROUNDS; round++) {
  const issues = (check && check.issues) || []
  log(`round ${round}: ${issues.length} open issues (${issues.filter((i) => i.severity !== 'minor').length} critical/major)`)
  history.push({ round, issues: issues.length, overall: check && check.overall })
  if (!issues.length) break
  if (round === MAX_ROUNDS) { log(`stopping after ${MAX_ROUNDS} rounds with ${issues.length} open issues`); break }
  const byDoc = DOCS.map((d) => ({ doc: d, issues: issues.filter((x) => x.doc === d) })).filter((g) => g.issues.length)
  const reports = await parallel(byDoc.map((g) => () => agent(FIX(g.doc, g.issues), { label: `fix${round}:${g.doc}`, phase: 'Fix', effort: 'xhigh' })))
  const fixReports = byDoc.map((g, i) => ({ doc: g.doc, report: reports[i] }))
  history[history.length - 1].fixes = fixReports
  check = await agent(RECHECK(fixReports), { label: `recheck${round}`, phase: 'Recheck', effort: 'xhigh', schema: ISSUES_SCHEMA })
}

return { history, open: (check && check.issues) || [] }
