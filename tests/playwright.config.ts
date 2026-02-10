import { defineConfig, devices } from '@playwright/test';

/**
 * Read environment variables from file.
 * https://github.com/motdotla/dotenv
 */
// require('dotenv').config();

/**
 * See https://playwright.dev/docs/test-configuration.
 */
export default defineConfig({
  testDir: './e2e',
  fullyParallel: false,
  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 2 : 0,
  workers: 1,
  reporter: process.env.CI ? 'github' : 'list',
  timeout: 30000,
  use: {
    baseURL: process.env.MCB_TEST_PORT 
      ? `http://localhost:${process.env.MCB_TEST_PORT}` 
      : 'http://localhost:18080',
    trace: 'on-first-retry',
    screenshot: 'only-on-failure',
    video: 'retain-on-failure',
  },

  /* Configure projects for major browsers */
  projects: [
    {
      name: 'chromium',
      use: { ...devices['Desktop Chrome'] },
    },
  ],

  webServer: {
    command: `rm -f /tmp/mcb-playwright.db && MCP__AUTH__USER_DB_PATH=/tmp/mcb-playwright.db MCP__SERVER__TRANSPORT_MODE=http cargo run --release --bin mcb -- serve --server`,
    url: process.env.MCB_TEST_PORT 
      ? `http://localhost:${process.env.MCB_TEST_PORT}` 
      : 'http://localhost:18080',
    reuseExistingServer: !process.env.CI,
    timeout: 600 * 1000,
    env: {
      'MCP__SERVER__NETWORK__PORT': process.env.MCB_TEST_PORT || '18080',
      'MCP__SERVER__TRANSPORT_MODE': 'http',
      'MCP__AUTH__USER_DB_PATH': '/tmp/mcb-playwright.db',
      'RUST_LOG': 'info',
    },
  },
});
