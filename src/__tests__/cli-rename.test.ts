import { describe, it, expect } from "vitest";
import { readFileSync } from "fs";
import { join } from "path";

const ROOT = join(import.meta.dirname, "..", "..");

describe("CLI executable name", () => {
  it("package.json bin entry uses 'work' not 'fm'", () => {
    const pkg = JSON.parse(readFileSync(join(ROOT, "package.json"), "utf-8"));
    expect(pkg.bin).toHaveProperty("work");
    expect(pkg.bin).not.toHaveProperty("fm");
  });

  it("CLI usage message references 'work'", async () => {
    const cli = readFileSync(join(ROOT, "src", "cli.tsx"), "utf-8");
    expect(cli).toContain("Usage: work");
    expect(cli).not.toContain("Usage: fm");
  });

  it("App header displays 'work pipeline'", async () => {
    const app = readFileSync(join(ROOT, "src", "ui", "App.tsx"), "utf-8");
    expect(app).toContain("work pipeline");
    expect(app).not.toContain("fm pipeline");
  });
});
