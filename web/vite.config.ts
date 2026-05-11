import path from "node:path";
import react from "@vitejs/plugin-react";
import { defineConfig } from "vitest/config";

export default defineConfig({
  plugins: [react()],
  server: {
    fs: {
      // Allow importing the wasm-pack bundle under `src/wasm/` and any workspace paths if needed later.
      allow: [path.resolve(__dirname, "."), path.resolve(__dirname, "..")],
    },
  },
  test: {
    environment: "jsdom",
    setupFiles: "./src/setupTests.ts",
    include: ["src/**/*.{test,spec}.{ts,tsx}"],
  },
});
