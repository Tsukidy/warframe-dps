import type { ReactNode } from "react";
import type { EffectStatus } from "../../api/generated";
import { useT } from "../../i18n/I18nProvider";
import type { MessageId } from "../../i18n/messages";
import { cx } from "./cx";

export type BadgeTone = "neutral" | "accent" | "stat" | "success" | "warning" | "danger";

const TONES: Record<BadgeTone, string> = {
  neutral: "text-fg-2 border-border",
  accent: "text-accent border-accent",
  stat: "text-stat border-stat",
  success: "text-success border-success",
  warning: "text-warning border-warning",
  danger: "text-danger border-danger",
};

export function Badge({ tone = "neutral", children, className }: { tone?: BadgeTone; children: ReactNode; className?: string }) {
  return (
    <span
      className={cx(
        "inline-flex items-center gap-1 rounded border bg-surface-1 px-1.5 py-px text-[11px] leading-4 font-medium",
        TONES[tone],
        className,
      )}
    >
      {children}
    </span>
  );
}

const STATUS_TONE: Record<EffectStatus, BadgeTone> = {
  verified: "success",
  auto: "stat",
  draft: "warning",
  stale: "warning",
  unmodelled: "danger",
  no_damage_effect: "neutral",
};

/** The curation badge of an item (domain-model.md §15.3; frontend.md §7 "honest data"). */
export function StatusBadge({ status }: { status: EffectStatus }) {
  const t = useT();
  return <Badge tone={STATUS_TONE[status]}>{t(`design.badge.${status}` as MessageId)}</Badge>;
}
