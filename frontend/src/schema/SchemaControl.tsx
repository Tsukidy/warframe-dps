// Schema-driven controls (AD-17, frontend.md §4 OptionsPanel/BuffsPanel "schema-driven"): one control per
// `ToggleSchema`, bound to the document by its field path. A new toggle in the schema renders without code
// changes as long as its value is a bool, a number, a percent, a select or an optional number.
import { Button as AriaButton, Label, ListBox, ListBoxItem, Popover, Select, SelectValue, type Key } from "react-aria-components";
import type { ToggleSchema } from "../api/generated";
import { useAppStore } from "../state/store";
import { getPath } from "../state/paths";
import { useDynamicT, useT } from "../i18n/I18nProvider";
import { Icon } from "../design/Icon";
import { NumberField, PercentField, Segmented, Toggle } from "../design/primitives";
import { cx, INTERACTIVE } from "../design/primitives/cx";
import {
  fromNumber,
  isGenericField,
  isOn,
  isVisible,
  NUMBER_MEMBER,
  switchedOnValue,
  toNumber,
  type VisibilityContext,
} from "./visibility";

export interface SchemaControlProps {
  toggle: ToggleSchema;
  /** Current value; defaults to the document value at `toggle.field`. */
  value?: unknown;
  /** Called with the new document value; defaults to the store's `setToggle(field, value)`. */
  onChange?: (value: unknown) => void;
}

function useBinding(toggle: ToggleSchema, value: unknown, onChange?: (v: unknown) => void) {
  const docValue = useAppStore((s) => getPath(s.doc, toggle.field));
  const setToggle = useAppStore((s) => s.setToggle);
  return {
    current: value !== undefined ? value : docValue !== undefined ? docValue : toggle.default,
    write: onChange ?? ((v: unknown) => setToggle(toggle.field, v)),
  };
}

const SEGMENTED_MAX = 4;

/** One control for one toggle descriptor. Returns null for fields a generic control cannot bind. */
export function SchemaControl({ toggle, value, onChange }: SchemaControlProps) {
  const dt = useDynamicT();
  const t = useT();
  const { current, write } = useBinding(toggle, value, onChange);
  if (!isGenericField(toggle)) return null;
  const label = dt(toggle.label);
  const help = dt(toggle.help, undefined, "");
  const c = toggle.control;
  const description = help.length > 0 ? help : undefined;
  const common = { minValue: c.min, maxValue: c.max, step: c.step };

  switch (c.kind) {
    case "bool":
      return <Toggle label={label} isSelected={current === true} onChange={(v) => write(v)} />;
    case "number":
      return (
        <NumberField
          label={label}
          description={description}
          value={toNumber(toggle.field, current) ?? Number.NaN}
          onChange={(n) => write(Number.isNaN(n) ? toggle.default : fromNumber(toggle.field, n))}
          {...common}
        />
      );
    case "percent":
      return (
        <PercentField
          label={label}
          description={description}
          value={toNumber(toggle.field, current) ?? Number.NaN}
          onChange={(n) => write(Number.isNaN(n) ? toggle.default : fromNumber(toggle.field, n))}
          minValue={c.min}
          maxValue={c.max}
        />
      );
    case "optional_number": {
      const n = toNumber(toggle.field, current);
      const on = isOn(toggle.field, current);
      // Object-member fields (T-08, T-09) leave the number to the server's default until one is entered.
      const serverDefault = NUMBER_MEMBER[toggle.field] !== undefined;
      const fallback = toNumber(toggle.field, toggle.default) ?? c.min ?? 0;
      const cleared = () => (serverDefault ? {} : fromNumber(toggle.field, fallback));
      return (
        <div className="flex flex-col gap-1">
          <Toggle
            label={label}
            isSelected={on}
            onChange={(v) => write(v ? switchedOnValue(toggle.field, fallback) : null)}
          />
          {on && (
            <NumberField
              label={label}
              hideLabel
              description={description}
              value={n ?? Number.NaN}
              placeholder={serverDefault ? t("shell.schema.serverDefault") : undefined}
              onChange={(x) => write(Number.isNaN(x) ? cleared() : fromNumber(toggle.field, x))}
              {...common}
            />
          )}
        </div>
      );
    }
    case "select": {
      const options = (c.options ?? []).map((o) => ({ value: o.value, label: dt(o.label, undefined, o.value) }));
      const selected =
        typeof current === "string" ? current : typeof toggle.default === "string" ? toggle.default : (options[0]?.value ?? "");
      if (options.length <= SEGMENTED_MAX) {
        return (
          <div className="flex flex-col gap-1">
            <span className="text-xs text-fg-2" aria-hidden>
              {label}
            </span>
            <Segmented label={label} options={options} value={selected} onChange={(v) => write(v)} />
          </div>
        );
      }
      return (
        <Select
          selectedKey={selected}
          onSelectionChange={(k: Key | null) => k != null && write(String(k))}
          className="flex flex-col gap-1"
        >
          <Label className="text-xs text-fg-2">{label}</Label>
          <AriaButton className={cx("flex h-8 items-center justify-between rounded-md border border-border bg-surface-1 px-2 text-sm", INTERACTIVE)}>
            <SelectValue />
            <Icon name="chevron-down" size={14} />
          </AriaButton>
          <Popover className="min-w-[var(--trigger-width)] rounded-md border border-border bg-surface-1 shadow-[var(--shadow-raised)]">
            <ListBox className="p-1 outline-none">
              {options.map((o) => (
                <ListBoxItem
                  key={o.value}
                  id={o.value}
                  className="flex min-h-8 items-center rounded px-2 text-sm outline-none data-[focused]:bg-surface-3 data-[selected]:font-semibold"
                >
                  {o.label}
                </ListBoxItem>
              ))}
            </ListBox>
          </Popover>
        </Select>
      );
    }
  }
}

export interface SchemaFieldsProps {
  toggles: readonly ToggleSchema[];
  context: VisibilityContext;
  /** Only toggles of these groups (all when absent). */
  groups?: readonly string[];
  className?: string;
}

/** Renders every visible, generically bindable toggle of the schema in schema order. */
export function SchemaFields({ toggles, context, groups, className }: SchemaFieldsProps) {
  const shown = toggles.filter(
    (t) => (!groups || groups.includes(t.group)) && isGenericField(t) && isVisible(t.visible_if, t.advanced, context),
  );
  return (
    <div className={cx("flex flex-col gap-3", className)}>
      {shown.map((t) => (
        <SchemaControl key={t.id} toggle={t} />
      ))}
    </div>
  );
}
