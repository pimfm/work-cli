import { describe, it, expect } from "vitest";
import { GitHubProvider, type GhCliRunner } from "../providers/github/github-provider.js";

function mockCli(responses: Record<string, string>): GhCliRunner {
  return {
    run(args: string[]): string {
      const key = args[0];
      if (key && key in responses) return responses[key]!;
      throw new Error(`Unexpected gh command: ${args.join(" ")}`);
    },
  };
}

const sampleIssues = [
  {
    number: 42,
    title: "Fix auth flow",
    body: "The login page is broken",
    state: "open",
    url: "https://github.com/acme/app/issues/42",
    labels: [{ name: "bug" }, { name: "priority:high" }],
    repository: { name: "app", nameWithOwner: "acme/app" },
  },
  {
    number: 7,
    title: "Add dark mode",
    body: "",
    state: "open",
    url: "https://github.com/acme/app/issues/7",
    labels: [],
    repository: { name: "app", nameWithOwner: "acme/app" },
  },
];

const sampleRepos = [
  { nameWithOwner: "acme/app", name: "app" },
  { nameWithOwner: "acme/api", name: "api" },
];

describe("GitHubProvider", () => {
  describe("fetchAssignedItems", () => {
    it("maps gh CLI output to WorkItem[]", async () => {
      const cli = mockCli({ search: JSON.stringify(sampleIssues) });
      const provider = new GitHubProvider("acme", cli);

      const items = await provider.fetchAssignedItems();

      expect(items).toHaveLength(2);
      expect(items[0]).toEqual({
        id: "acme/app#42",
        title: "Fix auth flow",
        description: "The login page is broken",
        status: "open",
        labels: ["bug", "priority:high"],
        source: "GitHub",
        team: "acme/app",
        url: "https://github.com/acme/app/issues/42",
      });
      expect(items[1]).toEqual({
        id: "acme/app#7",
        title: "Add dark mode",
        description: "",
        status: "open",
        labels: [],
        source: "GitHub",
        team: "acme/app",
        url: "https://github.com/acme/app/issues/7",
      });
    });

    it("returns empty array when no issues", async () => {
      const cli = mockCli({ search: "[]" });
      const provider = new GitHubProvider("acme", cli);

      const items = await provider.fetchAssignedItems();
      expect(items).toEqual([]);
    });

    it("throws on CLI failure", async () => {
      const cli: GhCliRunner = {
        run() {
          throw new Error("not installed");
        },
      };
      const provider = new GitHubProvider("acme", cli);

      await expect(provider.fetchAssignedItems()).rejects.toThrow(
        "Failed to fetch GitHub issues"
      );
    });

    it("passes repo filter when board filter is set", async () => {
      let capturedArgs: string[] = [];
      const cli: GhCliRunner = {
        run(args: string[]): string {
          capturedArgs = args;
          return "[]";
        },
      };
      const provider = new GitHubProvider("acme", cli);
      provider.setBoardFilter("acme/app");

      await provider.fetchAssignedItems();

      expect(capturedArgs).toContain("--repo");
      expect(capturedArgs).toContain("acme/app");
    });

    it("truncates long descriptions to 500 chars", async () => {
      const longBody = "x".repeat(1000);
      const issues = [
        {
          ...sampleIssues[0],
          body: longBody,
        },
      ];
      const cli = mockCli({ search: JSON.stringify(issues) });
      const provider = new GitHubProvider("acme", cli);

      const items = await provider.fetchAssignedItems();
      expect(items[0].description).toHaveLength(500);
    });
  });

  describe("fetchBoards", () => {
    it("returns repos as boards", async () => {
      const cli = mockCli({ repo: JSON.stringify(sampleRepos) });
      const provider = new GitHubProvider("acme", cli);

      const boards = await provider.fetchBoards();

      expect(boards).toEqual([
        { id: "acme/app", name: "app" },
        { id: "acme/api", name: "api" },
      ]);
    });

    it("throws on CLI failure", async () => {
      const cli: GhCliRunner = {
        run() {
          throw new Error("not installed");
        },
      };
      const provider = new GitHubProvider("acme", cli);

      await expect(provider.fetchBoards()).rejects.toThrow(
        "Failed to list GitHub repositories"
      );
    });
  });

  describe("addComment", () => {
    it("calls gh issue comment with correct args", async () => {
      let capturedArgs: string[] = [];
      const cli: GhCliRunner = {
        run(args: string[]): string {
          capturedArgs = args;
          return "";
        },
      };
      const provider = new GitHubProvider("acme", cli);

      await provider.addComment("acme/app#42", "Agent failed");

      expect(capturedArgs).toEqual([
        "issue",
        "comment",
        "42",
        "--repo",
        "acme/app",
        "--body",
        "Agent failed",
      ]);
    });

    it("throws on invalid item ID format", async () => {
      const cli: GhCliRunner = {
        run(): string {
          return "";
        },
      };
      const provider = new GitHubProvider("acme", cli);

      await expect(provider.addComment("bad-id", "test")).rejects.toThrow(
        "Invalid GitHub item ID format"
      );
    });

    it("throws on CLI failure", async () => {
      const cli: GhCliRunner = {
        run() {
          throw new Error("not installed");
        },
      };
      const provider = new GitHubProvider("acme", cli);

      await expect(provider.addComment("acme/app#42", "test")).rejects.toThrow(
        "Failed to add GitHub comment"
      );
    });
  });
});
