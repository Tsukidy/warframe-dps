Status: draft (architecture)

# Frontend

The frontend is a single-page app served by the Rust binary. It must match every feature of the inspiration's parity
checklist (`research/findings/inspiration-feature-inventory.md`, "Parity checklist") and look clearly better. All
calculation happens in Rust; the browser edits a document, renders results and drives the optimizer.

Contents: 1 Stack · 2 Project layout · 3 Page layout · 4 Component tree · 5 State and data flow · 6 Feature-parity
mapping · 7 Beyond parity · 8 i18n · 9 Build sharing and inspiration-link import · 10 Visual design direction ·
11 Accessibility · 12 Performance budgets

---

## 1. Stack

Versions are the current lines on npm on 2026-11-23; `package.json` pins exact versions and `package-lock.json` is
committed.

| Concern | Choice | Version | Why |
|---|---|---|---|
| Framework | React + TypeScript (strict) + Vite | react 19.3, typescript 7.0, vite 8.3 | largest ecosystem of accessible primitives; the most predictable stack for implementing agents; rendering speed is not the bottleneck (the engine is in Rust) |
| Accessible components | React Aria Components | 1.22 | the hard UI problems are accessibility problems: weapon/enemy/mod comboboxes, a mod grid with keyboard and screen-reader drag and drop, dialogs, popovers, tooltips, number fields with locale formatting, sliders, tabs; React Aria implements the ARIA patterns (including keyboard DnD) |
| Styling | Tailwind CSS v4 with CSS custom-property design tokens | 4.3 | consistent spacing and colour tokens, dark and light themes from one token set, no runtime CSS-in-JS |
| Client state | Zustand + Immer | zustand 5.1, immer 11.2 | one serializable document store; Immer patches give undo/redo |
| Server state | TanStack Query | 5.108 | request deduplication, `AbortSignal` cancellation of superseded calculations, caching by data version |
| Routing | TanStack Router | 1.170 | typed search params (`?b=wfd1-…`) |
| i18n | react-intl (ICU MessageFormat) + @formatjs/cli + eslint-plugin-formatjs | 12.2, 6.23, 8.1 | plural and number formatting per locale; message extraction; lint rule against literal strings |
| Charts | Recharts | 3.11 | declarative SVG charts themed by CSS variables (DPS over time with MC bands, breakdowns, uptime, crit tiers); ≤ a few thousand points per chart; every chart has a table view |
| Virtualization | TanStack Virtual | 3.14 | hit timeline with thousands of hits |
| Fuzzy search | fuse.js | 7.5 | client-side search over the catalogue (names, stat keys) |
| Fonts | Inter and JetBrains Mono, self-hosted (`@fontsource-variable/*`) | — | offline app; tabular numerals |
| Tests | Vitest + Testing Library, MSW, Playwright + @axe-core/playwright | 5.1, 16.3, 3.0, 1.65, 4.14 | testing.md §8 |
| API types | generated from Rust by ts-rs 12 (`cargo xtask gen-ts` → `src/api/generated/`) | — | one contract (AD-02) |

---

## 2. Project layout

```text
frontend/
  package.json  package-lock.json  vite.config.ts  tsconfig.json  eslint.config.js  playwright.config.ts  index.html
  src/
    main.tsx  App.tsx  router.tsx
    api/         client.ts (fetch wrapper, error → Message)  queries.ts (TanStack hooks)  sse.ts (EventSource with
                 Last-Event-ID resume)  generated/*.ts (ts-rs; do not edit)
    i18n/        en.json (ICU messages)  messages.ts (typed MessageId union, generated)  I18nProvider.tsx
                 format.ts (formatDamage = floor(x + 0.5) (U-5), percent, duration, compact)  pseudo.ts (en-XA)
    state/       store.ts (Zustand + Immer)  history.ts (patch-based undo/redo)  urlSync.ts  baseline.ts  prefs.ts
                 selectors.ts
    design/      tokens.css  themes.css  damageTypes.ts (colour token + glyph + label key per DamageType)
                 icons/sprite.svg  primitives/ (Button, IconButton, Card, StatRow, NumberField, PercentField, Toggle,
                 Segmented, Chip, Badge, InfoPopover, Tooltip, DataTable, Sheet, Dialog, Tabs, Slider, ComboBox,
                 EmptyState, Skeleton, DeltaBadge)
    features/
      shell/     TopBar  AppLayout  PaneTabs (narrow screens)  DataVersionBadge  ThemeToggle  LiveRegion
      loadout/   LoadoutPane  WeaponPicker  VariantControls  KitgunParts  ZawParts  ProgenitorPicker  ItemParamEditor
                 ModGrid  ModSlot  ModTile  ModPickerSheet  ExilusSlot  StanceSlot  ComboSelect  ArcaneSlots
                 IncarnonTrack  PerkPicker  RivenEditor  FramePanel  FrameModSlot  AuraSlot  FrameArcaneSlot  ShardSlot
      target/    TargetPane  EnemyPicker  EnemyPanel  OptionsPanel (schema-driven)  BuffsPanel (schema-driven)
                 ConditionsControl  CustomStatsDialog (schema-driven)  CombineAttacksDialog  SimulationPanel
      results/   ResultsPane  HeadlineCards  McRangeStrip  LayerTabs  AttackCard  AttackTable  ParityColumns
                 DpsOverTimeChart  BreakdownChart  StatusUptimeChart  CritTierChart  ChartTableToggle
                 WeaponStatsPanel  HitTimelineDialog  ExplainDrawer  AssumptionsList  WarningsList  BaselineBar
      optimizer/ OptimizerPage  ObjectivePanel  ConstraintsPanel  GroupsPanel  BuffPortfolioPanel  ProhibitedMods
                 RunPanel  ProgressBar  LogPane  IncumbentPreview  ResultPanel  BuildDiff  BuffTable
      share/     ShareDialog  ImportDialog  ImportReport
      data/      DataPage (version, check, refresh with progress, report, activate, rollback)
      about/     AboutPage (licences, sources, coverage, keyboard shortcuts)
    mocks/       handlers.ts (MSW, built from data/fixtures/mini via a generated catalogue JSON)
  e2e/
    parity/PAR-01-weapon-search.spec.ts … PAR-39-optimizer-run.spec.ts
    keyboard.spec.ts  a11y.spec.ts  visual.spec.ts  pseudo-locale.spec.ts  share.spec.ts  import.spec.ts
```

---

## 3. Page layout

Desktop (≥ 1280 px): three persistent panes, no modal for the core workflow (the inspiration hides most inputs in
popups).

```text
┌ TopBar: logo · build name (editable) · Undo/Redo · Pin baseline · Share · Import · Optimizer · Data vX · theme ┐
├──────────── Loadout (left, 420 px) ───┬────────── Results (centre, fluid) ─────────┬── Target (right, 360 px) ─┤
│ WeaponPicker + VariantControls        │ HeadlineCards: Window DPS (EV) with MC     │ EnemyPicker               │
│ (kitgun, zaw, progenitor, params)     │   range strip · Burst · Sustained · TTK    │ level · Steel Path ·      │
│ ModGrid: 8 main + exilus              │   (+ kill probability) · Δ vs baseline     │ Eximus · party            │
│ StanceSlot + ComboSelect (melee)      │ LayerTabs: Armor | Health | Shield |       │ EnemyPanel: pools, armor, │
│ ArcaneSlots (1, or 2 on arch-guns)    │   Overguard | Layered (TTK)                │ DR, immunities, unique-DR │
│ IncarnonTrack (tiers I-V)             │ AttackCards (one per attack + groups):     │ OptionsPanel (schema)     │
│ Riven (inline card + editor)          │   DPS, parity columns, sparkline           │ BuffsPanel (schema)       │
│ FramePanel (collapsible): 4 frame     │ Charts: DPS over time · breakdown by type  │ ConditionsControl         │
│ mods, aura, 2 frame arcanes,          │   and source · status uptime · crit tiers  │ Custom stats (dialog)     │
│ 5 shards (colour + Tauforged)         │ WeaponStatsPanel (base → modded per attack)│ Combine attacks (dialog)  │
│                                       │ Assumptions · Warnings                     │ SimulationPanel           │
└───────────────────────────────────────┴────────────────────────────────────────────┴───────────────────────────┘
```

900-1280 px: two panes; Target becomes a tab next to Results. Below 900 px: one pane with a bottom tab bar
(Loadout, Results, Target). The optimizer is its own route (`/optimizer`): configuration on the left, live progress,
leaderboard and log on the right; "Apply" and "Compare" bring a result back to the calculator. `/data` and `/about`
are plain pages.

---

## 4. Component tree

```text
<App> (I18nProvider · QueryClientProvider · RouterProvider · ThemeProvider · LiveRegion)
├── <TopBar> <BuildName> <UndoRedo> <PinBaseline> <ShareButton>→<ShareDialog> <ImportButton>→<ImportDialog>
│            <OptimizerLink> <DataVersionBadge> <ThemeToggle>
├── route "/"  <CalculatorPage>
│   ├── <LoadoutPane>
│   │   ├── <WeaponPicker> (ComboBox: substring + fuzzy over localized names, slot/class filter chips, images)
│   │   ├── <VariantControls> <KitgunParts> <ZawParts> <ProgenitorPicker> (element select + 25-60 % field/slider)
│   │   ├── <ModGrid> (React Aria GridList with drag and drop)
│   │   │   ├── 8 × <ModSlot> → <ModTile> | empty state ("accepts: …")
│   │   │   ├── <ExilusSlot> <StanceSlot><ComboSelect/> <ArcaneSlots>
│   │   │   └── <ModPickerSheet> (search by name or stat, legal/blocked with reason, set badges, rank, params)
│   │   ├── <IncarnonTrack> 5 tier tiles → <PerkPicker> popover
│   │   ├── <RivenEditor> (4 trait rows, class-filtered list, units, ≤ 3 base elements, live card preview,
│   │   │                  value range hint from the riven formula)
│   │   └── <FramePanel> 4 × <FrameModSlot> <AuraSlot> 2 × <FrameArcaneSlot> 5 × <ShardSlot>
│   ├── <ResultsPane>
│   │   ├── <BaselineBar> (when pinned)  <HeadlineCards> <McRangeStrip>
│   │   ├── <LayerTabs> (present layers; "expand" = focus mode)
│   │   │   └── <AttackCard>* / <AttackTable> (toggle): DPS window/burst/sustained, parity columns (queue DPS,
│   │   │        avg per bullet direct/status/total, median per bullet, TTK), "Timeline" → <HitTimelineDialog>
│   │   ├── <DpsOverTimeChart> <BreakdownChart> <StatusUptimeChart> <CritTierChart> (each with <ChartTableToggle>)
│   │   ├── <WeaponStatsPanel> (per attack: base → modded, per-type damage bars, Δ)
│   │   ├── <AssumptionsList> (touched register rows)  <WarningsList> (unmodelled, draft, UNVERIFIED, EV-MC deviation)
│   │   └── <ExplainDrawer> (opens from any number that has a probe)
│   └── <TargetPane>
│       ├── <EnemyPicker> (grouped by faction; search by name or faction)  <EnemyPanel>
│       ├── <OptionsPanel> (renders ScenarioSchema.toggles)  <BuffsPanel> (renders ScenarioSchema.buffs)
│       ├── <ConditionsControl> (T-02 mode + per-effect overrides list from CalcReport.conditions)
│       ├── <CustomStatsDialog> <CombineAttacksDialog>
│       └── <SimulationPanel> (window 1-60 s, advanced 600; MC runs; seed; "re-roll"; auto-MC switch)
├── route "/optimizer"  <OptimizerPage>
│   ├── <ObjectivePanel> <ConstraintsPanel> (mirrored mod grid with lock icons) <GroupsPanel>
│   │   <BuffPortfolioPanel> <ProhibitedMods>
│   ├── <RunPanel> start/abort · <ProgressBar> (phase, rate, gap, ETA) · <IncumbentPreview> · <LogPane>
│   └── <ResultPanel> best · alternatives · per mix · leave-one-out · next-best per slot · condition sensitivity ·
│        <BuffTable> · MC confirmation · <BuildDiff> · Apply · Compare
├── route "/data"  <DataPage>
└── route "/about" <AboutPage>
```

---

## 5. State and data flow

- **One document.** `store.ts` holds `doc: CalcInput` (generated type) plus UI-only state (open panels, selected
  attack and view, chart modes). Every user action is a named store action (`setWeapon`, `placeItem(slot, pick)`,
  `moveItem(from, to)`, `removeItem(slot)`, `setRiven`, `setToggle(path, value)`, `setBuff`, `setOverride`, …). The
  store applies the action with Immer; the patches feed undo/redo (`history.ts`, 200 steps, Ctrl+Z / Ctrl+Shift+Z).
- **Switching weapons** keeps installed mods that are still legal for the new weapon (inspiration behaviour, `I:feat`
  §2) and drops the rest with an undoable toast; Incarnon perks and arch-gun arcanes are cleared.
- **Derived server state.** `useCalcEv(doc)`: TanStack Query keyed by `(data_version, stableHash(doc))`, debounced
  120 ms, `want` without MC. `useCalcMc(doc)`: separate query, debounced 400 ms, `want.mc` set; can be switched off.
  Superseded requests are aborted (the server stops their work). The previous result stays visible, dimmed, while a
  new one loads (`placeholderData`).
- **Catalogue and schema** are fetched once per `data_version` (`staleTime: Infinity`); `useCandidates(doc, slot,
  query)` calls `/api/candidates`.
- **URL.** Every `CalcReport` carries the share `code` of its inputs, so the page URL (`?b=wfd1-…`, replaceState,
  debounced 300 ms) never needs a second encoder; Rust is the only codec (AD-18).
- **Persistence** (`prefs.ts`, `localStorage` with try/catch fallbacks): last build, prohibited-mods list (StableKeys),
  theme, density, collapsed sections, optimizer form.
- **Baseline.** "Pin baseline" stores a second `CalcInput` and its last report; every headline number shows Δ % against
  it; the optimizer's "Compare" uses the same mechanism and opens `<BuildDiff>`.
- **Optimizer.** The job is server state: `POST /api/optimize`, then an `EventSource` on the events route with
  `Last-Event-ID` resume; the store keeps only the form.

---

## 6. Feature-parity mapping

Every row of the inspiration's parity checklist, where our UI provides it, and the end-to-end test that proves it
(`frontend/e2e/parity/PAR-nn-*.spec.ts`, run against `warframe-dps --no-browser` on the mini fixture bundle with a
fixed MC seed).

| # | Inspiration feature (checklist row) | Must-have | Our implementation | Test |
|---|---|---|---|---|
| 1 | Weapon search (localized substring, keyboard nav) | yes | `WeaponPicker`: React Aria ComboBox, substring + fuzzy over localized names and categories, arrows/Enter/Escape, slot and class filter chips, images | PAR-01 |
| 2 | All attacks of a weapon simulated and listed separately | yes | `ResultsPane` renders one `AttackCard` per attack in every layer tab, plus parent+child and combined group cards | PAR-02 |
| 3 | Melee stance slot + stance combo selector | yes | `StanceSlot` + `ComboSelect`; stanceless defaults shown as "default combo"; the combo is saved in the share code (the inspiration does not) | PAR-03 |
| 4 | Heavy / slam attacks with `isHeavy` auto-mode | yes | attack roles from data; the heavy toggle (T-09) shows "always heavy" and is disabled with an explanation on explicit heavy attacks | PAR-04 |
| 5 | Weapon info panel with live modded stats and per-type damage | yes | `WeaponStatsPanel`: per attack, base → modded with Δ, per-type damage bars with colour + glyph, magazine, Incarnon magazine, reload, burst, charge, falloff, forced procs | PAR-05 |
| 6 | Incarnon evolutions (tiers) and Incarnon-form attack | yes | `IncarnonTrack` (5 tier tiles, `PerkPicker` popovers with effect text); the form attack has its own card using the Incarnon magazine and ignoring ammo efficiency | PAR-06 |
| 7 | Kitgun grip + loader | yes | `KitgunParts` (grip filtered by chamber class, loader list); saved in the code | PAR-07 |
| 8 | Zaw grip + link | yes | `ZawParts` (grip filtered by strike class); saved | PAR-08 |
| 9 | Progenitor element + 25-60 % | yes | `ProgenitorPicker`: 7 elements, NumberField + Slider clamped 25-60 %, default 60 % | PAR-09 |
| 10 | Chromatic Blade / Reactive Storm element choice | yes | generic `ItemParamEditor` from the item's `ParamSpec` (shown on the tile and in the picker); saved in the code | PAR-10 |
| 11 | Faction-weakness element mods (`add_weakness_dmg`) | yes | engine `target_weakness` element; the mod tile shows the resolved element for the current enemy | PAR-11 |
| 12 | 8 mod slots + exilus + arcane (+2 arch-gun arcanes) | yes | `ModGrid` from `ScenarioSchema.slot_layouts`; `ArcaneSlots` shows 2 on arch-guns | PAR-12 |
| 13 | 4 warframe mods, aura, 2 warframe arcanes, 5 archon shards incl. Tauforged | yes | `FramePanel` (collapsible, remembers state); `ShardSlot` with colour, bonus and a Tauforged switch | PAR-13 |
| 14 | Mod search by name or stat, compatibility, exclusivity, Semi filter, set bonuses | yes | `ModPickerSheet` backed by `/api/candidates`: name or stat query, blocked items shown greyed with the reason, set badge with piece count, rank selector | PAR-14 |
| 15 | Mod tile interactions (remove, swap by drag), keyboard equivalents | yes | React Aria DnD (mouse, touch, keyboard: Space/Enter to lift, arrows, Enter to drop), context menu (Remove, Move to…, Rank), Delete/Backspace removes, drag outside removes | PAR-15 |
| 16 | Riven builder (4 stats, per-class list, units, bow speed ×2, faction multiplier) | yes | `RivenEditor`: units from `riven_traits`, faction entered as a multiplier, at most 3 base elements, live card preview, legal value range hint (our addition) | PAR-16 |
| 17 | Enemy search by name or faction | yes | `EnemyPicker`: grouped by faction, matches name or faction | PAR-17 |
| 18 | Enemy level, Steel Path, Eximus, party size, innate DR, immunity, unique-DR warning | yes | `TargetPane` inputs + `EnemyPanel`; party size only for party-scaled enemies; unique-DR badge with explanation (T-28 switch) | PAR-18 |
| 19 | Enemy scaling (health, shield, armor, overguard, Eximus, 200/2700 armor clamp) | yes | `EnemyPanel` from `CalcReport.enemy` (engine 05 rules) | PAR-19 |
| 20 | Every "additional option" (19 controls incl. hidden `o.w`) | yes | `OptionsPanel` + `BuffsPanel` from the schema; mapping below | PAR-20 |
| 21 | Custom stat values popup (26 stats) | yes | `CustomStatsDialog` generated from `ScenarioSchema.custom_stats`, grouped (damage, crit, status, weapon, melee, target), "Reset all" | PAR-21 |
| 22 | Combine attack effects | yes | `CombineAttacksDialog` lists the weapon's groups (T-22); saved in the code | PAR-22 |
| 23 | Trigger all conditions toggle (default on) | yes | `ConditionsControl`: mode `all at max` (default) / `none` / `simulated`, plus per-effect overrides | PAR-23 |
| 24 | Queue length 1-60 s | yes | window slider in `SimulationPanel` (advanced up to 600 s); saved in the code | PAR-24 |
| 25 | Random queue simulation re-rolled on every change | yes | MC with seed "auto" re-rolls on every change; "re-roll" button; the parity column shows one-run "random queue DPS"; the headline is the deterministic EV (no jitter) | PAR-25 |
| 26 | Average over many cycles with min/median/max | yes | fixed `mc_runs` (default 1000) with mean, median, min, max and percentiles; tooltip "Min / Median / Mean / Max" | PAR-26 |
| 27 | Result panels per layer (armor, health, shield, overguard) with expand | yes | `LayerTabs` (present layers only; layer explanations on hover/focus) + focus mode; "Layered" tab for TTK | PAR-27 |
| 28 | Per attack: queue DPS, avg per bullet (direct/status/total), median per bullet, TTK | yes | `ParityColumns` in `AttackCard`/`AttackTable` | PAR-28 |
| 29 | Extended queue timeline (per-hit cards, reloads, status stacks) | yes | `HitTimelineDialog`: virtualized horizontal cards and a table view; reload markers; live stacks; drag and wheel scroll | PAR-29 |
| 30 | Tooltips (mod, arcane, shard info; attack damage types; layer explanations) | yes | React Aria `Tooltip` / `InfoPopover`; item info = rendered effects (i18n) + upstream text + curation status; attack hover shows modded per-type damage | PAR-30 |
| 31 | Build link encode/decode (all keys) | yes | `?b=wfd1-…` kept in the URL; every scenario input is in the code, including those the inspiration drops (window, stance combo, combine groups, exalted element, party size, heavy start combo) | PAR-31 |
| 32 | Copy build link button | yes | `ShareButton` copies the URL (and the bare code); toast confirmation | PAR-32 |
| 33 | Localization (7 languages) | no | English only through the i18n layer (§8); locales are data later | pseudo-locale |
| 34 | Optimizer: metrics DPS / min TTK / max hit | yes | `ObjectivePanel`: DPS (window), burst, sustained, min TTK, max hit in fight, max hit fresh | PAR-34 |
| 35 | Optimizer: attack, defense layer, heavy + combo, search window, beam width, element exclusion | yes | `ObjectivePanel` (attacks with weights, view, heavy + start combo, window with the inspiration's auto value offered), `ConstraintsPanel` (require/forbid elements as chips; free text "gas, blast" is parsed too), context beam as "beam width" under Advanced | PAR-35 |
| 36 | Optimizer: build rules (headshot, no faction, no on-kill, conditions, keep installed, keep progenitor, Incarnon perks) | yes | `ConstraintsPanel` rule switches; per-slot locks in the mirrored grid ("keep installed" locks all occupied slots) | PAR-36 |
| 37 | Optimizer: warframe slot toggles, strength, allowed external buffs with values | yes | `GroupsPanel` (frame mods 0-4, aura, frame arcanes, shards with owned list), strength field, `BuffPortfolioPanel` (buff + value, portfolio switch) | PAR-37 |
| 38 | Optimizer: prohibited-mods list with persistence and defaults | yes | `ProhibitedMods`: search by name or stat, chips, inspiration defaults with reasons on first run, persisted in `localStorage` | PAR-38 |
| 39 | Optimizer: progress bar, log, abort, apply result to the page | yes | `RunPanel` (phase, fraction, rate, gap, ETA), `LogPane` (i18n messages), Abort (DELETE), `ResultPanel` with Apply and Compare | PAR-39 |
| 40 | Optimizer: per-phase diagnostic buttons | no | not provided; the result's statistics, proof and log replace them | — |
| 41 | Help popup and changelog | no | `AboutPage` links to our docs; no changelog popup | — |
| 42 | Building guide article | no | not provided | — |
| 43 | TOP 20 monthly chart, donations, Telegram, cookie consent | no | out of scope (CLAUDE.md) | — |

Page-chrome behaviours of `I:feat` §1 are also covered: Escape closes popovers and dialogs, keyboard navigation in
every list (React Aria), and an empty-state hint on the enemy picker replaces the inspiration's blinking outline.

**Row 20 detail: the inspiration's "additional options" and where they are.**

| Inspiration option (link key) | Spec toggle | Control |
|---|---|---|
| External Virus + stacks (`o.v`) | T-06 | OptionsPanel: switch + stacks 1-10 |
| Strength (`o.t`) | T-20 | OptionsPanel: percent field (shows 100 %, fixes quirk 1) |
| Headshot/Weak Point bonus active (`o.hs`) | T-03 | OptionsPanel: segmented body / head / weak point |
| External Decrease Armor (`o.d`) | T-07 | OptionsPanel: switch + percent |
| Stealth bonus (melee) (`o.s`) | T-08 | OptionsPanel (melee only): switch + weapon rank |
| Heavy attack + start combo (`o.h`) | T-09 | OptionsPanel (melee only): switch + start combo |
| Light attacks combo multiplier | T-10 | OptionsPanel (melee only): switch + multiplier |
| Combo Ability Multiplier (`o.ac`) | T-11 | OptionsPanel (exalted only) |
| Sniper combo multiplier (`o.sc`) | T-12 | OptionsPanel (sniper attacks only) |
| Roar (`o.r`), Eclipse (`o.z`), Xata's Whisper (`o.x`), Toxic Lash (`o.l`), Nourish (`o.n`), Flat crit chance (`o.j`), Madurai (`o.g`) | T-13 … T-19 | BuffsPanel: switch + value each (Madurai = Phoenix Talons switch) |
| Manual number of statuses (`o.ms`) | T-05 | OptionsPanel: CO status count mode expected / weapon types / manual N |
| Damage per status multiplicative (`o.md`) | T-04 | OptionsPanel |
| Hidden weak-point CC multiplicative (`o.w`) | T-21 | OptionsPanel → Advanced (hidden by default; set by link import) |
| Trigger all conditions (`o.m`) | T-02 | ConditionsControl |

---

## 7. Beyond parity

What makes the app clearly better than the inspiration:

- **Deterministic headline**: window DPS from the EV evaluator, so numbers do not jitter on every change; the MC
  distribution (median and p5-p95) is a thin strip under it, with a CI.
- **Explain drawer**: any number with a probe opens a breakdown ("Critical chance 75.0 % = 12 % base × (1 + 150 %
  Point Strike) + 45 % Arcane Avenger (absolute, E-5)") with links to items and spec citations.
- **Charts the inspiration lacks**: DPS over time (EV line + MC p10-p90 band, reload dips visible), damage by type and
  by source, status uptime and stacks over time, crit-tier distribution; each with a table view.
- **Baseline and diff**: pin a build, change things, see Δ on every number; compare optimizer results stat by stat.
- **Honest data**: unmodelled, draft and auto-updated items are badged; assumptions list shows which discrepancy
  decisions (D-xxx) the current scenario touches; EV-vs-MC deviation badge.
- **Per-effect condition overrides** instead of one global switch.
- **Undo/redo**, keyboard-first editing, responsive layout down to phone width, dark and light themes.
- **Optimizer**: all cores, proven optimum or gap, best build per damage mix, leave-one-out, next-best per slot,
  buff portfolio table, MC confirmation.
- **Data page**: check and refresh game data in-app after a hotfix, with a diff report and rollback.

---

## 8. i18n

- All UI text lives in `src/i18n/en.json` (ICU MessageFormat), ids namespaced by feature
  (`loadout.modGrid.empty`, `toggle.T-13.label`, `warning.item.unmodelled`). `messages.ts` is a generated union type of
  ids, so an unknown id is a type error. ESLint `formatjs/no-literal-string-in-jsx` (and `enforce-description` off)
  fails CI on any literal text in JSX or in `aria-label`/`title`/`placeholder`; `formatjs extract` in CI finds unused
  and missing ids.
- Server-produced text is never English: warnings, errors, optimizer logs, legality reasons and import reports are
  `Message { key, args }`. `cargo xtask gen-ts` exports the list of `MessageKey`s; a Rust test fails when a key has
  no `en.json` entry (I-8).
- Game names (weapons, mods, enemies, perks) come from the bundle's `NameTable` (`/api/catalog.names`); WFCD offers 14
  more locales that can be added as data later. Toggle, buff and custom-stat labels are i18n keys named in the schema.
- Numbers, percents and durations use `Intl` through react-intl; damage uses `formatDamage` = `floor(x + 0.5)` (spec
  U-5); percents shown with one decimal; compact notation (`34.4 k`) only in dense tables, full numbers in tooltips.
- A pseudo-locale `en-XA` (accented, 30 % longer) is generated in development and run in one e2e suite to catch
  hard-coded strings and truncation.
- Adding a language = adding `src/i18n/<locale>.json` and a bundle name table; no code change.

---

## 9. Build sharing and inspiration-link import

- **Share**: `ShareDialog` shows the URL `http://127.0.0.1:<port>/?b=wfd1-…` and the bare code, both with copy
  buttons. Opening a URL with `?b=` decodes through `/api/build/decode`; decode warnings (retired items, migrated
  version) appear as a dismissible report.
- **Import**: `ImportDialog` accepts our code, our URL, or a warframe-damage.com URL (`?p=<LZString>&n=1`, or legacy
  base64 without `n`). The server maps the inspiration's key map (`I:feat` §15: `w`, `e`, `l`, `sp`, `ex`, `j`, `jv`,
  `z.*`, `k.*`, `m.*`, `r.*`, `o.*`, `p.*`) through `data/curated/aliases/inspiration.toml` and returns
  `ImportReport` (unmapped names, unit conversions, the inspiration's never-touched defaults such as
  `o.v.stucks = 0`). The dialog shows the report before applying, with Undo after.
- **Export to the inspiration** exists only in the API for the oracle harness (`/api/build/export-inspiration`).

---

## 10. Visual design direction

The inspiration is a dense jQuery page: everything in popups, no hierarchy between the headline and secondary
values, no charts, numbers that jitter. The target is a calm, dark **instrument panel**:

- **Hierarchy**: one headline number per view (window DPS) set large in tabular figures, its MC range as a thin strip
  under it, Δ against the baseline in a muted colour with an arrow glyph; then attack cards; then detail panels. Each
  pane scrolls independently; everything for one weapon fits a 1440×900 screen without page scrolling.
- **Tokens** (`design/tokens.css`, CSS custom properties, consumed by Tailwind v4 `@theme`):

| Token | Dark | Light | Use |
|---|---|---|---|
| `--surface-0` | `#0f1216` | `#f6f7f9` | page background |
| `--surface-1` / `-2` / `-3` | `#161b22` / `#1d2430` / `#252e3c` | `#ffffff` / `#eef1f5` / `#e3e8ef` | panes, cards, raised elements |
| `--text-1` / `--text-2` / `--text-3` | `#e8ecf1` / `#aab4c3` / `#7a8699` | `#141a22` / `#3e4a5c` / `#5f6b7d` | primary, secondary, muted (all ≥ 4.5:1 on their surface, checked by a unit test) |
| `--accent` | `#f5b544` (warm "Orokin" amber) | `#a86a00` | primary actions, headline, focus ring |
| `--stat` | `#5cc8e0` (cool cyan) | `#0b7a93` | statistics: MC bands, CIs |
| `--success` / `--warning` / `--danger` | `#4cc38a` / `#e6a23c` / `#ef5d5d` | darker variants | deltas, badges |
| `--dt-<type>` | one per damage type | same hue, adjusted lightness | damage-type bars, charts, slot stripes |

- **Damage types**: every type has a colour token, an SVG glyph (sprite) and a short label; colour is never the only
  cue; the palette is checked for colour-vision deficiencies (deuteranopia and protanopia simulations in the visual
  test).
- **Mod tiles**: compact cards with a rarity edge, polarity glyph, rank pips, drain, name, the key stat line rendered
  from the effect (not DE text), a damage-type stripe for elements, curation badges; hover/focus shows the full effect
  text and the item's contribution to the current result ("+12.4 % window DPS", from the leave-one-out of a cheap EV
  call). Empty slots show what they accept.
- **Typography**: Inter (UI) and JetBrains Mono with `font-variant-numeric: tabular-nums` for every number, so columns
  align; three sizes for numbers (headline 40 px, card 20 px, table 13 px).
- **Charts**: thin lines, no chart junk, the same damage-type colours, light grid on `--surface-2`, tooltips with full
  numbers, legends that double as toggles.
- **Motion**: 150 ms transitions on value changes only (numbers tween); none under `prefers-reduced-motion`.
- **No DE artwork or branding** beyond item images served from the WFCD CDN; generated glyph tiles offline.

---

## 11. Accessibility

Target WCAG 2.2 AA:

- Every action reachable by keyboard, including drag and drop (React Aria keyboard DnD and equivalent menu actions:
  Move to…, Remove); visible focus rings (`--accent`, 2 px, offset); a roving tab index in the mod grid; Enter opens
  the picker, Delete removes, Ctrl+Arrow moves a mod; Escape closes; documented shortcuts on the About page.
- Comboboxes, grid lists, dialogs, tabs, sliders and tooltips use React Aria's ARIA patterns; dialogs trap and restore
  focus.
- Labels and units on every input; error messages linked with `aria-describedby`.
- A polite live region announces a throttled summary after recalculation ("Window DPS 34,381, plus 12 %").
- Every chart has a "show as table" toggle; no information conveyed by colour alone.
- Targets at least 24 × 24 px; zoom to 200 % without horizontal scrolling of the panes; `prefers-reduced-motion` and
  `prefers-color-scheme` honoured; high-contrast mode keeps focus visible.
- `@axe-core/playwright` runs in every e2e test with zero serious or critical violations allowed.

---

## 12. Performance budgets

| Budget | Target |
|---|---|
| First load (localhost, cold cache) | ≤ 1.5 s to interactive; JS ≤ 600 kB gzip (code-split: optimizer, data and timeline are lazy routes/dialogs) |
| Catalogue fetch and index build | ≤ 300 ms |
| Input → EV result visible | ≤ 200 ms (120 ms debounce + server ≤ 5 ms + render) |
| Input → MC distribution | ≤ 700 ms for 1,000 runs |
| Timeline with 2,000 hits | 60 fps scrolling (virtualized) |
| Re-render scope | a slot change re-renders the grid cell, results and charts only (selector-based subscriptions) |
