/**
 * Compiles the default highlight-rule bundle into runnable regexes.
 * The universal rule set is shipped as JSON so it can be edited by
 * designers without touching TypeScript.
 */
import defaultRules from "./default-highlight-rules.json";

import type { HighlightSeverity } from "@/types/ipc";

export interface HighlightRule {
  id: string;
  severity: HighlightSeverity;
  pattern: string;
  flags: string;
}

export interface CompiledHighlightRule extends HighlightRule {
  regex: RegExp;
}

export interface HighlightRuleSet {
  version: number;
  ruleSet: string;
  rules: HighlightRule[];
}

export function loadDefaultRuleSet(): HighlightRuleSet {
  return defaultRules as HighlightRuleSet;
}

export function compileRuleSet(set: HighlightRuleSet): CompiledHighlightRule[] {
  return set.rules.map((r) => ({ ...r, regex: new RegExp(r.pattern, r.flags) }));
}

export function firstMatch(
  line: string,
  rules: CompiledHighlightRule[],
): CompiledHighlightRule | null {
  for (const rule of rules) {
    if (rule.regex.test(line)) return rule;
  }
  return null;
}
