import { describe, it, expect } from "vitest";
import { mkdtempSync, rmSync, readFileSync } from "fs";
import { join } from "path";
import { tmpdir } from "os";
import { PERSONALITIES } from "../model/personality.js";
import { AGENT_NAMES } from "../model/agent.js";
import type { AgentName } from "../model/agent.js";
import { buildClaudePrompt } from "../agents/claude-prompt.js";
import { writeClaudeMd } from "../agents/claude-md.js";
import type { WorkItem } from "../model/work-item.js";

describe("Agent personalities", () => {
  describe("PERSONALITIES", () => {
    it("defines a personality for every agent", () => {
      for (const name of AGENT_NAMES) {
        expect(PERSONALITIES[name]).toBeDefined();
        expect(PERSONALITIES[name].tagline).toBeTruthy();
        expect(PERSONALITIES[name].traits.length).toBeGreaterThan(0);
        expect(PERSONALITIES[name].systemPrompt).toBeTruthy();
      }
    });

    it("each agent has unique tagline", () => {
      const taglines = AGENT_NAMES.map((n) => PERSONALITIES[n].tagline);
      expect(new Set(taglines).size).toBe(taglines.length);
    });

    it("each agent has unique traits", () => {
      const allTraits = AGENT_NAMES.flatMap((n) => PERSONALITIES[n].traits);
      expect(new Set(allTraits).size).toBe(allTraits.length);
    });
  });

  describe("buildClaudePrompt with personality", () => {
    const item: WorkItem = {
      id: "LIN-10",
      title: "Add feature",
      labels: [],
      source: "Linear",
    };

    it("includes personality tagline for ember", () => {
      const prompt = buildClaudePrompt(item, "Ember");
      expect(prompt).toContain("Move fast, ship clean");
    });

    it("includes personality tagline for flow", () => {
      const prompt = buildClaudePrompt(item, "Flow");
      expect(prompt).toContain("Steady and thorough");
    });

    it("includes personality tagline for tempest", () => {
      const prompt = buildClaudePrompt(item, "Tempest");
      expect(prompt).toContain("Creative problem solver");
    });

    it("includes personality tagline for terra", () => {
      const prompt = buildClaudePrompt(item, "Terra");
      expect(prompt).toContain("Build to last");
    });
  });

  describe("writeClaudeMd with personality", () => {
    let tmpDir: string;

    beforeEach(() => {
      tmpDir = mkdtempSync(join(tmpdir(), "personality-test-"));
    });

    afterEach(() => {
      rmSync(tmpDir, { recursive: true, force: true });
    });

    it("includes personality system prompt for ember", () => {
      writeClaudeMd(tmpDir, "Ember");
      const content = readFileSync(join(tmpDir, "CLAUDE.md"), "utf-8");
      expect(content).toContain("## Personality");
      expect(content).toContain("speed and pragmatism");
    });

    it("includes personality system prompt for flow", () => {
      writeClaudeMd(tmpDir, "Flow");
      const content = readFileSync(join(tmpDir, "CLAUDE.md"), "utf-8");
      expect(content).toContain("correctness and thoroughness");
    });

    it("includes personality system prompt for tempest", () => {
      writeClaudeMd(tmpDir, "Tempest");
      const content = readFileSync(join(tmpDir, "CLAUDE.md"), "utf-8");
      expect(content).toContain("elegant patterns");
    });

    it("includes personality system prompt for terra", () => {
      writeClaudeMd(tmpDir, "Terra");
      const content = readFileSync(join(tmpDir, "CLAUDE.md"), "utf-8");
      expect(content).toContain("stability and backward compatibility");
    });

    it("still includes agent identity section", () => {
      writeClaudeMd(tmpDir, "Ember");
      const content = readFileSync(join(tmpDir, "CLAUDE.md"), "utf-8");
      expect(content).toContain("You are **Ember**");
      expect(content).toContain("fm pipeline");
    });
  });
});
