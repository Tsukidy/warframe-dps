// Pseudo-locale en-XA (frontend.md §8): accented letters and about 30 % more length, with brackets, so hard-coded
// strings (not accented) and truncation (no closing bracket) show up. ICU arguments, plural and select keys and
// `#` stay untouched.

const ACCENTS: Record<string, string> = {
  a: "á", b: "ƀ", c: "ç", d: "ð", e: "é", f: "ƒ", g: "ĝ", h: "ĥ", i: "í", j: "ĵ", k: "ķ", l: "ļ", m: "ɱ",
  n: "ñ", o: "ó", p: "þ", q: "ǫ", r: "ŕ", s: "š", t: "ţ", u: "ú", v: "ṽ", w: "ŵ", x: "ẋ", y: "ý", z: "ž",
  A: "Á", B: "Ɓ", C: "Ç", D: "Ð", E: "É", F: "Ƒ", G: "Ĝ", H: "Ĥ", I: "Í", J: "Ĵ", K: "Ķ", L: "Ļ", M: "Ṁ",
  N: "Ñ", O: "Ó", P: "Þ", Q: "Ǫ", R: "Ŕ", S: "Š", T: "Ţ", U: "Ú", V: "Ṽ", W: "Ŵ", X: "Ẋ", Y: "Ý", Z: "Ž",
};

/** The pseudo-locale tag. */
export const PSEUDO_LOCALE = "en-XA";

function accent(text: string): string {
  let out = "";
  for (const ch of text) out += ACCENTS[ch] ?? ch;
  return out;
}

/** Finds the index of the `}` that closes the `{` at `open`. */
function closing(msg: string, open: number): number {
  let depth = 0;
  for (let i = open; i < msg.length; i++) {
    if (msg[i] === "{") depth++;
    else if (msg[i] === "}") {
      depth--;
      if (depth === 0) return i;
    }
  }
  return msg.length - 1;
}

/** Transforms the literal text of an ICU message, keeping its syntax. */
function transform(msg: string, inPlural: boolean): string {
  let out = "";
  let i = 0;
  while (i < msg.length) {
    const ch = msg[i] as string;
    if (ch === "{") {
      const end = closing(msg, i);
      out += argument(msg.slice(i + 1, end));
      i = end + 1;
    } else if (ch === "#" && inPlural) {
      out += ch;
      i++;
    } else {
      out += ACCENTS[ch] ?? ch;
      i++;
    }
  }
  return out;
}

/** An argument body (between the outer braces): `name`, `name, number`, `name, plural, one {…} other {…}`. */
function argument(body: string): string {
  const m = /^\s*([\w.-]+)\s*,\s*(plural|select|selectordinal)\s*,(.*)$/s.exec(body);
  if (!m) return `{${body}}`;
  const [, name, kind, rest] = m as unknown as [string, string, string, string];
  let out = `{${name}, ${kind},`;
  let i = 0;
  while (i < rest.length) {
    const open = rest.indexOf("{", i);
    if (open < 0) {
      out += rest.slice(i);
      break;
    }
    out += rest.slice(i, open);
    const end = closing(rest, open);
    out += `{${transform(rest.slice(open + 1, end), kind !== "select")}}`;
    i = end + 1;
  }
  return `${out}}`;
}

/** One message in the pseudo-locale: `[Ŵíñðóŵ ÐÞŠ ~~~]`. */
export function pseudoMessage(msg: string): string {
  const letters = msg.replace(/\{[^}]*\}/g, "").length;
  const pad = "~".repeat(Math.max(1, Math.round(letters * 0.3)));
  return `[${transform(msg, false)} ${pad}]`;
}

/** The whole catalogue in the pseudo-locale. */
export function pseudoMessages(messages: Record<string, string>): Record<string, string> {
  return Object.fromEntries(Object.entries(messages).map(([k, v]) => [k, pseudoMessage(v)]));
}

export { accent as pseudoAccent };
