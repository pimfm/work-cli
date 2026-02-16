import { useState, useEffect, useCallback, useRef } from "react";
import { appendFileSync, mkdirSync } from "fs";
import { join } from "path";
import { homedir } from "os";
import type { Agent, AgentName } from "../../model/agent.js";
import type { WorkItem } from "../../model/work-item.js";
import type { WorkItemProvider } from "../../providers/provider.js";
import { AgentStore } from "../../agents/agent-store.js";
import { dispatchToAgent, retryAgent } from "../../agents/dispatch.js";
import { appendEvent } from "../../persistence/agent-log.js";

const MAX_RETRIES = 3;

interface UseAgentsResult {
  agents: Agent[];
  dispatchItem: (item: WorkItem) => void;
  isDispatching: boolean;
  flashMessage?: string;
  agentForItem: (item: WorkItem) => Agent | undefined;
  releaseAgent: (name: AgentName) => void;
}

export function useAgents(repoRoot?: string, providers?: WorkItemProvider[]): UseAgentsResult {
  const storeRef = useRef<AgentStore | null>(null);
  if (!storeRef.current) {
    storeRef.current = new AgentStore();
  }
  const store = storeRef.current;

  const [agents, setAgents] = useState<Agent[]>(store.getAll());
  const [isDispatching, setIsDispatching] = useState(false);
  const [flashMessage, setFlashMessage] = useState<string | undefined>();
  const retryingRef = useRef<Set<AgentName>>(new Set());
  const completingRef = useRef<Set<AgentName>>(new Set());

  // Poll for updates every 2s
  useEffect(() => {
    const root = repoRoot ?? process.cwd();

    const interval = setInterval(() => {
      store.reload();
      const current = store.getAll();

      for (const agent of current) {
        // Mark work item done and auto-release completed agents
        if (agent.status === "done" && !completingRef.current.has(agent.name)) {
          completingRef.current.add(agent.name);
          markWorkItemDone(agent, providers ?? [])
            .finally(() => {
              appendEvent({
                timestamp: new Date().toISOString(),
                agent: agent.name,
                event: "released",
                workItemId: agent.workItemId,
                workItemTitle: agent.workItemTitle,
                message: "Auto-released after completion",
              });
              store.release(agent.name);
              completingRef.current.delete(agent.name);
            });
        }

        // Retry errored agents
        if (agent.status === "error" && !retryingRef.current.has(agent.name)) {
          const retryCount = agent.retryCount ?? 0;

          if (retryCount < MAX_RETRIES) {
            retryingRef.current.add(agent.name);
            store.incrementRetry(agent.name);
            appendEvent({
              timestamp: new Date().toISOString(),
              agent: agent.name,
              event: "retry",
              workItemId: agent.workItemId,
              workItemTitle: agent.workItemTitle,
              message: `Retry ${retryCount + 1}/${MAX_RETRIES}: ${agent.error ?? "Unknown error"}`,
            });
            retryAgent(agent.name, root, store)
              .catch(() => {
                // retryAgent already marks error via markError
              })
              .finally(() => {
                retryingRef.current.delete(agent.name);
              });
          } else {
            // Max retries exceeded — log, comment, and release
            retryingRef.current.add(agent.name);
            appendEvent({
              timestamp: new Date().toISOString(),
              agent: agent.name,
              event: "max-retries",
              workItemId: agent.workItemId,
              workItemTitle: agent.workItemTitle,
              message: `Failed after ${MAX_RETRIES} attempts: ${agent.error ?? "Unknown error"}`,
            });
            handleMaxRetriesExceeded(agent, providers ?? [])
              .finally(() => {
                appendEvent({
                  timestamp: new Date().toISOString(),
                  agent: agent.name,
                  event: "released",
                  workItemId: agent.workItemId,
                  workItemTitle: agent.workItemTitle,
                  message: "Released after max retries exceeded",
                });
                store.release(agent.name);
                retryingRef.current.delete(agent.name);
              });
          }
        }
      }

      setAgents(store.getAll());
    }, 2000);
    return () => clearInterval(interval);
  }, [store, repoRoot, providers]);

  const dispatchItem = useCallback(
    (item: WorkItem) => {
      const root = repoRoot ?? process.cwd();
      const freeAgent = store.getNextFreeAgent();
      if (!freeAgent) {
        setFlashMessage("All agents are busy");
        setTimeout(() => setFlashMessage(undefined), 3000);
        return;
      }

      setIsDispatching(true);
      dispatchToAgent(freeAgent, item, root, store).finally(() => {
        store.reload();
        setAgents(store.getAll());
        setIsDispatching(false);
      });
    },
    [store, repoRoot],
  );

  const agentForItem = useCallback(
    (item: WorkItem) => {
      return agents.find((a) => a.workItemId === item.id && a.status !== "idle");
    },
    [agents],
  );

  const releaseAgent = useCallback(
    (name: AgentName) => {
      store.release(name);
      setAgents(store.getAll());
    },
    [store],
  );

  return { agents, dispatchItem, isDispatching, flashMessage, agentForItem, releaseAgent };
}

export async function markWorkItemDone(agent: Agent, providers: WorkItemProvider[]): Promise<void> {
  if (!agent.workItemId || !agent.workItemSource) return;

  const provider = providers.find((p) => p.name === agent.workItemSource);
  if (!provider?.markItemDone) return;

  try {
    await provider.markItemDone(agent.workItemId);
    appendEvent({
      timestamp: new Date().toISOString(),
      agent: agent.name,
      event: "marked-done",
      workItemId: agent.workItemId,
      workItemTitle: agent.workItemTitle,
      message: `Marked as done in ${agent.workItemSource}`,
    });
  } catch (err) {
    const message = err instanceof Error ? err.message : String(err);
    appendEvent({
      timestamp: new Date().toISOString(),
      agent: agent.name,
      event: "mark-done-failed",
      workItemId: agent.workItemId,
      workItemTitle: agent.workItemTitle,
      message,
    });
  }
}

async function handleMaxRetriesExceeded(agent: Agent, providers: WorkItemProvider[]): Promise<void> {
  const errorMsg = agent.error ?? "Unknown error";
  const comment = `Agent ${agent.name} failed after ${MAX_RETRIES} attempts: ${errorMsg}`;

  // Log to file
  const logDir = join(homedir(), ".localpipeline", "logs");
  mkdirSync(logDir, { recursive: true });
  const logPath = join(logDir, `agent-${agent.name}-failures.log`);
  appendFileSync(logPath, `[${new Date().toISOString()}] ${comment}\n`);

  // Comment on the work item if we can find a matching provider
  if (agent.workItemId) {
    for (const provider of providers) {
      if (provider.addComment) {
        try {
          await provider.addComment(agent.workItemId, comment);
          break;
        } catch {
          // Provider didn't match or API failed — try next
        }
      }
    }
  }
}
