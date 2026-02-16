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

  async addComment(itemId: string, comment: string): Promise<void> {
    const baseUrl = `https://${this.domain}.atlassian.net`;
    const auth = Buffer.from(`${this.email}:${this.apiToken}`).toString("base64");

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
      headers: {
        Authorization: `Basic ${auth}`,
        "Content-Type": "application/json",
        Accept: "application/json",
      },
      body: JSON.stringify(body),
    });

    if (!res.ok) {
      throw new Error(`Jira API error: ${res.status} ${res.statusText}`);
    }
  }

  async fetchAssignedItems(): Promise<WorkItem[]> {
    const baseUrl = `https://${this.domain}.atlassian.net`;
    const auth = Buffer.from(`${this.email}:${this.apiToken}`).toString("base64");

    const params = new URLSearchParams({
      jql: "assignee = currentUser() AND statusCategory != Done ORDER BY priority ASC",
      maxResults: "50",
      fields: "summary,description,status,priority,labels,project",
    });

    const res = await fetch(`${baseUrl}/rest/api/3/search?${params}`, {
      headers: {
        Authorization: `Basic ${auth}`,
        Accept: "application/json",
      },
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

  async markItemDone(itemId: string): Promise<void> {
    const baseUrl = `https://${this.domain}.atlassian.net`;
    const auth = Buffer.from(`${this.email}:${this.apiToken}`).toString("base64");
    const headers = {
      Authorization: `Basic ${auth}`,
      "Content-Type": "application/json",
      Accept: "application/json",
    };

    // Get available transitions for the issue
    const transitionsRes = await fetch(`${baseUrl}/rest/api/3/issue/${itemId}/transitions`, {
      headers,
    });
    if (!transitionsRes.ok) {
      throw new Error(`Jira API error: ${transitionsRes.status} ${transitionsRes.statusText}`);
    }

    const transitionsJson = await transitionsRes.json() as {
      transitions: { id: string; name: string; to: { statusCategory: { key: string } } }[];
    };

    const doneTransition = transitionsJson.transitions.find(
      (t) => t.to.statusCategory.key === "done",
    );
    if (!doneTransition) {
      throw new Error(`No transition to Done found for issue ${itemId}`);
    }

    // Execute the transition
    const res = await fetch(`${baseUrl}/rest/api/3/issue/${itemId}/transitions`, {
      method: "POST",
      headers,
      body: JSON.stringify({ transition: { id: doneTransition.id } }),
    });
    if (!res.ok) {
      throw new Error(`Jira API error: ${res.status} ${res.statusText}`);
    }
  }
}
