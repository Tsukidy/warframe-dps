Status: living (project management; updated at every pause)

# Continuation guide

How to pick this project up in a new session, as the manager that runs it. Read this file first, then
[`manager-log.md`](manager-log.md) (every user decision, effort approval and manager ruling), then the documents it
points to. Everything needed to continue is in the repository; nothing depends on the earlier conversation.

Contents: 1 The assignment · 2 How the work is organised · 3 State · 4 Setting up a fresh container ·
5 Next steps · 6 Running the orchestration · 7 Operational lessons

---

## 1. The assignment

### 1.1 The user's instructions (verbatim)

> Manage subagents to complete this full project to completion in one go utilizing ultracode workflows. If you have
> any questions for the user please use the AskUserQuestion tool to properly notify the user.
>
> You are a manager. Delegate where it makes sense and is needed to a subagent with a clean context window for the
> task. Do not allow subagents to continue to spawn their own.
>
> Subagents should be run at the 'High' effort level. If you believe a task requires a higher or would benefit from a
> higher effort level substantially. Inform the user and ask for permission. This must be done every time.
>
> Ensure you utilize the information within the 'Prompt engineering best practices' skill whenever creating prompts
> for subagents.
>
> **Goal:** Create a fully functional DPS calculator with an automated DPS optimizer for the game Warframe.
>
> **Features:**
> - Must at LEAST match the full feature set of the inspiration.
> - Must have a backend written in RUST. Frontend can be whatever would be best.
> - Ensure proper research into the damage calculations that are done.
> - Ensure "edge cases" are not forgotten/missed. (such as condition overload inconsistnetly being a multiplier with
>   others on certain weapons.)
> - Ensure the final DPS is as accurate as possible.
>
> **Setup:** Setup a common docs location alongside a seperate resarch directory for unfinished findings. Include any
> required changes if mistakes continually get made to correct them as we go.
>
> **Resources:**
> - Inspiration: This is the project that inspired this. It is a DPS calculator. However I would like ours to look a
>   bit better and function locally instead. As the automated DPS optimizer takes a while.
>   (A warframe-damage.com link to a Furis build; decoded in `research/test-cases/user-example-furis.json`.)
> - Abandoned Project: This is an abandoned project. It has useful information as it involved gathering information
>   about the game. https://github.com/Tsukidy/warframe-connector

The user's standing preference: **"Verify then act."**

On 2026-11-24 the user asked the manager to pause after the step then in flight and write these continuation
documents. Resuming means continuing the plan in section 5 with the same rules.

### 1.2 Rules that follow from the instructions

- The manager delegates; subagents never start subagents (every agent prompt says "do not use the Agent or Workflow
  tools").
- Subagents run at **high** effort. Any higher level needs the user's permission, asked with AskUserQuestion **every
  time** (each new task or group). Approvals already given are listed in `manager-log.md` §2; they cover only the
  tasks named there.
- Agent prompts follow the prompt-engineering best-practices skill (context and purpose first, explicit scope and
  ownership, the exact acceptance check, a structured final output, evidence for every claim).
- Questions for the user go through AskUserQuestion.
- Repeated agent mistakes become rules in CLAUDE.md "Standing corrections" (none so far; candidates are in
  `manager-log.md` §5).

---

## 2. How the work is organised

| Where | What |
|---|---|
| `CLAUDE.md` | Project rules every agent receives (layout, user decisions, citation and data conventions, DoD). |
| `docs/mechanics/00-09` | The damage spec. Normative for every number and rule. 00 is the master (notation, master formula, evaluators, discrepancy register D-xxx, toggles T-xx, ownership map §7). |
| `docs/architecture/` | Crate layout, domain model, engine, optimizer, server API, frontend, testing, and `work-breakdown.md`: every implementation task with Depends / Owns / Provides / Acceptance, the waves, the rules for every task (§2) and the ownership map (§8). |
| `docs/data/pipeline.md` | Game-data sources and the build pipeline. |
| `CONTRACTS.md` | Every change to a frozen interface, and the manager's rulings on contract requests. |
| `research/findings/` | Research inputs (wiki game truth, reverse-engineering of the inspiration, data-source evaluations). |
| `research/external/` | Gitignored third-party material (inspiration bundle, wiki/WFCD/Public Export caches). Rebuild it by hand (section 4); no script yet. |
| `tools/orchestration/` | The workflow scripts the manager runs (section 6). `archive/` holds the scripts of finished phases for reference. |
| `docs/project/` | This guide and the manager log. |

The project went through these phases (commit history on branch `claude/tender-planck-bpibcq` shows each):

1. **Research**: inspiration reverse-engineering, wiki mechanics, data sources (20 findings files, 2 architecture
   proposals).
2. **Spec**: one writer per topic document, two adversarial verifiers (source fidelity, completeness), one reviser, all
   at xhigh (`archive/spec-topics.js`); then a whole-spec consistency pass with three critics
   (`archive/spec-consistency.js`), a follow-up round (`archive/spec-followup.js`) and a converge loop
   (`spec-converge.js`).
3. **Architecture**: `docs/architecture/*` and `docs/data/pipeline.md` (`archive/architecture.js`).
4. **Implementation**: waves of `work-breakdown.md`, run with `impl-phase.js` (implement in a worktree, adversarial
   review, fix, merge). In progress (section 3).

---

## 3. State

Second session, started 2026-11-27 on branch `ccr-659c593d-av1db8` (same history as `claude/tender-planck-bpibcq`;
each new session is given its own branch). **Open work in detail: [`open-items.md`](open-items.md).**

| Area | State |
|---|---|
| Research | Done: `research/findings/` (21 topic files, 2 architecture proposals). `inspiration-update-2026-11-27.md` (cross-checked) maps the old inspiration line references onto the bundle published that day. |
| `research/external/` | Rebuilt with the committed kit `research/tools/` (571 wiki pins at 2026-11-24, WFCD 1.1276.51, current inspiration and Public Export); tested from an empty directory. |
| Spec `docs/mechanics/00-09` | All "reviewed (spec)". The unrechecked converge round of 2026-11-24 was rechecked and its tail fixed (2026-11-27); then updated for the 2026-11-27 inspiration bundle (D-014, D-056 now `User (T-05) + Compat`). Every recheck ended clean. |
| Architecture | All "reviewed (architecture)"; stale toggle numbers fixed (T-30 melee plan, T-35 capacity mode, T-36 lower ranks). |
| T01a, T01b, T01c | Accepted and merged; all contract requests ruled (`CONTRACTS.md`, rulings of 2026-11-27). |
| T01d contract sync | Done (implemented, reviewed, 11 issues fixed), in the dev branch; manager rulings on it recorded. Full Definition of done green, 75 Compat flags. |
| Wave 0 | Done 2026-11-27: T02a-d, T03b, T03c, T12, T03 fix run; reviewed, fixed, merged (93e1e69). Open requests C-31 … C-40 and task-scoped T02b-n, T02c-Rn, T02d-n, T03b-Rn, T03c-n await rulings (step 2). |
| Wave 1 | Running since 2026-11-28 (section 5 step 3). |
| Wave 2 and later | Not started. |

## 4. Setting up a fresh container

1. **Repository**: use the branch the session is given and merge the previous session's branch into it first (the
   second session used `ccr-659c593d-av1db8`). Pass the branch to `impl-phase.js` as `args.dev`.
2. **Rust**: `rust-toolchain.toml` pins the toolchain; `cargo` installs it on first use. Optional build cache, which
   matters because agents build in several worktrees at once:
   `cargo install sccache --locked` (0.20.0 was used), then in `~/.cargo/config.toml`:
   `[build]` `rustc-wrapper = "<path to sccache>"`.
3. **research/external/** (gitignored; needed by agents that check sources, not by the build): rebuild it with the kit,
   `cd research/tools && npm ci && node rebuild-external.js` (about 6-7 minutes; `README.md` there explains pins,
   drift and `--verify-only`). The inspiration cannot be pinned: on 2026-11-27 the site published a new bundle, and
   every older `JS L…` reference in the docs points to the earlier one (CLAUDE.md; the line map is in
   `research/findings/inspiration-update-2026-11-27.md`). If the site changes again, rerun that analysis
   (`tools/orchestration/external-rebuild.js`, its third agent) and a spec update round.
4. **Node** is needed for the frontend (T12 onwards) and for the research tools; Playwright with Chromium is
   preinstalled in Claude Code cloud containers.
5. **Commit trailer**: each session has its own trailer lines (the system tells the session which lines to use).
   Pass them to `impl-phase.js` as `args.trailer`.

---

## 5. Next steps

In order. Efforts: **high** unless a line says otherwise; every xhigh below is already approved by the user
(`manager-log.md` §2). Anything else above high needs a new AskUserQuestion.

1. **Wave 0: done 2026-11-27.** T02a-d, T03b, T03c, T12 and the T03 fix run were implemented, reviewed, fixed and
   merged (merge commit 93e1e69; Definition of done green). `cargo xtask lint-engine` fails on the fixture weapon
   name "Glaive" against spec identifiers (a C-22 extension, handled in step 2); `cargo xtask ci` gates 7-8 fail on
   todo! until T05/T18/T19 (T02c-R4).
2. **Whole-contract review: done 2026-11-27/28.** Pass 1 found 133 items; passes 2, 3 and the pass-3 tail fixed them
   (last commit 9ab5363; every recheck item resolved; full Definition of done green, frontend included). Records:
   `docs/project/inputs/contract-review-*.json`, CONTRACTS.md "Manager · Rulings on the whole-contract review" and
   "Contract review fixes (…)" sections. Session-local cargo config (`~/.cargo/config.toml`, not in the repository):
   sccache wrapper, `incremental = false`, `[profile.dev] debug = "line-tables-only"`, so about nine worktrees fit the
   disk; set it again in a fresh container.
3. **Wave 1, started 2026-11-28** as five `impl-phase.js` runs with `merge: false`: `W1-engine-a` (T06, T07; xhigh,
   run wf_e36d3e27-fac), `W1-engine-b` (T08a, T08b; xhigh, wf_2dd7f5d0-511), `W1-data` (T04, T10; wf_17a27754-023),
   `W1-pipeline-vectors` (T05, T09; wf_c02d0632-2c6), `W1-server` (T11; wf_2aa33795-b53). After all five, one
   `merge_only` run in the order T06, T07, T08a, T08b, T04, T10, T05, T09, T11. After a container restart, resume each
   unfinished run with its `resumeFromRunId` and the identical args (the args are in the Workflow call of the session
   transcript; the notes differ per task).
   **Waves 1-4** of `work-breakdown.md`, honouring each task's Depends. xhigh (approved) for the engine tasks T06, T07,
   T08a, T08b, T13, T14, T15a, T15b, T15c, T16, T17; for T32 (optimizer core); and for T40 (accuracy audit against the
   inspiration). High for everything else. Pass each task the items open-items §4 assigns to it.
4. **Open user decision Q-U1** (licence and publication of wiki-derived data, `overview.md` AD-11): ask with
   AskUserQuestion before T18 first commits `data/generated/`; until then the AD-11 interim rule applies.
5. **Final report to the user** must include: the licence situation of wiki-derived data (CC BY-NC-SA 3.0, AD-11, and
   the user's answer to Q-U1; it matters if the repository goes public or commercial); that the
   inspiration's server-only features (monthly top-20 usage chart, donations, Telegram) are out of scope; that the
   inspiration changed on 2026-11-27 and the oracle tests follow the new bundle; and the spec's open questions that
   need in-game testing (the `OQ-` items marked UNVERIFIED in `docs/mechanics`).

---

## 6. Running the orchestration

### 6.1 `tools/orchestration/impl-phase.js`

Implements a set of independent `work-breakdown.md` tasks. Per task: an implementer (in its own git worktree
`/home/user/wt/<id>` on branch `wip/<id>`, or in the main tree for `mode: "main"`), an adversarial reviewer, and a
fixer for the confirmed issues; then one merge agent merges the branches with `--no-ff`, runs the Definition of done
(`work-breakdown.md` §2.3) and removes the worktrees. Agents commit but never push; the manager pushes.

```text
Workflow({ scriptPath: "<repo>/tools/orchestration/impl-phase.js", args: {
  label: "W1-engine-a", trailer: "<the session's trailer lines>", mode: "worktree", merge: true,
  tasks: [ { id: "T06", effort: "xhigh", review: true, note: "optional manager note" }, ... ] } })
```

`args.dev` names the session's working branch. `merge_only: true` skips implementing and merges existing `wip/<id>`
branches with one integrator: use it after several parallel runs with `merge: false` (each workflow runs at most two
agents at a time on a 4-CPU container, so parallel runs are how a wave gets more agents).
The Workflow tool runs only scripts it can read: `Read` the script once before the first run in a session.
Manager notes are the place for anything an agent cannot find in the documents: what changed since the plan was
written, which files are being edited by someone else at the same time, where gitignored inputs are.

### 6.2 `tools/orchestration/spec-converge.js`

Critic → per-document fixers → recheck loop for `docs/mechanics` (at most two fix rounds). `args.input` is a file that
lists the residual items (the output of a previous review); `args.note` is a manager note appended to every prompt
(use it to turn the triage into a different job, as the 2026-11-27 inspiration update did); `args.issues` passes
already-confirmed issues (the `open` list of a previous run) and skips the triage.

### 6.2b `tools/orchestration/external-rebuild.js`

Builds `research/external/` with the kit, tests the kit from an empty directory, and analyses an inspiration update
(line map and behaviour changes). Rerun its analysis agent whenever the site publishes a new bundle.

### 6.3 Concurrency and safety

- Each workflow runs at most min(16, CPUs − 2) agents at a time: 2 on the 4-CPU containers used so far. Run several
  workflows side by side to use more, but builds then share the 4 cores.
- **Never run two merge agents on the main tree at once.** Put tasks that will finish together in one
  `impl-phase.js` run, or start the next run after the previous merge.
- A container restart kills running workflows but keeps files. Resume with the same `scriptPath`, the same `args`
  (identical, or the cache misses) and `resumeFromRunId`; finished agents are replayed from cache.
- The session has a stop hook that requires a clean, pushed tree at the end of every turn. While agents are editing,
  commit a snapshot of only the tracked files they touch (`git add -u <paths>`), named "WIP: … (snapshot)", and push.
  Never stage `research/external/` or `.cache/`.

---

## 7. Operational lessons

- Worktrees do not contain gitignored files: tell agents in worktrees to read `research/external/` from the main tree
  (`/home/user/warframe-dps/research/external/`).
- The shell's working directory can reset between commands; give agents absolute paths.
- Parallel editing of interdependent documents leaves a tail of "document X must now follow document Y" items. Edit
  the master document first, then the documents that depend on it, then run a converge loop until a recheck is clean.
- Reviewers catch implementers' overstated acceptance claims (T01a claimed every documented encoding parsed; two had
  been rewritten in the test). Keep the review stage on for every contract and engine task.
- Contract changes discovered late are cheaper as one manager-added task (T01d) than as scattered edits by later
  tasks.
- `Bash` refuses `sleep` loops: wait for workflow notifications instead of polling.
- Third-party sources move: the inspiration published a new bundle on the day the second session started. Pin what
  can be pinned (wiki revisions, npm versions), hash what cannot, and keep a line map when it changes.
- Changing the rustc wrapper (sccache) forces a full rebuild: set it before the first build of a session, not while
  agents are building.
