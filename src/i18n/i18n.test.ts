/**
 * T157 — i18n key parity lint.
 *
 * Guards against missing translations by diffing the full key tree of
 * `en.json` and `zh-TW.json`. Every locale bundle must expose the same
 * set of leaf keys; a mismatch fails this test and blocks CI
 * (Constitution IV — UX consistency).
 */
import { describe, expect, it } from "vitest";

import en from "./en.json";
import zhTW from "./zh-TW.json";

type JsonTree = { [key: string]: unknown };

function collectLeafKeys(tree: JsonTree, prefix = ""): string[] {
  const out: string[] = [];
  for (const [key, value] of Object.entries(tree)) {
    const path = prefix ? `${prefix}.${key}` : key;
    if (value && typeof value === "object" && !Array.isArray(value)) {
      out.push(...collectLeafKeys(value as JsonTree, path));
    } else {
      out.push(path);
    }
  }
  return out.sort();
}

describe("i18n key parity", () => {
  const enKeys = collectLeafKeys(en as JsonTree);
  const zhKeys = collectLeafKeys(zhTW as JsonTree);

  it("en.json has at least one key", () => {
    expect(enKeys.length).toBeGreaterThan(0);
  });

  it("en.json and zh-TW.json expose identical key trees", () => {
    const missingInZh = enKeys.filter((k) => !zhKeys.includes(k));
    const missingInEn = zhKeys.filter((k) => !enKeys.includes(k));
    expect(missingInZh).toEqual([]);
    expect(missingInEn).toEqual([]);
  });

  it("all translated values are non-empty strings", () => {
    for (const keys of [enKeys, zhKeys]) {
      for (const key of keys) {
        const value = key.split(".").reduce<unknown>((acc, part) => {
          if (acc && typeof acc === "object") {
            return (acc as JsonTree)[part];
          }
          return undefined;
        }, keys === enKeys ? en : zhTW);
        expect(typeof value, `key ${key} should be a string`).toBe("string");
        expect((value as string).length, `key ${key} should be non-empty`).toBeGreaterThan(0);
      }
    }
  });
});
