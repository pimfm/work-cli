import { describe, it, expect } from "vitest";
import { GitHubProvider, type GhCliRunner } from "../providers/github/github-provider.js";
import type { WorkItem } from "../model/work-item.js";

function makeItem(overrides: Partial<WorkItem> = {}): WorkItem {
  return {
    id: "#42",
    title: "Fix auth flow",
    source: "GitHub",
    labels: [],
    team: "acme/app",
    url: "https://github.com/acme/app/issues/42",
    ...overrides,
  };
}

describe("GitHubProvider.markDone", () => {
  it("closes the issue via gh CLI", async () => {
    let capturedArgs: string[] = [];
    const cli: GhCliRunner = {
      run(args: string[]): string {
        capturedArgs = args;
        return "";
      },
    };
    const provider = new GitHubProvider("acme", cli);

    await provider.markDone(makeItem());

    expect(capturedArgs).toEqual(["issue", "close", "42", "--repo", "acme/app"]);
  });

  it("strips # prefix from issue ID", async () => {
    let capturedArgs: string[] = [];
    const cli: GhCliRunner = {
      run(args: string[]): string {
        capturedArgs = args;
        return "";
      },
    };
    const provider = new GitHubProvider("acme", cli);

    await provider.markDone(makeItem({ id: "#7" }));

    expect(capturedArgs).toContain("7");
  });

  it("throws when CLI fails", async () => {
    const cli: GhCliRunner = {
      run(): string {
        throw new Error("not installed");
      },
    };
    const provider = new GitHubProvider("acme", cli);

    await expect(provider.markDone(makeItem())).rejects.toThrow(
      "Failed to close GitHub issue #42",
    );
  });

  it("throws when repo is missing", async () => {
    const cli: GhCliRunner = {
      run(): string {
        return "";
      },
    };
    const provider = new GitHubProvider("acme", cli);

    await expect(
      provider.markDone(makeItem({ team: undefined })),
    ).rejects.toThrow("Cannot close GitHub issue");
  });
});
