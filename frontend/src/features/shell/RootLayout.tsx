// The app shell around every route: skip link, top bar, main landmark, session effects and global shortcuts.
import { useEffect } from "react";
import { Link, Outlet } from "@tanstack/react-router";
import { useAppStore } from "../../state/store";
import { useT } from "../../i18n/I18nProvider";
import { EmptyState } from "../../design/primitives";
import { TopBar } from "./TopBar";
import { CalcSessionEffects } from "./useCalcSession";
import { useApplyTheme } from "./ThemeToggle";

/** True when the event target edits text (native undo applies there). */
function inTextField(target: EventTarget | null): boolean {
  if (!(target instanceof HTMLElement)) return false;
  return target.isContentEditable || target instanceof HTMLInputElement || target instanceof HTMLTextAreaElement;
}

/** Ctrl+Z / Ctrl+Shift+Z / Ctrl+Y (frontend.md §5). */
export function useHistoryShortcuts(): void {
  const undo = useAppStore((s) => s.undo);
  const redo = useAppStore((s) => s.redo);
  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if (!(e.ctrlKey || e.metaKey) || e.altKey || inTextField(e.target)) return;
      const k = e.key.toLowerCase();
      if (k === "z" && !e.shiftKey) {
        e.preventDefault();
        undo();
      } else if ((k === "z" && e.shiftKey) || k === "y") {
        e.preventDefault();
        redo();
      }
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [undo, redo]);
}

export function RootLayout() {
  const t = useT();
  useApplyTheme();
  useHistoryShortcuts();
  return (
    <div className="flex h-full flex-col bg-surface-0 text-fg-1">
      <a
        href="#main"
        className="sr-only-focusable absolute top-1 left-1 z-50 rounded-md bg-accent-fill px-3 py-1 text-on-accent"
      >
        {t("shell.skipToContent")}
      </a>
      <h1 className="sr-only">{t("shell.appName")}</h1>
      <TopBar />
      <main id="main" tabIndex={-1} className="flex min-h-0 flex-1 flex-col outline-none">
        <Outlet />
      </main>
      <CalcSessionEffects />
    </div>
  );
}

export function NotFound() {
  const t = useT();
  return (
    <div className="flex flex-1 items-center justify-center p-6">
      <EmptyState icon="warning" title={t("shell.notFound.title")}>
        <Link to="/" className="text-accent underline">
          {t("shell.notFound.back")}
        </Link>
      </EmptyState>
    </div>
  );
}

export function RouteError({ error }: { error: unknown }) {
  const t = useT();
  return (
    <div className="flex flex-1 items-center justify-center p-6">
      <EmptyState icon="warning" title={t("shell.error.title")}>
        <pre className="max-w-xl overflow-auto text-left whitespace-pre-wrap">{String(error)}</pre>
      </EmptyState>
    </div>
  );
}
