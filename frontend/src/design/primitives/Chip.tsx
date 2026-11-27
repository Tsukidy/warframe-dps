import type { ReactNode } from "react";
import { Button as AriaButton } from "react-aria-components";
import { Icon } from "../Icon";
import { cx, INTERACTIVE } from "./cx";

export interface ChipProps {
  children: ReactNode;
  /** Shows a remove button with this accessible name. */
  removeLabel?: string;
  onRemove?: () => void;
  /** Colour token for a leading stripe (a damage type, `var(--dt-HEA)`). */
  stripe?: string;
  className?: string;
}

/** A compact token (filters, prohibited mods, elements), optionally removable. */
export function Chip({ children, removeLabel, onRemove, stripe, className }: ChipProps) {
  return (
    <span
      className={cx(
        "inline-flex min-h-6 items-center gap-1 rounded-full border border-border bg-surface-2 pr-1 pl-2 text-xs text-fg-1",
        className,
      )}
      style={stripe ? { boxShadow: `inset 3px 0 0 ${stripe}` } : undefined}
    >
      {children}
      {onRemove && removeLabel && (
        <AriaButton
          aria-label={removeLabel}
          onPress={onRemove}
          className={cx("inline-flex size-5 items-center justify-center rounded-full text-fg-2 data-[hovered]:bg-surface-3", INTERACTIVE)}
        >
          <Icon name="close" size={10} />
        </AriaButton>
      )}
    </span>
  );
}
