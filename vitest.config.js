import { defineConfig } from "vitest/config";

// Separate from vite.config.js on purpose: that one is what Tauri builds the app with,
// and these tests run plain TypeScript modules — no SvelteKit plugin needed. The `test`
// script still runs `svelte-kit sync` first, because tsconfig.json extends the file it
// generates and esbuild reads that while transforming each test.
export default defineConfig({
  test: {
    include: ["src/**/*.test.ts"],
  },
});
