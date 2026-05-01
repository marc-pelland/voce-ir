import { defineConfig } from "vite";

export default defineConfig({
  root: ".",
  // Relative asset paths so the build works whether it's served from
  // the domain root or a subpath (GitHub Pages deploys it under /playground/).
  base: "./",
  build: {
    outDir: "dist",
    target: "es2022",
  },
  server: {
    port: 5173,
  },
});
