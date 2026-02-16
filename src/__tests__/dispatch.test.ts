import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { mkdtempSync, rmSync, existsSync, readFileSync } from "fs";
import { join } from "path";
import { tmpdir } from "os";
import { AgentStore } from "../agents/agent-store.js";
import { buildClaudePrompt } from "../agents/claude-prompt.js";
import { branchName, worktreePath } from "../agents/branch-utils.js";
import { writeClaudeMd } from "../agents/claude-md.js";
import type { WorkItem } from "../model/work-item.js";

describe("Dispatch integration", () => {
  let tmpDir: string;

  beforeEach(() => {
    tmpDir = mkdtempSync(join(tmpdir(), "dispatch-test-"));
  });

  afterEach(() => {
    rmSync(tmpDir, { recursive: true, force: true });
  });

  it("writes CLAUDE.md to worktree directory", () => {
    writeClaudeMd(tmpDir, "Ember");
    const content = readFileSync(join(tmpDir, "CLAUDE.md"), "utf-8");
    expect(content).toContain("work pipeline");
    expect(content).toContain("Ember");
    expect(content).toContain("vitest");
  });

  it("builds correct branch name for dispatch", () => {
    const item: WorkItem = { id: "LIN-42", title: "Fix auth flow", labels: [], source: "Linear" };
    const branch = branchName("ember", item.id, item.title);
    expect(branch).toBe("agent/ember/LIN-42-fix-auth-flow");
  });

  it("computes worktree path as sibling", () => {
    const path = worktreePath("/Users/pim/fm/workflow/main", "flow");
    expect(path).toBe("/Users/pim/fm/workflow/agent-flow");
  });

  it("agent store tracks full dispatch lifecycle", () => {
    const storePath = join(tmpDir, "agents.json");
    const store = new AgentStore(storePath);

    // Initially idle
    expect(store.getAgent("ember").status).toBe("idle");

    // Mark busy
    store.markBusy("ember", "LIN-42", "Fix auth", "Linear", "agent/ember/LIN-42", "/tmp/wt", process.pid);
    const busy = store.getAgent("ember");
    expect(busy.status).toBe("working");
    expect(busy.workItemId).toBe("LIN-42");
    expect(busy.pid).toBe(process.pid);

    // Next free agent should skip ember
    expect(store.getNextFreeAgent()).toBe("flow");

    // Mark done
    store.markDone("ember");
    expect(store.getAgent("ember").status).toBe("done");

    // Release
    store.release("ember");
    expect(store.getAgent("ember").status).toBe("idle");
    expect(store.getNextFreeAgent()).toBe("ember");
  });

  it("prompt includes all work item fields", () => {
    const item: WorkItem = {
      id: "PROJ-99",
      title: "Add dark mode",
      description: "Support dark theme",
      status: "Todo",
      priority: "Medium",
      labels: ["frontend", "ux"],
      source: "Jira",
      team: "Design",
      url: "https://jira.example.com/PROJ-99",
    };
    const prompt = buildClaudePrompt(item, "Terra");
    expect(prompt).toContain("PROJ-99");
    expect(prompt).toContain("Add dark mode");
    expect(prompt).toContain("Support dark theme");
    expect(prompt).toContain("Jira");
    expect(prompt).toContain("Medium");
    expect(prompt).toContain("frontend, ux");
  });
});
