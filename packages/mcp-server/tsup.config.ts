import { defineConfig } from "tsup";

// Three entry points: the binary (index), and two library sub-paths that
// downstream consumers (cli-chat S66) import from. dts is on so the .d.ts
// files are emitted alongside the .js — the package's exports map wires
// both to each sub-path.
export default defineConfig({
  entry: {
    index: "src/index.ts",
    memory: "src/memory/index.ts",
    workflow: "src/workflow/index.ts",
  },
  format: ["esm"],
  target: "node20",
  outDir: "dist",
  dts: true,
  clean: true,
});
