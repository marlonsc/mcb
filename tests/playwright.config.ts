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
    trace: process.env.CI ? 'off' : 'on-first-retry',
    screenshot: 'only-on-failure',
    video: process.env.CI ? 'off' : 'retain-on-failure',
  },

  /* Configure projects for major browsers */
  projects: [
    {
      name: 'chromium',
      use: { ...devices['Desktop Chrome'] },
    },
  ],

  webServer: {
    command: `rm -f /tmp/mcb-playwright.db && if [ -x target/release/mcb ]; then MCP__AUTH__USER_DB_PATH=/tmp/mcb-playwright.db MCP__SERVER__TRANSPORT_MODE=http target/release/mcb serve --server; else MCP__AUTH__USER_DB_PATH=/tmp/mcb-playwright.db MCP__SERVER__TRANSPORT_MODE=http cargo run --release --bin mcb -- serve --server; fi`,
    url: process.env.MCB_TEST_PORT
      ? `http://localhost:${process.env.MCB_TEST_PORT}`
      : 'http://localhost:18080',
    reuseExistingServer: !process.env.CI,
    timeout: 600 * 1000,
    env: {
      'MCP__SERVER__NETWORK__PORT': process.env.MCB_TEST_PORT || '18080',
      'MCP__SERVER__TRANSPORT_MODE': 'http',
      'MCP__AUTH__USER_DB_PATH': '/tmp/mcb-playwright.db',
      'RUST_LOG': process.env.CI ? 'warn' : 'info',
    },
  },
});
