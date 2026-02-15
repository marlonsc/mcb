import { test, expect } from '@playwright/test';

/**
 * CRITICAL E2E Tests for Admin Web UI Routes
 *
 * These tests verify that the admin web UI is accessible via HTTP.
 *
 * WHY THESE TESTS EXIST:
 * v0.2.0 shipped with broken admin UI (404 on all routes) because web routes
 * were only mounted in web_rocket() test fixture but NOT in admin_rocket()
 * production server.
 *
 * These E2E tests would have caught this bug by actually hitting the HTTP
 * server and verifying 200 OK responses.
 */

test.describe('Admin UI Routes - HTTP Accessibility', () => {
  const testPort = process.env.MCB_TEST_PORT || '18080';
  const baseURL = `http://localhost:${testPort}`;

  test('Dashboard (/) should return 200 OK with HTML', async ({ page }) => {
    const response = await page.goto(`${baseURL}/`);

    expect(response?.status()).toBe(200);
    expect(response?.headers()['content-type']).toContain('text/html');

    const html = await page.content();
    expect(html).toContain('<!DOCTYPE html>');
    expect(html).toContain('Dashboard');
  });

  test('/ui alias should also return dashboard', async ({ page }) => {
    const response = await page.goto(`${baseURL}/ui`);

    expect(response?.status()).toBe(200);
    const html = await page.content();
    expect(html).toContain('Dashboard');
  });

  test('/ui/config should return configuration page', async ({ page }) => {
    const response = await page.goto(`${baseURL}/ui/config`);

    expect(response?.status()).toBe(200);
    const html = await page.content();
    expect(html).toContain('Configuration');
  });

  test('/ui/health should return health status page', async ({ page }) => {
    const response = await page.goto(`${baseURL}/ui/health`);

    expect(response?.status()).toBe(200);
    const html = await page.content();
    expect(html).toContain('Health');
  });

  test('/ui/jobs should return jobs status page', async ({ page }) => {
    const response = await page.goto(`${baseURL}/ui/jobs`);

    expect(response?.status()).toBe(200);
    const html = await page.content();
    expect(html).toMatch(/Jobs|Indexing/);
  });

  test('/ui/browse should return browse collections page', async ({ page }) => {
    const response = await page.goto(`${baseURL}/ui/browse`);

    expect(response?.status()).toBe(200);
    const html = await page.content();
    expect(html).toContain('Browse');
  });

  test('/favicon.ico should return SVG icon', async ({ page }) => {
    const response = await page.goto(`${baseURL}/favicon.ico`);

    expect(response?.status()).toBe(200);
    expect(response?.headers()['content-type']).toContain('image/svg');
  });

  test('/ui/theme.css should return CSS file', async ({ page }) => {
    const response = await page.goto(`${baseURL}/ui/theme.css`);

    expect(response?.status()).toBe(200);
    expect(response?.headers()['content-type']).toContain('text/css');
  });

  test('/ui/shared.js should return JavaScript file', async ({ page }) => {
    const response = await page.goto(`${baseURL}/ui/shared.js`);

    expect(response?.status()).toBe(200);
    expect(response?.headers()['content-type']).toContain('javascript');
  });

  test('All critical routes should NOT return 404', async ({ page }) => {
    const criticalRoutes = [
      '/',
      '/ui',
      '/ui/config',
      '/ui/health',
      '/ui/jobs',
      '/ui/browse',
      '/favicon.ico',
      '/ui/theme.css',
      '/ui/shared.js',
    ];

    for (const route of criticalRoutes) {
      const response = await page.goto(`${baseURL}${route}`);
      expect(response?.status()).not.toBe(404);
      expect(response?.status()).toBeLessThan(400);
    }
  });

  test('Dashboard should have navigation links to other pages', async ({ page }) => {
    await page.goto(`${baseURL}/`);

    const configLink = page.locator('a[href*="config"]');
    const healthLink = page.locator('a[href*="health"]');
    const browseLink = page.locator('a[href*="browse"]');

    await expect(configLink).toBeVisible();
    await expect(healthLink).toBeVisible();
    await expect(browseLink).toBeVisible();
  });

  test('Theme CSS should contain valid CSS rules', async ({ page }) => {
    const response = await page.goto(`${baseURL}/ui/theme.css`);
    const css = await response?.text();

    expect(css).toBeTruthy();
    expect(css).toContain(':root');
    expect(css).toContain('background');
  });

  test('Shared JS should contain valid JavaScript', async ({ page }) => {
    const response = await page.goto(`${baseURL}/ui/shared.js`);
    const js = await response?.text();

    expect(js).toBeTruthy();
    expect(js).toContain('function');
  });
});

test.describe('Admin UI Routes - Error Cases', () => {
  const testPort = process.env.MCB_TEST_PORT || '18080';
  const baseURL = `http://localhost:${testPort}`;

  test('Non-existent route should return 404', async ({ page }) => {
    const response = await page.goto(`${baseURL}/non-existent-route-12345`, {
      waitUntil: 'domcontentloaded',
    });

    expect(response?.status()).toBe(404);
  });

  test('404 page should have helpful error message', async ({ page }) => {
    await page.goto(`${baseURL}/non-existent-route-12345`, {
      waitUntil: 'domcontentloaded',
    });

    const html = await page.content();
    expect(html).toContain('404');
    expect(html).toContain('Not Found');
  });
});
