// The top bar (frontend.md §3): logo · build name · Undo/Redo · Pin baseline · Share · Import · navigation · data
// version · theme.
import { Suspense, useState } from "react";
import { Link } from "@tanstack/react-router";
import { Input, Label, TextField } from "react-aria-components";
import { useAppStore } from "../../state/store";
import { selectCanRedo, selectCanUndo } from "../../state/selectors";
import { useT } from "../../i18n/I18nProvider";
import type { MessageId } from "../../i18n/messages";
import { Icon, type IconName } from "../../design/Icon";
import { Button, IconButton, cx } from "../../design/primitives";
import { ImportDialog, ShareDialog } from "../registry";
import { DataVersionBadge } from "./DataVersionBadge";
import { ThemeToggle } from "./ThemeToggle";
import { useCalcSession } from "./useCalcSession";

const NAV: { to: "/" | "/optimizer" | "/data" | "/about"; label: MessageId; icon: IconName }[] = [
  { to: "/", label: "shell.nav.calculator", icon: "calculator" },
  { to: "/optimizer", label: "shell.nav.optimizer", icon: "optimizer" },
  { to: "/data", label: "shell.nav.data", icon: "data" },
  { to: "/about", label: "shell.nav.about", icon: "about" },
];

function BuildName() {
  const t = useT();
  const name = useAppStore((s) => s.ui.buildName);
  const setUi = useAppStore((s) => s.setUi);
  return (
    <TextField value={name} onChange={(v) => setUi({ buildName: v })} className="min-w-24 flex-1 sm:flex-none">
      <Label className="sr-only">{t("shell.buildName.label")}</Label>
      <Input
        placeholder={t("shell.buildName.placeholder")}
        className="h-8 w-full min-w-0 rounded-md border border-transparent bg-transparent px-2 text-sm text-fg-1 outline-none placeholder:text-fg-3 hover:border-border focus:border-accent sm:w-44 lg:w-56"
      />
    </TextField>
  );
}

function HistoryButtons() {
  const t = useT();
  const canUndo = useAppStore(selectCanUndo);
  const canRedo = useAppStore(selectCanRedo);
  const undo = useAppStore((s) => s.undo);
  const redo = useAppStore((s) => s.redo);
  return (
    <div className="flex items-center">
      <IconButton icon="undo" label={t("shell.undo.shortcut")} isDisabled={!canUndo} onPress={undo} />
      <IconButton icon="redo" label={t("shell.redo.shortcut")} isDisabled={!canRedo} onPress={redo} />
    </div>
  );
}

function BaselineButton() {
  const t = useT();
  const pinned = useAppStore((s) => s.baseline !== null);
  const pin = useAppStore((s) => s.pinBaseline);
  const clear = useAppStore((s) => s.clearBaseline);
  const { report } = useCalcSession();
  return (
    <IconButton
      icon="pin"
      label={pinned ? t("shell.baseline.unpin") : t("shell.baseline.pin")}
      pressed={pinned}
      onPress={() => (pinned ? clear() : pin(report ?? null))}
    />
  );
}

function ShareImport() {
  const t = useT();
  const [open, setOpen] = useState<"share" | "import" | null>(null);
  return (
    <>
      <Button variant="ghost" size="sm" isDisabled={!ShareDialog} onPress={() => setOpen("share")}>
        <Icon name="share" size={14} />
        <span className="sr-only md:not-sr-only">{t("shell.share")}</span>
      </Button>
      <Button variant="ghost" size="sm" isDisabled={!ImportDialog} onPress={() => setOpen("import")}>
        <Icon name="import" size={14} />
        <span className="sr-only md:not-sr-only">{t("shell.import")}</span>
      </Button>
      <Suspense fallback={null}>
        {ShareDialog && open === "share" && <ShareDialog isOpen onOpenChange={(o) => setOpen(o ? "share" : null)} />}
        {ImportDialog && open === "import" && <ImportDialog isOpen onOpenChange={(o) => setOpen(o ? "import" : null)} />}
      </Suspense>
    </>
  );
}

function Nav() {
  const t = useT();
  return (
    <nav aria-label={t("shell.nav.label")} className="flex items-center gap-0.5">
      {NAV.map((n) => (
        <Link
          key={n.to}
          to={n.to}
          activeOptions={{ exact: n.to === "/", includeSearch: false }}
          className={cx(
            "inline-flex min-h-8 items-center gap-1.5 rounded-md px-2 text-sm text-fg-2 hover:bg-surface-2 hover:text-fg-1",
            "aria-[current=page]:bg-surface-2 aria-[current=page]:text-fg-1",
          )}
        >
          <Icon name={n.icon} size={14} />
          <span className="hidden lg:inline">{t(n.label)}</span>
          <span className="sr-only lg:hidden">{t(n.label)}</span>
        </Link>
      ))}
    </nav>
  );
}

export function TopBar() {
  const t = useT();
  return (
    <header className="flex min-h-12 shrink-0 flex-wrap items-center gap-x-2 gap-y-1 border-b border-border bg-surface-1 px-2 py-1 md:px-3">
      <Link to="/" className="flex items-center gap-2 rounded-md px-1 font-semibold text-fg-1">
        <svg width={20} height={20} viewBox="0 0 32 32" aria-hidden>
          <path d="M6 24 L12 8 L16 18 L20 8 L26 24" fill="none" stroke="var(--accent)" strokeWidth={3} strokeLinejoin="round" strokeLinecap="round" />
        </svg>
        <span className="sr-only sm:not-sr-only">{t("shell.appName")}</span>
      </Link>
      <BuildName />
      <div className="mx-1 hidden h-6 w-px bg-border sm:block" aria-hidden />
      <HistoryButtons />
      <BaselineButton />
      <div className="hidden items-center sm:flex">
        <ShareImport />
      </div>
      <div className="ml-auto flex items-center gap-1">
        <Nav />
        <DataVersionBadge />
        <ThemeToggle />
      </div>
    </header>
  );
}
