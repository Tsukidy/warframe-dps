// Feature registry: the shell plugs in the panes, pages and dialogs that feature tasks own (work-breakdown.md §8:
// loadout and share T30, target and results T31, optimizer, data and about T35) by file name, so adding a feature
// needs no edit of a shell file. A missing module leaves a placeholder (empty state, disabled button).
//
// Contract (CONTRACTS.md, T12): each file below exports a component of the same name.
//   loadout/LoadoutPane.tsx   export function LoadoutPane(): JSX
//   results/ResultsPane.tsx   export function ResultsPane(): JSX
//   target/TargetPane.tsx     export function TargetPane(): JSX
//   optimizer/OptimizerPage.tsx, data/DataPage.tsx, about/AboutPage.tsx   export function <Name>(): JSX  (lazy)
//   share/ShareDialog.tsx, share/ImportDialog.tsx   export function <Name>(props: DialogSlotProps): JSX  (lazy)
import { lazy, type ComponentType, type LazyExoticComponent } from "react";

export interface DialogSlotProps {
  isOpen: boolean;
  onOpenChange: (open: boolean) => void;
}

type Module = Record<string, unknown>;

const panes = import.meta.glob<Module>(
  ["./loadout/LoadoutPane.tsx", "./results/ResultsPane.tsx", "./target/TargetPane.tsx"],
  { eager: true },
);

const lazyModules = import.meta.glob<Module>([
  "./optimizer/OptimizerPage.tsx",
  "./data/DataPage.tsx",
  "./about/AboutPage.tsx",
  "./share/ShareDialog.tsx",
  "./share/ImportDialog.tsx",
]);

function eagerExport<P>(path: string, name: string): ComponentType<P> | undefined {
  const m = panes[path];
  const c = m?.[name];
  return typeof c === "function" ? (c as ComponentType<P>) : undefined;
}

function lazyExport<P extends object>(path: string, name: string): LazyExoticComponent<ComponentType<P>> | undefined {
  const load = lazyModules[path];
  if (!load) return undefined;
  return lazy(async () => {
    const m = await load();
    const c = m[name];
    if (typeof c !== "function") throw new Error(`${path} does not export ${name}`);
    return { default: c as ComponentType<P> };
  });
}

export const LoadoutPane = eagerExport<object>("./loadout/LoadoutPane.tsx", "LoadoutPane");
export const ResultsPane = eagerExport<object>("./results/ResultsPane.tsx", "ResultsPane");
export const TargetPane = eagerExport<object>("./target/TargetPane.tsx", "TargetPane");

export const OptimizerPage = lazyExport<object>("./optimizer/OptimizerPage.tsx", "OptimizerPage");
export const DataPage = lazyExport<object>("./data/DataPage.tsx", "DataPage");
export const AboutPage = lazyExport<object>("./about/AboutPage.tsx", "AboutPage");
export const ShareDialog = lazyExport<DialogSlotProps>("./share/ShareDialog.tsx", "ShareDialog");
export const ImportDialog = lazyExport<DialogSlotProps>("./share/ImportDialog.tsx", "ImportDialog");
