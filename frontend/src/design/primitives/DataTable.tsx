import type { ReactNode } from "react";
import { Cell, Column, Row, Table, TableBody, TableHeader } from "react-aria-components";
import { useT } from "../../i18n/I18nProvider";
import { cx } from "./cx";

export interface DataColumn<R> {
  id: string;
  header: ReactNode;
  cell: (row: R) => ReactNode;
  /** Numeric columns align right with tabular figures. */
  numeric?: boolean;
  isRowHeader?: boolean;
}

export interface DataTableProps<R> {
  /** Accessible name of the table. */
  label: string;
  columns: readonly DataColumn<R>[];
  rows: readonly R[];
  rowKey: (row: R, index: number) => string | number;
  className?: string;
}

/** An accessible data table (React Aria Table; also the "show as table" view of every chart, frontend.md §11). */
export function DataTable<R>({ label, columns, rows, rowKey, className }: DataTableProps<R>) {
  const t = useT();
  const keyed = rows.map((r, i) => ({ id: rowKey(r, i), row: r }));
  return (
    <Table aria-label={label} className={cx("w-full border-collapse text-sm", className)}>
      <TableHeader columns={columns}>
        {(c: DataColumn<R>) => (
          <Column
            id={c.id}
            isRowHeader={c.isRowHeader}
            className={cx("border-b border-border px-2 py-1 text-left text-xs font-medium text-fg-2", c.numeric && "text-right")}
          >
            {c.header}
          </Column>
        )}
      </TableHeader>
      <TableBody items={keyed} renderEmptyState={() => <div className="p-2 text-fg-3">{t("design.dataTable.empty")}</div>}>
        {(item) => (
          <Row id={item.id} columns={columns} className="border-b border-border/60 outline-none data-[focus-visible]:bg-surface-3">
            {(c: DataColumn<R>) => (
              <Cell className={cx("px-2 py-1 text-fg-1", c.numeric && "num-table text-right")}>{c.cell(item.row)}</Cell>
            )}
          </Row>
        )}
      </TableBody>
    </Table>
  );
}
