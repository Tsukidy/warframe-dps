import { ToggleButton, ToggleButtonGroup, type Key } from "react-aria-components";
import { cx, INTERACTIVE } from "./cx";

export interface SegmentedOption<V extends string> {
  value: V;
  label: string;
}

export interface SegmentedProps<V extends string> {
  /** Accessible name of the group. */
  label: string;
  options: readonly SegmentedOption<V>[];
  value: V;
  onChange: (value: V) => void;
  isDisabled?: boolean;
  className?: string;
}

/** A single-choice segmented control (React Aria ToggleButtonGroup, one selection, never empty). */
export function Segmented<V extends string>({ label, options, value, onChange, isDisabled, className }: SegmentedProps<V>) {
  return (
    <ToggleButtonGroup
      aria-label={label}
      selectionMode="single"
      disallowEmptySelection
      selectedKeys={[value]}
      isDisabled={isDisabled}
      onSelectionChange={(keys: Set<Key>) => {
        const [k] = [...keys];
        if (k != null) onChange(String(k) as V);
      }}
      className={cx("inline-flex rounded-md border border-border bg-surface-1 p-0.5", className)}
    >
      {options.map((o) => (
        <ToggleButton
          key={o.value}
          id={o.value}
          className={cx(
            "min-h-7 rounded px-2.5 text-xs text-fg-2 data-[hovered]:text-fg-1 data-[selected]:bg-surface-3 data-[selected]:text-fg-1",
            INTERACTIVE,
          )}
        >
          {o.label}
        </ToggleButton>
      ))}
    </ToggleButtonGroup>
  );
}
