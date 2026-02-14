import { readFileSync } from "fs";
import { join } from "path";
import { homedir } from "os";
import { parse } from "smol-toml";
import { z } from "zod";

const LinearConfigSchema = z.object({
  api_key: z.string(),
});

const TrelloConfigSchema = z.object({
  api_key: z.string(),
  token: z.string(),
});

const JiraConfigSchema = z.object({
  domain: z.string(),
  email: z.string(),
  api_token: z.string(),
});

const WakaTimeConfigSchema = z.object({
  api_key: z.string(),
});

const RescueTimeConfigSchema = z.object({
  api_key: z.string(),
});

const GitHubConfigSchema = z.object({
  owner: z.string().optional(),
  repo: z.string().optional(),
});

const AgentsConfigSchema = z.object({
  repo_root: z.string().optional(),
  webhook_port: z.number().optional(),
  webhook_secret: z.string().optional(),
});

const AppConfigSchema = z.object({
  linear: LinearConfigSchema.optional(),
  trello: TrelloConfigSchema.optional(),
  jira: JiraConfigSchema.optional(),
  github: GitHubConfigSchema.optional(),
  wakatime: WakaTimeConfigSchema.optional(),
  rescuetime: RescueTimeConfigSchema.optional(),
  agents: AgentsConfigSchema.optional(),
});

export type AppConfig = z.infer<typeof AppConfigSchema>;
export type LinearConfig = z.infer<typeof LinearConfigSchema>;
export type TrelloConfig = z.infer<typeof TrelloConfigSchema>;
export type JiraConfig = z.infer<typeof JiraConfigSchema>;

export function loadConfig(): AppConfig {
  const configPath = join(homedir(), ".localpipeline", "config.toml");
  let content: string;
  try {
    content = readFileSync(configPath, "utf-8");
  } catch {
    return {};
  }
  const raw = parse(content);
  return AppConfigSchema.parse(raw);
}
