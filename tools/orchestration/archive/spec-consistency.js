export const meta = {
  name: 'spec-consistency',
  description: 'Whole-spec critique (overview correctness, cross-document consistency, edge-case completeness) and per-document fixes (xhigh)',
  phases: [
    { title: 'Critique', detail: '3 xhigh critics with distinct lenses over docs/mechanics/00-09' },
    { title: 'Fix', detail: 'one xhigh fixer per document that has issues' },
  ],
}

const DOCS = ['00-overview', '01-damage-composition', '02-crit-status-multishot', '03-status-effects', '04-conditional-bonuses-and-buffs', '05-enemies', '06-ranged-firing-model', '07-melee', '08-evaluation-and-outputs', '09-build-rules']

const COMMON = `The damage-calculation spec of the warframe-dps project (a local Warframe DPS calculator and build optimizer) is complete in docs/mechanics/: 00-overview.md (master: notation, master formula, evaluators, discrepancy register D-xxx, toggles T-xx, topic map) and topic documents 01-09, each written, adversarially verified and revised separately. The project's CLAUDE.md is in your context. Research inputs are in research/findings/ (wiki-*.md game-truth research, inspiration-*.md reverse-engineering of warframe-damage.com, data-source-*.md); the inspiration's code is research/external/inspiration/min.pretty.js; wiki pages can be fetched with ?action=raw (check research/external/cache/ first). Implementation of a Rust engine from this spec has started, so every inconsistency you find now prevents a bug later.
Do all work yourself; do not use the Agent or Workflow tools. Do not edit any file in this step. Report only problems you can support with evidence (document, section or equation id, quote). Use severity critical (would make computed damage wrong in common builds, or two documents prescribe incompatible behaviour), major (wrong in specific builds or edge cases, or blocks implementation), minor (clarity, citation, dangling reference). For each issue, name the document that must change (one of: ${DOCS.join(', ')}) in the "doc" field.`

const LENSES = [
  { key: 'overview', prompt: `${COMMON}

Your lens: the correctness of 00-overview.md. It was never independently verified, and the topic documents were revised after it was written. Check every formula, constant, symbol definition, the worked example in §3.8 (recompute it with python3), the evaluator definitions in §4, every register row in §5.2 (is the game behaviour stated correctly per its source? does the topic document that owns it agree?), the toggle definitions in §6 and the topic map in §7. Where a topic document corrected or refined something, the overview must follow it (or the topic document is wrong: decide from the sources).
The topic documents also raise explicit proposals addressed to "the manager" to change 00 (for example 02 §8.1's list of deviations from 00, 03 OQ-301 moving the crit multiplier and body part into the DoT seed, 03 OQ-322/-323/-327/-333/-335, 05's rewording of toggle T-28, 06 OQ-620 and OQ-622, 08 OQ-804, and any other "proposal", "request to the manager" or "R-" item you find in 01-09). The manager's ruling: each is decided on the evidence. For every such proposal, report an issue against the document that must change (usually 00-overview) stating the decision and the evidence, so the fixer applies it. Where the evidence is genuinely balanced, prefer the topic document (it was researched and verified in more depth) and mark the point UNVERIFIED.` },
  { key: 'consistency', prompt: `${COMMON}

Your lens: consistency across documents. Check that symbols, units, equation ids (E-n, E-n.m), register ids (D-xxx and per-document D-<nn>-xx), toggle ids, test-vector ids and cross-references agree between documents; that no two documents give different formulas, constants or orders of operations for the same thing (e.g. faction application on DoTs, CO adding vs multiplying form, crit tier math, Heat strip ramp, Viral/Magnetic multipliers, status caps, quantization, layer routing, how the EV evaluator takes expectations); that every cross-reference points to something that exists; that ownership follows 00-overview.md §7 (each mechanic defined once, consumed elsewhere by reference); and that test vectors in different documents that touch the same quantities agree numerically.` },
  { key: 'completeness', prompt: `${COMMON}

Your lens: edge-case completeness of the whole spec. Go through every edge-case, quirk and open-question list in every research/findings/*.md file (wiki-*, inspiration-*) and every discrepancy the research found, and check that each one is either specified in some docs/mechanics document, listed there as explicitly out of scope with a reason, or recorded as an open question. The user specifically asked that edge cases not be missed, giving as an example Condition Overload being a multiplier with other bonuses on some weapons but additive on others; check that one with particular care (per-attack catalogue, CO base rules, weapons where CO does nothing, interaction with Incarnon flat damage and final multipliers). Also flag mechanics a thorough Warframe DPS calculator needs that the research and the spec both missed, if you can source them from the wiki.` },
]

const ISSUES_SCHEMA = {
  type: 'object',
  properties: {
    issues: { type: 'array', items: { type: 'object', properties: {
      doc: { type: 'string', enum: DOCS },
      severity: { type: 'string', enum: ['critical', 'major', 'minor'] },
      location: { type: 'string' }, problem: { type: 'string' }, evidence: { type: 'string' }, fix: { type: 'string' },
    }, required: ['doc', 'severity', 'location', 'problem', 'evidence', 'fix'] } },
    overall: { type: 'string' },
  },
  required: ['issues', 'overall'],
}

phase('Critique')
const crit = await parallel(LENSES.map((l) => () =>
  agent(l.prompt, { label: `critic:${l.key}`, phase: 'Critique', effort: 'xhigh', schema: ISSUES_SCHEMA })))
const all = crit.filter(Boolean).flatMap((c, i) => c.issues.map((x) => ({ ...x, lens: LENSES[i].key })))
log(`${all.length} issues: ${all.filter((x) => x.severity === 'critical').length} critical, ${all.filter((x) => x.severity === 'major').length} major`)

phase('Fix')
const byDoc = DOCS.map((d) => ({ doc: d, issues: all.filter((x) => x.doc === d) })).filter((g) => g.issues.length)
const fixes = await parallel(byDoc.map((g) => () => agent(`${COMMON.replace('Do not edit any file in this step. ', '')}

Revise docs/mechanics/${g.doc}.md. Three independent critics reviewed the whole spec; the issues below concern this document. Critics can be wrong: check each issue's evidence against the sources and the other documents before changing anything. Apply every issue that holds up. Where an issue says another document is right and this one must follow, follow it; where you conclude the other document is wrong instead, do not edit it: say so in your final report. For rejected issues add a one-line note to the document's "Review notes" section. Recheck any test vector you add or change with python3. Keep the first line "Status: reviewed (spec)" (set it if it says draft). Edit only docs/mechanics/${g.doc}.md. Other agents are revising the other documents at the same time.

Issues:
${JSON.stringify(g.issues, null, 1)}

Final response: a plain-text report under 200 words: applied, rejected (why), and any issue you believe belongs to another document.`, { label: `fix:${g.doc}`, phase: 'Fix', effort: 'xhigh' })))

return {
  counts: { total: all.length, critical: all.filter((x) => x.severity === 'critical').length, major: all.filter((x) => x.severity === 'major').length },
  perDoc: byDoc.map((g, i) => ({ doc: g.doc, issues: g.issues.length, report: fixes[i] })),
  overall: crit.filter(Boolean).map((c, i) => ({ lens: LENSES[i].key, overall: c.overall })),
}
