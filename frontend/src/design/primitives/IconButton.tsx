import { Button as AriaButton, type ButtonProps as AriaButtonProps } from "react-aria-components";
import { Icon, type IconName } from "../Icon";
import { cx, INTERACTIVE } from "./cx";
import { Tooltip } from "./Tooltip";

export interface IconButtonProps extends Omit<AriaButtonProps, "className" | "children"> {
  icon: IconName;
  /** Accessible name; also shown as the tooltip. */
  label: string;
  /** Pressed state for toggle-like buttons. */
  pressed?: boolean;
  className?: string;
}

/** A square icon-only button with a tooltip carrying its name (at least 32 × 32 px). */
export function IconButton({ icon, label, pressed, className, ...rest }: IconButtonProps) {
  return (
    <Tooltip content={label}>
      <AriaButton
        {...rest}
        aria-label={label}
        aria-pressed={pressed}
        className={cx(
          "inline-flex size-8 items-center justify-center rounded-md text-fg-2 data-[hovered]:bg-surface-2 data-[hovered]:text-fg-1 data-[pressed]:bg-surface-3",
          pressed && "bg-surface-3 text-fg-1",
          INTERACTIVE,
          className,
        )}
      >
        <Icon name={icon} />
      </AriaButton>
    </Tooltip>
  );
}
