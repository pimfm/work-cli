import { describe, it, expect, vi, beforeEach } from "vitest";

vi.mock("child_process", () => ({
  execFile: vi.fn(),
}));

vi.mock("util", async (importOriginal) => {
  const actual = await importOriginal<typeof import("util")>();
  return {
    ...actual,
    promisify: (fn: unknown) => {
      return (...args: unknown[]) =>
        new Promise((resolve, reject) => {
          (fn as Function)(...args, (err: Error | null, result: unknown) => {
            if (err) reject(err);
            else resolve(result);
          });
        });
    },
  };
});

import { execFile } from "child_process";
import { GitHubProvider } from "../providers/github/github-provider.js";

const mockExecFile = vi.mocked(execFile);

describe("GitHubProvider", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("fetches assigned issues via gh CLI", async () => {
    const ghOutput = JSON.stringify([
      {
        number: 42,
        title: "Fix login bug",
        body: "Users cannot log in with SSO",
        state: "open",
        url: "https://github.com/org/repo/issues/42",
        labels: [{ name: "bug" }, { name: "auth" }],
        repository: { nameWithOwner: "org/repo" },
      },
      {
        number: 7,
        title: "Add dark mode",
        body: "",
        state: "open",
        url: "https://github.com/org/repo/issues/7",
        labels: [],
        repository: { nameWithOwner: "org/repo" },
      },
    ]);

    mockExecFile.mockImplementation((...args: unknown[]) => {
      const cb = args[args.length - 1] as Function;
      cb(null, { stdout: ghOutput, stderr: "" });
    });

    const provider = new GitHubProvider();
    const items = await provider.fetchAssignedItems();

    expect(items).toHaveLength(2);
    expect(items[0]).toEqual({
      id: "#42",
      title: "Fix login bug",
      description: "Users cannot log in with SSO",
      status: "open",
      labels: ["bug", "auth"],
      source: "GitHub",
      team: "org/repo",
      url: "https://github.com/org/repo/issues/42",
    });
    expect(items[1].id).toBe("#7");
    expect(items[1].description).toBeUndefined();
  });

  it("passes repo flag when owner and repo are set", async () => {
    mockExecFile.mockImplementation((...args: unknown[]) => {
      const cb = args[args.length - 1] as Function;
      cb(null, { stdout: "[]", stderr: "" });
    });

    const provider = new GitHubProvider("myorg", "myrepo");
    await provider.fetchAssignedItems();

    const callArgs = mockExecFile.mock.calls[0];
    expect(callArgs[0]).toBe("gh");
    const flags = callArgs[1] as string[];
    expect(flags).toContain("-R");
    expect(flags).toContain("myorg/myrepo");
  });

  it("does not pass repo flag when owner/repo are omitted", async () => {
    mockExecFile.mockImplementation((...args: unknown[]) => {
      const cb = args[args.length - 1] as Function;
      cb(null, { stdout: "[]", stderr: "" });
    });

    const provider = new GitHubProvider();
    await provider.fetchAssignedItems();

    const flags = mockExecFile.mock.calls[0][1] as string[];
    expect(flags).not.toContain("-R");
  });

  it("throws when gh CLI fails", async () => {
    mockExecFile.mockImplementation((...args: unknown[]) => {
      const cb = args[args.length - 1] as Function;
      cb(new Error("gh: command not found"), null);
    });

    const provider = new GitHubProvider();
    await expect(provider.fetchAssignedItems()).rejects.toThrow("gh: command not found");
  });

  it("truncates description to 500 chars", async () => {
    const longBody = "x".repeat(1000);
    const ghOutput = JSON.stringify([
      {
        number: 1,
        title: "Long issue",
        body: longBody,
        state: "open",
        url: "https://github.com/org/repo/issues/1",
        labels: [],
        repository: { nameWithOwner: "org/repo" },
      },
    ]);

    mockExecFile.mockImplementation((...args: unknown[]) => {
      const cb = args[args.length - 1] as Function;
      cb(null, { stdout: ghOutput, stderr: "" });
    });

    const provider = new GitHubProvider();
    const items = await provider.fetchAssignedItems();

    expect(items[0].description).toHaveLength(500);
  });

  it("has name set to GitHub", () => {
    const provider = new GitHubProvider();
    expect(provider.name).toBe("GitHub");
  });
});
