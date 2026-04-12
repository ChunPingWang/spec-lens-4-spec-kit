/**
 * T157 / T156 — i18n key parity + unknown-key lint.
 *
 * Guards against missing translations by diffing the full key tree of
 * `en.json` and `zh-TW.json`. Every locale bundle must expose the same
 * set of leaf keys; a mismatch fails this test and blocks CI
 * (Constitution IV — UX consistency).
 *
 * T156: extends the suite with a **static audit** that scans every
 * `src/**\/*.{ts,tsx}` file for literal `t("key")` / `t('key')` calls
 * and asserts the key exists in the bundle. Dynamic lookups such as
 * ``t(`phase.docStatus.${status}`)`` are intentionally skipped — they
 * carry their own `defaultValue` fallback in the call site.
 */
import fs from "node:fs";
import path from "node:path";

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

/**
 * Recursive `.ts` / `.tsx` walker rooted at `src/`. Skips test files so
 * we only audit production call sites.
 */
function walkSourceFiles(dir: string, out: string[] = []): string[] {
  for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
    if (entry.name === "node_modules" || entry.name.startsWith(".")) continue;
    const full = path.join(dir, entry.name);
    if (entry.isDirectory()) {
      walkSourceFiles(full, out);
    } else if (/\.(ts|tsx)$/.test(entry.name) && !/\.test\.(ts|tsx)$/.test(entry.name)) {
      out.push(full);
    }
  }
  return out;
}

describe("i18n unknown-key audit (T156)", () => {
  const srcRoot = path.resolve(__dirname, "..");
  const enKeys = new Set(collectLeafKeys(en as JsonTree));
  const files = walkSourceFiles(srcRoot);

  // Literal `t("namespace.key")` / `t('namespace.key')`. We require at
  // least one dot to avoid matching non-i18n helpers like `t('foo')`.
  const literalCall = /\bt\(\s*(['"])([a-zA-Z][\w]+(?:\.[\w]+)+)\1/g;

  it("walks the frontend source tree", () => {
    expect(files.length).toBeGreaterThan(10);
  });

  it("every literal t('key') call resolves to an en.json entry", () => {
    const unknown = new Map<string, string[]>();
    for (const file of files) {
      const source = fs.readFileSync(file, "utf8");
      for (const match of source.matchAll(literalCall)) {
        const key = match[2];
        if (!key || enKeys.has(key)) continue;
        const sites = unknown.get(key) ?? [];
        sites.push(path.relative(srcRoot, file));
        unknown.set(key, sites);
      }
    }
    const formatted = Array.from(unknown.entries())
      .map(([key, sites]) => `${key}  (${sites.join(", ")})`)
      .sort();
    expect(formatted).toEqual([]);
  });
});
