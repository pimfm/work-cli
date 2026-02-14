import { execSync } from "child_process";
import type { WorkItem } from "../../model/work-item.js";
import type { WorkItemProvider, Board } from "../provider.js";
import { GhIssueListSchema, type GhIssue } from "./github-types.js";

export interface GhCliRunner {
  run(args: string[]): string;
}

const defaultRunner: GhCliRunner = {
  run(args: string[]): string {
    return execSync(`gh ${args.join(" ")}`, {
      encoding: "utf-8",
      stdio: ["pipe", "pipe", "pipe"],
    });
  },
};

export class GitHubProvider implements WorkItemProvider {
  name = "GitHub";
  private repoFilter: string | undefined;

  constructor(
    private owner: string,
    private cli: GhCliRunner = defaultRunner,
  ) {}

  async fetchAssignedItems(): Promise<WorkItem[]> {
    const args = [
      "search",
      "issues",
      "--assignee",
      this.owner,
      "--state",
      "open",
      "--json",
      "number,title,body,state,url,labels,repository",
      "--limit",
      "50",
    ];

    if (this.repoFilter) {
      args.push("--repo", this.repoFilter);
    }

    let output: string;
    try {
      output = this.cli.run(args);
    } catch {
      throw new Error("Failed to fetch GitHub issues. Is the gh CLI installed and authenticated?");
    }

    const parsed = JSON.parse(output);
    const issues = GhIssueListSchema.parse(parsed);

    return issues.map((issue) => mapIssueToWorkItem(issue));
  }

  async fetchBoards(): Promise<Board[]> {
    const args = [
      "repo",
      "list",
      this.owner,
      "--json",
      "nameWithOwner,name",
      "--limit",
      "50",
    ];

    let output: string;
    try {
      output = this.cli.run(args);
    } catch {
      throw new Error("Failed to list GitHub repositories. Is the gh CLI installed and authenticated?");
    }

    const repos: Array<{ nameWithOwner: string; name: string }> = JSON.parse(output);
    return repos.map((r) => ({ id: r.nameWithOwner, name: r.name }));
  }

  setBoardFilter(repoFullName: string): void {
    this.repoFilter = repoFullName;
  }
}

function mapIssueToWorkItem(issue: GhIssue): WorkItem {
  return {
    id: `#${issue.number}`,
    title: issue.title,
    description: issue.body?.slice(0, 500),
    status: issue.state,
    labels: issue.labels.map((l) => l.name),
    source: "GitHub",
    team: issue.repository.nameWithOwner,
    url: issue.url,
  };
}
