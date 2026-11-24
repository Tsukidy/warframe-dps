export const meta = {
  name: 'architecture',
  description: 'Design the system architecture: two independent proposals, a synthesis into docs/architecture, adversarial review, revision',
  phases: [
    { title: 'Propose', detail: '2 independent architecture proposals with different priorities' },
    { title: 'Synthesize', detail: 'judge picks and merges into docs/architecture/*.md and docs/data/pipeline.md' },
    { title: 'Review', detail: 'parity and spec-coverage review' },
    { title: 'Revise', detail: 'apply confirmed review issues' },
  ],
}

const CONTEXT = `Project: a local Warframe DPS calculator with an automated build optimizer, in /home/user/warframe-dps (currently only docs and research; no code yet). The project's CLAUDE.md is in your context: note the user's decisions there (local web app: one Rust binary with Axum serving an API and the built frontend on localhost and opening the browser; English only but every UI string through an i18n layer; optimizer runs locally in Rust using all cores; server-only inspiration features out of scope). The backend must be Rust. The frontend can be whatever is best; it must look clearly better than warframe-damage.com ("the inspiration") and at least match all of its features.

Inputs to read:
- docs/mechanics/00-overview.md: the master damage spec (notation, master formula, the two evaluators: a deterministic expected-value evaluator for the optimizer and an event-driven seeded Monte Carlo simulator, discrepancy register, toggles, topic map). Topic documents docs/mechanics/01-09 are being written in parallel right now; read any that exist, but rely on the overview and the research findings for what they will contain.
- research/findings/inspiration-feature-inventory.md (every feature to match, with a parity checklist), inspiration-data-model.md (the inspiration's mod action keys, weapon/attack/enemy fields, slots), inspiration-optimizer.md (how its optimizer works and its weaknesses, plus a proposal for a Rust optimizer), inspiration-dps-aggregation.md (outputs).
- research/findings/data-source-wfcd.md, data-source-wiki-modules.md, data-source-public-export-and-connector.md: what each data source provides and lacks. Key conclusions: wiki Lua modules (parsed with the mlua crate, vendored Lua 5.1; a proof of concept is in research/external/cache/poc/lua2json/) are the best source for weapon attacks, enemies, stances and faction damage tables; WFCD (MIT) for the catalogue, images and i18n names; the Public Export for canonical uniqueName keys and freshness. Mod, arcane, Incarnon-evolution and archon-shard effects have no machine-readable source and need a curated effects table. Wiki content is CC BY-NC-SA 3.0.
- The wiki-*.md findings for the range of mechanics the effect system must express (buckets, conditional stacks, per-attack applicability, extra hits, CO adding vs multiplying per attack).
Toolchain available here: Rust stable 1.97 with cargo (crates.io reachable), Node 22 with npm, Python 3, Playwright with Chromium.`

const PROPOSAL = (angle) => `${CONTEXT}

Write an architecture proposal for the whole system. Your guiding priority: ${angle}
Write it to research/findings/architecture-proposal-${angle.split(':')[0].trim().toLowerCase().replace(/[^a-z]+/g, '-')}.md (first line "Status: draft (proposal)"). Another architect is writing a competing proposal with a different priority; a judge will merge the best of both, so argue for your choices concretely.

Cover, with concrete Rust types, file layouts and API shapes rather than generalities:
1. Cargo workspace layout (crates and their responsibilities and dependencies), and where the frontend lives.
2. The data pipeline: a Rust build tool that fetches and caches the sources, parses wiki Lua modules, joins sources on uniqueName, applies curated effect tables, validates, and writes a versioned data bundle the app loads (format, size, how it is embedded or shipped, how the user refreshes data after a game update, licence attribution). Curated tables: file format and layout, how a curator adds a mod's effects, and how they are validated against the source text.
3. The domain model: weapons, attacks (multi-part attacks, Incarnon forms, kitgun/zaw assembly), mods, arcanes, archon shards, Incarnon evolutions, rivens, enemies, factions, buffs, builds. Above all the effect/modifier encoding: show how it expresses every bucket and mechanic in the spec (additive base damage, elemental with slot order, faction, separate multipliers, crit chance relative/absolute, crit damage, status, multishot, fire rate, Condition-Overload family in adding vs multiplying form per attack, conditional stacks with max/ramp, per-attack and per-weapon-class applicability, extra hits, target debuffs, set bonuses, toggles such as "trigger all conditions"). Give encoded examples for at least 12 diverse mods/buffs (Serration, Galvanized Aptitude, Condition Overload, Primary Deadhead, Hunter Munitions, a riven, Roar, Xata's Whisper, a Violet archon shard, an Incarnon evolution perk, Blood Rush, Arcane Avenger).
4. The engine crate: public API, internal pipeline (build -> resolved per-attack stats -> per-hit model -> expected-value evaluator and Monte Carlo simulator -> outputs), how the spec's test vectors become unit tests, numeric types, determinism, and performance targets (builds per second per core for the evaluator).
5. The optimizer: search algorithm (consider beam search, local search with swaps, element-order search, exhaustive pruning where possible), use of all cores, scoring (expected-value evaluator for search, Monte Carlo confirmation of finalists), constraints (locked slots, incompatibilities, optional capacity/polarity, allowed buffs, rivens, arcanes, exilus, Incarnon tiers), progress reporting and cancellation, and how it beats the inspiration's weaknesses.
6. The server: Axum routes (REST + server-sent events or WebSocket for optimizer progress), request/response types, static frontend embedding, opening the browser, config, logging.
7. The frontend: framework and libraries (justify), page layout and component tree, state management, how it reaches feature parity with every item in the inspiration's parity checklist, i18n approach, build sharing (encoded build strings; also importing a warframe-damage.com build link), visual design direction (it must look clearly better than the inspiration), and accessibility.
8. Testing strategy: unit tests from spec test vectors, data-pipeline validation, golden tests against the inspiration oracle (research/findings/oracle-feasibility.md), optimizer tests, frontend end-to-end tests with Playwright.
9. A work breakdown: an ordered list of implementation tasks sized for one agent each, with the interfaces each task must honour so independent tasks can run in parallel, dependencies, and acceptance criteria.

Do all work yourself; do not use the Agent or Workflow tools. Edit only your proposal file. Do not commit. Final response: a plain-text summary under 200 words of your key choices.`

const ANGLES = [
  'Correctness and extensibility: a data-driven effect system where game mechanics live in data and typed rules, minimal special-casing in code, easy to update after game patches, and every spec mechanic expressible.',
  'Optimizer performance: the expected-value evaluator must score very large numbers of candidate builds fast on all cores (compile builds into flat stat vectors, precompute per-mod contributions, incremental re-evaluation), without sacrificing the accuracy the spec demands.',
]

phase('Propose')
const proposals = await parallel(ANGLES.map((a, i) => () =>
  agent(PROPOSAL(a), { label: `propose:${i === 0 ? 'extensibility' : 'performance'}`, phase: 'Propose', effort: 'high' })
))

phase('Synthesize')
const SYNTH = `${CONTEXT}

Two architects wrote competing architecture proposals for this project with different priorities (correctness/extensibility vs optimizer performance). They are in research/findings/architecture-proposal-*.md. Their summaries:
${proposals.filter(Boolean).map((p, i) => `Proposal ${i + 1}: ${p}`).join('\n')}

Judge them and write the final architecture. Evaluate each proposal on: can the effect encoding express every mechanic in docs/mechanics/00-overview.md and the wiki-*.md findings; will it reach feature parity with every row of the inspiration's parity checklist; optimizer speed and result quality; simplicity and how easy it is to update after game patches; testability; risk. Pick the stronger base and graft in the better ideas from the other. Where both miss something, design it yourself.

Write these files (create the directories), each starting with "Status: draft (architecture)":
- docs/architecture/overview.md: system diagram (text), workspace and crate layout, data flow, key decisions with rationale (record the judging outcome briefly).
- docs/architecture/domain-model.md: the Rust domain types and the complete effect/modifier encoding with encoded examples (at least the 12 from the proposals' brief), and how toggles and conditions are represented.
- docs/data/pipeline.md: sources, fetch/caching etiquette, parsing (mlua), joining, curated tables format and workflow, validation, bundle format, refresh command, licence attribution.
- docs/architecture/engine.md: engine API, internal stages, both evaluators, determinism, performance targets, how spec test vectors map to tests.
- docs/architecture/optimizer.md: algorithm, parallelism, scoring, constraints, progress and cancellation.
- docs/architecture/server-api.md: every route with request/response JSON shapes, SSE/WebSocket events, static serving, startup behaviour.
- docs/architecture/frontend.md: stack, layout, component tree, state, a feature-parity mapping table (every row of the inspiration's parity checklist -> where our UI provides it), i18n, build sharing and inspiration-link import, visual design direction, accessibility.
- docs/architecture/testing.md: test layers and tools, oracle golden tests.
- docs/architecture/work-breakdown.md: ordered implementation tasks, one agent each, with exact interfaces/contracts (file paths, public types and function signatures, JSON schemas) so independent tasks can run in parallel without conflicts, dependencies between tasks, and acceptance criteria (commands that must pass).
Do all work yourself; do not use the Agent or Workflow tools. Edit only these files. Do not commit. Final response: a plain-text summary under 250 words of the final architecture and the work breakdown order.`
const synth = await agent(SYNTH, { label: 'synthesize', phase: 'Synthesize', effort: 'high' })

phase('Review')
const REVIEW = `${CONTEXT}

Adversarially review the architecture in docs/architecture/*.md and docs/data/pipeline.md (written by another agent from two proposals in research/findings/architecture-proposal-*.md). Find problems; do not praise. Check:
1. Feature parity: go through every row of the parity checklist in research/findings/inspiration-feature-inventory.md and every feature in its body; flag each feature with no clear home in the architecture.
2. Spec coverage: go through docs/mechanics/00-overview.md (and any docs/mechanics/0*.md that exist) and the wiki-*.md findings' edge-case lists; flag mechanics the domain model or effect encoding cannot express, or can express only by special-casing that is not planned.
3. Data: flag data the engine needs that the planned pipeline cannot provide according to data-source-*.md, unless a curated source is planned for it.
4. Implementability of the work breakdown: missing tasks, circular dependencies, interfaces too vague for parallel agents, acceptance criteria that cannot be checked, tasks too large for one agent.
5. Technical risks: crate choices that will not build here (Rust 1.97, offline-unfriendly steps), performance targets that the design cannot meet, frontend choices that complicate embedding in the Rust binary.
For each problem: location, problem, evidence, fix, severity (critical = blocks parity or correctness; major = significant rework later; minor). Do not edit any file.`
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
const review = await agent(REVIEW, { label: 'review', phase: 'Review', effort: 'high', schema: ISSUES_SCHEMA })

phase('Revise')
const REVISE = `${CONTEXT}

Revise the architecture documents (docs/architecture/*.md and docs/data/pipeline.md). An independent reviewer reported the issues below. Check each one's evidence yourself; apply every issue that holds up, and for each rejected issue add a one-line note under a "Review notes" heading in docs/architecture/overview.md saying why. Keep all documents consistent with each other, especially the interfaces in work-breakdown.md. Change each file's first line to "Status: reviewed (architecture)". Edit only those files. Do all work yourself; do not use the Agent or Workflow tools. Do not commit.

Issues:
${JSON.stringify((review && review.issues) || [], null, 1)}

Final response: a plain-text summary under 250 words: changes made, issues rejected, and the final work-breakdown order with task names.`
const revised = await agent(REVISE, { label: 'revise', phase: 'Revise', effort: 'high' })

return {
  synth,
  reviewCounts: review ? {
    critical: review.issues.filter((i) => i.severity === 'critical').length,
    major: review.issues.filter((i) => i.severity === 'major').length,
    minor: review.issues.filter((i) => i.severity === 'minor').length,
  } : null,
  reviewOverall: review && review.overall,
  revised,
}
