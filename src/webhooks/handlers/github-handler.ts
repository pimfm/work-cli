import type { WorkItem } from "../../model/work-item.js";
import type { CardStatus } from "../../providers/provider.js";

interface GitHubWebhookBody {
  action?: string;
  issue?: {
    number?: number;
    title?: string;
    body?: string;
    html_url?: string;
    labels?: Array<{ name?: string }>;
    state?: string;
  };
  pull_request?: {
    merged?: boolean;
    head?: { ref?: string };
    title?: string;
  };
  repository?: {
    full_name?: string;
  };
}

export interface CardStatusChange {
  itemId: string;
  status: CardStatus;
}

export function parseGitHubWebhook(body: GitHubWebhookBody): WorkItem | undefined {
  const action = body.action;
  if (action !== "assigned" && action !== "labeled") return undefined;

  const issue = body.issue;
  if (!issue?.number || !issue?.title) return undefined;

  return {
    id: `#${issue.number}`,
    title: issue.title,
    description: issue.body ?? undefined,
    status: issue.state,
    labels: issue.labels?.map((l) => l.name).filter((n): n is string => !!n) ?? [],
    source: "GitHub",
    team: body.repository?.full_name,
    url: issue.html_url,
  };
}

/**
 * Extract a work item ID from an agent branch name.
 * Branch format: agent/{agentName}/{itemId}-{slug}
 */
function extractItemIdFromBranch(branch: string): string | undefined {
  const match = branch.match(/^agent\/\w+\/([^-]+-?\d*)/);
  if (!match) return undefined;
  // The item ID can be like "LIN-42", "PROJ-99", "#123", or "69932610"
  // The branch format is: agent/{name}/{id}-{slug}
  // We need to extract just the ID part before the slug
  const parts = branch.replace(/^agent\/\w+\//, "").split("-");
  // Check if it looks like a compound ID (e.g., LIN-42, PROJ-99)
  if (parts.length >= 2 && /^[A-Z]+$/.test(parts[0]!) && /^\d+/.test(parts[1]!)) {
    return `${parts[0]}-${parts[1]}`;
  }
  // Otherwise it's a simple ID (e.g., 69932610 or abcd1234)
  return parts[0];
}

export function parseGitHubPrMerge(body: GitHubWebhookBody): CardStatusChange | undefined {
  if (body.action !== "closed") return undefined;
  if (!body.pull_request?.merged) return undefined;

  const branch = body.pull_request.head?.ref;
  if (!branch) return undefined;

  const itemId = extractItemIdFromBranch(branch);
  if (!itemId) return undefined;

  return { itemId, status: "done" };
}
