// The data version in the top bar (frontend.md §3 "Data vX"), linking to the data page.
import { Link } from "@tanstack/react-router";
import { useMeta } from "../../api/queries";
import { useT } from "../../i18n/I18nProvider";
import { Icon } from "../../design/Icon";
import { cx } from "../../design/primitives";

export function DataVersionBadge() {
  const t = useT();
  const meta = useMeta();
  const text = meta.isError
    ? t("shell.dataVersion.offline")
    : meta.data
      ? t("shell.dataVersion", { version: meta.data.data_version })
      : t("shell.dataVersion.loading");
  return (
    <Link
      to="/data"
      className={cx(
        "inline-flex min-h-8 items-center gap-1.5 rounded-md border border-border px-2 text-xs",
        meta.isError ? "text-danger" : "text-fg-2 hover:text-fg-1",
      )}
      data-testid="data-version"
    >
      <Icon name={meta.isError ? "warning" : "data"} size={14} />
      <span className="num sr-only md:not-sr-only">{text}</span>
      {__WFD_MOCKS__ && <span className="sr-only text-warning lg:not-sr-only">{t("shell.dataVersion.mock")}</span>}
    </Link>
  );
}
