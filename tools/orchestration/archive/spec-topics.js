export const meta = {
  name: 'spec-topics',
  description: 'Write, adversarially verify and revise damage-spec topic documents (xhigh)',
  phases: [
    { title: 'Write', detail: 'one xhigh writer per topic document' },
    { title: 'Verify', detail: 'two xhigh verifiers per document: source fidelity, completeness and math' },
    { title: 'Revise', detail: 'one xhigh reviser per document applies confirmed issues' },
  ],
}

// args: array of topic objects {key, file, title, scope, inputs: [findings file names], focus}
const TOPICS = args

const COMMON = `You are working on the damage-calculation spec of a local Warframe DPS calculator and build optimizer (Rust engine, browser UI) in /home/user/warframe-dps. The project's CLAUDE.md is already in your context; follow its conventions. The spec lives in docs/mechanics/. docs/mechanics/00-overview.md is the master document: it fixes notation, symbol identifiers, units, the master formula, the two evaluators (expected-value and Monte Carlo), the discrepancy policy and register (D-xxx IDs), user toggles, and the scope boundary of every topic document. Read it fully first and follow it exactly; use its symbols and IDs. Other agents are writing the other topic documents at the same time, so stay inside your document's scope and reference the owning document for anything outside it.

Research inputs live in research/findings/ (game-truth research from wiki.warframe.com in wiki-*.md, reverse-engineering of the warframe-damage.com calculator "the inspiration" in inspiration-*.md, data-source evaluations in data-source-*.md). The inspiration's code is research/external/inspiration/min.pretty.js. You can fetch wiki pages (https://wiki.warframe.com/w/PAGE?action=raw; check research/external/cache/ first, and cache new downloads there) and run python3 to check arithmetic. warframe.fandom.com is blocked.

Do all work yourself; do not use the Agent or Workflow tools. Do not commit.`

const WRITER = (t) => `${COMMON}

Write ${t.file} (${t.title}).
Scope (from the overview's topic map): ${t.scope}
Primary research inputs (read all of them fully): ${t.inputs.join(', ')}. Also search the other findings files for anything in your scope; several topics are touched by more than one file.
${t.focus ? 'Pay particular attention to: ' + t.focus : ''}

The document is what the Rust engine will be implemented and unit-tested from, so it must be exact and complete:
- First line "Status: draft (spec)". Then a short scope statement and links to related documents.
- Every formula written precisely (fenced pseudocode or math) using the overview's symbols, with its source cited inline (findings file and section, plus the underlying wiki page or min.pretty.js line range). Give the game update a rule came from when known.
- Explicit order of operations and rounding/quantization rules wherever they matter.
- An "Edge cases" section listing every exception, weapon-specific behaviour and special case from the research that falls in your scope, each with ID (e.g. EC-${t.key}-01), the rule, and the source. Missing an edge case that the research documented is the most likely way this document fails, so go through every findings file's edge-case and quirk lists and account for each item in your scope: either specify it, or list it as explicitly out of scope with the reason.
- An "Inspiration differences" section: for each place the inspiration's behaviour differs from the game in your scope, reference the overview's D-xxx entry (or propose a new one, numbered D-${t.key}-xx, if the overview missed it) and state what the engine does.
- A "Test vectors" section: at least 6 worked numeric examples covering normal and edge cases, with inputs and expected outputs, computed and checked with python3. These become the engine's unit tests, so give enough inputs to reproduce each one exactly.
- An "Open questions" section: anything UNVERIFIED, with the evidence that would settle it.
Do not guess. If sources are silent or contradict each other, say so, pick the best-supported behaviour, mark it UNVERIFIED, and explain the choice.
Edit only ${t.file}.

Final response: a plain-text report under 200 words: what the document covers, the number of edge cases and test vectors, new discrepancies proposed, and open questions.`

const VERIFY_SOURCES = (t) => `${COMMON}

Adversarially verify ${t.file} (${t.title}) for SOURCE FIDELITY. Another agent just wrote it from the research findings. Your job is to find errors, not to praise it. For every formula, number, rule and claim in the document: check it against the cited findings file AND against the primary source behind it (re-read the wiki page wikitext, or the cited min.pretty.js lines). Look for: misquoted numbers, wrong constants, formulas whose structure differs from the source (additive written as multiplicative, wrong order of operations, wrong base for a percentage), citations that do not support the claim, outdated rules presented as current, claims attributed to the game that actually come from the inspiration, and inspiration behaviour described incorrectly. Recompute every test vector with python3 and report any that are wrong or not reproducible from the stated inputs.
Report only problems you can support with evidence. For each, give the exact location in the document, what it says, what the source says (quote it, with page/section or line numbers), and the fix. Use severity critical (would make computed damage wrong in common builds), major (wrong in specific builds or edge cases, or a broken test vector), minor (wording, citation, clarity). If the document is correct in some area, say nothing about it. Do not edit any file.`

const VERIFY_COMPLETE = (t) => `${COMMON}

Adversarially verify ${t.file} (${t.title}) for COMPLETENESS, CONSISTENCY and IMPLEMENTABILITY. Another agent just wrote it from the research findings. Your job is to find what is missing or unusable, not to praise it. Check:
1. Completeness: go through every findings file in research/findings/ (especially ${t.inputs.join(', ')}) and list every mechanic, edge case, weapon-specific rule, quirk or discrepancy that falls inside this document's scope (scope: ${t.scope}) but is absent from the document or only mentioned vaguely. Also flag mechanics in scope that a thorough Warframe DPS calculator needs but the research itself missed, if you can source them from the wiki.
2. Consistency with docs/mechanics/00-overview.md: symbols, units, bucket names, D-xxx decisions, toggle definitions, evaluator semantics, scope boundaries. Flag contradictions.
3. Implementability: could a Rust developer implement every rule without guessing? Flag ambiguous order of operations, undefined symbols, missing rounding rules, unstated defaults, rules that need data the data sources do not provide (see data-source-*.md) without saying where the data comes from.
4. Test vectors: are normal and edge cases covered? Name important cases with no test vector.
For each problem give the location (or "missing"), the issue, the evidence (findings file and section, or wiki page), and the fix, with severity critical / major / minor as defined: critical = would make computed damage wrong in common builds; major = wrong in specific builds or edge cases, or blocks implementation; minor = clarity. Do not edit any file.`

const REVISER = (t, issues) => `${COMMON}

Revise ${t.file} (${t.title}). Two independent verifiers reviewed the draft. Their reported issues are below as JSON. Verifiers can be wrong too: for each issue, check the evidence yourself against the sources before changing anything. Apply every issue whose evidence holds up; for an issue you reject, add a one-line note to a "Review notes" section at the end of the document saying why. Keep the document's structure, symbols and IDs consistent with docs/mechanics/00-overview.md. Recheck every test vector you add or change with python3. When done, change the first line to "Status: reviewed (spec)". Edit only ${t.file}.

Issues:
${JSON.stringify(issues, null, 1)}

Final response: a plain-text report under 200 words: issues applied, issues rejected and why, anything still open.`

const ISSUES_SCHEMA = {
  type: 'object',
  properties: {
    issues: {
      type: 'array',
      items: {
        type: 'object',
        properties: {
          severity: { type: 'string', enum: ['critical', 'major', 'minor'] },
          location: { type: 'string' },
          problem: { type: 'string' },
          evidence: { type: 'string' },
          fix: { type: 'string' },
        },
        required: ['severity', 'location', 'problem', 'evidence', 'fix'],
      },
    },
    overall: { type: 'string' },
  },
  required: ['issues', 'overall'],
}

const results = await pipeline(
  TOPICS,
  (t) => agent(WRITER(t), { label: `write:${t.key}`, phase: 'Write', effort: 'xhigh' }),
  async (writeReport, t) => {
    const [a, b] = await parallel([
      () => agent(VERIFY_SOURCES(t), { label: `verify-src:${t.key}`, phase: 'Verify', effort: 'xhigh', schema: ISSUES_SCHEMA }),
      () => agent(VERIFY_COMPLETE(t), { label: `verify-complete:${t.key}`, phase: 'Verify', effort: 'xhigh', schema: ISSUES_SCHEMA }),
    ])
    const issues = [
      ...((a && a.issues) || []).map((i) => ({ ...i, lens: 'source-fidelity' })),
      ...((b && b.issues) || []).map((i) => ({ ...i, lens: 'completeness' })),
    ]
    if (!a || !b) log(`${t.key}: a verifier returned nothing`)
    log(`${t.key}: ${issues.length} issues (${issues.filter((i) => i.severity === 'critical').length} critical)`)
    return { writeReport, issues }
  },
  async (v, t) => {
    const reviseReport = await agent(REVISER(t, v.issues), { label: `revise:${t.key}`, phase: 'Revise', effort: 'xhigh' })
    return {
      key: t.key,
      file: t.file,
      writeReport: v.writeReport,
      issueCounts: {
        critical: v.issues.filter((i) => i.severity === 'critical').length,
        major: v.issues.filter((i) => i.severity === 'major').length,
        minor: v.issues.filter((i) => i.severity === 'minor').length,
      },
      reviseReport,
    }
  },
)
return results.filter(Boolean)
