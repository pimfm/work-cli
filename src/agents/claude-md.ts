import { writeFileSync } from "fs";
import { join } from "path";
import type { AgentName } from "../model/agent.js";
import { PERSONALITIES } from "../model/personality.js";

export function writeClaudeMd(worktreePath: string, agentName: string): void {
  const personality = PERSONALITIES[agentName.toLowerCase() as AgentName];
  const personalitySection = personality
    ? `\n## Personality\n${personality.systemPrompt}\n`
    : "";

  const content = `# fm pipeline

## Project Overview
A terminal dashboard CLI (\`fm\`) that aggregates work items from Trello, Linear, and Jira.
Built with TypeScript, React Ink (terminal UI), and Node.js.

## Tech Stack
- **Runtime**: Node.js (ES2022)
- **Language**: TypeScript (strict mode)
- **UI**: React 18 + Ink 5 (terminal rendering)
- **Build**: tsup
- **Test**: vitest
- **Package manager**: npm

## Conventions
- ESM modules (\`"type": "module"\` in package.json)
- Imports use \`.js\` extension (e.g., \`import { foo } from "./bar.js"\`)
- No default exports — use named exports
- Zod for runtime validation at system boundaries
- Models in \`src/model/\`, providers in \`src/providers/\`, UI in \`src/ui/\`

## Testing
- Test files: \`src/__tests__/*.test.ts\` or \`src/__tests__/*.test.tsx\`
- Use \`describe/it/expect\` from vitest
- Mock external APIs, use real file system with temp directories
- Run: \`npm test\`

## Commit Format
- Short imperative subject line (e.g., "Add login validation")
- Reference the work item ID in the commit body

## Agent Identity
You are **${agentName}**, an autonomous agent working in a git worktree.
Your changes will be submitted as a pull request for review.
${personalitySection}`;

  writeFileSync(join(worktreePath, "CLAUDE.md"), content);
}
