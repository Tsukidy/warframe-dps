import type { ReactNode } from "react";
import { Button as AriaButton, Dialog as AriaDialog, Heading, Modal, ModalOverlay } from "react-aria-components";
import { useT } from "../../i18n/I18nProvider";
import { Icon } from "../Icon";
import { cx, INTERACTIVE } from "./cx";

export interface SheetProps {
  title: ReactNode;
  children: ReactNode;
  isOpen?: boolean;
  onOpenChange?: (open: boolean) => void;
  side?: "right" | "bottom";
}

/** A side or bottom panel over the page (mod picker, explain drawer); modal, Escape closes. */
export function Sheet({ title, children, isOpen, onOpenChange, side = "right" }: SheetProps) {
  const t = useT();
  return (
    <ModalOverlay isOpen={isOpen} onOpenChange={onOpenChange} isDismissable className="fixed inset-0 z-40 bg-black/40">
      <Modal
        className={cx(
          "fixed border-border bg-surface-1 shadow-[var(--shadow-raised)]",
          side === "right"
            ? "top-0 right-0 h-full w-full max-w-md border-l"
            : "right-0 bottom-0 left-0 max-h-[80vh] rounded-t-lg border-t",
        )}
      >
        <AriaDialog className="flex h-full flex-col outline-none">
          {({ close }) => (
            <>
              <div className="flex items-center justify-between gap-2 border-b border-border px-4 py-2">
                <Heading slot="title" className="text-base font-semibold">
                  {title}
                </Heading>
                <AriaButton
                  aria-label={t("design.sheet.close")}
                  onPress={close}
                  className={cx("inline-flex size-8 items-center justify-center rounded text-fg-2 data-[hovered]:bg-surface-2", INTERACTIVE)}
                >
                  <Icon name="close" />
                </AriaButton>
              </div>
              <div className="min-h-0 flex-1 overflow-auto p-4">{children}</div>
            </>
          )}
        </AriaDialog>
      </Modal>
    </ModalOverlay>
  );
}
