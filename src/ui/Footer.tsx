import React from "react";
import { Box, Text } from "ink";
import type { DashboardMode, AgentSubMode } from "./hooks/use-navigation.js";

interface Props {
  canGoBack?: boolean;
  canGoForward?: boolean;
  mode?: DashboardMode;
  agentSubMode?: AgentSubMode;
}

export function Footer({ canGoBack, canGoForward, mode, agentSubMode }: Props) {
  let hints: string;

  const navHints = canGoBack || canGoForward
    ? "[←] back" + (canGoForward ? "  [→] forward" : "") + "  "
    : "";

  if (mode === "agents" && agentSubMode === "detail") {
    hints = "[↑/↓] scroll  [←/esc] back  [q] quit";
  } else if (mode === "agents") {
    hints = "[↑/↓] select agent  [enter] view log  " + navHints + "[a/esc] back  [q] quit";
  } else {
    hints =
      navHints +
      "[↑/↓] navigate  [enter] start/stop  [d] dispatch  [a] agents  [t] time  [r] refresh  [c] complete" +
      (canGoBack ? "" : "  [q/esc] quit");
  }

  return (
    <Box paddingX={1}>
      <Text dimColor>{hints}</Text>
    </Box>
  );
}
