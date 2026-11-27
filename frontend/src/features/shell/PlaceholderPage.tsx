// A plain page frame for /optimizer, /data and /about (frontend.md §3); shows the feature's page when it exists
// (T35) and a placeholder until then.
import { Suspense, type ComponentType } from "react";
import { useT } from "../../i18n/I18nProvider";
import type { MessageId } from "../../i18n/messages";
import { EmptyState, Skeleton } from "../../design/primitives";

export function PageFrame({ titleId, page: Page }: { titleId: MessageId; page?: ComponentType }) {
  const t = useT();
  return (
    <div className="mx-auto flex w-full max-w-6xl min-h-0 flex-1 flex-col overflow-y-auto p-4">
      <h2 className="mt-0 mb-4 text-lg font-semibold">{t(titleId)}</h2>
      {Page ? (
        <Suspense fallback={<Skeleton lines={6} />}>
          <Page />
        </Suspense>
      ) : (
        <EmptyState title={t("shell.placeholder.page")} />
      )}
    </div>
  );
}
