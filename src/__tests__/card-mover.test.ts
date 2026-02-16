import { describe, it, expect, vi, beforeEach } from "vitest";
import { moveCard } from "../agents/card-mover.js";
import { parseGitHubPrMerge } from "../webhooks/handlers/github-handler.js";
import type { WorkItemProvider, CardStatus } from "../providers/provider.js";

function mockProvider(name: string, moveImpl?: (id: string, status: CardStatus) => Promise<void>): WorkItemProvider {
  return {
    name,
    fetchAssignedItems: vi.fn().mockResolvedValue([]),
    moveCard: moveImpl ? vi.fn(moveImpl) : undefined,
  };
}

describe("moveCard", () => {
  it("calls the first provider that supports moveCard", async () => {
    const move = vi.fn().mockResolvedValue(undefined);
    const providers = [
      mockProvider("NoMove"),
      mockProvider("WithMove", move),
      mockProvider("AlsoMove", vi.fn().mockResolvedValue(undefined)),
    ];

    await moveCard(providers, "LIN-42", "in_progress");

    expect(move).toHaveBeenCalledWith("LIN-42", "in_progress");
    // The third provider should not be called since the second succeeded
    expect(providers[2]!.moveCard).not.toHaveBeenCalled();
  });

  it("tries the next provider when one fails", async () => {
    const failMove = vi.fn().mockRejectedValue(new Error("Not found"));
    const succeedMove = vi.fn().mockResolvedValue(undefined);
    const providers = [
      mockProvider("Failing", failMove),
      mockProvider("Succeeding", succeedMove),
    ];

    await moveCard(providers, "PROJ-99", "in_review");

    expect(failMove).toHaveBeenCalledWith("PROJ-99", "in_review");
    expect(succeedMove).toHaveBeenCalledWith("PROJ-99", "in_review");
  });

  it("does not throw when no provider supports moveCard", async () => {
    const providers = [mockProvider("NoMove1"), mockProvider("NoMove2")];

    await expect(moveCard(providers, "ITEM-1", "done")).resolves.toBeUndefined();
  });

  it("does not throw when all providers fail", async () => {
    const providers = [
      mockProvider("Fail1", vi.fn().mockRejectedValue(new Error("fail"))),
      mockProvider("Fail2", vi.fn().mockRejectedValue(new Error("fail"))),
    ];

    await expect(moveCard(providers, "ITEM-1", "done")).resolves.toBeUndefined();
  });

  it("passes all three statuses correctly", async () => {
    const move = vi.fn().mockResolvedValue(undefined);
    const providers = [mockProvider("P", move)];

    await moveCard(providers, "id1", "in_progress");
    await moveCard(providers, "id2", "in_review");
    await moveCard(providers, "id3", "done");

    expect(move).toHaveBeenCalledWith("id1", "in_progress");
    expect(move).toHaveBeenCalledWith("id2", "in_review");
    expect(move).toHaveBeenCalledWith("id3", "done");
  });
});

describe("parseGitHubPrMerge", () => {
  it("detects a merged PR and extracts compound item ID from branch", () => {
    const result = parseGitHubPrMerge({
      action: "closed",
      pull_request: {
        merged: true,
        head: { ref: "agent/ember/LIN-42-fix-auth-flow" },
        title: "Fix auth flow",
      },
    });
    expect(result).toEqual({ itemId: "LIN-42", status: "done" });
  });

  it("extracts simple item ID from branch", () => {
    const result = parseGitHubPrMerge({
      action: "closed",
      pull_request: {
        merged: true,
        head: { ref: "agent/tide/69932610-move-card-feature" },
        title: "Move card feature",
      },
    });
    expect(result).toEqual({ itemId: "69932610", status: "done" });
  });

  it("extracts Trello-style short ID from branch", () => {
    const result = parseGitHubPrMerge({
      action: "closed",
      pull_request: {
        merged: true,
        head: { ref: "agent/gale/abc12345-add-dark-mode" },
        title: "Add dark mode",
      },
    });
    expect(result).toEqual({ itemId: "abc12345", status: "done" });
  });

  it("ignores non-closed PR events", () => {
    expect(parseGitHubPrMerge({
      action: "opened",
      pull_request: { merged: false, head: { ref: "agent/ember/LIN-42-fix" } },
    })).toBeUndefined();
  });

  it("ignores closed but not merged PRs", () => {
    expect(parseGitHubPrMerge({
      action: "closed",
      pull_request: { merged: false, head: { ref: "agent/ember/LIN-42-fix" } },
    })).toBeUndefined();
  });

  it("ignores PRs without agent branch pattern", () => {
    expect(parseGitHubPrMerge({
      action: "closed",
      pull_request: { merged: true, head: { ref: "feature/new-thing" } },
    })).toBeUndefined();
  });

  it("ignores PRs without branch info", () => {
    expect(parseGitHubPrMerge({
      action: "closed",
      pull_request: { merged: true },
    })).toBeUndefined();
  });
});
