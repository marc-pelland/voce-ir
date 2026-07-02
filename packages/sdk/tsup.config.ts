import { defineConfig } from "tsup";

// Single entry: the SDK's public surface. dts is on so the .d.ts ships
// alongside the .js (package.json points main/types at dist/index).
export default defineConfig({
  entry: {
    index: "src/index.ts",
  },
  format: ["esm"],
  target: "node20",
  outDir: "dist",
  dts: true,
  clean: true,
});
