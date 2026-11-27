import { Button as AriaButton, type ButtonProps as AriaButtonProps } from "react-aria-components";
import { cx, INTERACTIVE } from "./cx";

export type ButtonVariant = "primary" | "secondary" | "ghost" | "danger";

const VARIANTS: Record<ButtonVariant, string> = {
  primary: "bg-accent-fill text-on-accent data-[hovered]:brightness-110 data-[pressed]:brightness-95",
  secondary:
    "bg-surface-2 text-fg-1 border border-border data-[hovered]:bg-surface-3 data-[pressed]:bg-surface-3",
  ghost: "bg-transparent text-fg-1 data-[hovered]:bg-surface-2 data-[pressed]:bg-surface-3",
  danger: "bg-transparent text-danger border border-danger data-[hovered]:bg-surface-2",
};

export interface ButtonProps extends Omit<AriaButtonProps, "className"> {
  variant?: ButtonVariant;
  size?: "sm" | "md";
  className?: string;
}

/** A button (React Aria): press events, keyboard and focus ring handled. */
export function Button({ variant = "secondary", size = "md", className, ...rest }: ButtonProps) {
  return (
    <AriaButton
      {...rest}
      className={cx(
        "inline-flex items-center justify-center gap-1.5 rounded-md font-medium transition-colors duration-[var(--motion)] cursor-default select-none",
        size === "sm" ? "min-h-7 px-2 text-xs" : "min-h-8 px-3 text-sm",
        VARIANTS[variant],
        INTERACTIVE,
        className,
      )}
    />
  );
}
