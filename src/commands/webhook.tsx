import React from "react";
import { Box, Text } from "ink";
import { startWebhookServer } from "../webhooks/server.js";
import { loadConfig } from "../config/config.js";
import { createProviders } from "../providers/registry.js";

interface Props {
  port?: number;
  repoRoot?: string;
}

export function WebhookCommand({ port, repoRoot }: Props) {
  const config = loadConfig();
  const actualPort = port ?? config.agents?.webhook_port ?? 7890;
  const actualRoot = repoRoot ?? config.agents?.repo_root ?? process.cwd();
  const secret = config.agents?.webhook_secret;
  const providers = React.useMemo(() => createProviders(config), []);

  React.useEffect(() => {
    const server = startWebhookServer(actualPort, actualRoot, secret, providers);
    return () => {
      server.close();
    };
  }, [actualPort, actualRoot, secret, providers]);

  return (
    <Box flexDirection="column" paddingX={1}>
      <Text bold>fm webhook server</Text>
      <Text>Listening on port {actualPort}</Text>
      <Text dimColor>Repo root: {actualRoot}</Text>
      {secret && <Text dimColor>Webhook secret: configured</Text>}
      <Text> </Text>
      <Text dimColor>Press Ctrl+C to stop</Text>
    </Box>
  );
}
