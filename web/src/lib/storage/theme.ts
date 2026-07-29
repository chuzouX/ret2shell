import { defaultDict, fetchDictionary, hasLocale, type Locale } from "@lib/i18n";
import { resolveTemplate, translator } from "@solid-primitives/i18n";
import { createPrefersDark } from "@solid-primitives/media";
import { makePersisted } from "@solid-primitives/storage";
import { createEffect, createResource, createRoot, untrack } from "solid-js";
import { createStore, type StoreReturn } from "solid-js/store";

type ThemeStore = {
  theme: string;
  locale: Locale;
  colorScheme: string;
  colorSchemeFollowsSystem: boolean;
  showBackgroundImg: boolean;
};

let systemPrefersLocale = (window.navigator.language || window.navigator.languages[0])
  .replace("-", "_")
  .toLowerCase() as Locale;

if (!hasLocale(systemPrefersLocale)) {
  systemPrefersLocale = "zh_cn" as Locale;
}

const themeRoot = createRoot(() => {
  const prefersDark = createPrefersDark();
  const [themeStore, setThemeStore] = makePersisted<ThemeStore, StoreReturn<ThemeStore>>(
    createStore<ThemeStore>({
      theme: "cyber",
      locale: systemPrefersLocale,
      colorScheme: "dark",
      colorSchemeFollowsSystem: true,
      showBackgroundImg: true,
    }),
    { name: "theme" }
  );
  const [dict] = createResource<Record<string, string>, Locale>(
    () => themeStore.locale || systemPrefersLocale,
    async (locale) => (await fetchDictionary(locale)) as Record<string, string>,
    {
      initialValue: defaultDict as Record<string, string>,
    }
  );
  return { prefersDark, themeStore, setThemeStore, dict };
});

export const themeStore = themeRoot.themeStore;
export const setThemeStore = themeRoot.setThemeStore;

const prefersDark = themeRoot.prefersDark;
const dict = themeRoot.dict;

export function setTheme(theme: string) {
  setThemeStore({ theme });
}

export function setColorScheme(colorScheme: "dark" | "light") {
  setThemeStore({ colorScheme });
}

export function setLocale(locale: Locale) {
  setThemeStore({ locale });
  setTimeout(() => location.reload());
}

export function toggleBackgroundImg() {
  setThemeStore("showBackgroundImg", !themeStore.showBackgroundImg);
}

export function fullTheme() {
  return `${themeStore.theme}-${themeStore.colorScheme}`;
}

export function initTheme() {
  createEffect(() => {
    document.documentElement.setAttribute("data-theme", fullTheme());
    document.documentElement.setAttribute("data-style", themeStore.colorScheme);
  });
  createEffect(() => {
    if (themeStore.colorSchemeFollowsSystem)
      if (prefersDark()) untrack(() => setColorScheme("dark"));
      else untrack(() => setColorScheme("light"));
  });

  function onBeforePrint() {
    document.documentElement.setAttribute("data-theme", `${themeStore.theme}-light`);
    document.documentElement.setAttribute("data-style", "light");
  }
  function onAfterPrint() {
    document.documentElement.setAttribute("data-theme", fullTheme());
    document.documentElement.setAttribute("data-style", themeStore.colorScheme);
  }
  window.onbeforeprint = onBeforePrint;
  window.onafterprint = onAfterPrint;
}
export const t = translator(dict as () => Record<string, string>, resolveTemplate);
export const colorPalette = {
  fg: () => (themeStore.colorScheme === "dark" ? "#eee" : "#121212"),
  primary: "#0991ed",
  secondary: "#bd63c5",
  accent: "#699f08",
  info: "#0991ed",
  success: "#17a750",
  warning: "#db640e",
  error: "#e05864",
};

export const breakpoints = {
  "2xl": "1536px",
  xl: "1280px",
  lg: "1024px",
  md: "768px",
  sm: "640px",
  xs: "480px",
  "2xs": "320px",
} as const;
