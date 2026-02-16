import type { WorkItem } from "../../model/work-item.js";
import type { WorkItemProvider } from "../provider.js";
import type { LinearGraphQLResponse } from "./linear-types.js";

export class LinearProvider implements WorkItemProvider {
  name = "Linear";

  constructor(private apiKey: string) {}

  async fetchAssignedItems(): Promise<WorkItem[]> {
    const query = `{
      viewer {
        assignedIssues(
          filter: { state: { type: { nin: ["completed", "canceled"] } } }
          first: 50
        ) {
          nodes {
            id
            identifier
            title
            description
            priority
            url
            state { name }
            team { name }
            labels { nodes { name } }
          }
        }
      }
    }`;

    const res = await fetch("https://api.linear.app/graphql", {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        Authorization: this.apiKey,
      },
      body: JSON.stringify({ query }),
    });

    if (!res.ok) {
      throw new Error(`Linear API error: ${res.status} ${res.statusText}`);
    }

    const json: LinearGraphQLResponse = await res.json();
    const nodes = json.data?.viewer.assignedIssues.nodes ?? [];

    return nodes.map((issue) => ({
      id: issue.identifier,
      title: issue.title,
      description: issue.description?.slice(0, 500),
      status: issue.state?.name,
      priority: priorityLabel(issue.priority),
      labels: issue.labels?.nodes.map((l) => l.name) ?? [],
      source: "Linear",
      team: issue.team?.name,
      url: issue.url,
    }));
  }
  async addComment(itemId: string, comment: string): Promise<void> {
    // Linear work items use `identifier` (e.g. LIN-42) but comments need the internal UUID
    const lookupQuery = `{
      issues(filter: { identifier: { eq: "${itemId}" } }, first: 1) {
        nodes { id }
      }
    }`;

    const lookupRes = await fetch("https://api.linear.app/graphql", {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        Authorization: this.apiKey,
      },
      body: JSON.stringify({ query: lookupQuery }),
    });

    if (!lookupRes.ok) {
      throw new Error(`Linear API error: ${lookupRes.status} ${lookupRes.statusText}`);
    }

    const lookupJson = await lookupRes.json() as { data?: { issues: { nodes: { id: string }[] } } };
    const issueId = lookupJson.data?.issues.nodes[0]?.id;
    if (!issueId) {
      throw new Error(`Linear issue not found: ${itemId}`);
    }

    const mutation = `mutation {
      commentCreate(input: { issueId: "${issueId}", body: ${JSON.stringify(comment)} }) {
        success
      }
    }`;

    const res = await fetch("https://api.linear.app/graphql", {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        Authorization: this.apiKey,
      },
      body: JSON.stringify({ query: mutation }),
    });

    if (!res.ok) {
      throw new Error(`Linear API error: ${res.status} ${res.statusText}`);
    }
  }

  async markItemDone(itemId: string): Promise<void> {
    // Look up internal UUID and team for the issue
    const lookupQuery = `{
      issues(filter: { identifier: { eq: "${itemId}" } }, first: 1) {
        nodes { id team { states { nodes { id type } } } }
      }
    }`;

    const lookupRes = await this.graphql<{
      data?: {
        issues: {
          nodes: {
            id: string;
            team: { states: { nodes: { id: string; type: string }[] } };
          }[];
        };
      };
    }>(lookupQuery);

    const issue = lookupRes.data?.issues.nodes[0];
    if (!issue) {
      throw new Error(`Linear issue not found: ${itemId}`);
    }

    const doneState = issue.team.states.nodes.find((s) => s.type === "completed");
    if (!doneState) {
      throw new Error(`No completed state found for issue ${itemId}`);
    }

    const mutation = `mutation {
      issueUpdate(id: "${issue.id}", input: { stateId: "${doneState.id}" }) {
        success
      }
    }`;

    await this.graphql(mutation);
  }

  private async graphql<T = unknown>(query: string): Promise<T> {
    const res = await fetch("https://api.linear.app/graphql", {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        Authorization: this.apiKey,
      },
      body: JSON.stringify({ query }),
    });

    if (!res.ok) {
      throw new Error(`Linear API error: ${res.status} ${res.statusText}`);
    }

    return res.json() as Promise<T>;
  }
}

function priorityLabel(priority: number): string {
  switch (priority) {
    case 1: return "Urgent";
    case 2: return "High";
    case 3: return "Medium";
    case 4: return "Low";
    default: return "None";
  }
}
