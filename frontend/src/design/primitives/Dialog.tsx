import type { ReactElement, ReactNode } from "react";
import { Button as AriaButton, Dialog as AriaDialog, DialogTrigger, Heading, Modal, ModalOverlay } from "react-aria-components";
import { useT } from "../../i18n/I18nProvider";
import { Icon } from "../Icon";
import { cx, INTERACTIVE } from "./cx";

export interface DialogProps {
  title: ReactNode;
  children: ReactNode | ((close: () => void) => ReactNode);
  /** Controlled open state (omit when used inside `DialogTrigger`). */
  isOpen?: boolean;
  onOpenChange?: (open: boolean) => void;
  size?: "sm" | "md" | "lg";
  /** Allow closing by clicking outside (default true). */
  isDismissable?: boolean;
}

const WIDTH = { sm: "max-w-sm", md: "max-w-lg", lg: "max-w-3xl" } as const;

/** A modal dialog (React Aria): focus trapped and restored, Escape closes (frontend.md §11). */
export function Dialog({ title, children, isOpen, onOpenChange, size = "md", isDismissable = true }: DialogProps) {
  const t = useT();
  return (
    <ModalOverlay
      isOpen={isOpen}
      onOpenChange={onOpenChange}
      isDismissable={isDismissable}
      className="fixed inset-0 z-40 flex items-center justify-center bg-black/50 p-4"
    >
      <Modal className={cx("w-full rounded-lg border border-border bg-surface-1 shadow-[var(--shadow-raised)]", WIDTH[size])}>
        <AriaDialog className="flex max-h-[85vh] flex-col outline-none">
          {({ close }) => (
            <>
              <div className="flex items-center justify-between gap-2 border-b border-border px-4 py-2">
                <Heading slot="title" className="text-base font-semibold">
                  {title}
                </Heading>
                <AriaButton
                  aria-label={t("design.dialog.close")}
                  onPress={close}
                  className={cx("inline-flex size-8 items-center justify-center rounded text-fg-2 data-[hovered]:bg-surface-2", INTERACTIVE)}
                >
                  <Icon name="close" />
                </AriaButton>
              </div>
              <div className="overflow-auto p-4">{typeof children === "function" ? children(close) : children}</div>
            </>
          )}
        </AriaDialog>
      </Modal>
    </ModalOverlay>
  );
}

/** Pairs a trigger (a React Aria Button) with a `Dialog`. */
export function DialogButton({ trigger, children }: { trigger: ReactElement; children: ReactElement<DialogProps> }) {
  return (
    <DialogTrigger>
      {trigger}
      {children}
    </DialogTrigger>
  );
}
