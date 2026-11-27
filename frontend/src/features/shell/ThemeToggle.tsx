// Theme switch (frontend.md §10, §11): system / dark / light, stored in prefs and applied as `data-theme` on <html>.
import { useEffect } from "react";
import { usePrefsStore, type ThemePref } from "../../state/prefs";
import { useT } from "../../i18n/I18nProvider";
import { IconButton } from "../../design/primitives";
import type { IconName } from "../../design/Icon";

const NEXT: Record<ThemePref, ThemePref> = { system: "dark", dark: "light", light: "system" };
const ICON: Record<ThemePref, IconName> = { system: "monitor", dark: "moon", light: "sun" };

/** Applies the theme preference to the document element. */
export function useApplyTheme(): void {
  const theme = usePrefsStore((s) => s.prefs.theme);
  useEffect(() => {
    document.documentElement.dataset.theme = theme;
  }, [theme]);
}

export function ThemeToggle() {
  const t = useT();
  const theme = usePrefsStore((s) => s.prefs.theme);
  const setPrefs = usePrefsStore((s) => s.setPrefs);
  const name = { system: t("shell.theme.system"), dark: t("shell.theme.dark"), light: t("shell.theme.light") }[theme];
  return (
    <IconButton
      icon={ICON[theme]}
      label={t("shell.theme.current", { theme: name })}
      onPress={() => setPrefs({ theme: NEXT[theme] })}
      data-testid="theme-toggle"
    />
  );
}
