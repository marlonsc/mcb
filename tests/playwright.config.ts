import { defineConfig, devices } from '@playwright/test';

const testPort = process.env.MCB_TEST_PORT || '18080';
const testDbPath = `/tmp/mcb-playwright-${testPort}.db`;

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
    baseURL: `http://localhost:${testPort}`,
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
    command: `cd .. && rm -f ${testDbPath} && MCB_BIN=$([ -x target/release/mcb ] && echo target/release/mcb || echo "") && if [ -n "$MCB_BIN" ]; then $MCB_BIN serve --server; else cargo run --release --bin mcb -- serve --server; fi`,
    url: `http://localhost:${testPort}`,
    reuseExistingServer: false,
    timeout: 600 * 1000,
    env: {
      'SERVER_PORT': testPort,
      'LOCO_ENV': 'test',
      'DATABASE_URL': `sqlite://${testDbPath}?mode=rwc`,
      'MCP__SERVER__TRANSPORT_MODE': 'http',
      'MCP__AUTH__USER_DB_PATH': testDbPath,
      'MCB_MODEL_ID': process.env.MCB_MODEL_ID || 'playwright-e2e',
      'RUST_LOG': process.env.CI ? 'warn' : 'info',
    },
  },
});
