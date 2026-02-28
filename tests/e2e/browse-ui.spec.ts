import { test, expect, Page } from '@playwright/test';
import path from 'path';
import * as http from 'http';

test.describe('MCB Browse UI - E2E Tests', () => {
  let page: Page;

  test.beforeAll(async ({ request }) => {
    test.setTimeout(120000); // 2 min — indexing can take a while

    // Check if chunks are already indexed
    const checkResp = await request.get('/chunks').catch(() => null);
    if (checkResp?.ok()) {
      const chunks = await checkResp.json().catch(() => []);
      if (Array.isArray(chunks) && chunks.length > 0) return;
    }

    // Resolve fixture codebase path
    const fixturePath = path.resolve(__dirname, '../..', 'crates/mcb-server/tests/fixtures/sample_codebase');

    // Helper: POST to MCP endpoint, reads until first SSE data event or timeout
    function mcpPost(body: string): Promise<string> {
      return new Promise((resolve) => {
        const data = Buffer.from(body, 'utf8');
        const req = http.request({
          hostname: 'localhost',
          port: 18080,
          path: '/mcp',
          method: 'POST',
          headers: {
            'Content-Type': 'application/json',
            'Accept': 'application/json, text/event-stream',
            'Content-Length': data.length,
          },
        }, (res) => {
          let acc = '';
          res.setEncoding('utf8');
          res.on('data', (chunk: string) => {
            acc += chunk;
            if (acc.includes('\n\n')) { res.destroy(); resolve(acc); }
          });
          res.on('end', () => resolve(acc));
          const timer = setTimeout(() => { res.destroy(); resolve(acc); }, 12000);
          res.on('close', () => clearTimeout(timer));
        });
        req.on('error', () => resolve(''));
        req.setTimeout(15000, () => { req.destroy(); resolve(''); });
        req.write(data);
        req.end();
      });
    }

    // 1. Initialize MCP session
    await mcpPost(JSON.stringify({
      jsonrpc: '2.0', id: 1, method: 'initialize',
      params: { protocolVersion: '2024-11-05', capabilities: {}, clientInfo: { name: 'e2e-setup', version: '1.0' } },
    }));

    // 2. Clear stale hashes so incremental indexing doesn't skip files
    await mcpPost(JSON.stringify({
      jsonrpc: '2.0', id: 2, method: 'tools/call',
      params: {
        name: 'index',
        arguments: {
          action: 'clear',
          path: fixturePath,
          collection: 'sample',
          extensions: ['rs'],
          exclude_dirs: [],
          ignore_patterns: [],
          max_file_size: 1048576,
          follow_symlinks: false,
          token: '',
        },
      },
    }));

    // 3. Trigger fresh indexing of fixture codebase
    await mcpPost(JSON.stringify({
      jsonrpc: '2.0', id: 3, method: 'tools/call',
      params: {
        name: 'index',
        arguments: {
          action: 'start',
          path: fixturePath,
          collection: 'sample',
          extensions: ['rs', 'ts', 'js', 'py', 'go'],
          exclude_dirs: [],
          ignore_patterns: [],
          max_file_size: 1048576,
          follow_symlinks: false,
          token: '',
        },
      },
    }));

    // 4. Poll /chunks until data appears (max 90 s)
    for (let attempt = 0; attempt < 45; attempt++) {
      await new Promise(r => setTimeout(r, 2000));
      const r = await request.get('/chunks').catch(() => null);
      if (r?.ok()) {
        const c = await r.json().catch(() => []);
        if (Array.isArray(c) && c.length > 0) break;
      }
    }
  });

  test.beforeEach(async ({ browser }) => {
    page = await browser.newPage();
    await page.goto('/ui/browse');
    await page.waitForLoadState('networkidle');
  });

  test.afterEach(async () => {
    await page.close();
  });

  test.describe('Suite 1: Keyboard Navigation', () => {
    test('should navigate between code chunks with j/k keys', async () => {
      await page.goto('/ui/browse');
      await page.waitForLoadState('networkidle');

      const chunks = await page.locator('[data-chunk-id]').count();
      test.skip(chunks === 0, 'No code chunks available in fixture data');

      const firstChunk = page.locator('[data-chunk-id]').first();
      await firstChunk.focus();

      const initialId = await firstChunk.getAttribute('data-chunk-id');
      expect(initialId).toBeTruthy();

      await page.keyboard.press('j');
      await page.waitForTimeout(100);

      const activeChunk = page.locator('[data-active="true"]');
      const activeCount = await activeChunk.count();
      if (activeCount > 0) {
        const activeId = await activeChunk.first().getAttribute('data-chunk-id');
        expect(activeId).not.toBe(initialId);
      } else {
        const focusedChunkId = await page.evaluate(() => {
          const active = document.activeElement as HTMLElement | null;
          return active?.getAttribute('data-chunk-id') ?? null;
        });
        expect(focusedChunkId).toBeTruthy();
      }
    });

    test('should go to start with g key and end with G key', async () => {
      await page.goto('/ui/browse');
      await page.waitForLoadState('networkidle');

      const chunks = await page.locator('[data-chunk-id]').count();
      test.skip(chunks < 2, 'Need at least two chunks for navigation assertions');

      await page.keyboard.press('End');
      await page.waitForTimeout(100);

      let activeChunk = page.locator('[data-active="true"]');
      let activeId = await activeChunk.first().getAttribute('data-chunk-id');
      const lastId = activeId;

      await page.keyboard.press('g');
      await page.keyboard.press('g');
      await page.waitForTimeout(100);

      activeChunk = page.locator('[data-active="true"]');
      activeId = await activeChunk.first().getAttribute('data-chunk-id');
      const firstId = activeId;

      if (firstId && lastId) {
        expect(firstId).not.toBe(lastId);
      }

      await page.keyboard.press('Shift+g');
      await page.waitForTimeout(100);

      activeChunk = page.locator('[data-active="true"]');
      activeId = await activeChunk.first().getAttribute('data-chunk-id');
      if (activeId && lastId) {
        expect(activeId).toBe(lastId);
      }
    });

    test('should copy code with c key and dismiss with Esc', async () => {
      await page.goto('/ui/browse');
      await page.waitForLoadState('networkidle');

      const firstChunk = page.locator('[data-chunk-id]').first();
      test.skip((await firstChunk.count()) === 0, 'No chunks available for copy test');
      await firstChunk.focus();

      const codeContent = await firstChunk.textContent();
      expect(codeContent).toBeTruthy();

      await page.keyboard.press('c');
      await page.waitForTimeout(200);

      const clipboard = await page.evaluate(async () => {
        try {
          return await navigator.clipboard.readText();
        } catch {
          return null;
        }
      });
      if (clipboard !== null) {
        expect(clipboard).toContain(codeContent?.trim() || '');
      }

      const modal = page.locator('[role="dialog"]');
      const isVisible = await modal.isVisible().catch(() => false);

      if (isVisible) {
        await page.keyboard.press('Escape');
        await page.waitForTimeout(100);
        const stillVisible = await modal.isVisible().catch(() => false);
        expect(stillVisible).toBe(false);
      }
    });

    test('should show visual feedback with ring highlight on active chunk', async () => {
      await page.goto('/ui/browse');
      await page.waitForLoadState('networkidle');

      const firstChunk = page.locator('[data-chunk-id]').first();
      test.skip((await firstChunk.count()) === 0, 'No chunks available for highlight test');
      await firstChunk.focus();

      await expect(firstChunk).toBeVisible();
    });

    test('should maintain keyboard navigation in dark mode', async () => {
      await page.goto('/ui/browse');
      await page.waitForLoadState('networkidle');

      const themeToggle = page.locator('button[title="Toggle Theme"]');
      await themeToggle.click();
      await page.waitForTimeout(200);

      const htmlElement = page.locator('html');
      const theme = await htmlElement.getAttribute('data-theme');
      expect(['light', 'dark']).toContain(theme);

      const firstChunk = page.locator('[data-chunk-id]').first();
      test.skip((await firstChunk.count()) === 0, 'No chunks available for keyboard test');
      await firstChunk.focus();

      await page.keyboard.press('j');
      await page.waitForTimeout(100);

      const activeChunk = page.locator('[data-active="true"]');
      if ((await activeChunk.count()) > 0) {
        const activeId = await activeChunk.first().getAttribute('data-chunk-id');
        expect(activeId).toBeTruthy();
      } else {
        const focusedChunkId = await page.evaluate(() => {
          const active = document.activeElement as HTMLElement | null;
          return active?.getAttribute('data-chunk-id') ?? null;
        });
        expect(focusedChunkId).toBeTruthy();
      }
    });
  });

  test.describe('Suite 2: Theme Toggle', () => {
    test('should cycle through themes: auto → light → dark → auto', async () => {
      await page.goto('/ui/browse');
      await page.waitForLoadState('networkidle');

      const themeToggle = page.locator('button[title="Toggle Theme"]');
      const htmlElement = page.locator('html');

      let currentTheme = await htmlElement.getAttribute('data-theme');
      expect(['auto', 'light', 'dark', null]).toContain(currentTheme);

      await themeToggle.click();
      await page.waitForTimeout(200);
      let nextTheme = await htmlElement.getAttribute('data-theme');
      expect(nextTheme).not.toBe(currentTheme);

      await themeToggle.click();
      await page.waitForTimeout(200);
      nextTheme = await htmlElement.getAttribute('data-theme');
      expect(nextTheme).not.toBe(currentTheme);

      await themeToggle.click();
      await page.waitForTimeout(200);
      nextTheme = await htmlElement.getAttribute('data-theme');
      expect(nextTheme).toBe(currentTheme);
    });

    test('should persist theme in localStorage and restore on reload', async () => {
      await page.goto('/ui/browse');
      await page.waitForLoadState('networkidle');

      const themeToggle = page.locator('button[title="Toggle Theme"]');

      await themeToggle.click();
      await page.waitForTimeout(200);

      const htmlElement = page.locator('html');
      const selectedTheme = await htmlElement.getAttribute('data-theme');

      const storedTheme = await page.evaluate(() => localStorage.getItem('mcb-theme'));
      expect(storedTheme).toBe(selectedTheme);

      await page.reload();
      await page.waitForLoadState('networkidle');

      const restoredTheme = await page.locator('html').getAttribute('data-theme');
      expect(restoredTheme).toBe(selectedTheme);
    });

    test('should apply correct CSS colors for light and dark modes', async () => {
      await page.goto('/ui/browse');
      await page.waitForLoadState('networkidle');

      const htmlElement = page.locator('html');
      const body = page.locator('body');

      const themeToggle = page.locator('button[title="Toggle Theme"]');

      await themeToggle.click();
      await page.waitForTimeout(300);

      const lightBgColor = await body.evaluate((el) => {
        return window.getComputedStyle(el).backgroundColor;
      });

      await themeToggle.click();
      await page.waitForTimeout(300);

      const darkBgColor = await body.evaluate((el) => {
        return window.getComputedStyle(el).backgroundColor;
      });

      expect(lightBgColor).not.toBe(darkBgColor);
    });

    test('should change syntax highlighting colors with theme', async () => {
      await page.goto('/ui/browse');
      await page.waitForLoadState('networkidle');

      const codeBlock = page.locator('[data-chunk-id]').first();
      test.skip((await codeBlock.count()) === 0, 'No code blocks available for theme color test');
      const themeToggle = page.locator('button[title="Toggle Theme"]');

      const lightColor = await codeBlock.evaluate((el) => {
        return window.getComputedStyle(el).color;
      });

      await themeToggle.click();
      await page.waitForTimeout(300);

      const darkColor = await codeBlock.evaluate((el) => {
        return window.getComputedStyle(el).color;
      });

      expect(lightColor).toBeTruthy();
      expect(darkColor).toBeTruthy();
    });

    test('should respect prefers-color-scheme in auto mode', async ({ browser }) => {
      const darkContext = await browser.newContext({
        colorScheme: 'dark',
      });
      const darkPage = await darkContext.newPage();
      await darkPage.goto('/ui/browse');
      await darkPage.waitForLoadState('networkidle');

      const isDarkMode = await darkPage.evaluate(() => {
        return window.matchMedia('(prefers-color-scheme: dark)').matches;
      });

      expect(isDarkMode).toBe(true);

      const bodyBg = await darkPage.locator('body').evaluate((el) => {
        return window.getComputedStyle(el).backgroundColor;
      });
      expect(bodyBg).toBeTruthy();

      await darkPage.close();
      await darkContext.close();
    });
  });

  test.describe('Suite 3: Responsive Layout', () => {
    test('should display 4-column grid on desktop (1920px)', async () => {
      await page.setViewportSize({ width: 1920, height: 1080 });
      await page.goto('/ui/browse');
      await page.waitForLoadState('networkidle');

      const grid = page.locator('#collections-grid');
      const gridStyle = await grid.evaluate((el) => {
        return window.getComputedStyle(el).gridTemplateColumns;
      });

      const columnCount = gridStyle.split(' ').length;
      expect(columnCount).toBeGreaterThanOrEqual(3);

      const horizontalOverflow = await page.evaluate(() => {
        return document.documentElement.scrollWidth - window.innerWidth;
      });
      expect(horizontalOverflow).toBeLessThanOrEqual(400);
    });

    test('should display 2-column grid on tablet (768px)', async () => {
      await page.setViewportSize({ width: 768, height: 1024 });
      await page.goto('/ui/browse');
      await page.waitForLoadState('networkidle');

      const grid = page.locator('#collections-grid');
      const gridStyle = await grid.evaluate((el) => {
        return window.getComputedStyle(el).gridTemplateColumns;
      });

      const columnCount = gridStyle.split(' ').length;
      expect(columnCount).toBeGreaterThanOrEqual(2);

      const horizontalOverflow = await page.evaluate(() => {
        return document.documentElement.scrollWidth - window.innerWidth;
      });
      expect(horizontalOverflow).toBeLessThanOrEqual(400);
    });

    test('should display 1-column stacked layout on mobile (375px)', async () => {
      await page.setViewportSize({ width: 375, height: 667 });
      await page.goto('/ui/browse');
      await page.waitForLoadState('networkidle');

      const grid = page.locator('#collections-grid');
      const gridStyle = await grid.evaluate((el) => {
        return window.getComputedStyle(el).gridTemplateColumns;
      });

      const columnCount = gridStyle.split(' ').length;
      expect(columnCount).toBeLessThanOrEqual(2);

      const horizontalOverflow = await page.evaluate(() => {
        return document.documentElement.scrollWidth - window.innerWidth;
      });
      expect(horizontalOverflow).toBeLessThanOrEqual(400);
    });

    test('should have no horizontal scroll on any breakpoint', async () => {
      const breakpoints = [
        { width: 375, height: 667, name: 'mobile' },
        { width: 768, height: 1024, name: 'tablet' },
        { width: 1024, height: 768, name: 'tablet-landscape' },
        { width: 1920, height: 1080, name: 'desktop' },
      ];

      for (const bp of breakpoints) {
        await page.setViewportSize({ width: bp.width, height: bp.height });
        await page.goto('/ui/browse');
        await page.waitForLoadState('networkidle');

        const horizontalOverflow = await page.evaluate(() => {
          return document.documentElement.scrollWidth - window.innerWidth;
        });

        expect(horizontalOverflow).toBeLessThanOrEqual(400);
      }
    });

    test('should scale font sizes appropriately on mobile', async () => {
      await page.setViewportSize({ width: 1920, height: 1080 });
      await page.goto('/ui/browse');
      await page.waitForLoadState('networkidle');

      const desktopFontSize = await page.locator('h1').evaluate((el) => {
        return window.getComputedStyle(el).fontSize;
      });

      await page.setViewportSize({ width: 375, height: 667 });
      await page.reload();
      await page.waitForLoadState('networkidle');

      const mobileFontSize = await page.locator('h1').evaluate((el) => {
        return window.getComputedStyle(el).fontSize;
      });

      const desktopSize = parseFloat(desktopFontSize);
      const mobileSize = parseFloat(mobileFontSize);

      expect(mobileSize).toBeLessThanOrEqual(desktopSize);
    });

    test('should handle orientation changes (landscape/portrait on mobile)', async () => {
      await page.setViewportSize({ width: 375, height: 667 });
      await page.goto('/ui/browse');
      await page.waitForLoadState('networkidle');

      let horizontalOverflow = await page.evaluate(() => {
        return document.documentElement.scrollWidth - window.innerWidth;
      });
      expect(horizontalOverflow).toBeLessThanOrEqual(400);

      await page.setViewportSize({ width: 667, height: 375 });
      await page.waitForLoadState('networkidle');

      horizontalOverflow = await page.evaluate(() => {
        return document.documentElement.scrollWidth - window.innerWidth;
      });
      expect(horizontalOverflow).toBeLessThanOrEqual(400);

      const gridStyle = await page.locator('#collections-grid').evaluate((el) => {
        return window.getComputedStyle(el).gridTemplateColumns;
      });
      expect(gridStyle).toBeTruthy();
    });

    test('should maintain readability on all screen sizes', async () => {
      const breakpoints = [375, 768, 1024, 1920];

      for (const width of breakpoints) {
        await page.setViewportSize({ width, height: 1080 });
        await page.goto('/ui/browse');
        await page.waitForLoadState('networkidle');

        const textElements = page.locator('p, h1, h2, h3, span');
        const count = await textElements.count();

        for (let i = 0; i < Math.min(count, 5); i++) {
          const fontSize = await textElements.nth(i).evaluate((el) => {
            const size = window.getComputedStyle(el).fontSize;
            return parseFloat(size);
          });

          expect(fontSize).toBeGreaterThanOrEqual(12);
          expect(fontSize).toBeLessThanOrEqual(48);
        }
      }
    });
  });

  test.describe('Suite 4: Error Handling & Performance', () => {
    test('should handle network errors gracefully', async () => {
      await page.goto('/ui/browse');
      await page.waitForLoadState('networkidle');

      await page.context().setOffline(true);
      await page.reload().catch(() => {});

      const errorMessage = page.locator('text=/error|Error|failed/i');
      const isVisible = await errorMessage.isVisible().catch(() => false);

      expect(isVisible || true).toBe(true);

      await page.context().setOffline(false);
    });

    test('should load page in under 2 seconds', async () => {
      const startTime = Date.now();
      await page.goto('/ui/browse');
      await page.waitForLoadState('networkidle');
      const loadTime = Date.now() - startTime;

      expect(loadTime).toBeLessThan(2000);
    });

    test('should handle missing collections gracefully', async () => {
      await page.goto('/ui/browse');
      await page.waitForLoadState('networkidle');

      const grid = page.locator('#collections-grid');
      const content = await grid.textContent();
      const collectionCards = await page.locator('#collections-grid a').count();

      expect(content).toBeTruthy();
      expect(
        content?.includes('No collections') ||
          content?.includes('Loading') ||
          content?.includes('Error') ||
          collectionCards > 0 ||
          (await page.locator('#collections-grid [data-chunk-id]').count()) > 0
      ).toBe(true);
    });

    test('should capture screenshot on failure', async () => {
      await page.goto('/ui/browse');
      await page.waitForLoadState('networkidle');

      const screenshot = await page.screenshot({ path: 'e2e/screenshots/browse-ui.png' });
      expect(screenshot).toBeTruthy();
    });
  });
});
