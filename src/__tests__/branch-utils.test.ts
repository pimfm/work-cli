import { describe, it, expect } from "vitest";
import { slugify, branchName, worktreePath } from "../agents/branch-utils.js";

describe("slugify", () => {
  it("converts to lowercase with dashes", () => {
    expect(slugify("Fix Login Bug")).toBe("fix-login-bug");
  });

  it("removes special characters", () => {
    expect(slugify("Add feature: OAuth 2.0!")).toBe("add-feature-oauth-2-0");
  });

  it("truncates to 40 characters", () => {
    const long = "This is a very long title that should be truncated to forty characters maximum";
    expect(slugify(long).length).toBeLessThanOrEqual(40);
  });

  it("strips leading and trailing dashes", () => {
    expect(slugify("--hello--world--")).toBe("hello-world");
  });

  it("handles empty string", () => {
    expect(slugify("")).toBe("");
  });
});

describe("branchName", () => {
  it("generates correct branch name format", () => {
    expect(branchName("ember", "LIN-123", "Fix login bug")).toBe("agent/ember/LIN-123-fix-login-bug");
  });

  it("truncates long item IDs to 8 chars", () => {
    const result = branchName("flow", "1234567890abcdef", "Short title");
    expect(result).toBe("agent/flow/12345678-short-title");
  });
});

describe("worktreePath", () => {
  it("creates sibling directory path", () => {
    expect(worktreePath("/Users/pim/fm/workflow/main", "ember")).toBe("/Users/pim/fm/workflow/agent-ember");
  });

  it("works for different agent names", () => {
    expect(worktreePath("/Users/pim/fm/workflow/main", "terra")).toBe("/Users/pim/fm/workflow/agent-terra");
  });
});
