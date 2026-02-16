import { readFileSync, writeFileSync, mkdirSync } from "fs";
import { join } from "path";
import { homedir } from "os";
import type { Agent, AgentName } from "../model/agent.js";
import { AGENT_NAMES } from "../model/agent.js";

interface StoreData {
  agents: Record<AgentName, Agent>;
}

function defaultAgent(name: AgentName): Agent {
  return { name, status: "idle" };
}

function defaultData(): StoreData {
  return {
    agents: {
      ember: defaultAgent("ember"),
      flow: defaultAgent("flow"),
      tempest: defaultAgent("tempest"),
      terra: defaultAgent("terra"),
    },
  };
}

export class AgentStore {
  private filePath: string;
  private data: StoreData;

  constructor(customPath?: string) {
    if (customPath) {
      this.filePath = customPath;
    } else {
      const dir = join(homedir(), ".localpipeline");
      mkdirSync(dir, { recursive: true });
      this.filePath = join(dir, "agents.json");
    }
    this.data = this.load();
    this.cleanStaleProcesses();
  }

  private load(): StoreData {
    try {
      const content = readFileSync(this.filePath, "utf-8");
      const parsed = JSON.parse(content);
      const defaults = defaultData();
      for (const name of AGENT_NAMES) {
        if (parsed.agents?.[name]) {
          defaults.agents[name] = { ...defaults.agents[name], ...parsed.agents[name] };
        }
      }
      return defaults;
    } catch {
      return defaultData();
    }
  }

  private save(): void {
    writeFileSync(this.filePath, JSON.stringify(this.data, null, 2));
  }

  private cleanStaleProcesses(): void {
    let changed = false;
    for (const name of AGENT_NAMES) {
      const agent = this.data.agents[name];
      if (agent.pid && (agent.status === "working" || agent.status === "provisioning")) {
        try {
          process.kill(agent.pid, 0);
        } catch {
          agent.status = "error";
          agent.error = "Process died unexpectedly";
          agent.pid = undefined;
          changed = true;
        }
      }
    }
    if (changed) this.save();
  }

  getAll(): Agent[] {
    return AGENT_NAMES.map((n) => this.data.agents[n]);
  }

  getAgent(name: AgentName): Agent {
    return this.data.agents[name];
  }

  updateAgent(name: AgentName, partial: Partial<Agent>): void {
    this.data.agents[name] = { ...this.data.agents[name], ...partial };
    this.save();
  }

  getNextFreeAgent(): AgentName | undefined {
    return AGENT_NAMES.find((n) => this.data.agents[n].status === "idle");
  }

  markBusy(name: AgentName, workItemId: string, workItemTitle: string, workItemSource: string, branch: string, worktreePath: string, pid: number): void {
    this.data.agents[name] = {
      name,
      status: "working",
      workItemId,
      workItemTitle,
      workItemSource,
      branch,
      worktreePath,
      pid,
      startedAt: new Date().toISOString(),
    };
    this.save();
  }

  markDone(name: AgentName): void {
    const agent = this.data.agents[name];
    this.data.agents[name] = {
      ...agent,
      status: "done",
      pid: undefined,
    };
    this.save();
  }

  markError(name: AgentName, error: string): void {
    const agent = this.data.agents[name];
    this.data.agents[name] = {
      ...agent,
      status: "error",
      error,
      pid: undefined,
    };
    this.save();
  }

  incrementRetry(name: AgentName): number {
    const agent = this.data.agents[name];
    const count = (agent.retryCount ?? 0) + 1;
    this.data.agents[name] = { ...agent, retryCount: count };
    this.save();
    return count;
  }

  release(name: AgentName): void {
    this.data.agents[name] = defaultAgent(name);
    this.save();
  }

  reload(): void {
    this.data = this.load();
    this.cleanStaleProcesses();
  }
}
