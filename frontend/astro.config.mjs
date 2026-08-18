import { defineConfig } from "astro/config";
import svelte from "@astrojs/svelte";

// Static build: the PWA is pre-rendered and served by nginx (see Dockerfile).
export default defineConfig({
  integrations: [svelte()],
  output: "static",
});
