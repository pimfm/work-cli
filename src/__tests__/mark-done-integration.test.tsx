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

const ITEMS: WorkItem[] = [
  { id: "1", title: "First task", source: "Mock", labels: [], status: "Todo" },
  { id: "2", title: "Second task", source: "Mock", labels: [], status: "Todo" },
];

function delay(ms: number): Promise<void> {
  return new Promise((r) => setTimeout(r, ms));
}

class MockProvider implements WorkItemProvider {
  name = "Mock";
  private items: WorkItem[];
  markDoneCalls: WorkItem[] = [];

  constructor(items: WorkItem[]) {
    this.items = [...items];
  }

  async fetchAssignedItems(): Promise<WorkItem[]> {
    return this.items;
  }

  async markDone(item: WorkItem): Promise<void> {
    this.markDoneCalls.push(item);
    // Simulate the item being removed from subsequent fetches (like real providers filter done items)
    this.items = this.items.filter((i) => i.id !== item.id);
  }
}

describe("Mark done integration", () => {
  let tmpDir: string;
  let store: TimeStore;

  beforeEach(() => {
    tmpDir = mkdtempSync(join(tmpdir(), "lp-test-"));
    store = new TimeStore(join(tmpDir, "time.json"));
  });

  afterEach(() => {
    rmSync(tmpDir, { recursive: true, force: true });
  });

  it("calls markDone on the provider when pressing 'c'", async () => {
    const provider = new MockProvider(ITEMS);
    const { stdin, unmount } = render(<App providers={[provider]} store={store} />);

    await delay(50);

    // Press 'c' to complete the first item
    stdin.write("c");
    await delay(50);

    expect(provider.markDoneCalls).toHaveLength(1);
    expect(provider.markDoneCalls[0]!.id).toBe("1");
    expect(provider.markDoneCalls[0]!.title).toBe("First task");

    unmount();
  });

  it("refreshes the item list after marking done", async () => {
    const provider = new MockProvider(ITEMS);
    const { lastFrame, stdin, unmount } = render(<App providers={[provider]} store={store} />);

    await delay(50);

    // Both items visible initially
    let frame = lastFrame();
    expect(frame).toContain("First task");
    expect(frame).toContain("Second task");

    // Press 'c' to complete first item
    stdin.write("c");
    // Wait for markDone + refresh
    await delay(200);

    // First item should be gone after refresh
    frame = lastFrame();
    expect(frame).not.toContain("First task");
    expect(frame).toContain("Second task");

    unmount();
  });

  it("stops the active timer when pressing 'c'", async () => {
    const provider = new MockProvider(ITEMS);
    const { stdin, unmount } = render(<App providers={[provider]} store={store} />);

    await delay(50);

    // Start timer
    stdin.write("\r");
    await delay(50);
    expect(store.getActiveTimer()).toBeDefined();

    // Complete
    stdin.write("c");
    await delay(50);

    expect(store.getActiveTimer()).toBeUndefined();

    unmount();
  });
});
