/** Joins class names, skipping falsy values. */
export function cx(...parts: (string | false | null | undefined)[]): string {
  return parts.filter(Boolean).join(" ");
}

/** Shared focus ring and disabled styles for interactive elements (frontend.md §11: 24 × 24 px targets). */
export const INTERACTIVE =
  "outline-none data-[focus-visible]:outline-2 data-[focus-visible]:outline-offset-2 data-[focus-visible]:outline-accent data-[disabled]:opacity-50 data-[disabled]:cursor-not-allowed";
