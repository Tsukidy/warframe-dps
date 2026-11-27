import type { ReactNode } from "react";
import { Tab, TabList, TabPanel, Tabs as AriaTabs, type Key } from "react-aria-components";
import { cx, INTERACTIVE } from "./cx";

export interface TabItem {
  id: string;
  label: ReactNode;
  content: ReactNode;
  isDisabled?: boolean;
}

export interface TabsProps {
  /** Accessible name of the tab list. */
  label: string;
  items: readonly TabItem[];
  selected?: string;
  onSelect?: (id: string) => void;
  className?: string;
}

/** Tabs with arrow-key navigation (React Aria). */
export function Tabs({ label, items, selected, onSelect, className }: TabsProps) {
  return (
    <AriaTabs
      selectedKey={selected}
      onSelectionChange={onSelect ? (k: Key) => onSelect(String(k)) : undefined}
      className={cx("flex flex-col", className)}
    >
      <TabList aria-label={label} className="flex gap-1 border-b border-border">
        {items.map((it) => (
          <Tab
            key={it.id}
            id={it.id}
            isDisabled={it.isDisabled}
            className={cx(
              "-mb-px min-h-8 border-b-2 border-transparent px-3 text-sm text-fg-2 data-[hovered]:text-fg-1 data-[selected]:border-accent data-[selected]:text-fg-1",
              INTERACTIVE,
            )}
          >
            {it.label}
          </Tab>
        ))}
      </TabList>
      {items.map((it) => (
        <TabPanel key={it.id} id={it.id} className="pt-3 outline-none">
          {it.content}
        </TabPanel>
      ))}
    </AriaTabs>
  );
}
