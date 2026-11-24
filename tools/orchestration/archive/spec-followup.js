export const meta = {
  name: 'spec-followup',
  description: 'Apply cross-document follow-ups from the spec consistency pass (overview first, then topic documents), then re-check consistency (xhigh)',
  phases: [
    { title: 'Overview', detail: 'settle the last 00-overview items so topic documents follow a fixed master' },
    { title: 'Topics', detail: 'one xhigh agent per document group applies the follow-ups addressed to it' },
    { title: 'Recheck', detail: 'one xhigh consistency critic over the whole spec' },
    { title: 'Fix', detail: 'fix critical and major issues the recheck found' },
  ],
}

const REPORTS_FILE = '/tmp/claude-0/-home-user/fba0a26e-e0d2-55c2-9445-fb2cf89f09fb/scratchpad/wf/round1-reports.json'
const DOCS = ['00-overview', '01-damage-composition', '02-crit-status-multishot', '03-status-effects', '04-conditional-bonuses-and-buffs', '05-enemies', '06-ranged-firing-model', '07-melee', '08-evaluation-and-outputs', '09-build-rules']

const COMMON = `The damage-calculation spec of the warframe-dps project (a local Warframe DPS calculator and build optimizer; repository /home/user/warframe-dps, all relative paths below are inside it) is in docs/mechanics/: 00-overview.md is the master (notation, master formula, evaluators, discrepancy register D-xxx, toggles T-xx, topic ownership map in §7) and 01-09 are topic documents. The project's CLAUDE.md is in your context. A Rust engine will be implemented directly from these documents, so two documents that disagree become a bug.

What happened so far: three critics reviewed the whole spec and one fixer per document applied the issues, all in parallel. Each fixer could edit only its own document, so each one ended its report with changes that OTHER documents must now make to follow. Those follow-ups have not been applied yet; that is the job of this round. All round-1 fixer reports (the source of the follow-up items) are in ${REPORTS_FILE}, one entry per document; read that file first.

Sources: research/findings/ (wiki-*.md game-truth research, inspiration-*.md reverse-engineering), research/external/inspiration/min.pretty.js, wiki pages via https://wiki.warframe.com/w/PAGE?action=raw (check research/external/cache/ and /tmp/claude-0/-home-user/fba0a26e-e0d2-55c2-9445-fb2cf89f09fb/scratchpad/wiki/ first). Each document's "Review notes" section records what was decided in earlier rounds and why; read it before re-deciding anything.
Do all work yourself; do not use the Agent or Workflow tools. Do not commit.`

const OVERVIEW_ITEMS = `- From 03: 00 §6.3 should describe the custom stat vuln_status_damage as additive inside V_sd (03 kept the additive form on the evidence of JS L205-209, L10467-10469, L10498-10502). Check 00's current text and the evidence, and make 00 and 03 agree.
- From 04: §7.1 should make 08 the only owner of the EV expectation of sequential-pellet / status counts (04 E-4.2 now states only the counting rule).
- From 04: the D-067 example is reversed (abs gives 5.401).
- From 08: the stance formula in §3.3 is written with ρ_co·B_ref, while 04 E-4.6 uses B_co, which also covers fixed rows.
- From 01: §7.2 could add an ownership row for elemental DoT membership (03 §9.3 owns which sources count; 01 E-1.15 sums the flagged contributions).`

const GROUPS = [
  { key: '01-07-09', docs: ['01-damage-composition', '07-melee', '09-build-rules'], items: `01:
- From 00: §4.14 E-11 wording must match 00's revised E-11 (ρ_co / κ, catalogue rows such as Tenet Agendus 20 % and Tenet Detron 65 %).
- From 04: R2 adopts 04's new exclusion rule for uncatalogued conditional augment lines.
07:
- Check 07 against 00's revised text: toggle T-30 (melee plan) and T-31, stance and heavy multipliers S_j and H_j in the base (not in G), the CO ratio for shared stance combos and B_co (04 E-4.6), Melee Animosity per input. 07 was the reference for several of these; confirm 00 and 04 now agree with it and fix 07 only where 07 is the one that is now out of date.
09:
- From 00: replace placeholder toggle names with T-35 (capacity mode), T-36 (lower mod ranks in the optimizer) and T-37 (caster of T-13 … T-16).
- Check every toggle, D-xxx and equation reference in 09 against 00 as it is now.` },
  { key: '02', docs: ['02-crit-status-multishot'], items: `- From 00: drop the §8.1 V-1 … V-8(/V-9) deviation markers: 00 adopted them (check each against 00 as it is now; any deviation 00 did not adopt stays, stated as an open proposal).
- From 03: remove the stale remark(s) 03's report mentions (time model / ragdoll "dormant" wording); add Finisher to §4.7.2 (03 §4.2 now lists it as CC only).
- From 04: adopt 04's exclusion rule for uncatalogued conditional augment lines.
- Check 02 against 00's revised E-7 (Part on body and radial hits, head-crit factor, ΣHS only on heads), E-11 and §4.2 (Hunter Munitions EV form).` },
  { key: '03', docs: ['03-status-effects'], items: `- From 00: E-3.4 and E-3.5 put the crit factor c_k and Part_dot inside the DoT seed, with Part_dot = Part (00 E-13, W:Damage/Calculation L632-635); update §15 and TV-03-01 (recompute with python3); E-3.2 og_present is per view.
- From 00: Sonar and Detect Vulnerability now apply to DoTs (D-095, E-13); M_OG applies to Slash ticks on overguard; Saryn passive scope (D-037) and Toxic Lash (E-14 radial triggers) as 00 now states them; boss status caps per 05 (00 CD-07 follows 05 now).
- From 01: §9.3 must add Sentient Incision to "Excluded everywhere" (W:Sentient_Incision L46: "Elemental damage from this augment does not scale status damage of the corresponding element"), and say that sources 04 does not catalogue take their dot_scaling flag from 01 (§4.10.5, Table 4.10-B).
- From 02: §8.10 still calls the ragdoll rule "dormant"; 02 now reads the ragdoll state from E-3.13 (02 E-2.13a).` },
  { key: '04', docs: ['04-conditional-bonuses-and-buffs'], items: `- From 00 (04's round-1 fixer says it already did several of these; verify each against 00 as it is now and finish what is missing): S_j and H_j out of G (§2.12, §3.2); remove the Toxic-Lash-only Saryn rule (E-4.14); §5.12 to ΣCD_abs; §6.2 radial extra hits; the "after E-5" wording.
- From 07: rows 188-189 (Exalted Umbra Blade waves) should also list Exalted Blade and Exalted Prime Blade, whose data is identical; E-4.6's "r_in applies to every hit of the input" must exclude waves that have their own catalogue row (07 E-7.21, OQ-741).
- From 08: E-4.5 uses an unclamped (1 + A_w), while 01 E-1.11 clamps each factor; make them agree on the evidence.
- From 03: §10 describes vuln_status_damage as a separate factor; 03 keeps it additive inside V_sd (see 03's report and Review notes) and 00 §6.3 is being aligned to 03 in this round.` },
  { key: '05-06', docs: ['05-enemies', '06-ranged-firing-model'], items: `05:
- From 02: 05 has no "not a bodyshot" flag for non-body hitboxes (W:Enemy_Body_Parts L37), so 02 still applies Meticulous Aim to them. Add the flag to 05's part flags (alongside crit_disabled, wp_bonus_ok, is_object) with its source, and say which document consumes it.
- From 03: §18 has a stale remark (the time model or OQ-804); remove or update it.
- Check 05 against 00's revised E-7 (weak points that are not heads take no ΣHS; Part and h on body and radial hits) and CD-07 (status caps on bosses).
06:
- From 00: §9.6 radial Part rule; Duplex (D-608) cadence; sustained metrics (dps_sustained with spool and ramp as 00 §4 now defines it).
- From 03: E-6.11 must use the integer microsecond time grid of 00 CD-14 / 08 E-8.0a (ε only for the tick-count floor).
- From 04: adopt 04's exclusion rule for uncatalogued conditional augment lines.
- From 02: the Zymos exception (02 flag radial_parent_part, 06 OQ-625) must be described the same way in 06.` },
  { key: '08', docs: ['08-evaluation-and-outputs'], items: `- From 00: A-3, A-4, A-10, E-8.2, G-1, G-13, G-14, §4.1 (S_j display-only), §4.5, §13 step 5 (the worked value moves from 2,153.158 to about 2,143.49 after 00's E-13 change; recompute with python3), and n_ext tied to T-43.
- From 02: define the probability of ragdoll and mix whole proc events (ragdolled and not) in E-8.10, as 02 E-2.13a requires; do not substitute an expected δ_rag.
- From 03: E-8.9 and E-8.10 add the K_j term of 04 E-4.2.
- From 04: add Pain Points, Hata-Satya and D-420 … D-422 to the oracle-unsafe list; OQ-810 and OQ-811 are now resolved by 04.` },
]

const APPLY = (docs, items, extra) => `${COMMON}

Your task: apply the follow-ups addressed to ${docs.map((d) => 'docs/mechanics/' + d + '.md').join(', ')}. Edit only ${docs.length > 1 ? 'these documents' : 'this document'}; other agents are editing the other documents at the same time.${extra || ''}

Follow-up items for you:
${items}

Also scan the round-1 reports for any other item addressed to your document(s) that this list missed, and handle it the same way.

For each item: read the current text of every document involved (the other documents are the state after round 1), check the item's evidence against the sources, then apply it. A follow-up is a claim by another agent, not an order: if you find that your document was right and the other document is wrong, keep your text, add a one-line note to your document's "Review notes" saying why, and report it. Where the evidence is genuinely balanced, follow the document that owns the mechanic per 00 §7 and mark the point UNVERIFIED. Recompute with python3 every test vector you add or change. Keep each document's first line "Status: reviewed (spec)". Keep IDs stable: do not renumber equations, test vectors, D- or OQ- items; add new ones at the end of their series.

Final response: a plain-text report under 200 words: items applied, items rejected (why), and anything you believe another document must still change.`

const RECHECK = `${COMMON}

Round 2 has just applied the follow-ups. Your task: check that the spec is now consistent. Do not edit any file.
1. For every follow-up item in the round-1 reports, check that the target document now carries it (or records in its Review notes why it was rejected, with evidence that holds up).
2. Check for contradictions that remain or were introduced: two documents giving different formulas, constants, orders of operations, bucket placements, flags or defaults for the same thing; references (E-, D-, T-, OQ-, TV-, EC-, §) that no longer resolve; ownership violations of 00 §7 (a mechanic defined in two places).
3. Recompute with python3 any test vector in two documents that touch the same quantities and confirm they agree.
Report only problems you can support with evidence (document, section or id, quote). Severity: critical (wrong damage in common builds, or two documents prescribe incompatible behaviour), major (wrong in specific builds or edge cases, or blocks implementation), minor (clarity, citation, dangling reference). Name the document that must change in "doc".`

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

phase('Overview')
const ov = await agent(APPLY(['00-overview'], OVERVIEW_ITEMS, ' The topic documents will be revised after you, to follow 00 as you leave it; so change 00 only where an item or a clear contradiction requires it.'), { label: 'apply:00', phase: 'Overview', effort: 'xhigh' })

phase('Topics')
const topics = await parallel(GROUPS.map((g) => () => agent(APPLY(g.docs, g.items, ' 00-overview.md was finalised just before this step; follow it as it is now.'), { label: `apply:${g.key}`, phase: 'Topics', effort: 'xhigh' })))

phase('Recheck')
const rc = await agent(RECHECK, { label: 'recheck', phase: 'Recheck', effort: 'xhigh', schema: ISSUES_SCHEMA })
const serious = ((rc && rc.issues) || []).filter((i) => i.severity !== 'minor')
const minor = ((rc && rc.issues) || []).filter((i) => i.severity === 'minor')
log(`recheck: ${serious.length} critical/major, ${minor.length} minor`)

phase('Fix')
const byDoc = DOCS.map((d) => ({ doc: d, issues: ((rc && rc.issues) || []).filter((x) => x.doc === d) })).filter((g) => g.issues.some((i) => i.severity !== 'minor'))
const fixes = await parallel(byDoc.map((g) => () => agent(`${COMMON}

Revise docs/mechanics/${g.doc}.md. After the follow-up round, a consistency critic reported the issues below for this document. Critics can be wrong: check each issue's evidence against the sources and the other documents before changing anything. Apply every issue that holds up (minor ones too); for rejected issues add a one-line note to the "Review notes" section. Recompute with python3 any test vector you add or change. Keep the first line "Status: reviewed (spec)" and keep IDs stable. Edit only docs/mechanics/${g.doc}.md; other agents are revising other documents at the same time.

Issues:
${JSON.stringify(g.issues, null, 1)}

Final response: a plain-text report under 150 words: applied, rejected (why), anything another document must change.`, { label: `fix:${g.doc}`, phase: 'Fix', effort: 'xhigh' })))

return {
  overview: ov,
  topics: GROUPS.map((g, i) => ({ group: g.key, report: topics[i] })),
  recheck: rc && { overall: rc.overall, issues: rc.issues },
  fixes: byDoc.map((g, i) => ({ doc: g.doc, issues: g.issues.length, report: fixes[i] })),
  minorUnfixed: minor.filter((m) => !byDoc.find((g) => g.doc === m.doc)),
}
