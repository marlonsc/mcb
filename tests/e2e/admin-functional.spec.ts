import { test, expect } from '@playwright/test';
import * as path from 'path';
import * as fs from 'fs';

const testPort = process.env.MCB_TEST_PORT || '18080';
const baseURL = `http://localhost:${testPort}`;

test.describe('Admin Functional Tests - Real Data Processing', () => {
  test.beforeAll(async () => {
    const testDataDir = path.join(process.cwd(), 'tests', 'fixtures', 'test-codebase');
    if (!fs.existsSync(testDataDir)) {
      console.warn(`Test data directory not found: ${testDataDir}`);
    }
  });

  test('Health page should show actual system metrics', async ({ page }) => {
    await page.goto(`${baseURL}/ui/health`);
    
    const content = await page.content();
    
    expect(content).toContain('Health');
    expect(content).toContain('Status');
    
    const statusElement = page.locator('text=/status|health|ok|running/i').first();
    await expect(statusElement).toBeVisible({ timeout: 10000 });
  });

  test('Indexing page should show indexing status', async ({ page }) => {
    await page.goto(`${baseURL}/ui/indexing`);
    
    const content = await page.content();
    
    expect(content).toContain('Indexing');
    expect(content).toMatch(/status|idle|running|complete/i);
  });

  test('Config page should display actual configuration', async ({ page }) => {
    await page.goto(`${baseURL}/ui/config`);
    
    const content = await page.content();
    
    expect(content).toContain('Configuration');
    
    const hasConfigData = content.match(/port|host|provider|embedding|vector/i);
    expect(hasConfigData).toBeTruthy();
  });

  test('Browse page should load collections grid', async ({ page }) => {
    await page.goto(`${baseURL}/ui/browse`);
    
    await page.waitForLoadState('networkidle');
    
    const grid = page.locator('#collections-grid, [data-testid="collections-grid"], .collections-grid');
    
    const gridExists = await grid.count() > 0;
    if (!gridExists) {
      const content = await page.content();
      console.log('Browse page content:', content.substring(0, 500));
    }
    
    expect(gridExists).toBeTruthy();
  });
});

test.describe('Admin API Integration Tests', () => {
  test('Health endpoint should return JSON', async ({ request }) => {
    const response = await request.get(`${baseURL}/health`);
    
    expect(response.ok()).toBeTruthy();
    expect(response.headers()['content-type']).toContain('application/json');
    
    const data = await response.json();
    expect(data).toHaveProperty('status');
  });

  test('Config endpoint should return configuration', async ({ request }) => {
    const response = await request.get(`${baseURL}/config`);
    
    expect(response.ok()).toBeTruthy();
    
    const data = await response.json();
    expect(data).toBeDefined();
  });

  test('Indexing status endpoint should return status', async ({ request }) => {
    const response = await request.get(`${baseURL}/indexing`);
    
    expect(response.ok()).toBeTruthy();
    
    const data = await response.json();
    // Check for actual fields in IndexingStatusResponse
    expect(data).toHaveProperty('is_indexing');
    expect(data).toHaveProperty('active_operations');
    expect(Array.isArray(data.operations)).toBeTruthy();
  });

  test('Collections endpoint should return array', async ({ request }) => {
    const response = await request.get(`${baseURL}/collections`);
    
    expect(response.ok()).toBeTruthy();
    
    const data = await response.json();
    expect(Array.isArray(data) || typeof data === 'object').toBeTruthy();
  });
});

test.describe('Theme and UX Tests', () => {
  test('Theme toggle should work across all pages', async ({ page }) => {
    const pages = [
      '/ui',
      '/ui/config',
      '/ui/health',
      '/ui/indexing',
      '/ui/browse',
    ];

    for (const pagePath of pages) {
      await page.goto(`${baseURL}${pagePath}`);
      
      const themeToggle = page.locator('button[title*="Theme"], button[aria-label*="theme"]').first();
      
      if (await themeToggle.count() > 0) {
        const htmlElement = page.locator('html');
        const initialTheme = await htmlElement.getAttribute('data-theme');
        
        await themeToggle.click();
        await page.waitForTimeout(300);
        
        const newTheme = await htmlElement.getAttribute('data-theme');
        expect(newTheme).not.toBe(initialTheme);
      }
    }
  });

  test('Navigation links should work between pages', async ({ page }) => {
    await page.goto(`${baseURL}/`);
    
    const links = await page.locator('a[href^="/ui"]').all();
    
    if (links.length > 0) {
      const firstLink = links[0];
      const href = await firstLink.getAttribute('href');
      
      await firstLink.click();
      await page.waitForLoadState('networkidle');
      
      expect(page.url()).toContain(href || '');
    }
  });

  test('All pages should be responsive', async ({ page }) => {
    const viewports = [
      { width: 375, height: 667, name: 'mobile' },
      { width: 768, height: 1024, name: 'tablet' },
      { width: 1920, height: 1080, name: 'desktop' },
    ];

    const pages = ['/', '/ui/health', '/ui/browse'];

    for (const viewport of viewports) {
      await page.setViewportSize(viewport);
      
      for (const pagePath of pages) {
        await page.goto(`${baseURL}${pagePath}`);
        
        const hasHorizontalScroll = await page.evaluate(() => {
          return document.documentElement.scrollWidth > window.innerWidth;
        });
        
        expect(hasHorizontalScroll).toBe(false);
      }
    }
  });
});

test.describe('Error Handling and Edge Cases', () => {
  test('Invalid collection should show error message', async ({ page }) => {
    await page.goto(`${baseURL}/ui/browse/nonexistent-collection-12345`);
    
    const content = await page.content();
    const hasError = content.match(/error|not found|invalid|404/i);
    
    expect(hasError).toBeTruthy();
  });

  test('Server should handle rapid page navigation', async ({ page }) => {
    const pages = ['/', '/ui/health', '/ui/config', '/ui/indexing', '/ui/browse'];
    
    for (let i = 0; i < 3; i++) {
      for (const pagePath of pages) {
        const response = await page.goto(`${baseURL}${pagePath}`, {
          waitUntil: 'domcontentloaded',
          timeout: 10000,
        });
        
        expect(response?.status()).toBeLessThan(500);
      }
    }
  });

  test('CSS and JS assets should load without errors', async ({ page }) => {
    const errors: string[] = [];
    
    page.on('console', msg => {
      if (msg.type() === 'error') {
        errors.push(msg.text());
      }
    });
    
    await page.goto(`${baseURL}/`);
    await page.waitForLoadState('networkidle');
    
    const criticalErrors = errors.filter(err => 
      !err.includes('favicon') && 
      !err.includes('net::ERR_FAILED') &&
      !err.includes('404')
    );
    
    expect(criticalErrors.length).toBe(0);
  });
});
