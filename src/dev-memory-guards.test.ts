import { readFileSync } from "node:fs";
import { describe, expect, it } from "vitest";
import type { UserConfig } from "vite";

import viteConfig from "../vite.config";

describe("development memory guards", () => {
  it("disables Tauri's Rust file watcher after the one-shot prebuild", () => {
    const packageJson = JSON.parse(
      readFileSync(new URL("../package.json", import.meta.url), "utf8"),
    ) as { scripts?: Record<string, string> };

    expect(packageJson.scripts?.["tauri:dev"]).toContain("tauri dev --no-watch");
  });

  it("keeps Rust build output outside Vite's watch set", () => {
    const ignored = (viteConfig as UserConfig).server?.watch?.ignored;
    const ignoredPatterns = Array.isArray(ignored) ? ignored : ignored ? [ignored] : [];

    expect(ignoredPatterns).toContain("**/target/**");
  });
});
