// i18n provider (frontend.md §8): react-intl with the merged English catalogue (`en.json`, generated from
// `en/*.json`) and the pseudo-locale en-XA. Adding a language means adding `en/`-shaped files for it and an entry in
// CATALOGUES; no component changes.
import type { ReactNode } from "react";
import { useMemo } from "react";
import { IntlProvider, useIntl, type IntlShape } from "react-intl";
import en from "./en.json";
import type { MessageId } from "./messages";
import { PSEUDO_LOCALE, pseudoMessages } from "./pseudo";
import type { Message } from "../api/generated";

/** The English catalogue, merged from `src/i18n/en/*.json`. */
export const EN_MESSAGES: Readonly<Record<MessageId, string>> = en;

/** Locales the app can show. `en-XA` is the pseudo-locale (accented, 30 % longer). */
export const LOCALES = ["en", PSEUDO_LOCALE] as const;
export type Locale = (typeof LOCALES)[number];

export function isLocale(x: unknown): x is Locale {
  return typeof x === "string" && (LOCALES as readonly string[]).includes(x);
}

function catalogue(locale: Locale): Record<string, string> {
  return locale === PSEUDO_LOCALE ? pseudoMessages(EN_MESSAGES) : EN_MESSAGES;
}

export interface I18nProviderProps {
  locale?: Locale;
  children: ReactNode;
  /** Called for missing ids and format errors; defaults to a console warning. */
  onError?: (err: Error) => void;
}

export function I18nProvider({ locale = "en", children, onError }: I18nProviderProps) {
  const messages = useMemo(() => catalogue(locale), [locale]);
  return (
    <IntlProvider
      locale={locale === PSEUDO_LOCALE ? "en" : locale}
      messages={messages}
      defaultLocale="en"
      onError={onError ?? ((err) => console.warn(err.message))}
    >
      {children}
    </IntlProvider>
  );
}

/** Values of an ICU message. */
export type MsgValues = Record<string, string | number | boolean | Date | null | undefined>;

/** Typed message lookup: an unknown id is a type error. */
export function useT() {
  const intl = useIntl();
  return useMemo(() => (id: MessageId, values?: MsgValues) => intl.formatMessage({ id }, values), [intl]);
}

/**
 * Lookup of an id that comes from data (toggle, buff and custom-stat labels of the schema route, frontend.md §8).
 * A missing id shows `fallback` (or the id itself) instead of failing.
 */
export function formatDynamic(intl: IntlShape, id: string, values?: MsgValues, fallback?: string): string {
  if (Object.prototype.hasOwnProperty.call(intl.messages, id)) {
    return intl.formatMessage({ id }, values);
  }
  return fallback ?? id;
}

export function useDynamicT() {
  const intl = useIntl();
  return useMemo(
    () => (id: string, values?: MsgValues, fallback?: string) => formatDynamic(intl, id, values, fallback),
    [intl],
  );
}

/** Resolves numeric item arguments (`item`, `a`, `b`, `by`, `ability`, `host`, `key`) to display names. */
export type NameResolver = (key: number) => string | undefined;

const ITEM_ARGS = new Set(["item", "a", "b", "by", "ability", "host", "key"]);

/**
 * Renders a server `Message` (server-api.md §2: errors, warnings, logs, legality reasons are `{ key, args }`, never
 * English). Numeric item arguments are shown by name when `resolve` knows them.
 */
export function formatServerMessage(intl: IntlShape, msg: Message, resolve?: NameResolver): string {
  const values: MsgValues = {};
  for (const [k, v] of Object.entries(msg.args)) {
    values[k] = typeof v === "number" && ITEM_ARGS.has(k) && resolve ? (resolve(v) ?? v) : v;
  }
  return formatDynamic(intl, msg.key, values, msg.key);
}

export function useServerMessage(resolve?: NameResolver) {
  const intl = useIntl();
  return useMemo(() => (msg: Message) => formatServerMessage(intl, msg, resolve), [intl, resolve]);
}
