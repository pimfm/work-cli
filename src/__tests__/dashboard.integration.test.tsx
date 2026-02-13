import React from "react";
import { describe, it, expect, beforeEach, afterEach } from "vitest";
import { render } from "ink-testing-library";
import { App } from "../ui/App.js";
import { TimeStore } from "../persistence/time-store.js";
import type { WorkItemProvider } from "../providers/provider.js";
import type { WorkItem } from "../model/work-item.js";
import { mkdtempSync, rmSync } from "fs";
import { join } from "path";
import { tmpdir } from "os";

// ANSI escape sequences for key input
const ARROW_DOWN = "\x1B[B";
const ARROW_UP = "\x1B[A";
const ENTER = "\r";

const MOCK_ITEMS: WorkItem[] = [
  { id: "1", title: "Fix login bug", source: "Linear", labels: [], status: "In Progress", priority: "High" },
  { id: "2", title: "Add dark mode", source: "Trello", labels: ["frontend"], status: "Todo" },
  { id: "3", title: "Update API docs", source: "Jira", labels: [], status: "Open", priority: "Low" },
];

class MockProvider implements WorkItemProvider {
  name = "Mock";
  constructor(private items: WorkItem[]) {}
  async fetchAssignedItems(): Promise<WorkItem[]> {
    return this.items;
  }
}

function delay(ms: number): Promise<void> {
  return new Promise((r) => setTimeout(r, ms));
}

describe("Dashboard Integration", () => {
  let tmpDir: string;
  let store: TimeStore;
  let storePath: string;

  beforeEach(() => {
    tmpDir = mkdtempSync(join(tmpdir(), "lp-test-"));
    storePath = join(tmpDir, "time.json");
    store = new TimeStore(storePath);
  });

  afterEach(() => {
    rmSync(tmpDir, { recursive: true, force: true });
  });

  it("renders work items after loading", async () => {
    const provider = new MockProvider(MOCK_ITEMS);
    const { lastFrame, unmount } = render(<App providers={[provider]} store={store} />);

    // Wait for async fetch
    await delay(50);

    const frame = lastFrame();
    expect(frame).toContain("Fix login bug");
    expect(frame).toContain("Add dark mode");
    expect(frame).toContain("Update API docs");
    expect(frame).toContain("Work Items");
    expect(frame).toContain("work pipeline");

    unmount();
  });

  it("selects items with arrow keys", async () => {
    const provider = new MockProvider(MOCK_ITEMS);
    const { lastFrame, stdin, unmount } = render(<App providers={[provider]} store={store} />);

    await delay(50);

    // First item should be selected by default ("> " prefix)
    let frame = lastFrame();
    // The first item should have the ">" indicator
    expect(frame).toContain("Fix login bug");

    // Navigate down
    stdin.write(ARROW_DOWN);
    await delay(20);

    frame = lastFrame();
    // Detail panel should now show second item's info
    expect(frame).toContain("Detail");

    // Navigate down again
    stdin.write(ARROW_DOWN);
    await delay(20);

    frame = lastFrame();
    expect(frame).toContain("Update API docs");

    // Navigate up
    stdin.write(ARROW_UP);
    await delay(20);

    unmount();
  });

  it("starts a timer when pressing Enter on a selected item", async () => {
    const provider = new MockProvider(MOCK_ITEMS);
    const { lastFrame, stdin, unmount } = render(<App providers={[provider]} store={store} />);

    await delay(50);

    // Press Enter to start timer on first item
    stdin.write(ENTER);
    await delay(50);

    const frame = lastFrame();
    // Timer indicator should appear
    expect(frame).toContain("⏱");

    // Store should have an active timer
    const activeTimer = store.getActiveTimer();
    expect(activeTimer).toBeDefined();
    expect(activeTimer!.workItemId).toBe("1");
    expect(activeTimer!.workItemSource).toBe("Linear");
    expect(activeTimer!.workItemTitle).toBe("Fix login bug");

    unmount();
  });

  it("stops a timer when pressing Enter again on the same item", async () => {
    const provider = new MockProvider(MOCK_ITEMS);
    const { lastFrame, stdin, unmount } = render(<App providers={[provider]} store={store} />);

    await delay(50);

    // Start timer
    stdin.write(ENTER);
    await delay(50);

    expect(store.getActiveTimer()).toBeDefined();

    // Stop timer by pressing Enter again
    stdin.write(ENTER);
    await delay(50);

    // Active timer should be cleared
    expect(store.getActiveTimer()).toBeUndefined();

    // A completed entry should exist
    const entries = store.getAllEntries();
    expect(entries).toHaveLength(1);
    expect(entries[0]!.workItemId).toBe("1");
    expect(entries[0]!.workItemSource).toBe("Linear");
    expect(entries[0]!.workItemTitle).toBe("Fix login bug");
    expect(entries[0]!.endTime).toBeDefined();
    expect(entries[0]!.durationMinutes).toBeDefined();

    unmount();
  });

  it("switches timer when pressing Enter on a different item", async () => {
    const provider = new MockProvider(MOCK_ITEMS);
    const { stdin, unmount } = render(<App providers={[provider]} store={store} />);

    await delay(50);

    // Start timer on first item
    stdin.write(ENTER);
    await delay(50);

    expect(store.getActiveTimer()!.workItemId).toBe("1");

    // Navigate to second item and press Enter
    stdin.write(ARROW_DOWN);
    await delay(20);
    stdin.write(ENTER);
    await delay(50);

    // Timer should now be on second item
    const activeTimer = store.getActiveTimer();
    expect(activeTimer).toBeDefined();
    expect(activeTimer!.workItemId).toBe("2");
    expect(activeTimer!.workItemTitle).toBe("Add dark mode");

    // First item's timer should be saved as a completed entry
    const entries = store.getAllEntries();
    expect(entries).toHaveLength(1);
    expect(entries[0]!.workItemId).toBe("1");

    unmount();
  });

  it("stops timer with 'c' key", async () => {
    const provider = new MockProvider(MOCK_ITEMS);
    const { stdin, unmount } = render(<App providers={[provider]} store={store} />);

    await delay(50);

    // Start timer
    stdin.write(ENTER);
    await delay(50);

    expect(store.getActiveTimer()).toBeDefined();

    // Press 'c' to complete/stop
    stdin.write("c");
    await delay(50);

    expect(store.getActiveTimer()).toBeUndefined();
    expect(store.getAllEntries()).toHaveLength(1);

    unmount();
  });

  it("shows time panel with timer info", async () => {
    const provider = new MockProvider(MOCK_ITEMS);
    const { lastFrame, stdin, unmount } = render(<App providers={[provider]} store={store} />);

    await delay(50);

    // Time panel should show "No active timer" initially
    let frame = lastFrame();
    expect(frame).toContain("Time");
    expect(frame).toContain("No active timer");

    // Start a timer
    stdin.write(ENTER);
    await delay(50);

    // Time panel should now show the timer
    frame = lastFrame();
    expect(frame).toContain("00:00");

    unmount();
  });

  it("toggles time panel expanded mode with 't'", async () => {
    const provider = new MockProvider(MOCK_ITEMS);
    const { lastFrame, stdin, unmount } = render(<App providers={[provider]} store={store} />);

    await delay(50);

    // Normal mode should show Work Items panel
    let frame = lastFrame();
    expect(frame).toContain("Work Items");

    // Press 't' to expand time panel
    stdin.write("t");
    await delay(20);

    frame = lastFrame();
    expect(frame).toContain("Time Analytics");

    // Press 't' again to go back to normal
    stdin.write("t");
    await delay(20);

    frame = lastFrame();
    expect(frame).toContain("Work Items");

    unmount();
  });

  it("persists time entries across store instances", async () => {
    const provider = new MockProvider(MOCK_ITEMS);
    const { stdin, unmount } = render(<App providers={[provider]} store={store} />);

    await delay(50);

    // Start and stop a timer
    stdin.write(ENTER);
    await delay(50);
    stdin.write(ENTER);
    await delay(50);

    unmount();

    // Create a new store from the same file — entries should persist
    const store2 = new TimeStore(storePath);
    const entries = store2.getAllEntries();
    expect(entries).toHaveLength(1);
    expect(entries[0]!.workItemId).toBe("1");
    expect(entries[0]!.workItemTitle).toBe("Fix login bug");
  });

  it("shows footer with all keybindings", async () => {
    const provider = new MockProvider(MOCK_ITEMS);
    const { lastFrame, unmount } = render(<App providers={[provider]} store={store} />);

    await delay(50);

    const frame = lastFrame();
    expect(frame).toContain("navigate");
    expect(frame).toContain("start/stop");
    expect(frame).toContain("time");
    expect(frame).toContain("complete");
    expect(frame).toContain("quit");

    unmount();
  });
});
