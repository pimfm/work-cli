import { useState, useCallback } from "react";
import { useInput, useApp } from "ink";
import type { AgentName } from "../../model/agent.js";
import { AGENT_NAMES } from "../../model/agent.js";

export type DashboardMode = "normal" | "time-expanded" | "agents";
export type AgentSubMode = "list" | "detail";

export const MODE_LABELS: Record<DashboardMode, string> = {
  "normal": "Dashboard",
  "time-expanded": "Time Analytics",
  "agents": "Agents",
};

interface NavigationCallbacks {
  onEnter?: () => void;
  onComplete?: () => void;
  onDispatch?: () => void;
  onRefresh?: () => void;
}

export function useNavigation(itemCount: number, callbacks?: NavigationCallbacks) {
  const { exit } = useApp();
  const [selectedIndex, setSelectedIndex] = useState(0);
  const [modeStack, setModeStack] = useState<DashboardMode[]>(["normal"]);
  const [forwardStack, setForwardStack] = useState<DashboardMode[]>([]);
  const [agentSelectedIndex, setAgentSelectedIndex] = useState(0);
  const [expandedAgent, setExpandedAgent] = useState<AgentName | null>(null);
  const [agentSubMode, setAgentSubMode] = useState<AgentSubMode>("list");
  const [detailScrollOffset, setDetailScrollOffset] = useState(0);

  const mode = modeStack[modeStack.length - 1]!;
  const canGoBack = modeStack.length > 1;
  const canGoForward = forwardStack.length > 0;

  const navigateTo = useCallback((target: DashboardMode) => {
    setModeStack((stack) => {
      if (stack[stack.length - 1] === target) return stack;
      return [...stack, target];
    });
    setForwardStack([]);
  }, []);

  const navigateBack = useCallback(() => {
    setModeStack((stack) => {
      if (stack.length <= 1) return stack;
      return stack.slice(0, -1);
    });
    setForwardStack((fwd) => {
      const currentMode = modeStack[modeStack.length - 1];
      if (modeStack.length <= 1 || !currentMode) return fwd;
      return [currentMode, ...fwd];
    });
  }, [modeStack]);

  const navigateForward = useCallback(() => {
    const next = forwardStack[0];
    if (!next) return;
    setModeStack((stack) => [...stack, next]);
    setForwardStack((fwd) => fwd.slice(1));
  }, [forwardStack]);

  const breadcrumbs = modeStack.map((m) => MODE_LABELS[m]);
  const forwardBreadcrumbs = forwardStack.map((m) => MODE_LABELS[m]);

  useInput((input, key) => {
    // Agent detail mode
    if (mode === "agents" && agentSubMode === "detail") {
      if (key.escape || key.backspace || key.delete || key.leftArrow) {
        setAgentSubMode("list");
        setExpandedAgent(null);
        setDetailScrollOffset(0);
        return;
      }
      if (key.upArrow) {
        setDetailScrollOffset((o) => Math.max(0, o - 1));
        return;
      }
      if (key.downArrow) {
        setDetailScrollOffset((o) => o + 1);
        return;
      }
      if (input === "q") {
        exit();
        return;
      }
      return;
    }

    // Agent list mode
    if (mode === "agents") {
      if (key.escape || input === "a" || input === "b" || key.backspace || key.delete || key.leftArrow) {
        navigateBack();
        return;
      }
      if (key.rightArrow) {
        navigateForward();
        return;
      }
      if (input === "t") {
        navigateTo("time-expanded");
        return;
      }
      if (key.upArrow) {
        setAgentSelectedIndex((i) => Math.max(0, i - 1));
        return;
      }
      if (key.downArrow) {
        setAgentSelectedIndex((i) => Math.min(AGENT_NAMES.length - 1, i + 1));
        return;
      }
      if (key.return) {
        setExpandedAgent(AGENT_NAMES[agentSelectedIndex]!);
        setAgentSubMode("detail");
        setDetailScrollOffset(0);
        return;
      }
      if (input === "q") {
        exit();
        return;
      }
      return;
    }

    // Normal / time-expanded mode
    if (input === "q" || key.escape) {
      if (canGoBack) {
        navigateBack();
        return;
      }
      exit();
      return;
    }

    if (key.leftArrow) {
      if (canGoBack) {
        navigateBack();
        return;
      }
    }

    if (key.rightArrow) {
      if (canGoForward) {
        navigateForward();
        return;
      }
    }

    if (key.backspace || key.delete) {
      if (canGoBack) {
        navigateBack();
        return;
      }
    }

    if (key.upArrow) {
      setSelectedIndex((i) => Math.max(0, i - 1));
    }
    if (key.downArrow) {
      setSelectedIndex((i) => Math.min(itemCount - 1, i + 1));
    }
    if (key.return) {
      callbacks?.onEnter?.();
    }
    if (input === "t") {
      if (mode === "time-expanded") {
        navigateBack();
      } else {
        navigateTo("time-expanded");
      }
    }
    if (input === "c") {
      callbacks?.onComplete?.();
    }
    if (input === "d") {
      callbacks?.onDispatch?.();
    }
    if (input === "a") {
      navigateTo("agents");
    }
    if (input === "r") {
      callbacks?.onRefresh?.();
    }
    if (input === "b") {
      navigateBack();
    }
  });

  const clampedIndex = Math.min(selectedIndex, Math.max(0, itemCount - 1));

  return {
    selectedIndex: clampedIndex,
    mode,
    breadcrumbs,
    forwardBreadcrumbs,
    canGoBack,
    canGoForward,
    agentSelectedIndex,
    expandedAgent,
    agentSubMode,
    detailScrollOffset,
  };
}
