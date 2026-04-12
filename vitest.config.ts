import { defineConfig } from "vitest/config";
import react from "@vitejs/plugin-react";
import { fileURLToPath } from "node:url";
import { dirname, resolve } from "node:path";

const __dirname = dirname(fileURLToPath(import.meta.url));

export default defineConfig({
  plugins: [react()],
  resolve: {
    alias: {
      "@": resolve(__dirname, "src"),
    },
  },
  test: {
    environment: "jsdom",
    globals: true,
    setupFiles: ["src/test/setup.ts"],
    coverage: {
      provider: "v8",
      reporter: ["text", "html", "lcov"],
      // Constitution II: line coverage on **changed** code MUST be ≥ 80 %
      // and overall project coverage MUST NOT decrease on any PR. The
      // global floors below encode the current (2026-04-12) baseline;
      // they are a non-decreasing ratchet toward the 80 % target. Any
      // PR that lowers these numbers must fail CI and either raise the
      // floor back or add tests. Phase 10 polish ticket: T161.
      thresholds: {
        lines: 60,
        functions: 44,
        branches: 80,
        statements: 60,
      },
      exclude: [
        "src/main.tsx",
        "src/i18n/**",
        "src/**/*.d.ts",
        "src/test/**",
        "src-tauri/**",
      ],
    },
    include: ["src/**/*.{test,spec}.{ts,tsx}"],
  },
});
