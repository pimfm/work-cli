import { spawn } from "child_process";
import { createWriteStream, mkdirSync, rmSync, existsSync } from "fs";
import { join } from "path";
import { homedir } from "os";
import { execSync } from "child_process";
import type { AgentName } from "../model/agent.js";
import type { WorkItem } from "../model/work-item.js";
import { AgentStore } from "./agent-store.js";
import { branchName, worktreePath } from "./branch-utils.js";
import { buildClaudePrompt } from "./claude-prompt.js";
import { writeClaudeMd } from "./claude-md.js";
import { AGENTS } from "../model/agent.js";
import { appendEvent } from "../persistence/agent-log.js";

export async function dispatchToAgent(
  agentName: AgentName,
  item: WorkItem,
  repoRoot: string,
  store?: AgentStore,
): Promise<void> {
  const agentStore = store ?? new AgentStore();
  const branch = branchName(agentName, item.id, item.title);
  const wtPath = worktreePath(repoRoot, agentName);

  agentStore.updateAgent(agentName, {
    status: "provisioning",
    workItemId: item.id,
    workItemTitle: item.title,
    workItemSource: item.source,
    branch,
    worktreePath: wtPath,
  });

  appendEvent({
    timestamp: new Date().toISOString(),
    agent: agentName,
    event: "dispatched",
    workItemId: item.id,
    workItemTitle: item.title,
  });

  try {
    // Fetch latest main from origin
    execSync("git fetch origin main", { cwd: repoRoot, stdio: "pipe" });

    // Clean up existing worktree if it exists
    if (existsSync(wtPath)) {
      try {
        execSync(`git worktree remove ${wtPath} --force`, { cwd: repoRoot, stdio: "pipe" });
      } catch {
        rmSync(wtPath, { recursive: true, force: true });
        try {
          execSync("git worktree prune", { cwd: repoRoot, stdio: "pipe" });
        } catch {
          // ignore prune errors
        }
      }
    }

    // Create or reset branch from origin/main
    try {
      execSync(`git branch ${branch} origin/main`, { cwd: repoRoot, stdio: "pipe" });
    } catch {
      execSync(`git branch -f ${branch} origin/main`, { cwd: repoRoot, stdio: "pipe" });
    }

    // Create worktree
    execSync(`git worktree add ${wtPath} ${branch}`, { cwd: repoRoot, stdio: "pipe" });

    // Write CLAUDE.md
    writeClaudeMd(wtPath, AGENTS[agentName].display);

    // npm install in worktree
    execSync("npm install", { cwd: wtPath, stdio: "pipe" });

    // Build prompt
    const prompt = buildClaudePrompt(item, AGENTS[agentName].display);

    // Set up log file
    const logDir = join(homedir(), ".localpipeline", "logs");
    mkdirSync(logDir, { recursive: true });
    const logPath = join(logDir, `agent-${agentName}.log`);
    const logStream = createWriteStream(logPath, { flags: "w" });

    // Spawn claude process
    const child = spawn("claude", ["-p", prompt, "--dangerously-skip-permissions"], {
      cwd: wtPath,
      stdio: ["ignore", "pipe", "pipe"],
    });

    child.stdout.pipe(logStream);
    child.stderr.pipe(logStream);

    // Mark as working with PID
    agentStore.markBusy(agentName, item.id, item.title, item.source, branch, wtPath, child.pid!);

    appendEvent({
      timestamp: new Date().toISOString(),
      agent: agentName,
      event: "working",
      workItemId: item.id,
      workItemTitle: item.title,
    });

    // Handle exit
    child.on("close", (code) => {
      logStream.close();
      if (code === 0) {
        agentStore.markDone(agentName);
        appendEvent({
          timestamp: new Date().toISOString(),
          agent: agentName,
          event: "done",
          workItemId: item.id,
          workItemTitle: item.title,
        });
      } else {
        const message = `Process exited with code ${code}`;
        agentStore.markError(agentName, message);
        appendEvent({
          timestamp: new Date().toISOString(),
          agent: agentName,
          event: "error",
          workItemId: item.id,
          workItemTitle: item.title,
          message,
        });
      }
    });

    child.on("error", (err) => {
      logStream.close();
      agentStore.markError(agentName, err.message);
      appendEvent({
        timestamp: new Date().toISOString(),
        agent: agentName,
        event: "error",
        workItemId: item.id,
        workItemTitle: item.title,
        message: err.message,
      });
    });
  } catch (err) {
    const message = err instanceof Error ? err.message : String(err);
    agentStore.markError(agentName, message);
    appendEvent({
      timestamp: new Date().toISOString(),
      agent: agentName,
      event: "error",
      workItemId: item.id,
      workItemTitle: item.title,
      message,
    });
  }
}

export async function retryAgent(
  agentName: AgentName,
  repoRoot: string,
  store: AgentStore,
): Promise<void> {
  const agent = store.getAgent(agentName);
  if (!agent.workItemId || !agent.workItemTitle || !agent.branch) {
    throw new Error(`Agent ${agentName} missing work item info for retry`);
  }

  const wtPath = worktreePath(repoRoot, agentName);

  // Clean up old worktree
  try {
    execSync(`git worktree remove ${wtPath} --force`, { cwd: repoRoot, stdio: "pipe" });
  } catch {
    // Worktree may already be removed; clean up directory if it lingers
    if (existsSync(wtPath)) {
      rmSync(wtPath, { recursive: true, force: true });
    }
    try {
      execSync("git worktree prune", { cwd: repoRoot, stdio: "pipe" });
    } catch {
      // ignore prune errors
    }
  }

  // Fetch latest main from origin
  execSync("git fetch origin main", { cwd: repoRoot, stdio: "pipe" });

  // Reset the existing branch to origin/main
  try {
    execSync(`git branch -f ${agent.branch} origin/main`, { cwd: repoRoot, stdio: "pipe" });
  } catch {
    // Branch may not exist; create it
    execSync(`git branch ${agent.branch} origin/main`, { cwd: repoRoot, stdio: "pipe" });
  }

  // Re-create worktree
  execSync(`git worktree add ${wtPath} ${agent.branch}`, { cwd: repoRoot, stdio: "pipe" });

  // Reconstruct a WorkItem for dispatch
  const item: WorkItem = {
    id: agent.workItemId,
    title: agent.workItemTitle,
    labels: [],
    source: agent.workItemSource ?? "",
  };

  // Write CLAUDE.md
  writeClaudeMd(wtPath, AGENTS[agentName].display);

  // npm install in worktree
  execSync("npm install", { cwd: wtPath, stdio: "pipe" });

  // Build prompt
  const prompt = buildClaudePrompt(item, AGENTS[agentName].display);

  // Set up log file
  const logDir = join(homedir(), ".localpipeline", "logs");
  mkdirSync(logDir, { recursive: true });
  const logPath = join(logDir, `agent-${agentName}.log`);
  const logStream = createWriteStream(logPath, { flags: "w" });

  // Spawn claude process
  const child = spawn("claude", ["-p", prompt, "--dangerously-skip-permissions"], {
    cwd: wtPath,
    stdio: ["ignore", "pipe", "pipe"],
  });

  child.stdout.pipe(logStream);
  child.stderr.pipe(logStream);

  // Mark as working with PID
  store.markBusy(agentName, item.id, item.title, item.source, agent.branch, wtPath, child.pid!);

  appendEvent({
    timestamp: new Date().toISOString(),
    agent: agentName,
    event: "working",
    workItemId: item.id,
    workItemTitle: item.title,
  });

  // Handle exit
  child.on("close", (code) => {
    logStream.close();
    if (code === 0) {
      store.markDone(agentName);
      appendEvent({
        timestamp: new Date().toISOString(),
        agent: agentName,
        event: "done",
        workItemId: item.id,
        workItemTitle: item.title,
      });
    } else {
      const message = `Process exited with code ${code}`;
      store.markError(agentName, message);
      appendEvent({
        timestamp: new Date().toISOString(),
        agent: agentName,
        event: "error",
        workItemId: item.id,
        workItemTitle: item.title,
        message,
      });
    }
  });

  child.on("error", (err) => {
    logStream.close();
    store.markError(agentName, err.message);
    appendEvent({
      timestamp: new Date().toISOString(),
      agent: agentName,
      event: "error",
      workItemId: item.id,
      workItemTitle: item.title,
      message: err.message,
    });
  });
}
