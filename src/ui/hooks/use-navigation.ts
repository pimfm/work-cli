import { useState, useCallback } from "react";
import { useInput, useApp } from "ink";

export type DashboardMode = "normal" | "time-expanded" | "agents";

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

  const mode = modeStack[modeStack.length - 1]!;
  const canGoBack = modeStack.length > 1;

  const navigateTo = useCallback((target: DashboardMode) => {
    setModeStack((stack) => {
      if (stack[stack.length - 1] === target) return stack;
      return [...stack, target];
    });
  }, []);

  const navigateBack = useCallback(() => {
    setModeStack((stack) => {
      if (stack.length <= 1) return stack;
      return stack.slice(0, -1);
    });
  }, []);

  const breadcrumbs = modeStack.map((m) => MODE_LABELS[m]);

  useInput((input, key) => {
    if (input === "q" || key.escape) {
      if (canGoBack) {
        navigateBack();
        return;
      }
      exit();
      return;
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
      if (mode === "agents") {
        navigateBack();
      } else {
        navigateTo("agents");
      }
    }
    if (input === "r") {
      callbacks?.onRefresh?.();
    }
    if (input === "b") {
      navigateBack();
    }
  });

  const clampedIndex = Math.min(selectedIndex, Math.max(0, itemCount - 1));

  return { selectedIndex: clampedIndex, mode, breadcrumbs, canGoBack };
}
