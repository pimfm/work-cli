import type { WorkItem } from "../../model/work-item.js";
import type { WorkItemProvider } from "../provider.js";
import type { JiraSearchResponse } from "./jira-types.js";
import { extractTextFromAdf } from "../../utils/adf-parser.js";

export class JiraProvider implements WorkItemProvider {
  name = "Jira";

  constructor(
    private domain: string,
    private email: string,
    private apiToken: string,
  ) {}

  private authHeaders(): Record<string, string> {
    const auth = Buffer.from(`${this.email}:${this.apiToken}`).toString("base64");
    return {
      Authorization: `Basic ${auth}`,
      "Content-Type": "application/json",
      Accept: "application/json",
    };
  }

  async markDone(item: WorkItem): Promise<void> {
    const baseUrl = `https://${this.domain}.atlassian.net`;

    const transRes = await fetch(`${baseUrl}/rest/api/3/issue/${item.id}/transitions`, {
      headers: this.authHeaders(),
    });
    if (!transRes.ok) {
      throw new Error(`Jira API error: ${transRes.status} ${transRes.statusText}`);
    }

    const transJson = await transRes.json() as { transitions: { id: string; name: string; to: { statusCategory: { key: string } } }[] };
    const doneTrans = transJson.transitions.find(
      (t) => t.to.statusCategory.key === "done",
    );
    if (!doneTrans) {
      throw new Error(`No 'Done' transition available for ${item.id}`);
    }

    const res = await fetch(`${baseUrl}/rest/api/3/issue/${item.id}/transitions`, {
      method: "POST",
      headers: this.authHeaders(),
      body: JSON.stringify({ transition: { id: doneTrans.id } }),
    });
    if (!res.ok) {
      throw new Error(`Jira API error: ${res.status} ${res.statusText}`);
    }
  }

  async addComment(itemId: string, comment: string): Promise<void> {
    const baseUrl = `https://${this.domain}.atlassian.net`;

    const body = {
      body: {
        version: 1,
        type: "doc",
        content: [
          {
            type: "paragraph",
            content: [{ type: "text", text: comment }],
          },
        ],
      },
    };

    const res = await fetch(`${baseUrl}/rest/api/3/issue/${itemId}/comment`, {
      method: "POST",
      headers: this.authHeaders(),
      body: JSON.stringify(body),
    });

    if (!res.ok) {
      throw new Error(`Jira API error: ${res.status} ${res.statusText}`);
    }
  }

  async fetchAssignedItems(): Promise<WorkItem[]> {
    const baseUrl = `https://${this.domain}.atlassian.net`;

    const params = new URLSearchParams({
      jql: "assignee = currentUser() AND statusCategory != Done ORDER BY priority ASC",
      maxResults: "50",
      fields: "summary,description,status,priority,labels,project",
    });

    const res = await fetch(`${baseUrl}/rest/api/3/search?${params}`, {
      headers: this.authHeaders(),
    });

    if (!res.ok) {
      throw new Error(`Jira API error: ${res.status} ${res.statusText}`);
    }

    const json: JiraSearchResponse = await res.json();

    return json.issues.map((issue) => ({
      id: issue.key,
      title: issue.fields.summary,
      description: extractTextFromAdf(issue.fields.description)?.slice(0, 500),
      status: issue.fields.status?.name,
      priority: issue.fields.priority?.name,
      labels: issue.fields.labels,
      source: "Jira",
      team: issue.fields.project?.name,
      url: `${baseUrl}/browse/${issue.key}`,
    }));
  }
}
