import type { ReactNode } from "react";
import { Icon } from "../Icon";
import { cx } from "./cx";

export interface StatRowProps {
  label: ReactNode;
  value: ReactNode;
  /** Optional base value shown before the modded one (base → modded). */
  base?: ReactNode;
  delta?: ReactNode;
  className?: string;
}

/** A label/value row with tabular numbers; `base` renders as "base → value". */
export function StatRow({ label, value, base, delta, className }: StatRowProps) {
  return (
    <div className={cx("flex items-baseline justify-between gap-3 py-0.5", className)}>
      <dt className="text-fg-2">{label}</dt>
      <dd className="num-table m-0 flex items-baseline gap-1.5 text-fg-1">
        {base != null && (
          <>
            <span className="text-fg-3">{base}</span>
            <Icon name="chevron-right" size={12} className="self-center text-fg-3" />
          </>
        )}
        <span>{value}</span>
        {delta}
      </dd>
    </div>
  );
}
