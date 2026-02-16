import type { WorkItem } from "../../model/work-item.js";
import type { WorkItemProvider, CardStatus } from "../provider.js";
import type { JiraSearchResponse } from "./jira-types.js";
import { extractTextFromAdf } from "../../utils/adf-parser.js";

const STATUS_TRANSITION_NAMES: Record<CardStatus, string> = {
  in_progress: "In Progress",
  in_review: "In Review",
  done: "Done",
};

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

  async moveCard(itemId: string, status: CardStatus): Promise<void> {
    const baseUrl = `https://${this.domain}.atlassian.net`;
    const auth = Buffer.from(`${this.email}:${this.apiToken}`).toString("base64");
    const targetName = STATUS_TRANSITION_NAMES[status];

    // Get available transitions for the issue
    const transRes = await fetch(`${baseUrl}/rest/api/3/issue/${itemId}/transitions`, {
      headers: {
        Authorization: `Basic ${auth}`,
        Accept: "application/json",
      },
    });
    if (!transRes.ok) {
      throw new Error(`Jira API error: ${transRes.status} ${transRes.statusText}`);
    }

    const transJson = await transRes.json() as {
      transitions: Array<{ id: string; name: string; to: { name: string } }>;
    };

    const transition = transJson.transitions.find(
      (t) => t.to.name.toLowerCase() === targetName.toLowerCase(),
    );
    if (!transition) {
      throw new Error(`Jira transition to "${targetName}" not found for issue ${itemId}`);
    }

    // Execute transition
    const res = await fetch(`${baseUrl}/rest/api/3/issue/${itemId}/transitions`, {
      method: "POST",
      headers: {
        Authorization: `Basic ${auth}`,
        "Content-Type": "application/json",
        Accept: "application/json",
      },
      body: JSON.stringify({ transition: { id: transition.id } }),
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
}
