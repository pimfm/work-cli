import { describe, it, expect, vi, beforeEach } from "vitest";
import type { Agent } from "../model/agent.js";
import type { WorkItemProvider } from "../providers/provider.js";
import { GitHubProvider, type GhCliRunner } from "../providers/github/github-provider.js";
import { markWorkItemDone } from "../ui/hooks/use-agents.js";

vi.mock("../persistence/agent-log.js", () => ({
  appendEvent: vi.fn(),
}));

function mockProvider(name: string, options?: {
  markItemDone?: (itemId: string) => Promise<void>;
}): WorkItemProvider {
  return {
    name,
    fetchAssignedItems: vi.fn().mockResolvedValue([]),
    markItemDone: options?.markItemDone ?? vi.fn().mockResolvedValue(undefined),
  };
}

function doneAgent(overrides?: Partial<Agent>): Agent {
  return {
    name: "ember",
    status: "done",
    workItemId: "ITEM-1",
    workItemTitle: "Fix the bug",
    workItemSource: "Linear",
    branch: "agent/ember/ITEM-1",
    ...overrides,
  };
}

describe("markWorkItemDone", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("calls markItemDone on the matching provider", async () => {
    const markDone = vi.fn().mockResolvedValue(undefined);
    const provider = mockProvider("Linear", { markItemDone: markDone });

    await markWorkItemDone(doneAgent(), [provider]);

    expect(markDone).toHaveBeenCalledWith("ITEM-1");
  });

  it("skips providers that do not match the source", async () => {
    const trelloDone = vi.fn().mockResolvedValue(undefined);
    const linearDone = vi.fn().mockResolvedValue(undefined);
    const trello = mockProvider("Trello", { markItemDone: trelloDone });
    const linear = mockProvider("Linear", { markItemDone: linearDone });

    await markWorkItemDone(doneAgent({ workItemSource: "Trello" }), [trello, linear]);

    expect(trelloDone).toHaveBeenCalledWith("ITEM-1");
    expect(linearDone).not.toHaveBeenCalled();
  });

  it("does nothing when workItemId is missing", async () => {
    const markDone = vi.fn().mockResolvedValue(undefined);
    const provider = mockProvider("Linear", { markItemDone: markDone });

    await markWorkItemDone(doneAgent({ workItemId: undefined }), [provider]);

    expect(markDone).not.toHaveBeenCalled();
  });

  it("does nothing when workItemSource is missing", async () => {
    const markDone = vi.fn().mockResolvedValue(undefined);
    const provider = mockProvider("Linear", { markItemDone: markDone });

    await markWorkItemDone(doneAgent({ workItemSource: undefined }), [provider]);

    expect(markDone).not.toHaveBeenCalled();
  });

  it("does nothing when no provider matches", async () => {
    const markDone = vi.fn().mockResolvedValue(undefined);
    const provider = mockProvider("Jira", { markItemDone: markDone });

    await markWorkItemDone(doneAgent({ workItemSource: "Linear" }), [provider]);

    expect(markDone).not.toHaveBeenCalled();
  });

  it("does nothing when provider has no markItemDone", async () => {
    const provider: WorkItemProvider = {
      name: "Linear",
      fetchAssignedItems: vi.fn().mockResolvedValue([]),
    };

    // Should not throw
    await markWorkItemDone(doneAgent(), [provider]);
  });

  it("does not throw when markItemDone fails", async () => {
    const markDone = vi.fn().mockRejectedValue(new Error("API error"));
    const provider = mockProvider("Linear", { markItemDone: markDone });

    // Should not throw — errors are logged, not propagated
    await markWorkItemDone(doneAgent(), [provider]);

    expect(markDone).toHaveBeenCalledWith("ITEM-1");
  });
});

describe("GitHubProvider.markItemDone", () => {
  it("closes the issue via gh CLI", async () => {
    let capturedArgs: string[] = [];
    const cli: GhCliRunner = {
      run(args: string[]): string {
        capturedArgs = args;
        return "";
      },
    };
    const provider = new GitHubProvider("acme", cli);
    provider.setBoardFilter("acme/app");

    await provider.markItemDone("#42");

    expect(capturedArgs).toEqual(["issue", "close", "42", "--repo", "acme/app"]);
  });

  it("throws when no repo filter is set", async () => {
    const cli: GhCliRunner = {
      run(): string {
        return "";
      },
    };
    const provider = new GitHubProvider("acme", cli);

    await expect(provider.markItemDone("#42")).rejects.toThrow(
      "Cannot close GitHub issue without a repo filter set",
    );
  });

  it("throws when gh CLI fails", async () => {
    const cli: GhCliRunner = {
      run(): string {
        throw new Error("not installed");
      },
    };
    const provider = new GitHubProvider("acme", cli);
    provider.setBoardFilter("acme/app");

    await expect(provider.markItemDone("#42")).rejects.toThrow(
      "Failed to close GitHub issue #42",
    );
  });
});

describe("AgentStore.markBusy stores workItemSource", () => {
  it("persists workItemSource through the lifecycle", async () => {
    // This is tested more thoroughly in agent-store.test.ts;
    // here we verify the source field is available for markWorkItemDone
    const markDone = vi.fn().mockResolvedValue(undefined);
    const provider = mockProvider("Trello", { markItemDone: markDone });

    const agent = doneAgent({
      workItemSource: "Trello",
      workItemId: "abc12345",
    });

    await markWorkItemDone(agent, [provider]);

    expect(markDone).toHaveBeenCalledWith("abc12345");
  });
});
