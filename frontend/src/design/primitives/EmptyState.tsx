import type { ReactNode } from "react";
import { Icon, type IconName } from "../Icon";
import { cx } from "./cx";

/** A calm placeholder for empty panes and lists ("accepts: …", "pick an enemy"). */
export function EmptyState({
  icon = "info",
  title,
  children,
  className,
}: {
  icon?: IconName;
  title: ReactNode;
  children?: ReactNode;
  className?: string;
}) {
  return (
    <div className={cx("flex flex-col items-center justify-center gap-2 rounded-lg border border-dashed border-border p-6 text-center", className)}>
      <Icon name={icon} size={24} className="text-fg-3" />
      <p className="m-0 text-sm font-medium text-fg-2">{title}</p>
      {children != null && <div className="text-xs text-fg-3">{children}</div>}
    </div>
  );
}
