import { execFile } from "child_process";
import { promisify } from "util";
import type { WorkItem } from "../../model/work-item.js";
import type { WorkItemProvider } from "../provider.js";
import type { GhIssue } from "./github-types.js";

const execFileAsync = promisify(execFile);

export class GitHubProvider implements WorkItemProvider {
  name = "GitHub";

  constructor(private owner?: string, private repo?: string) {}

  async fetchAssignedItems(): Promise<WorkItem[]> {
    const repoFlag =
      this.owner && this.repo ? ["-R", `${this.owner}/${this.repo}`] : [];

    const { stdout } = await execFileAsync("gh", [
      "issue",
      "list",
      "--assignee",
      "@me",
      "--state",
      "open",
      "--json",
      "number,title,body,state,url,labels,repository",
      "--limit",
      "50",
      ...repoFlag,
    ]);

    const issues: GhIssue[] = JSON.parse(stdout);

    return issues.map((issue) => ({
      id: `#${issue.number}`,
      title: issue.title,
      description: issue.body?.slice(0, 500) || undefined,
      status: issue.state,
      labels: issue.labels?.map((l) => l.name) ?? [],
      source: "GitHub",
      team: issue.repository?.nameWithOwner,
      url: issue.url,
    }));
  }
}
