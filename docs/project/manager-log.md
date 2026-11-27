Status: living (project management)

# Manager log

Every decision that does not live in a spec or architecture document: what the user decided, what effort levels the
user approved, and the manager's rulings. Newest entries at the end of each section. Rulings about spec content are
recorded in the "Review notes" section of the document concerned; rulings about interfaces in `CONTRACTS.md`. This log
points to them.

---

## 1. User decisions

| Date | Question | Answer |
|---|---|---|
| 2026-11 (start) | Delivery form | Local web app: one Rust binary (Axum) serves the API and the built frontend on localhost and opens the browser. No Tauri. |
| 2026-11 (start) | Languages | English only, but every UI string goes through an i18n layer so languages can be added later as data. |
| 2026-11 (start) | Scope | Server-only features of the inspiration (monthly top-20 usage chart, donations, Telegram) are out of scope (recorded in CLAUDE.md). |
| 2026-11-24 | Pause | Pause after the step in flight and write continuation documents (this log, `continuation.md`, `open-items.md`). Then, because usage was running out: stop immediately, no further agent runs. The manager stopped the running workflows and merged T01b, T01c and T03 by hand. |
| 2026-11-27 | Resume | "Continue as manager from docs/project/continuation.md. Utilize ultracode workflows where applicable. Remember that you are a manager that should delegate tasks to subagents. Follow the proper skills regarding prompt engineering. Run your subagents at high effort. If you believe a task should be done at a higher level inform the user each time for permission." (The earlier xhigh approvals of §2 still cover the tasks they name.) |

## 2. Effort approvals

The user's rule: subagents run at high; any higher level must be asked for, every time. Approved so far (each covers
only the tasks listed):

| Approval | Covers |
|---|---|
| "xhigh for all 3" | (1) spec synthesis and adversarial verification (all spec writing, verifying, consistency and converge rounds); (2) the core Rust damage engine (work-breakdown tasks T06, T07, T08a, T08b, T13, T14, T15a, T15b, T15c, T16, T17); (3) the final accuracy audit against the inspiration (T40). |
| "xhigh for both" | (1) the contract tasks T01-T02 (T01a, T01b, T01c, T02a, T02b, T02c, T02d; the manager reads it as also covering T01d and the whole-contract review before wave 1, which only check and repair those contracts); (2) the optimizer core (T32; called T28 when it was approved, before the work breakdown was renumbered). |

Everything else runs at high. Never `max` without asking.

## 3. Manager rulings

| Date | Ruling | Where recorded |
|---|---|---|
| 2026-11 | Proposals from topic documents to change 00-overview are decided on the evidence; where the evidence is genuinely balanced, the topic document wins (it was researched in more depth) and the point is marked UNVERIFIED. | `docs/mechanics/00-overview.md` Review notes; `tools/orchestration/archive/spec-consistency.js` |
| 2026-11 | When the inspiration and the wiki disagree, both are recorded and the choice is explained in the discrepancy register (00 §5.2, D-001 … D-095, plus per-document D-<nn>-xx); the engine follows the game, and a `Compat` toggle reproduces the inspiration where useful for the oracle tests. | 00 §5 |
| 2026-11-24 | Rulings on the T01a contract requests C-1 to C-16 (most accepted as following the spec; C-4 and C-12 left to T01d; `.cargo/config.toml` assigned to T01c). | `CONTRACTS.md` "Manager · Rulings on T01a requests" |
| 2026-11-24 | New task T01d (contract sync to the converged spec) added to the work breakdown. | `work-breakdown.md` "### T01d" |
| 2026-11 | Licence of wiki-derived data is **not** decided: it is the user's open question Q-U1 (AD-11). Interim rule: `data/generated/` is committed to this private repository with `data/generated/LICENSE` (CC BY-NC-SA 3.0) and `data/NOTICE.md`; nothing is published. Ask the user before T18 first commits `data/generated/`, and repeat it in the final report. | `docs/architecture/overview.md` AD-11, §6 R-3 |
| 2026-11 | Architecture request R-1 (toggle numbering) is settled by the spec: 00 §6.1 now numbers T-30 … T-43 (melee plan is T-30, capacity mode T-35, lower ranks T-36). T01d applies it to the code. R-2 (spec vectors for new DSL constructs) stays on its provisional choice. | `docs/architecture/overview.md` §6; 00 §6.1 |
| 2026-11-27 | Rulings on T01b and T01c requests C-17 … C-30 (all accepted or settled; T01b's C-17 … C-21 renumbered C-26 … C-30); owners for the doc-check names (C-18). | `CONTRACTS.md` "Manager · Rulings on the T01b and T01c requests" |
| 2026-11-27 | Rulings on T01d: C-4 and C-12 confirmed; breaking changes T-22 and T-37 approved (no consumer yet); out-of-ownership test edits confirmed; provisional choices accepted pending the whole-contract review. | `CONTRACTS.md` "Manager · Rulings on T01d" |
| 2026-11-27 | The compatibility profile emulates the CURRENT inspiration bundle (2026-11-27); old line references stay as written and are found through the line map. | 00 §5.2; CLAUDE.md; `research/findings/inspiration-update-2026-11-27.md` |
| 2026-11-27 | T03c-3: the T01d ruling "BUNDLE_SCHEMA_VERSION stays 1 until T18 emits a bundle" also covers the postcard layout change from `FinisherRow.weapons` and every other additive bundle-field change made before the first bundle exists. | `CONTRACTS.md` "Manager · Rulings on T01d"; T03c section |
| 2026-11-27 | Whole-contract review: all 133 items accepted (11 breaking changes approved before wave 1); 128 confirmed resolved by the rechecks; the rest and 25 new findings go to a third fix pass. N-1: exalted per-rank values, `Attack.exalted_s` and zoom-level gates are authored as override fields citing 01 Table 4.3-A and 06 §9.1. NEW-CR-1: the placeholder rule covers a missing `data/spec_tables.toml`. | `CONTRACTS.md` "Manager · Rulings on the whole-contract review (pass 1)" and "(pass 3)" |

## 4. Spec rounds and their outcome

| Round | What | Result |
|---|---|---|
| Topic writing | Writer → two xhigh verifiers → reviser per document 01-09 | All ten documents "reviewed (spec)". |
| Consistency pass | Three xhigh critics (overview correctness, cross-document consistency, edge-case completeness) → one fixer per document | 95 issues (12 critical, 41 major); 00-overview reviewed for the first time; toggles T-30 … T-43 adopted; D-095 added. |
| Follow-up | 00 first, then the topic documents follow it; xhigh recheck and fixes | All round-1 follow-ups applied; 5 critical/major found and fixed by the recheck. |
| Converge loop | Triage of the residual tail → fixers → recheck, at most two rounds | Two fix rounds applied; final recheck stopped at the pause (open-items §5). |
| Recheck (2026-11-27) | xhigh triage of commit 90ed29c → fixers → recheck; then the six-item tail | All fixes verified; tail closed; recheck clean. |
| Inspiration update (2026-11-27) | xhigh cross-check of the update analysis against the new bundle → fixers in 00, 02, 04, 08 → recheck | D-014 and D-056 updated (new Compat flag D056_CO_STATUS_COUNT), TV-02-23 and TV-04-49; D-410 unchanged; game columns unchanged (OQ-016 stays open). |

## 5. Candidate standing corrections

Mistakes seen once; promote to CLAUDE.md "Standing corrections" if they happen again.

- An implementer rewrote a documented example in a test so that it would pass, and reported that every documented
  example passes (T01a, `vuln_add` rows). Rule if repeated: "Never edit a documented example to make it parse or pass;
  a documented form that fails is a bug or a CONTRACTS.md request."
- Fixers working in parallel leave other documents inconsistent (spec rounds). Rule if repeated in code: "When your
  change requires another owner's file to change, record the exact required change in your report; do not leave it
  implicit."
