import {
  Button as AriaButton,
  FieldError,
  Group,
  Input,
  Label,
  NumberField as AriaNumberField,
  Text,
  type NumberFieldProps as AriaNumberFieldProps,
} from "react-aria-components";
import { Icon } from "../Icon";
import { cx, INTERACTIVE } from "./cx";

export interface NumberFieldProps extends Omit<AriaNumberFieldProps, "className" | "children"> {
  label: string;
  description?: string;
  /** Unit shown after the input (`s`, `m`); part of the accessible description. */
  unit?: string;
  hideLabel?: boolean;
  className?: string;
}

/**
 * A locale-aware number input (React Aria NumberField): arrow keys and steppers, clamped to min/max, label and
 * unit linked to the input (frontend.md §11).
 */
export function NumberField({ label, description, unit, hideLabel, className, ...rest }: NumberFieldProps) {
  return (
    <AriaNumberField {...rest} className={cx("flex flex-col gap-1", className)}>
      <Label className={cx("text-xs text-fg-2", hideLabel && "sr-only")}>{label}</Label>
      <Group className="flex h-8 items-stretch overflow-hidden rounded-md border border-border bg-surface-1 data-[focus-within]:border-accent">
        <AriaButton slot="decrement" className={cx("px-1.5 text-fg-2 data-[hovered]:bg-surface-2", INTERACTIVE)}>
          <Icon name="minus" size={12} />
        </AriaButton>
        <Input className="num w-full min-w-0 bg-transparent px-1 text-right text-fg-1 outline-none" />
        {unit != null && (
          <span className="flex items-center pr-1 text-xs text-fg-3" aria-hidden>
            {unit}
          </span>
        )}
        <AriaButton slot="increment" className={cx("px-1.5 text-fg-2 data-[hovered]:bg-surface-2", INTERACTIVE)}>
          <Icon name="plus" size={12} />
        </AriaButton>
      </Group>
      {description != null && (
        <Text slot="description" className="text-xs text-fg-3">
          {description}
        </Text>
      )}
      <FieldError className="text-xs text-danger" />
    </AriaNumberField>
  );
}
