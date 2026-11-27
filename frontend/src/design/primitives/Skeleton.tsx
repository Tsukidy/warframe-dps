import { useT } from "../../i18n/I18nProvider";
import { cx } from "./cx";

/** A loading placeholder block (announced once as "Loading"). */
export function Skeleton({ className, lines = 1 }: { className?: string; lines?: number }) {
  const t = useT();
  return (
    <div role="status" aria-label={t("design.skeleton.label")} className={cx("flex flex-col gap-1.5", className)}>
      {Array.from({ length: lines }, (_, i) => (
        <div key={i} className="h-3 animate-pulse rounded bg-surface-3 motion-reduce:animate-none" />
      ))}
    </div>
  );
}
