import { describe, it, expect, beforeEach, afterEach } from "vitest";
import { AgentStore } from "../agents/agent-store.js";
import { mkdtempSync, rmSync } from "fs";
import { join } from "path";
import { tmpdir } from "os";

describe("AgentStore", () => {
  let tmpDir: string;
  let storePath: string;

  beforeEach(() => {
    tmpDir = mkdtempSync(join(tmpdir(), "agent-store-test-"));
    storePath = join(tmpDir, "agents.json");
  });

  afterEach(() => {
    rmSync(tmpDir, { recursive: true, force: true });
  });

  it("initializes with all agents idle", () => {
    const store = new AgentStore(storePath);
    const agents = store.getAll();
    expect(agents).toHaveLength(4);
    expect(agents.every((a) => a.status === "idle")).toBe(true);
    expect(agents.map((a) => a.name)).toEqual(["ember", "flow", "tempest", "terra"]);
  });

  it("returns next free agent in order", () => {
    const store = new AgentStore(storePath);
    expect(store.getNextFreeAgent()).toBe("ember");
  });

  it("skips busy agents when finding next free", () => {
    const store = new AgentStore(storePath);
    store.markBusy("ember", "item-1", "Task 1", "Trello", "branch/1", "/tmp/wt1", 99999);
    expect(store.getNextFreeAgent()).toBe("flow");
  });

  it("returns undefined when all agents are busy", () => {
    const store = new AgentStore(storePath);
    store.markBusy("ember", "1", "T1", "Trello", "b/1", "/tmp/1", 99999);
    store.markBusy("flow", "2", "T2", "Linear", "b/2", "/tmp/2", 99999);
    store.markBusy("tempest", "3", "T3", "Jira", "b/3", "/tmp/3", 99999);
    store.markBusy("terra", "4", "T4", "GitHub", "b/4", "/tmp/4", 99999);
    expect(store.getNextFreeAgent()).toBeUndefined();
  });

  it("marks agent as done", () => {
    const store = new AgentStore(storePath);
    store.markBusy("ember", "item-1", "Task 1", "Trello", "branch/1", "/tmp/wt1", 99999);
    store.markDone("ember");
    const agent = store.getAgent("ember");
    expect(agent.status).toBe("done");
    expect(agent.pid).toBeUndefined();
    expect(agent.workItemTitle).toBe("Task 1");
  });

  it("marks agent as error", () => {
    const store = new AgentStore(storePath);
    store.markBusy("flow", "item-2", "Task 2", "Linear", "branch/2", "/tmp/wt2", 99999);
    store.markError("flow", "Process crashed");
    const agent = store.getAgent("flow");
    expect(agent.status).toBe("error");
    expect(agent.error).toBe("Process crashed");
  });

  it("releases agent back to idle", () => {
    const store = new AgentStore(storePath);
    store.markBusy("tempest", "item-3", "Task 3", "Jira", "branch/3", "/tmp/wt3", 99999);
    store.markDone("tempest");
    store.release("tempest");
    const agent = store.getAgent("tempest");
    expect(agent.status).toBe("idle");
    expect(agent.workItemId).toBeUndefined();
  });

  it("persists state across instances", () => {
    const store1 = new AgentStore(storePath);
    store1.markBusy("ember", "item-1", "Task 1", "Trello", "branch/1", "/tmp/wt1", 99999);

    const store2 = new AgentStore(storePath);
    const agent = store2.getAgent("ember");
    // Will be error because PID 99999 is (almost certainly) not running
    expect(agent.status).toBe("error");
    expect(agent.workItemId).toBe("item-1");
  });

  it("detects stale processes on load", () => {
    const store1 = new AgentStore(storePath);
    store1.markBusy("ember", "item-1", "Task 1", "Trello", "branch/1", "/tmp/wt1", 999999);

    const store2 = new AgentStore(storePath);
    const agent = store2.getAgent("ember");
    expect(agent.status).toBe("error");
    expect(agent.error).toBe("Process died unexpectedly");
  });
});
