# Golden Tests - E2E Integration

## Overview

MCB has a **3-layer testing strategy** to ensure admin web UI routes are always
accessible:

1. **Unit Tests (Rust)**: Test isolated route handlers
2. **Integration Tests (Rust)**: Test full Rocket server with `admin_rocket()`
3. **E2E Tests (Playwright)**: Test actual HTTP server end-to-end

## Why All 3 Layers?

**v0.2.0 Bug**: Admin UI returned 404 on all routes because web routes were only
mounted in `web_rocket()` (test fixture) but NOT in `admin_rocket()` (production
server).

- ✅ **Unit tests passed** - They tested `web_rocket()` which had routes
- ❌ **Integration tests MISSING** - No tests for `admin_rocket()` production
  config
- ❌ **E2E tests NOT RUN** - Playwright tests existed but weren't integrated
  into CI

**Result**: Bug shipped to production.

## Test Structure

### Layer 1: Unit Tests (Rust)

**Location**: `crates/mcb-server/tests/admin/web_tests.rs`

Tests individual route handlers in isolation using `web_rocket()`:

```rust
#[rocket::async_test]
async fn test_dashboard_returns_html() {
    let client = Client::tracked(web_rocket()).await.expect("...");
    let response = client.get("/").dispatch().await;
    assert_eq!(response.status(), Status::Ok);
}
```

**What they catch**: Route handler bugs, template rendering issues.
**What they miss**: Routes not mounted in production `admin_rocket()`.

### Layer 2: Integration Tests (Rust)

**Location**: `crates/mcb-server/tests/integration/golden_admin_web_e2e.rs`

Tests the **REAL production server** using `admin_rocket()`:

```rust
#[rocket::async_test]
async fn test_admin_rocket_dashboard_is_accessible() {
    let (client, _, _) = AdminTestHarness::new().build_client().await;
    let response = client.get("/").dispatch().await;

    assert_eq!(
        response.status(),
        Status::Ok,
        "Dashboard (/) must return 200 OK, not 404. This is the PRODUCTION \
        route."
    );
}
```

**What they catch**: Routes not mounted in `admin_rocket()`, production config issues.
**What they miss**: HTTP server startup issues, network problems, browser rendering.

### Layer 3: E2E Tests (Playwright)

**Location**: `e2e/admin-ui-routes.spec.ts`

Tests the actual HTTP server end-to-end:

```typescript
test('Dashboard (/) should return 200 OK with HTML', async ({ page }) => {
  const response = await page.goto(`${baseURL}/`);
  expect(response?.status()).toBe(200);
  expect(response?.headers()['content-type']).toContain('text/html');
});
```

**What they catch**: Everything - HTTP server config, routing, rendering,
browser compatibility.
**What they miss**: Nothing (but slowest to run).

## Running Tests

### Quick (Development)

```bash

# Run just the critical integration tests (Layer 2)
make test SCOPE=integration

# Run specific test file
cargo test --package mcb-server --test integration golden_admin_web_e2e
```

### Full (Pre-commit)

```bash

# Run all Rust tests + E2E
make check  # Runs fmt + lint + test SCOPE=all
make test SCOPE=e2e  # Runs Playwright E2E tests
```

### E2E Only

```bash

# Run all Playwright tests
make test SCOPE=e2e

# Run with UI (interactive)
make test SCOPE=e2e

# Run in debug mode
make test SCOPE=e2e

# Run specific test suite
npm run test:admin  # Admin routes only
npm run test:browse  # Browse UI only
```

## CI Integration

### GitHub Actions Workflow

**File**: `.github/workflows/ci.yml`

```yaml
jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      -   uses: actions/checkout@v4
      -   name: Run Rust tests
        run: make test SCOPE=all

      -   name: Install Playwright
        run: npm ci

      -   name: Run E2E tests
        run: make test SCOPE=e2e
```

## Critical Test Coverage

### Routes That MUST Always Return 200

<!-- markdownlint-disable MD013 -->
| Route | Layer 1 | Layer 2 | Layer 3 |
| ------- | --------- | --------- | --------- |
| `/` | ✅ `test_dashboard_returns_html` | ✅ `test_admin_rocket_dashboard_is_accessible` | ✅ `Dashboard should return 200` |
| `/ui/config` | ✅ `test_config_page_returns_html` | ✅ `test_admin_rocket_config_page_is_accessible` | ✅ `config should return 200` |
| `/ui/health` | ✅ `test_health_page_returns_html` | ✅ `test_admin_rocket_health_page_is_accessible` | ✅ `health should return 200` |
| `/ui/jobs` | ✅ `test_jobs_page_returns_html` | ✅ `test_admin_rocket_jobs_page_is_accessible` | ✅ `jobs should return 200` |
| `/ui/browse` | ✅ | ✅ `test_admin_rocket_browse_page_is_accessible` | ✅ `browse should return 200` |
| `/favicon.ico` | ✅ `test_favicon_returns_svg` | ✅ `test_admin_rocket_favicon_is_accessible` | ✅ `favicon should return SVG` |
| `/ui/theme.css` | ❌ | ✅ `test_admin_rocket_theme_css_is_accessible` | ✅ `theme CSS should return 200` |
| `/ui/shared.js` | ❌ | ✅ `test_admin_rocket_shared_js_is_accessible` | ✅ `shared JS should return 200` |
<!-- markdownlint-enable MD013 -->

## Maintenance

### When Adding New Routes

1. **Add to `admin/web/handlers.rs`**
2. **Mount in `admin/routes.rs`** (CRITICAL - this is where v0.2.0 bug happened)
3. **Add Layer 2 test** in `golden_admin_web_e2e.rs`
4. **Add Layer 3 test** in `admin-ui-routes.spec.ts`

### When Routes Return 404

1. Check Layer 3 first: `make test SCOPE=e2e`
2. If failing, check Layer 2: `cargo test golden_admin_web_e2e`
3. If passing, check `admin/routes.rs` - routes might not be mounted

## Troubleshooting

### Playwright Tests Fail with "Connection Refused"

MCB server not running. Playwright config auto-starts server via
`webServer.command`.

**Fix**:

```bash

# Manual server start for debugging
./target/release/mcb serve --server &
MCB_BASE_URL=http://localhost:8080 npx playwright test
```

### Integration Tests Pass But E2E Fails

Routes mounted in `web_rocket()` but not `admin_rocket()`.

**Fix**: Check `crates/mcb-server/src/admin/routes.rs` - add routes to `mount()`
call.

### E2E Tests Skip (CI Only)

Playwright dependencies not installed.

**Fix**: Add to CI workflow:

```yaml
-   name: Install Playwright browsers
  run: npx playwright install --with-deps chromium
```

## Related Documentation

- [GOLDEN_TESTS_CONTRACT.md](./GOLDEN_TESTS_CONTRACT.md) - Test contract specifications
- [Testing Strategy](./INTEGRATION_TESTS.md) - Overall testing approach
- [CI/CD Pipeline](../../.github/workflows/ci.yml) - Continuous integration config
