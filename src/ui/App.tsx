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
import { useTimeStore } from "../persistence/store-context.js";
import { StoreProvider } from "../persistence/store-context.js";
import { ItemList } from "./ItemList.js";
import { DetailPanel } from "./DetailPanel.js";
import { TimePanel } from "./TimePanel.js";
import { Footer } from "./Footer.js";
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
  const state = useWorkItems(providers);
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
      providers={providers}
      contentHeight={contentHeight}
      wakatime={wakatime}
      rescuetime={rescuetime}
    />
  );
}

function Dashboard({
  items,
  providers,
  contentHeight,
  wakatime,
  rescuetime,
}: {
  items: { id: string; title: string; description?: string; status?: string; priority?: string; labels: string[]; source: string; team?: string; url?: string }[];
  providers: WorkItemProvider[];
  contentHeight: number;
  wakatime?: WakaTimeProvider;
  rescuetime?: RescueTimeProvider;
}) {
  const store = useTimeStore();
  const { activeTimer, toggleTimer, stopTimer, isTrackingItem } = useTimeTracking(store);
  const { analytics, localStats } = useAnalytics(store, wakatime, rescuetime);

  const selectedRef = React.useRef(0);

  const onEnter = useCallback(() => {
    const item = items[selectedRef.current];
    if (item) toggleTimer(item);
  }, [items, toggleTimer]);

  const onComplete = useCallback(() => {
    stopTimer();
  }, [stopTimer]);

  const { selectedIndex, mode } = useNavigation(items.length, { onEnter, onComplete });
  selectedRef.current = selectedIndex;
  const sources = [...new Set(items.map((i) => i.source))].join(" | ");
  const selectedItem = items[selectedIndex]!;

  return (
    <Box flexDirection="column" borderStyle="round" borderColor="cyan">
      <Box paddingX={1} justifyContent="space-between">
        <Text bold> work pipeline</Text>
        <Text dimColor>{sources}</Text>
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
      ) : (
        <Box height={contentHeight + 3} overflow="hidden">
          <ItemList items={items} selectedIndex={selectedIndex} height={contentHeight} isTrackingItem={isTrackingItem} />
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

      <Footer />
    </Box>
  );
}
