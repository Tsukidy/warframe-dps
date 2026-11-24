Status: living (project management; updated at every pause)

# Continuation guide

How to pick this project up in a new session, as the manager that runs it. Read this file first, then
[`manager-log.md`](manager-log.md) (every user decision, effort approval and manager ruling), then the documents it
points to. Everything needed to continue is in the repository; nothing depends on the earlier conversation.

Contents: 1 The assignment · 2 How the work is organised · 3 State at the pause · 4 Setting up a fresh container ·
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
| `research/external/` | Gitignored third-party material (inspiration bundle, wiki/WFCD/Public Export caches). Rebuild it with `research/tools/` (section 4). |
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

## 3. State at the pause

<!-- FILLED IN AT THE PAUSE -->

---

## 4. Setting up a fresh container

1. **Repository**: the working branch is `claude/tender-planck-bpibcq` (a new session may be given a different
   branch: merge this one into it first).
2. **Rust**: `rust-toolchain.toml` pins the toolchain; `cargo` installs it on first use. Optional build cache, which
   matters because agents build in several worktrees at once:
   `cargo install sccache --locked` (0.20.0 was used), then in `~/.cargo/config.toml`:
   `[build]` `rustc-wrapper = "<path to sccache>"`.
3. **research/external/**: follow `research/tools/README.md`. It rebuilds the inspiration bundle, the wiki page and
   Lua-module caches (pinned to the revisions the spec cites), WFCD and Public Export, and checks every file against
   `research/tools/` 's manifest. If `min.pretty.js` or a wiki page no longer matches its hash, line references in
   `docs/mechanics` and `research/findings` to that file may be off; the README says how to handle that.
4. **Node** is needed for the frontend (T12 onwards) and for the research tools; Playwright with Chromium is
   preinstalled in Claude Code cloud containers.
5. **Commit trailer**: each session has its own trailer lines (the system tells the session which lines to use).
   Pass them to `impl-phase.js` as `args.trailer`.

---

## 5. Next steps

In order. Efforts: **high** unless a line says otherwise; every xhigh below is already approved by the user
(`manager-log.md` §2). Anything else above high needs a new AskUserQuestion.

1. **T01d Contract sync** (xhigh; covered by the approval for contract tasks T01-T02). Section "### T01d" of
   `work-breakdown.md`: bring `wfd-model` (toggles T-30 … T-43, new flags), the generated TypeScript, JSON schema and
   `CompatFlags`, and `domain-model.md` / `server-api.md` in line with the converged spec; rule on C-4 and C-12.
   Run: `impl-phase.js` with `mode: "main"`, `merge: false`, `tasks: [{ id: "T01d", effort: "xhigh", review: true }]`.
2. **Rest of wave 0, in one `impl-phase.js` run** (`mode: "worktree"`, `merge: true`, so only one merge agent touches
   the main tree): T02a, T02b, T02c, T02d (xhigh, review on), T03b and T03c (high, review on). T12 (frontend
   foundation, high) can join the same run; it depends only on T01b.
3. **Whole-contract review** (xhigh; part of the contract-task approval) before wave 1: an adversarial review of every
   frozen interface (`wfd-model`, engine internal contracts from T02a, crate APIs, API DTOs, curated schema) against
   the spec and the architecture, then fixes. No script exists yet; adapt `spec-converge.js` (critic → fixer → recheck
   loop) to code.
4. **Waves 1-4** of `work-breakdown.md`, honouring each task's Depends. xhigh (approved) for the engine tasks T06, T07,
   T08a, T08b, T13, T14, T15a, T15b, T15c, T16, T17; for T32 (optimizer core); and for T40 (accuracy audit against the
   inspiration). High for everything else.
5. **Open user decision Q-U1** (licence and publication of wiki-derived data, `overview.md` AD-11): ask with
   AskUserQuestion before T18 first commits `data/generated/`; until then the AD-11 interim rule applies.
6. **Final report to the user** must include: the licence situation of wiki-derived data (CC BY-NC-SA 3.0, AD-11, and
   the user's answer to Q-U1; it matters if the repository goes public or commercial); that the
   inspiration's server-only features (monthly top-20 usage chart, donations, Telegram) are out of scope; and the
   spec's open questions that need in-game testing (the `OQ-` items marked UNVERIFIED in `docs/mechanics`).

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

The Workflow tool runs only scripts it can read: `Read` the script once before the first run in a session.
Manager notes are the place for anything an agent cannot find in the documents: what changed since the plan was
written, which files are being edited by someone else at the same time, where gitignored inputs are.

### 6.2 `tools/orchestration/spec-converge.js`

Critic → per-document fixers → recheck loop for `docs/mechanics` (at most two fix rounds). `args.input` is a file that
lists the residual items (the output of a previous review).

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
