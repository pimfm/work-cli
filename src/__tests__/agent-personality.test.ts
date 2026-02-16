import { describe, it, expect, beforeEach, afterEach } from "vitest";
import { mkdtempSync, rmSync, readFileSync } from "fs";
import { join } from "path";
import { tmpdir } from "os";
import { AGENT_NAMES } from "../model/agent.js";
import { PERSONALITIES } from "../model/personality.js";
import type { AgentPersonality } from "../model/personality.js";
import { writeClaudeMd } from "../agents/claude-md.js";
import { buildClaudePrompt } from "../agents/claude-prompt.js";
import type { WorkItem } from "../model/work-item.js";

describe("agent personalities", () => {
  describe("model", () => {
    it("every agent has a personality with tagline, traits, and systemPrompt", () => {
      for (const name of AGENT_NAMES) {
        const personality = PERSONALITIES[name];
        expect(personality).toBeDefined();
        expect(personality.tagline).toBeTruthy();
        expect(personality.traits.length).toBeGreaterThan(0);
        expect(personality.systemPrompt).toBeTruthy();
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

  describe("writeClaudeMd", () => {
    let tmpDir: string;

    beforeEach(() => {
      tmpDir = mkdtempSync(join(tmpdir(), "personality-test-"));
    });

    afterEach(() => {
      rmSync(tmpDir, { recursive: true, force: true });
    });

    it("includes personality section in CLAUDE.md", () => {
      writeClaudeMd(tmpDir, "Ember");
      const content = readFileSync(join(tmpDir, "CLAUDE.md"), "utf-8");

      expect(content).toContain("### Personality: Move fast, ship clean");
      expect(content).toContain("decisive, pragmatic, velocity-focused");
      expect(content).toContain("You value speed and pragmatism");
    });

    it("includes correct personality for each agent", () => {
      for (const name of AGENT_NAMES) {
        const dir = mkdtempSync(join(tmpdir(), `personality-${name}-`));
        const display = name.charAt(0).toUpperCase() + name.slice(1);
        writeClaudeMd(dir, display);
        const content = readFileSync(join(dir, "CLAUDE.md"), "utf-8");

        expect(content).toContain(`### Personality: ${PERSONALITIES[name].tagline}`);
        expect(content).toContain(PERSONALITIES[name].traits.join(", "));
        rmSync(dir, { recursive: true, force: true });
      }
    });

    it("still includes agent identity", () => {
      writeClaudeMd(tmpDir, "Flow");
      const content = readFileSync(join(tmpDir, "CLAUDE.md"), "utf-8");

      expect(content).toContain("You are **Flow**");
    });
  });

  describe("buildClaudePrompt", () => {
    const item: WorkItem = {
      id: "LIN-456",
      title: "Add dark mode",
      labels: ["feature"],
      source: "Linear",
    };

    it("includes personality tagline in opening line", () => {
      const prompt = buildClaudePrompt(item, "Flow");
      expect(prompt).toContain("Your personality: Steady and thorough.");
    });

    it("includes full personality section at end of prompt", () => {
      const prompt = buildClaudePrompt(item, "Ember");
      expect(prompt).toContain("## Personality: Move fast, ship clean");
      expect(prompt).toContain("decisive, pragmatic, velocity-focused");
    });

    it("includes personality for all agents", () => {
      for (const name of AGENT_NAMES) {
        const display = name.charAt(0).toUpperCase() + name.slice(1);
        const prompt = buildClaudePrompt(item, display);
        expect(prompt).toContain(`## Personality: ${PERSONALITIES[name].tagline}`);
      }
    });
  });
});
