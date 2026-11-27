import { useFormat } from "../../i18n/format";
import { useT } from "../../i18n/I18nProvider";
import { Icon } from "../Icon";
import { cx } from "./cx";

export interface DeltaBadgeProps {
  /** Relative change as a fraction (0.12 = +12 %). */
  delta: number;
  /** When higher is worse (time to kill), the colours swap. */
  lowerIsBetter?: boolean;
  className?: string;
}

/** Δ against the baseline (frontend.md §10): muted colour, arrow glyph, signed percent; not colour alone. */
export function DeltaBadge({ delta, lowerIsBetter = false, className }: DeltaBadgeProps) {
  const f = useFormat();
  const t = useT();
  const text = f.deltaPercent(delta);
  const same = Math.abs(delta) < 5e-4;
  const good = same ? null : (delta > 0) !== lowerIsBetter;
  const label = same
    ? t("common.delta.same")
    : delta > 0
      ? t("common.delta.up", { delta: text })
      : t("common.delta.down", { delta: text });
  return (
    <span
      className={cx(
        "num-table inline-flex items-center gap-0.5",
        good === null ? "text-fg-3" : good ? "text-success" : "text-danger",
        className,
      )}
      aria-label={t("design.delta.label", { delta: label })}
    >
      {!same && <Icon name={delta > 0 ? "arrow-up" : "arrow-down"} size={12} />}
      <span aria-hidden>{text}</span>
    </span>
  );
}
