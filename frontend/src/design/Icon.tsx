// Icons and damage-type glyphs from the sprite (frontend.md §10). Decorative by default (aria-hidden); pass `label`
// when the icon alone carries meaning.
import spriteUrl from "./icons/sprite.svg?url";
import type { TypeId } from "./damageTypes";
import { typeStyle } from "./damageTypes";
import { useT } from "../i18n/I18nProvider";

/** UI icon names in the sprite (`i-<name>`). */
export type IconName =
  | "about" | "arrow-down" | "arrow-up" | "calculator" | "check" | "chevron-down" | "chevron-right" | "close"
  | "data" | "image" | "import" | "info" | "loadout" | "menu" | "minus" | "monitor" | "moon" | "optimizer" | "pin"
  | "plus" | "redo" | "results" | "share" | "sun" | "target" | "undo" | "warning";

/** The URL of the sprite file (hashed in builds). */
export const SPRITE_URL: string = spriteUrl;

interface SvgUseProps {
  symbol: string;
  size?: number;
  label?: string;
  className?: string;
  color?: string;
}

function SvgUse({ symbol, size = 16, label, className, color }: SvgUseProps) {
  return (
    <svg
      width={size}
      height={size}
      className={className}
      style={color ? { color } : undefined}
      role={label ? "img" : undefined}
      aria-label={label}
      aria-hidden={label ? undefined : true}
      focusable="false"
    >
      <use href={`${SPRITE_URL}#${symbol}`} />
    </svg>
  );
}

export function Icon({ name, ...rest }: { name: IconName; size?: number; label?: string; className?: string }) {
  return <SvgUse symbol={`i-${name}`} {...rest} />;
}

/** A damage or status type glyph in its colour, labelled with the type name unless `decorative`. */
export function DamageGlyph({
  type,
  size = 16,
  decorative = false,
  className,
}: {
  type: TypeId;
  size?: number;
  decorative?: boolean;
  className?: string;
}) {
  const t = useT();
  const s = typeStyle(type);
  return (
    <SvgUse
      symbol={s.glyph}
      size={size}
      color={s.color}
      className={className}
      label={decorative ? undefined : t(s.label)}
    />
  );
}
