import type { AgentName } from "../model/agent.js";
import { PERSONALITIES } from "../model/personality.js";
import type { WorkItem } from "../model/work-item.js";

export function buildClaudePrompt(item: WorkItem, agentName: string): string {
  const personality = PERSONALITIES[agentName.toLowerCase() as AgentName];
  const personalityLine = personality
    ? ` Your personality: ${personality.tagline}.`
    : "";

  const lines: string[] = [
    `You are agent "${agentName}" working on the following task:${personalityLine}`,
    "",
    `# ${item.title}`,
    `- ID: ${item.id}`,
    `- Source: ${item.source}`,
  ];

  if (item.url) lines.push(`- URL: ${item.url}`);
  if (item.priority) lines.push(`- Priority: ${item.priority}`);
  if (item.labels.length > 0) lines.push(`- Labels: ${item.labels.join(", ")}`);
  if (item.status) lines.push(`- Status: ${item.status}`);
  if (item.team) lines.push(`- Team: ${item.team}`);

  if (item.description) {
    lines.push("", "## Description", item.description);
  }

  lines.push(
    "",
    "## Instructions",
    "1. Read CLAUDE.md in the project root for conventions and context.",
    "2. Implement the task described above.",
    "3. Write tests for your changes.",
    "4. Run `npm test` and ensure all tests pass.",
    `5. Commit your changes with a message referencing ${item.id}.`,
    `6. Run \`git push -u origin HEAD\`.`,
    `7. Run \`gh pr create --fill\` to open a pull request.`,
    "",
    "Work autonomously. Do not ask for clarification — make reasonable decisions.",
  );

  return lines.join("\n");
}
