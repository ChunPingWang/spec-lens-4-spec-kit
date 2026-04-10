/**
 * i18next bootstrap. Loads English + Traditional Chinese resource bundles
 * and picks the initial language from the backend-persisted config, or
 * from the browser locale as a fallback.
 */
import i18n from "i18next";
import { initReactI18next } from "react-i18next";

import en from "./en.json";
import zhTW from "./zh-TW.json";

export const SUPPORTED_LANGUAGES = ["en", "zh-TW"] as const;
export type SupportedLanguage = (typeof SUPPORTED_LANGUAGES)[number];

export function detectSystemLanguage(): SupportedLanguage {
  const nav = typeof navigator !== "undefined" ? navigator.language : "en";
  if (nav?.toLowerCase().startsWith("zh")) {
    return "zh-TW";
  }
  return "en";
}

void i18n.use(initReactI18next).init({
  resources: {
    en: { translation: en },
    "zh-TW": { translation: zhTW },
  },
  lng: detectSystemLanguage(),
  fallbackLng: "en",
  interpolation: { escapeValue: false },
  returnNull: false,
});

export default i18n;
