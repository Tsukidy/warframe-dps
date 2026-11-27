// The boot code's decode result (frontend.md §9): a URL `?b=` code or the stored last build that failed to decode
// (the calculator started a new build instead), or decoded with warnings (retired items, unknown local items,
// migrated version), shown as a dismissible notice above the panes. The result lives in the store (`bootDecode`), so
// the share/import feature can show the same report and dismiss it.
import { useMemo } from "react";
import type { Catalog, Message } from "../../api/generated";
import { useCatalog } from "../../api/queries";
import { useServerMessage, useT, type NameResolver } from "../../i18n/I18nProvider";
import { Icon } from "../../design/Icon";
import { IconButton, cx } from "../../design/primitives";
import { reportHasEntries, useAppStore, type BootDecode } from "../../state/store";

function nameResolver(catalog: Catalog | undefined): NameResolver | undefined {
  if (!catalog) return undefined;
  const names = new Map<number, string>();
  for (const list of [catalog.weapons, catalog.items, catalog.enemies, catalog.frames]) {
    for (const x of list) names.set(x.key, x.name);
  }
  return (key) => names.get(key);
}

/** Every line of a decode result as server messages. */
export function bootMessages(b: BootDecode): Message[] {
  if (b.error) return [b.error, ...(b.issues ?? [])];
  const r = b.report;
  if (!r) return [];
  return [
    ...r.warnings,
    ...r.retired_items.map((key): Message => ({ key: "import.retired_item", args: { key } })),
    ...r.unmapped.map((u): Message => ({ key: "import.unmapped", args: { field: u.field, value: u.value } })),
    ...r.converted,
    ...r.defaults_applied,
  ];
}

export function BootNotice() {
  const t = useT();
  const boot = useAppStore((s) => s.bootDecode);
  const dismiss = useAppStore((s) => s.setBootDecode);
  const catalog = useCatalog();
  const resolve = useMemo(() => nameResolver(catalog.data), [catalog.data]);
  const fmt = useServerMessage(resolve);
  if (!boot || (!boot.error && !reportHasEntries(boot.report))) return null;
  const failed = boot.error !== undefined;
  return (
    <div
      role={failed ? "alert" : "status"}
      data-testid="boot-notice"
      className={cx(
        "mx-2 mt-2 flex items-start gap-2 rounded-md border bg-surface-1 px-3 py-2 text-sm md:mx-3",
        failed ? "border-danger" : "border-warning",
      )}
    >
      <Icon name="warning" size={16} className={cx("mt-0.5 shrink-0", failed ? "text-danger" : "text-warning")} />
      <div className="min-w-0 flex-1">
        <p className="m-0 font-medium text-fg-1">{failed ? t("shell.boot.failed") : t("shell.boot.report")}</p>
        <ul className="m-0 mt-1 list-disc pl-4 text-xs text-fg-2">
          {bootMessages(boot).map((m, i) => (
            <li key={i}>{fmt(m)}</li>
          ))}
        </ul>
      </div>
      <IconButton icon="close" label={t("shell.toast.dismiss")} onPress={() => dismiss(null)} />
    </div>
  );
}
