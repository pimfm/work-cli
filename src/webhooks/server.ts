import { createServer, type IncomingMessage, type ServerResponse } from "http";
import { parseTrelloWebhook } from "./handlers/trello-handler.js";
import { parseJiraWebhook } from "./handlers/jira-handler.js";
import { parseLinearWebhook } from "./handlers/linear-handler.js";
import { parseClickUpWebhook } from "./handlers/clickup-handler.js";
import { parseAsanaWebhook } from "./handlers/asana-handler.js";
import { parseGitHubWebhook, parseGitHubPrMerge } from "./handlers/github-handler.js";
import { webhookDispatch } from "./webhook-dispatcher.js";
import { moveCard } from "../agents/card-mover.js";
import type { WorkItem } from "../model/work-item.js";
import type { WorkItemProvider } from "../providers/provider.js";

type Parser = (body: any) => WorkItem | undefined;

const HANDLERS: Record<string, Parser> = {
  trello: parseTrelloWebhook,
  jira: parseJiraWebhook,
  linear: parseLinearWebhook,
  clickup: parseClickUpWebhook,
  asana: parseAsanaWebhook,
  github: parseGitHubWebhook,
};

function readBody(req: IncomingMessage): Promise<string> {
  return new Promise((resolve, reject) => {
    let data = "";
    req.on("data", (chunk) => (data += chunk));
    req.on("end", () => resolve(data));
    req.on("error", reject);
  });
}

function json(res: ServerResponse, status: number, body: object): void {
  res.writeHead(status, { "Content-Type": "application/json" });
  res.end(JSON.stringify(body));
}

export function startWebhookServer(
  port: number,
  repoRoot: string,
  secret?: string,
  providers?: WorkItemProvider[],
): ReturnType<typeof createServer> {
  const server = createServer(async (req, res) => {
    const url = req.url ?? "";

    // Health check
    if (req.method === "GET" && url === "/health") {
      json(res, 200, { status: "ok" });
      return;
    }

    // HEAD requests for webhook verification (Trello sends HEAD to verify URL)
    if (req.method === "HEAD") {
      res.writeHead(200);
      res.end();
      return;
    }

    // Webhook routes
    const match = url.match(/^\/webhook\/(\w+)$/);
    if (req.method !== "POST" || !match) {
      json(res, 404, { error: "Not found" });
      return;
    }

    const provider = match[1]!;
    const parser = HANDLERS[provider];
    if (!parser) {
      json(res, 400, { error: `Unknown provider: ${provider}` });
      return;
    }

    // Optional secret verification
    if (secret) {
      const headerSecret = req.headers["x-webhook-secret"] as string | undefined;
      if (headerSecret !== secret) {
        json(res, 401, { error: "Invalid webhook secret" });
        return;
      }
    }

    try {
      const raw = await readBody(req);
      const body = JSON.parse(raw);

      // Check for PR merge events (GitHub only) → move card to "Done"
      if (provider === "github" && providers) {
        const prMerge = parseGitHubPrMerge(body);
        if (prMerge) {
          await moveCard(providers, prMerge.itemId, prMerge.status);
          json(res, 200, { cardMoved: true, itemId: prMerge.itemId, status: prMerge.status });
          return;
        }
      }

      const item = parser(body);

      if (!item) {
        json(res, 200, { ignored: true, reason: "Event not actionable" });
        return;
      }

      const result = await webhookDispatch(item, repoRoot, providers);
      if (result.dispatched) {
        json(res, 200, { dispatched: true, agent: result.agent, item: item.id });
      } else {
        json(res, 200, { dispatched: false, queued: true, item: item.id });
      }
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      json(res, 500, { error: message });
    }
  });

  server.listen(port, () => {
    console.log(`Webhook server listening on http://localhost:${port}`);
    console.log(`Routes: POST /webhook/{trello,jira,linear,clickup,asana,github}`);
    console.log(`Health: GET /health`);
  });

  return server;
}
