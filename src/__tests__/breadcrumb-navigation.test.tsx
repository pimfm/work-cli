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

const BACKSPACE = "\x7F";

const MOCK_ITEMS: WorkItem[] = [
  { id: "1", title: "Fix login bug", source: "Linear", labels: [], status: "In Progress", priority: "High" },
  { id: "2", title: "Add dark mode", source: "Trello", labels: ["frontend"], status: "Todo" },
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

describe("Breadcrumb Navigation", () => {
  let tmpDir: string;
  let store: TimeStore;

  beforeEach(() => {
    tmpDir = mkdtempSync(join(tmpdir(), "lp-test-"));
    store = new TimeStore(join(tmpDir, "time.json"));
  });

  afterEach(() => {
    rmSync(tmpDir, { recursive: true, force: true });
  });

  it("does not show breadcrumbs on the main dashboard", async () => {
    const provider = new MockProvider(MOCK_ITEMS);
    const { lastFrame, unmount } = render(<App providers={[provider]} store={store} />);
    await delay(50);

    const frame = lastFrame();
    // Breadcrumb component returns null when only one level deep
    // so we should NOT see the ">" separator
    expect(frame).not.toContain("Dashboard > ");
    unmount();
  });

  it("shows breadcrumbs when navigating to time analytics", async () => {
    const provider = new MockProvider(MOCK_ITEMS);
    const { lastFrame, stdin, unmount } = render(<App providers={[provider]} store={store} />);
    await delay(50);

    stdin.write("t");
    await delay(20);

    const frame = lastFrame();
    expect(frame).toContain("Dashboard");
    expect(frame).toContain("Time Analytics");
    expect(frame).toContain(">");
    unmount();
  });

  it("shows breadcrumbs when navigating to agents", async () => {
    const provider = new MockProvider(MOCK_ITEMS);
    const { lastFrame, stdin, unmount } = render(<App providers={[provider]} store={store} />);
    await delay(50);

    stdin.write("a");
    await delay(20);

    const frame = lastFrame();
    expect(frame).toContain("Dashboard");
    expect(frame).toContain("Agents");
    expect(frame).toContain(">");
    unmount();
  });

  it("navigates back with escape key", async () => {
    const provider = new MockProvider(MOCK_ITEMS);
    const { lastFrame, stdin, unmount } = render(<App providers={[provider]} store={store} />);
    await delay(50);

    // Navigate to time-expanded
    stdin.write("t");
    await delay(20);

    let frame = lastFrame();
    expect(frame).toContain("Time Analytics");
    expect(frame).toContain(">");

    // Press escape to go back
    stdin.write("\x1B");
    await delay(20);

    frame = lastFrame();
    // Should be back on normal dashboard, no breadcrumbs
    expect(frame).toContain("Work Items");
    expect(frame).not.toContain("Dashboard > ");
    unmount();
  });

  it("navigates back with backspace key", async () => {
    const provider = new MockProvider(MOCK_ITEMS);
    const { lastFrame, stdin, unmount } = render(<App providers={[provider]} store={store} />);
    await delay(50);

    // Navigate to agents
    stdin.write("a");
    await delay(20);

    let frame = lastFrame();
    expect(frame).toContain("Agents");
    expect(frame).toContain(">");

    // Press backspace to go back
    stdin.write(BACKSPACE);
    await delay(20);

    frame = lastFrame();
    expect(frame).toContain("Work Items");
    expect(frame).not.toContain("Dashboard > ");
    unmount();
  });

  it("shows back hint in footer when navigated away from root", async () => {
    const provider = new MockProvider(MOCK_ITEMS);
    const { lastFrame, stdin, unmount } = render(<App providers={[provider]} store={store} />);
    await delay(50);

    // On root: footer should show quit, not back
    let frame = lastFrame();
    expect(frame).toContain("quit");

    // Navigate to time-expanded
    stdin.write("t");
    await delay(20);

    // Footer should show back, not quit
    frame = lastFrame();
    expect(frame).toContain("back");
    unmount();
  });

  it("toggles same mode navigates back instead of pushing duplicate", async () => {
    const provider = new MockProvider(MOCK_ITEMS);
    const { lastFrame, stdin, unmount } = render(<App providers={[provider]} store={store} />);
    await delay(50);

    // Navigate to time-expanded
    stdin.write("t");
    await delay(20);

    let frame = lastFrame();
    expect(frame).toContain("Time Analytics");

    // Press 't' again — should go back to normal
    stdin.write("t");
    await delay(20);

    frame = lastFrame();
    expect(frame).toContain("Work Items");
    expect(frame).not.toContain("Dashboard > ");
    unmount();
  });
});
