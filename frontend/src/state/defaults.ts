// Default documents (domain-model.md §9.3 defaults; server-api.md §4). New documents of the UI use
// `eval.seed = "auto"` (frontend.md §5 "Seed", parity row 25); the server's serde default for a missing seed is 0.
import type { CalcInput, EnemySetup, EvalSettings, ScenarioDoc, Toggles } from "../api/generated";

/** Document version written by this client (server-api.md §2 `CalcInput.v`). */
export const DOC_VERSION = 1;

/** Placeholder key before the catalogue is known; documents with it are never sent to the server. */
export const NO_KEY = 0;

export function defaultToggles(): Toggles {
  return {
    hit_location: "body", // T-03
    co_multiplicative: false, // T-04
    co_status_count: { mode: "expected" }, // T-05
    external_viral: null, // T-06
    armor_strip: 0, // T-07
    stealth: null, // T-08
    heavy: null, // T-09
    light_combo: null, // T-10
    ability_combo: null, // T-11
    sniper_combo: null, // T-12
    strength: 1, // T-20
    wp_cc_multiplicative: false, // T-21
    combine_groups: null, // T-22: profile default
    distance_m: 0, // T-27
    unique_dr: true, // T-28
  };
}

export function defaultEval(mcRuns = 1000): EvalSettings {
  return { window_s: 20, ttk_horizon_s: 60, mc_runs: mcRuns, seed: "auto", series_bin_s: 1 }; // T-01, T-29
}

export function defaultEnemy(enemy: number = NO_KEY): EnemySetup {
  return { enemy, level: 1, steel_path: false, eximus: false, party: 1 }; // T-23
}

export function defaultScenario(enemy: number = NO_KEY, mcRuns?: number): ScenarioDoc {
  return {
    enemy: defaultEnemy(enemy),
    toggles: defaultToggles(),
    buffs: [],
    custom: {},
    conditions: { mode: "all_max", overrides: {} }, // T-02
    eval: defaultEval(mcRuns),
  };
}

export function newDocument(weapon: number = NO_KEY, enemy: number = NO_KEY, mcRuns?: number): CalcInput {
  return {
    v: DOC_VERSION,
    build: { weapon, variant: {}, slots: [] },
    scenario: defaultScenario(enemy, mcRuns),
  };
}

/** A document that can be sent to the server (weapon and enemy chosen). */
export function isComplete(doc: CalcInput): boolean {
  return doc.build.weapon !== NO_KEY && doc.scenario.enemy.enemy !== NO_KEY;
}
