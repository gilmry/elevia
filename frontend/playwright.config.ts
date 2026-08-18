import { defineConfig, devices } from "@playwright/test";

/**
 * These specs double as living documentation of Elevia's user journeys (see
 * e2e/README.md): every run records a video and a step-by-step HTML report,
 * so `npm run test:e2e:report` always shows the *current* app behaviour, not
 * a snapshot someone forgot to update.
 */
export default defineConfig({
  testDir: "./e2e",
  fullyParallel: false,
  workers: 1,
  retries: 0,
  reporter: [["html", { open: "never" }], ["list"]],
  timeout: 30_000,
  use: {
    baseURL: process.env.E2E_BASE_URL ?? "http://localhost:4321",
    video: "on",
    screenshot: "only-on-failure",
    trace: "retain-on-failure",
  },
  projects: [
    {
      name: "chromium",
      use: { ...devices["Desktop Chrome"] },
    },
  ],
});
