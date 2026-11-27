import type { ReactNode } from "react";
import { cx } from "./cx";

export interface CardProps {
  title?: ReactNode;
  actions?: ReactNode;
  children: ReactNode;
  className?: string;
  /** Heading level of the title (default 3). */
  level?: 2 | 3 | 4;
}

/** A card on `--surface-2` with an optional heading row. */
export function Card({ title, actions, children, className, level = 3 }: CardProps) {
  const H = `h${level}` as const;
  return (
    <section className={cx("rounded-lg border border-border bg-surface-2 p-3", className)}>
      {(title != null || actions != null) && (
        <div className="mb-2 flex items-center justify-between gap-2">
          {title != null && <H className="text-sm font-semibold text-fg-1">{title}</H>}
          {actions}
        </div>
      )}
      {children}
    </section>
  );
}
