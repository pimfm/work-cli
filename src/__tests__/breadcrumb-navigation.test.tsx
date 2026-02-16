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
const ARROW_LEFT = "\x1B[D";
const ARROW_RIGHT = "\x1B[C";

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

  it("navigates back with escape key and preserves forward history", async () => {
    const provider = new MockProvider(MOCK_ITEMS);
    const { lastFrame, stdin, unmount } = render(<App providers={[provider]} store={store} />);
    await delay(50);

    stdin.write("t");
    await delay(20);

    let frame = lastFrame();
    expect(frame).toContain("Time Analytics");

    stdin.write("\x1B");
    await delay(20);

    frame = lastFrame();
    expect(frame).toContain("Work Items");
    // Forward history is shown as dimmed breadcrumb trail
    expect(frame).toContain("Time Analytics");
    unmount();
  });

  it("navigates back with 'b' key and preserves forward history", async () => {
    const provider = new MockProvider(MOCK_ITEMS);
    const { lastFrame, stdin, unmount } = render(<App providers={[provider]} store={store} />);
    await delay(50);

    stdin.write("a");
    await delay(20);

    let frame = lastFrame();
    expect(frame).toContain("Agents");

    stdin.write("b");
    await delay(20);

    frame = lastFrame();
    expect(frame).toContain("Work Items");
    // Forward history shows Agents as reachable via right arrow
    expect(frame).toContain("Agents");
    unmount();
  });

  it("navigates back with backspace key", async () => {
    const provider = new MockProvider(MOCK_ITEMS);
    const { lastFrame, stdin, unmount } = render(<App providers={[provider]} store={store} />);
    await delay(50);

    stdin.write("a");
    await delay(20);

    let frame = lastFrame();
    expect(frame).toContain("Agents");

    stdin.write(BACKSPACE);
    await delay(20);

    frame = lastFrame();
    expect(frame).toContain("Work Items");
    unmount();
  });

  it("shows back hint in footer when navigated away from root", async () => {
    const provider = new MockProvider(MOCK_ITEMS);
    const { lastFrame, stdin, unmount } = render(<App providers={[provider]} store={store} />);
    await delay(50);

    let frame = lastFrame();
    expect(frame).toContain("quit");

    stdin.write("t");
    await delay(20);

    frame = lastFrame();
    expect(frame).toContain("back");
    unmount();
  });

  it("toggles same mode navigates back instead of pushing duplicate", async () => {
    const provider = new MockProvider(MOCK_ITEMS);
    const { lastFrame, stdin, unmount } = render(<App providers={[provider]} store={store} />);
    await delay(50);

    stdin.write("t");
    await delay(20);

    let frame = lastFrame();
    expect(frame).toContain("Time Analytics");

    // Press 't' again — should go back to normal
    stdin.write("t");
    await delay(20);

    frame = lastFrame();
    expect(frame).toContain("Work Items");
    unmount();
  });

  it("navigates back with left arrow key", async () => {
    const provider = new MockProvider(MOCK_ITEMS);
    const { lastFrame, stdin, unmount } = render(<App providers={[provider]} store={store} />);
    await delay(50);

    stdin.write("t");
    await delay(20);

    let frame = lastFrame();
    expect(frame).toContain("Time Analytics");

    stdin.write(ARROW_LEFT);
    await delay(20);

    frame = lastFrame();
    expect(frame).toContain("Work Items");
    unmount();
  });

  it("navigates forward with right arrow key after going back", async () => {
    const provider = new MockProvider(MOCK_ITEMS);
    const { lastFrame, stdin, unmount } = render(<App providers={[provider]} store={store} />);
    await delay(50);

    // Navigate to time analytics
    stdin.write("t");
    await delay(20);

    let frame = lastFrame();
    expect(frame).toContain("Time Analytics");

    // Go back with left arrow
    stdin.write(ARROW_LEFT);
    await delay(20);

    frame = lastFrame();
    expect(frame).toContain("Work Items");

    // Go forward with right arrow
    stdin.write(ARROW_RIGHT);
    await delay(20);

    frame = lastFrame();
    expect(frame).toContain("Time Analytics");
    unmount();
  });

  it("clears forward history when navigating to a new destination", async () => {
    const provider = new MockProvider(MOCK_ITEMS);
    const { lastFrame, stdin, unmount } = render(<App providers={[provider]} store={store} />);
    await delay(50);

    // Navigate to time analytics
    stdin.write("t");
    await delay(20);

    // Go back
    stdin.write(ARROW_LEFT);
    await delay(20);

    // Navigate to agents instead of using right arrow
    stdin.write("a");
    await delay(20);

    let frame = lastFrame();
    expect(frame).toContain("Agents");
    // Time Analytics should no longer be in forward trail
    expect(frame).not.toContain("Time Analytics");

    // Right arrow should do nothing (no forward history)
    stdin.write(ARROW_RIGHT);
    await delay(20);

    frame = lastFrame();
    expect(frame).toContain("Agents");
    expect(frame).not.toContain("Time Analytics");
    unmount();
  });

  it("shows forward hint in footer when forward history exists", async () => {
    const provider = new MockProvider(MOCK_ITEMS);
    const { lastFrame, stdin, unmount } = render(<App providers={[provider]} store={store} />);
    await delay(50);

    // Navigate to time analytics then go back
    stdin.write("t");
    await delay(20);
    stdin.write(ARROW_LEFT);
    await delay(20);

    const frame = lastFrame();
    expect(frame).toContain("forward");
    unmount();
  });

  it("supports multi-step forward navigation through history", async () => {
    const provider = new MockProvider(MOCK_ITEMS);
    const { lastFrame, stdin, unmount } = render(<App providers={[provider]} store={store} />);
    await delay(50);

    // Build stack: Dashboard > Agents > Time Analytics
    stdin.write("a");
    await delay(20);
    stdin.write("t");
    await delay(20);

    // Go back twice to dashboard
    stdin.write(ARROW_LEFT);
    await delay(20);
    stdin.write(ARROW_LEFT);
    await delay(20);

    let frame = lastFrame();
    expect(frame).toContain("Work Items");

    // Go forward once — should be at agents
    stdin.write(ARROW_RIGHT);
    await delay(20);

    frame = lastFrame();
    expect(frame).toContain("Agents");

    // Go forward again — should be at time analytics
    stdin.write(ARROW_RIGHT);
    await delay(20);

    frame = lastFrame();
    expect(frame).toContain("Time Analytics");
    unmount();
  });

  it("left arrow navigates back in agents mode", async () => {
    const provider = new MockProvider(MOCK_ITEMS);
    const { lastFrame, stdin, unmount } = render(<App providers={[provider]} store={store} />);
    await delay(50);

    stdin.write("a");
    await delay(20);

    let frame = lastFrame();
    expect(frame).toContain("Agents");

    stdin.write(ARROW_LEFT);
    await delay(20);

    frame = lastFrame();
    expect(frame).toContain("Work Items");
    unmount();
  });

  it("right arrow has no effect when no forward history exists", async () => {
    const provider = new MockProvider(MOCK_ITEMS);
    const { lastFrame, stdin, unmount } = render(<App providers={[provider]} store={store} />);
    await delay(50);

    // Navigate to time analytics
    stdin.write("t");
    await delay(20);

    let frame = lastFrame();
    expect(frame).toContain("Time Analytics");

    // Right arrow should do nothing
    stdin.write(ARROW_RIGHT);
    await delay(20);

    frame = lastFrame();
    expect(frame).toContain("Time Analytics");
    unmount();
  });

  it("left arrow has no effect on the root dashboard", async () => {
    const provider = new MockProvider(MOCK_ITEMS);
    const { lastFrame, stdin, unmount } = render(<App providers={[provider]} store={store} />);
    await delay(50);

    let frame = lastFrame();
    expect(frame).toContain("Work Items");

    // Left arrow should do nothing on root
    stdin.write(ARROW_LEFT);
    await delay(20);

    frame = lastFrame();
    expect(frame).toContain("Work Items");
    unmount();
  });

  it("builds navigation stack across multiple navigations", async () => {
    const provider = new MockProvider(MOCK_ITEMS);
    const { lastFrame, stdin, unmount } = render(<App providers={[provider]} store={store} />);
    await delay(50);

    stdin.write("a");
    await delay(20);

    stdin.write("t");
    await delay(20);

    let frame = lastFrame();
    expect(frame).toContain("Dashboard");
    expect(frame).toContain("Agents");
    expect(frame).toContain("Time Analytics");

    // Go back once — should be at agents, Time Analytics in forward trail
    stdin.write("b");
    await delay(20);

    frame = lastFrame();
    expect(frame).toContain("Dashboard");
    expect(frame).toContain("Agents");
    expect(frame).toContain("Time Analytics"); // now in forward breadcrumbs

    // Go back again — should be at dashboard with forward history
    stdin.write("b");
    await delay(20);

    frame = lastFrame();
    expect(frame).toContain("Work Items");
    // Forward history shows Agents and Time Analytics
    expect(frame).toContain("Agents");
    unmount();
  });
});
