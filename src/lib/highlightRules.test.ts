import { describe, expect, it } from "vitest";

import {
  compileRuleSet,
  firstMatch,
  loadDefaultRuleSet,
} from "./highlightRules";

describe("highlightRules", () => {
  const rules = compileRuleSet(loadDefaultRuleSet());

  it("marks ERROR-prefixed lines as error severity", () => {
    const match = firstMatch("ERROR: something blew up", rules);
    expect(match?.severity).toBe("error");
  });

  it("marks WARN-prefixed lines as warn severity", () => {
    const match = firstMatch("WARNING: low disk space", rules);
    expect(match?.severity).toBe("warn");
  });

  it("marks PASS tokens as success", () => {
    const match = firstMatch("  PASS   src/a.test.ts", rules);
    expect(match?.severity).toBe("success");
  });

  it("returns null for plain lines", () => {
    expect(firstMatch("nothing to see here", rules)).toBeNull();
  });
});
