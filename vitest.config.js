import { defineConfig } from "vitest/config";
import { sveltekit } from "@sveltejs/kit/vite";

// Separate from vite.config.js on purpose: that one is what Tauri builds the app with,
// and this one only has to make the modules under test importable. The SvelteKit plugin
// is what does that — the state modules are `.svelte.ts`, so their `$state` needs the
// Svelte compiler to mean anything, and they reach each other through `$lib`. The `test`
// script still runs `svelte-kit sync` first, because tsconfig.json extends the file it
// generates and the plugin reads that while transforming each test.
export default defineConfig({
  plugins: [sveltekit()],
  test: {
    include: ["src/**/*.test.ts"],
    setupFiles: ["./vitest.setup.ts"],
  },
});
