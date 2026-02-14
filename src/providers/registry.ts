import type { AppConfig } from "../config/config.js";
import type { WorkItemProvider } from "./provider.js";
import { LinearProvider } from "./linear/linear-provider.js";
import { TrelloProvider } from "./trello/trello-provider.js";
import { JiraProvider } from "./jira/jira-provider.js";
import { GitHubProvider } from "./github/github-provider.js";

export function createProviders(config: AppConfig): WorkItemProvider[] {
  const providers: WorkItemProvider[] = [];

  if (config.linear) {
    providers.push(new LinearProvider(config.linear.api_key));
  }
  if (config.trello) {
    providers.push(new TrelloProvider(config.trello.api_key, config.trello.token));
  }
  if (config.jira) {
    providers.push(
      new JiraProvider(config.jira.domain, config.jira.email, config.jira.api_token)
    );
  }
  if (config.github) {
    providers.push(new GitHubProvider(config.github.owner, config.github.repo));
  }

  return providers;
}
