import type { ReactNode } from "react";
import { Button as AriaButton, Dialog, DialogTrigger, OverlayArrow, Popover } from "react-aria-components";
import { useT } from "../../i18n/I18nProvider";
import { Icon } from "../Icon";
import { cx, INTERACTIVE } from "./cx";

export interface InfoPopoverProps {
  children: ReactNode;
  /** Accessible name of the trigger; defaults to "More information". */
  label?: string;
  /** A custom trigger; by default an info icon button. */
  trigger?: ReactNode;
}

/** Click/Enter-opened explanation (item info, layer explanations); Escape closes, focus returns to the trigger. */
export function InfoPopover({ children, label, trigger }: InfoPopoverProps) {
  const t = useT();
  return (
    <DialogTrigger>
      {trigger ?? (
        <AriaButton
          aria-label={label ?? t("design.info.label")}
          className={cx("inline-flex size-6 items-center justify-center rounded text-fg-3 data-[hovered]:text-fg-1", INTERACTIVE)}
        >
          <Icon name="info" size={14} />
        </AriaButton>
      )}
      <Popover
        offset={6}
        className="max-w-sm rounded-md border border-border bg-surface-3 p-3 text-sm text-fg-1 shadow-[var(--shadow-raised)]"
      >
        <OverlayArrow>
          <svg width={10} height={10} viewBox="0 0 10 10" className="fill-surface-3" aria-hidden>
            <path d="M0 0 L5 5 L10 0" />
          </svg>
        </OverlayArrow>
        <Dialog aria-label={label ?? t("design.info.label")} className="outline-none">
          {children}
        </Dialog>
      </Popover>
    </DialogTrigger>
  );
}
