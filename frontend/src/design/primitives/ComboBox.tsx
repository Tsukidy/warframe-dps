import type { ReactNode } from "react";
import {
  Button as AriaButton,
  ComboBox as AriaComboBox,
  Input,
  Label,
  ListBox,
  ListBoxItem,
  Popover,
  type Key,
} from "react-aria-components";
import { Icon } from "../Icon";
import { cx, INTERACTIVE } from "./cx";

export interface ComboBoxItem {
  id: string | number;
  /** Plain text used for type-ahead and the input value. */
  textValue: string;
  /** Rendered row; defaults to `textValue`. */
  content?: ReactNode;
  isDisabled?: boolean;
}

export interface ComboBoxProps {
  label: string;
  items: readonly ComboBoxItem[];
  selectedKey?: string | number | null;
  onSelectionChange?: (key: string | number | null) => void;
  /** Controlled input text (for server-side or fuzzy filtering, set `filterItems={false}` semantics by passing filtered items). */
  inputValue?: string;
  onInputChange?: (value: string) => void;
  placeholder?: string;
  hideLabel?: boolean;
  /** Empty-state text inside the list. */
  emptyText?: string;
  className?: string;
}

/**
 * A searchable picker (React Aria ComboBox: arrows, Enter, Escape, screen-reader announcements). When
 * `inputValue` is controlled the caller filters `items` (fuzzy search over localized names, frontend.md §6 row 1);
 * otherwise React Aria filters by substring.
 */
export function ComboBox({
  label,
  items,
  selectedKey,
  onSelectionChange,
  inputValue,
  onInputChange,
  placeholder,
  hideLabel,
  emptyText,
  className,
}: ComboBoxProps) {
  const controlled = inputValue !== undefined;
  return (
    <AriaComboBox
      items={items}
      selectedKey={selectedKey ?? null}
      onSelectionChange={(k: Key | null) => onSelectionChange?.(k)}
      inputValue={inputValue}
      onInputChange={onInputChange}
      defaultFilter={controlled ? () => true : undefined}
      menuTrigger="focus"
      className={cx("flex flex-col gap-1", className)}
    >
      <Label className={cx("text-xs text-fg-2", hideLabel && "sr-only")}>{label}</Label>
      <div className="flex h-8 items-stretch overflow-hidden rounded-md border border-border bg-surface-1 focus-within:border-accent">
        <Input placeholder={placeholder} className="w-full min-w-0 bg-transparent px-2 text-sm text-fg-1 outline-none placeholder:text-fg-3" />
        <AriaButton className={cx("px-2 text-fg-2", INTERACTIVE)}>
          <Icon name="chevron-down" size={14} />
        </AriaButton>
      </div>
      <Popover className="w-[var(--trigger-width)] overflow-auto rounded-md border border-border bg-surface-1 shadow-[var(--shadow-raised)]">
        <ListBox
          className="max-h-80 p-1 outline-none"
          renderEmptyState={() => <div className="p-2 text-sm text-fg-3">{emptyText}</div>}
        >
          {(item: ComboBoxItem) => (
            <ListBoxItem
              id={item.id}
              textValue={item.textValue}
              isDisabled={item.isDisabled}
              className="flex min-h-8 cursor-default items-center gap-2 rounded px-2 text-sm text-fg-1 outline-none data-[disabled]:text-fg-3 data-[focused]:bg-surface-3 data-[selected]:font-semibold"
            >
              {item.content ?? item.textValue}
            </ListBoxItem>
          )}
        </ListBox>
      </Popover>
    </AriaComboBox>
  );
}
