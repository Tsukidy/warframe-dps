// i18n coverage (frontend.md §8, overview.md I-8; testing.md §8): every server message key has an English message
// using exactly its declared arguments; every message parses as ICU; the pseudo-locale keeps the ICU syntax.
import { describe, expect, it } from "vitest";
import { createIntl } from "react-intl";
import { MESSAGE_KEYS } from "../api/generated/messageKeys";
import { EN_MESSAGES } from "./I18nProvider";
import { pseudoMessage, pseudoMessages } from "./pseudo";
import { schema } from "../mocks/fixtures";

/** Top-level argument names of an ICU message. */
function argNames(msg: string): Set<string> {
  const out = new Set<string>();
  for (const m of msg.matchAll(/\{\s*([\w.-]+)\s*[,}]/g)) out.add(m[1] as string);
  return out;
}

const messages = EN_MESSAGES as Record<string, string>;

describe("message catalogue", () => {
  it("has an English message for every server key, with the declared arguments", () => {
    for (const [key, spec] of Object.entries(MESSAGE_KEYS)) {
      const msg = messages[key];
      expect(msg, key).toBeTypeOf("string");
      expect([...argNames(msg as string)].sort(), key).toEqual([...spec.args].sort());
    }
  });

  it("has an English message for every toggle and condition-class key the fixture schema names", () => {
    // The toggle descriptors of data/fixtures/mini (the rows of rules/toggles.toml: `toggle.T-NN.label`, `.help`,
    // `.opt.<value>`, frontend.md §8) and the condition classes a, b, c, u (contract review N-6, N-7).
    const s = schema();
    const keys = [
      ...s.toggles.flatMap((t) => [
        t.label,
        t.help,
        ...("options" in t.control && Array.isArray(t.control.options) ? t.control.options.map((o) => o.label) : []),
      ]),
      ...s.conditions.classes.flatMap((c) => [c.label, c.help]),
    ];
    expect(keys.length).toBeGreaterThan(0);
    expect(keys.filter((k) => typeof messages[k] !== "string")).toEqual([]);
  });

  it("parses every message and formats it without errors", () => {
    const errors: string[] = [];
    const intl = createIntl({ locale: "en", messages, onError: (e) => errors.push(e.message) });
    for (const [id, msg] of Object.entries(messages)) {
      const values = Object.fromEntries([...argNames(msg)].map((a) => [a, 1]));
      intl.formatMessage({ id }, values);
    }
    expect(errors).toEqual([]);
  });

  it("pseudo-localizes text but keeps arguments, plurals and #", () => {
    expect(pseudoMessage("Window DPS {dps}")).toMatch(/^\[Ŵíñðóŵ ÐÞŠ \{dps\} ~+\]$/);
    const plural = pseudoMessage("{n, plural, one {# shot} other {# shots}}");
    expect(plural).toContain("{n, plural,");
    expect(plural).toContain("{# šĥóţ}");
    expect(plural).toContain("{# šĥóţš}");
    const intl = createIntl({ locale: "en", messages: { x: plural } });
    expect(intl.formatMessage({ id: "x" }, { n: 2 })).toContain("2 šĥóţš");
  });

  it("makes the pseudo catalogue about 30 % longer and still valid", () => {
    const pseudo = pseudoMessages(messages);
    const errors: string[] = [];
    const intl = createIntl({ locale: "en", messages: pseudo, onError: (e) => errors.push(e.message) });
    for (const [id, msg] of Object.entries(messages)) {
      const values = Object.fromEntries([...argNames(msg)].map((a) => [a, 1]));
      intl.formatMessage({ id }, values);
      expect((pseudo[id] as string).length).toBeGreaterThan(msg.length);
    }
    expect(errors).toEqual([]);
  });
});
