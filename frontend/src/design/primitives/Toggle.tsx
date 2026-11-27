import { Switch, type SwitchProps } from "react-aria-components";
import { cx, INTERACTIVE } from "./cx";

export interface ToggleProps extends Omit<SwitchProps, "className" | "children"> {
  label: string;
  hideLabel?: boolean;
  className?: string;
}

/** An on/off switch (React Aria Switch, role="switch"). */
export function Toggle({ label, hideLabel, className, ...rest }: ToggleProps) {
  return (
    <Switch {...rest} className={cx("group inline-flex min-h-6 items-center gap-2 text-sm text-fg-1", INTERACTIVE, className)}>
      <span
        aria-hidden
        className="relative inline-block h-5 w-9 rounded-full border border-border-strong bg-surface-3 transition-colors duration-[var(--motion)] group-data-[selected]:border-accent-fill group-data-[selected]:bg-accent-fill"
      >
        <span className="absolute top-0.5 left-0.5 size-3.5 rounded-full bg-fg-2 transition-transform duration-[var(--motion)] group-data-[selected]:translate-x-4 group-data-[selected]:bg-on-accent" />
      </span>
      <span className={cx(hideLabel && "sr-only")}>{label}</span>
    </Switch>
  );
}
