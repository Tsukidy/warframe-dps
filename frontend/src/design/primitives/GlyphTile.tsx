import { useState } from "react";
import { useT } from "../../i18n/I18nProvider";
import { cx } from "./cx";

/** A deterministic hue from a string (the same name always gets the same tile). */
export function glyphHue(name: string): number {
  let h = 2166136261;
  for (let i = 0; i < name.length; i++) {
    h ^= name.charCodeAt(i);
    h = Math.imul(h, 16777619);
  }
  return (h >>> 0) % 360;
}

/** Initials of a name: first letters of the first two words (`Braton Prime` → `BP`). */
export function glyphInitials(name: string): string {
  const words = name.split(/[\s-]+/).filter((w) => /\p{L}|\p{N}/u.test(w));
  const first = [...(words[0] ?? "?")][0] ?? "?";
  const second = words.length > 1 ? ([...(words[1] ?? "")][0] ?? "") : ([...(words[0] ?? "")][1] ?? "");
  return (first + second).toUpperCase();
}

/**
 * The generated tile shown instead of an item image (frontend.md §10: offline, image fetching off, tests): initials
 * on a hue derived from the name. Decorative unless `labelled`.
 */
export function GlyphTile({ name, size = 40, labelled = false, className }: { name: string; size?: number; labelled?: boolean; className?: string }) {
  const t = useT();
  const hue = glyphHue(name);
  return (
    <span
      role={labelled ? "img" : undefined}
      aria-label={labelled ? t("design.glyph.label", { name }) : undefined}
      aria-hidden={labelled ? undefined : true}
      className={cx("inline-flex shrink-0 items-center justify-center rounded-md font-semibold select-none", className)}
      style={{
        width: size,
        height: size,
        fontSize: Math.round(size * 0.38),
        background: `light-dark(hsl(${hue} 45% 88%), hsl(${hue} 30% 24%))`,
        color: `light-dark(hsl(${hue} 50% 22%), hsl(${hue} 60% 85%))`,
        border: `1px solid light-dark(hsl(${hue} 30% 70%), hsl(${hue} 30% 36%))`,
      }}
    >
      {glyphInitials(name)}
    </span>
  );
}

/**
 * An item, weapon or frame image from `GET /api/img/{key}` (server-api.md §10), falling back to the glyph tile when
 * the catalogue says there is none, the server answers 204, or loading fails.
 */
export function ItemImage({ imageKey, name, hasImage, size = 40, className }: { imageKey: number; name: string; hasImage: boolean; size?: number; className?: string }) {
  const [failed, setFailed] = useState(false);
  if (!hasImage || failed) return <GlyphTile name={name} size={size} className={className} />;
  return (
    <img
      src={`/api/img/${imageKey}`}
      alt=""
      width={size}
      height={size}
      loading="lazy"
      className={cx("shrink-0 rounded-md object-contain", className)}
      onError={() => setFailed(true)}
      onLoad={(e) => {
        if (e.currentTarget.naturalWidth === 0) setFailed(true);
      }}
    />
  );
}
