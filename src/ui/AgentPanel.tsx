import React from "react";
import { Box, Text } from "ink";
import type { Agent } from "../model/agent.js";
import type { AgentName } from "../model/agent.js";
import { AGENTS } from "../model/agent.js";
import { PERSONALITIES } from "../model/personality.js";
import { agentStatusColor } from "./theme.js";
import { AgentDetail } from "./AgentDetail.js";

interface Props {
  agents: Agent[];
  height: number;
  selectedIndex: number;
  expandedAgent: AgentName | null;
  detailScrollOffset: number;
}

function elapsed(startedAt?: string): string {
  if (!startedAt) return "";
  const ms = Date.now() - new Date(startedAt).getTime();
  const mins = Math.floor(ms / 60000);
  const secs = Math.floor((ms % 60000) / 1000);
  return `${String(mins).padStart(2, "0")}:${String(secs).padStart(2, "0")}`;
}

export function AgentPanel({ agents, height, selectedIndex, expandedAgent, detailScrollOffset }: Props) {
  if (expandedAgent) {
    return <AgentDetail agentName={expandedAgent} height={height} scrollOffset={detailScrollOffset} />;
  }

  return (
    <Box flexDirection="column" borderStyle="round" borderColor="cyan" width="50%" height={height + 3} overflow="hidden">
      <Box paddingX={1}>
        <Text bold>Agents</Text>
      </Box>
      <Box flexDirection="column" paddingX={1} overflow="hidden">
        {agents.map((agent, idx) => {
          const info = AGENTS[agent.name];
          const isSelected = idx === selectedIndex;
          return (
            <Box key={agent.name} height={1} overflow="hidden">
              <Text color={isSelected ? "cyan" : undefined}>{isSelected ? "▸ " : "  "}</Text>
              <Text color={info.color}>{info.emoji} {info.display.padEnd(6)}</Text>
              <Text> </Text>
              <Text color={agentStatusColor(agent.status)}>
                {agent.status.padEnd(13)}
              </Text>
              {agent.status === "idle" && PERSONALITIES[agent.name] && (
                <>
                  <Text> </Text>
                  <Text dimColor italic>{PERSONALITIES[agent.name].tagline}</Text>
                  <Text dimColor>{" · "}</Text>
                  <Text dimColor>{PERSONALITIES[agent.name].focus}</Text>
                </>
              )}
              {agent.workItemTitle && (
                <>
                  <Text> </Text>
                  <Text dimColor>{truncate(agent.workItemTitle, 40)}</Text>
                </>
              )}
              {agent.startedAt && agent.status === "working" && (
                <>
                  <Text> </Text>
                  <Text color="yellow">{elapsed(agent.startedAt)}</Text>
                </>
              )}
              {agent.error && (
                <>
                  <Text> </Text>
                  <Text color="red">{truncate(agent.error, 30)}</Text>
                </>
              )}
            </Box>
          );
        })}
        {Array.from({ length: Math.max(0, height - agents.length) }).map((_, i) => (
          <Text key={`pad-${i}`}> </Text>
        ))}
      </Box>
    </Box>
  );
}

function truncate(str: string, max: number): string {
  if (str.length <= max) return str;
  return str.slice(0, max - 1) + "…";
}
