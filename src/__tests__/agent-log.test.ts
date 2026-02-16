import { describe, it, expect, beforeEach, afterEach, vi } from "vitest";
import { mkdtempSync, rmSync, readFileSync } from "fs";
import { join } from "path";
import { tmpdir } from "os";

// Mock homedir to use temp directory
const mockHomeDir = vi.fn<() => string>();
vi.mock("os", async () => {
  const actual = await vi.importActual<typeof import("os")>("os");
  return { ...actual, homedir: () => mockHomeDir() };
});

import { appendEvent, readEvents, readEventsForAgent } from "../persistence/agent-log.js";
import type { AgentEvent } from "../persistence/agent-log.js";

describe("agent-log", () => {
  let tmpDir: string;

  beforeEach(() => {
    tmpDir = mkdtempSync(join(tmpdir(), "agent-log-test-"));
    mockHomeDir.mockReturnValue(tmpDir);
  });

  afterEach(() => {
    rmSync(tmpDir, { recursive: true, force: true });
  });

  function makeEvent(overrides: Partial<AgentEvent> = {}): AgentEvent {
    return {
      timestamp: new Date().toISOString(),
      agent: "ember",
      event: "dispatched",
      ...overrides,
    };
  }

  it("appends and reads events", () => {
    appendEvent(makeEvent({ event: "dispatched", message: "Starting" }));
    appendEvent(makeEvent({ event: "working" }));

    const events = readEvents();
    expect(events).toHaveLength(2);
    expect(events[0]!.event).toBe("dispatched");
    expect(events[0]!.message).toBe("Starting");
    expect(events[1]!.event).toBe("working");
  });

  it("returns empty array when no log file exists", () => {
    const events = readEvents();
    expect(events).toEqual([]);
  });

  it("filters events by agent name", () => {
    appendEvent(makeEvent({ agent: "ember", event: "dispatched" }));
    appendEvent(makeEvent({ agent: "flow", event: "dispatched" }));
    appendEvent(makeEvent({ agent: "ember", event: "working" }));

    const emberEvents = readEvents("ember");
    expect(emberEvents).toHaveLength(2);
    expect(emberEvents.every((e) => e.agent === "ember")).toBe(true);

    const flowEvents = readEvents("flow");
    expect(flowEvents).toHaveLength(1);
    expect(flowEvents[0]!.agent).toBe("flow");
  });

  it("limits returned events to most recent N", () => {
    for (let i = 0; i < 10; i++) {
      appendEvent(makeEvent({ message: `event-${i}` }));
    }

    const events = readEvents(undefined, 3);
    expect(events).toHaveLength(3);
    expect(events[0]!.message).toBe("event-7");
    expect(events[2]!.message).toBe("event-9");
  });

  it("readEventsForAgent is a convenience wrapper", () => {
    appendEvent(makeEvent({ agent: "tempest", event: "error", message: "fail" }));
    appendEvent(makeEvent({ agent: "ember", event: "done" }));

    const tempestEvents = readEventsForAgent("tempest");
    expect(tempestEvents).toHaveLength(1);
    expect(tempestEvents[0]!.agent).toBe("tempest");
    expect(tempestEvents[0]!.event).toBe("error");
  });

  it("writes valid JSON lines", () => {
    appendEvent(makeEvent({ event: "dispatched", workItemId: "PROJ-123", workItemTitle: "Fix bug" }));

    const raw = readFileSync(join(tmpDir, ".localpipeline", "agent-activity.jsonl"), "utf-8");
    const lines = raw.trim().split("\n");
    expect(lines).toHaveLength(1);

    const parsed = JSON.parse(lines[0]!);
    expect(parsed.event).toBe("dispatched");
    expect(parsed.workItemId).toBe("PROJ-123");
  });

  it("skips malformed lines gracefully", async () => {
    const { appendFileSync, mkdirSync: mkdirSyncFs } = await import("fs");
    const logPath = join(tmpDir, ".localpipeline", "agent-activity.jsonl");
    mkdirSyncFs(join(tmpDir, ".localpipeline"), { recursive: true });
    appendFileSync(logPath, "not-json\n");
    appendEvent(makeEvent({ event: "done" }));

    const events = readEvents();
    expect(events).toHaveLength(1);
    expect(events[0]!.event).toBe("done");
  });
});
