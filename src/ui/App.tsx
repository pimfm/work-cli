import React, { useCallback, useMemo } from "react";
import { Box, Text, useStdout } from "ink";
import type { WorkItemProvider } from "../providers/provider.js";
import type { WakaTimeProvider } from "../providers/analytics/wakatime-provider.js";
import type { RescueTimeProvider } from "../providers/analytics/rescuetime-provider.js";
import type { TimeStore } from "../persistence/time-store.js";
import { useWorkItems } from "./hooks/use-work-items.js";
import { useNavigation } from "./hooks/use-navigation.js";
import { useTimeTracking } from "./hooks/use-time-tracking.js";
import { useAnalytics } from "./hooks/use-analytics.js";
import { useAgents } from "./hooks/use-agents.js";
import { useTimeStore } from "../persistence/store-context.js";
import { StoreProvider } from "../persistence/store-context.js";
import { ItemList } from "./ItemList.js";
import { DetailPanel } from "./DetailPanel.js";
import { TimePanel } from "./TimePanel.js";
import { AgentPanel } from "./AgentPanel.js";
import { Footer } from "./Footer.js";
import { Breadcrumb } from "./Breadcrumb.js";
import { Spinner } from "@inkjs/ui";

interface Props {
  providers: WorkItemProvider[];
  wakatime?: WakaTimeProvider;
  rescuetime?: RescueTimeProvider;
  store?: TimeStore;
}

export function App({ store, ...props }: Props) {
  return (
    <StoreProvider store={store}>
      <AppInner {...props} />
    </StoreProvider>
  );
}

function AppInner({ providers, wakatime, rescuetime }: Props) {
  const { refresh, ...state } = useWorkItems(providers);
  const { stdout } = useStdout();
  const rows = stdout.rows ?? 24;
  const contentHeight = Math.max(6, rows - 9);

  if (state.status === "loading") {
    return (
      <Box borderStyle="round" borderColor="cyan" padding={1} flexDirection="column">
        <Text bold> work pipeline</Text>
        <Box paddingX={1} paddingY={1}>
          <Spinner label={`Fetching work items from ${providers.map((p) => p.name).join(", ")}...`} />
        </Box>
      </Box>
    );
  }

  if (state.status === "error") {
    return (
      <Box borderStyle="round" borderColor="cyan" padding={1} flexDirection="column">
        <Text bold> work pipeline</Text>
        <Box paddingX={1}>
          <Text color="red">{state.message}</Text>
        </Box>
      </Box>
    );
  }

  if (state.items.length === 0) {
    return (
      <Box borderStyle="round" borderColor="cyan" padding={1} flexDirection="column">
        <Text bold> work pipeline</Text>
        <Box paddingX={1}>
          <Text color="yellow">No assigned work items found.</Text>
        </Box>
      </Box>
    );
  }

  return (
    <Dashboard
      items={state.items}
      isRefreshing={state.status === "refreshing"}
      providers={providers}
      contentHeight={contentHeight}
      wakatime={wakatime}
      rescuetime={rescuetime}
      onRefresh={refresh}
    />
  );
}

function Dashboard({
  items,
  isRefreshing,
  providers,
  contentHeight,
  wakatime,
  rescuetime,
  onRefresh,
}: {
  items: { id: string; title: string; description?: string; status?: string; priority?: string; labels: string[]; source: string; team?: string; url?: string }[];
  isRefreshing: boolean;
  providers: WorkItemProvider[];
  contentHeight: number;
  wakatime?: WakaTimeProvider;
  rescuetime?: RescueTimeProvider;
  onRefresh: () => void;
}) {
  const store = useTimeStore();
  const { activeTimer, toggleTimer, stopTimer, isTrackingItem } = useTimeTracking(store);
  const { analytics, localStats } = useAnalytics(store, wakatime, rescuetime);
  const { agents, dispatchItem, flashMessage, agentForItem } = useAgents();

  const selectedRef = React.useRef(0);

  const onEnter = useCallback(() => {
    const item = items[selectedRef.current];
    if (item) toggleTimer(item);
  }, [items, toggleTimer]);

  const onComplete = useCallback(() => {
    stopTimer();
  }, [stopTimer]);

  const onDispatch = useCallback(() => {
    const item = items[selectedRef.current];
    if (item) dispatchItem(item);
  }, [items, dispatchItem]);

  const { selectedIndex, mode, breadcrumbs, canGoBack } = useNavigation(items.length, { onEnter, onComplete, onDispatch, onRefresh });
  selectedRef.current = selectedIndex;
  const sources = [...new Set(items.map((i) => i.source))].join(" | ");
  const selectedItem = items[selectedIndex]!;

  return (
    <Box flexDirection="column" borderStyle="round" borderColor="cyan">
      <Box paddingX={1} justifyContent="space-between">
        <Box>
          <Text bold> work pipeline</Text>
          {canGoBack && (
            <>
              <Text dimColor>  </Text>
              <Breadcrumb items={breadcrumbs} />
            </>
          )}
        </Box>
        <Box>
          {isRefreshing && <Text color="cyan">refreshing...  </Text>}
          {flashMessage && <Text color="yellow">{flashMessage}  </Text>}
          <Text dimColor>{sources}</Text>
        </Box>
      </Box>

      {mode === "time-expanded" ? (
        <Box height={contentHeight + 3} overflow="hidden">
          <TimePanel
            activeTimer={activeTimer}
            localStats={localStats}
            analytics={analytics}
            expanded={true}
            height={contentHeight}
          />
        </Box>
      ) : mode === "agents" ? (
        <Box height={contentHeight + 3} overflow="hidden">
          <AgentPanel agents={agents} height={contentHeight} />
          <DetailPanel item={selectedItem} height={contentHeight} />
        </Box>
      ) : (
        <Box height={contentHeight + 3} overflow="hidden">
          <ItemList items={items} selectedIndex={selectedIndex} height={contentHeight} isTrackingItem={isTrackingItem} agentForItem={agentForItem} />
          <DetailPanel item={selectedItem} height={contentHeight} />
          <TimePanel
            activeTimer={activeTimer}
            localStats={localStats}
            analytics={analytics}
            expanded={false}
            height={contentHeight}
          />
        </Box>
      )}

      <Footer canGoBack={canGoBack} />
    </Box>
  );
}
