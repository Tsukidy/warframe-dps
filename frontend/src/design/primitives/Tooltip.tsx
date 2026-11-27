import type { ReactElement, ReactNode } from "react";
import { Focusable, OverlayArrow, Tooltip as AriaTooltip, TooltipTrigger } from "react-aria-components";

export interface TooltipProps {
  content: ReactNode;
  /** A focusable element (a React Aria Button, or an element wrapped by `Focusable`). */
  children: ReactElement;
  delay?: number;
  placement?: "top" | "bottom" | "start" | "end";
}

/** A hover/focus tooltip (React Aria): shown on keyboard focus too, closed with Escape. */
export function Tooltip({ content, children, delay = 600, placement = "bottom" }: TooltipProps) {
  return (
    <TooltipTrigger delay={delay}>
      {children}
      <AriaTooltip
        placement={placement}
        offset={6}
        className="max-w-xs rounded-md border border-border bg-surface-3 px-2 py-1 text-xs text-fg-1 shadow-[var(--shadow-raised)]"
      >
        <OverlayArrow>
          <svg width={8} height={8} viewBox="0 0 8 8" className="fill-surface-3" aria-hidden>
            <path d="M0 0 L4 4 L8 0" />
          </svg>
        </OverlayArrow>
        {content}
      </AriaTooltip>
    </TooltipTrigger>
  );
}

export { Focusable };
