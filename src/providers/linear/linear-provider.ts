import type { WorkItem } from "../../model/work-item.js";
import type { WorkItemProvider } from "../provider.js";
import type { LinearGraphQLResponse } from "./linear-types.js";

export class LinearProvider implements WorkItemProvider {
  name = "Linear";

  constructor(private apiKey: string) {}

  private async gql(query: string): Promise<Response> {
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
    return res;
  }

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

    const res = await this.gql(query);
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

  async markDone(item: WorkItem): Promise<void> {
    const lookupQuery = `{
      issues(filter: { identifier: { eq: "${item.id}" } }, first: 1) {
        nodes { id, team { id } }
      }
    }`;

    const lookupRes = await this.gql(lookupQuery);
    const lookupJson = await lookupRes.json() as { data?: { issues: { nodes: { id: string; team: { id: string } }[] } } };
    const issue = lookupJson.data?.issues.nodes[0];
    if (!issue) {
      throw new Error(`Linear issue not found: ${item.id}`);
    }

    const statesQuery = `{
      workflowStates(filter: { team: { id: { eq: "${issue.team.id}" } }, type: { eq: "completed" } }, first: 1) {
        nodes { id }
      }
    }`;

    const statesRes = await this.gql(statesQuery);
    const statesJson = await statesRes.json() as { data?: { workflowStates: { nodes: { id: string }[] } } };
    const doneState = statesJson.data?.workflowStates.nodes[0];
    if (!doneState) {
      throw new Error(`No completed state found for team of ${item.id}`);
    }

    const mutation = `mutation {
      issueUpdate(id: "${issue.id}", input: { stateId: "${doneState.id}" }) {
        success
      }
    }`;

    await this.gql(mutation);
  }

  async addComment(itemId: string, comment: string): Promise<void> {
    const lookupQuery = `{
      issues(filter: { identifier: { eq: "${itemId}" } }, first: 1) {
        nodes { id }
      }
    }`;

    const lookupRes = await this.gql(lookupQuery);
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

    await this.gql(mutation);
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
