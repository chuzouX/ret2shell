import { type BaseDict, flatten } from "@solid-primitives/i18n";
import zhCN from "./zh-cn.json";

const localeList = ["zh_cn", "en_us", "zh_tw", "ja_jp"] as const;
export type Locale = (typeof localeList)[number];

/** Synchronously available default dict so translations work before async locale loads. */
export const defaultDict: BaseDict = flatten(zhCN);

export async function fetchDictionary(locale: Locale): Promise<BaseDict> {
  let dict: BaseDict;
  const dictModules = import.meta.glob("./*.json");
  const match = dictModules[`./${locale.replace("_", "-")}.json`];
  try {
    dict = (await match()) as BaseDict;
  } catch {
    dict = await import("./zh-cn.json");
  }
  // flatten the dictionary to make all nested keys available top-level
  return flatten(dict);
}

export function hasLocale(locale: unknown): locale is Locale {
  return typeof locale === "string" && localeList.includes(locale as Locale);
}
