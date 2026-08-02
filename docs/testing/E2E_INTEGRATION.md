<!-- markdownlint-disable MD013 MD024 MD025 MD003 MD022 MD031 MD032 MD036 MD041 MD060 -->
# Testing - HTTP, MCP, And UI E2E

## Overview

MCB validates real user and agent workflows across three current layers:

1. **Rust handler/golden tests**: exercise MCP handlers through the real test DI
   stack.
2. **Rust stdio MCP integration tests**: exercise the public command workflow an
   agent uses through MCP.
3. **Playwright E2E tests**: start the real Loco/Axum `mcb` server and drive the
   admin UI and HTTP endpoints through a browser.

The historical Rocket `web_rocket()` / `admin_rocket()` path is no longer the
runtime baseline. The current server path is Loco plus Axum, with standalone
routes from `mcb-server` embedded by `crates/mcb/src/initializers/mcp_server.rs`.

## Current Coverage

| Layer | Files | What It Must Prove |
| ----- | ----- | ------------------ |
| Golden MCP handlers | `crates/mcb-server/tests/e2e/golden_e2e_complete.rs`, `crates/mcb-server/tests/e2e/golden_tools_e2e.rs`, `crates/mcb-server/tests/e2e/test_project_operations.rs` | indexing, status, clear, search relevance, and project management behaviour through real handlers |
| Stdio MCP workflow | `crates/mcb/tests/integration/mcp_commands/workflow_tests.rs` | project creation, session start, memory persistence, repository indexing, and code search through public MCP commands |
| Admin routes and assets | `tests/e2e/admin-ui-routes.spec.ts` | authenticated route status, assets, navigation links, and 404 handling |
| Admin functional UI/API | `tests/e2e/admin-functional.spec.ts` | health, config, jobs, collections, theme/UX, responsive pages, and runtime browser errors |
| Browse UI | `tests/e2e/browse-ui.spec.ts` | real MCP indexing of `tests/fixtures/sample_codebase`, filtered browse rendering, keyboard navigation, theme persistence, responsive layout, and error states |

## Authentication

Playwright tests seed real admin credentials before touching protected admin
pages or APIs. The helper in `tests/e2e/helpers/mcp-auth.ts` calls the public
MCP `entity` tool to create an org, admin user, and API key, then sends the
resulting `X-API-Key` / `X-Admin-Key` headers in browser and request contexts.

Do not add unauthenticated admin-test shortcuts. If a page or API requires admin
access, extend the shared helper and keep the browser test on the real auth path.

## Running Tests

Run the smallest decisive scope first:

```bash
make test SCOPE=golden
make test SCOPE=integration
make test SCOPE=e2e APPLY=Y
```

For a focused Playwright browse run from the repository root:

```bash
npm exec -- playwright test --config=playwright.config.ts e2e/browse-ui.spec.ts --reporter=list
```

The Playwright config in `tests/playwright.config.ts` starts the real server with
`LOCO_ENV=test`, a per-port SQLite database under `/tmp`, HTTP MCP transport,
and `reuseExistingServer: false` so stale local processes cannot satisfy a test.

## Browse Workflow Contract

`browse-ui.spec.ts` indexes `tests/fixtures/sample_codebase` through real MCP
calls before opening the UI. The UI and chunk API support an optional collection
filter:

```text
/ui/browse?collection=<collection>
/chunks?collection=<collection>
```

The collection is normalized by the same server-side collection-name logic used
by indexing. Tests must wait for actual fixture files to render, not only for the
page shell.

## Route Contract

Critical admin routes must either return the expected authenticated page/API
response or a deliberate 404 for unknown paths:

| Route | Current Playwright Coverage |
| ----- | --------------------------- |
| `/` | dashboard HTML and navigation |
| `/ui` | dashboard alias |
| `/ui/config` | configuration page |
| `/ui/health` | health page and JSON API |
| `/ui/jobs` | jobs page and JSON API |
| `/ui/browse` | browse page and real indexed chunks |
| `/favicon.ico` | SVG icon |
| `/ui/theme.css` | CSS asset |
| `/ui/shared.js` | JavaScript asset |
| unknown admin path | 404 status and helpful 404 page |

If an unknown route returns 200, inspect the embedded-router composition in
`crates/mcb/src/initializers/mcp_server.rs` and the standalone router builder in
`crates/mcb-server/src/controllers/routes.rs`.

## Maintenance

When adding or changing an admin route:

1. Update the Axum route table in `crates/mcb-server/src/controllers/routes.rs`
   or the matching web/API controller.
2. Add focused Rust coverage when the behaviour belongs to a handler or MCP
   command.
3. Add or update Playwright coverage when the behaviour is user-visible in the
   browser.
4. If the route is protected, use `ensureE2eAdminAuth()` instead of adding an
   unauthenticated test-only path.

## Troubleshooting

If Playwright cannot connect, check the `webServer.command` in
`tests/playwright.config.ts`; it is responsible for starting `target/release/mcb`
when available or `cargo run --release` otherwise.

If browse UI tests show an empty page, verify that the MCP indexing step
completed and that `/chunks?collection=<collection>` returns fixture files before
asserting on the browser view.

## Related Documentation

- [GOLDEN_TESTS_CONTRACT.md](./GOLDEN_TESTS_CONTRACT.md) - golden MCP contract
- [INTEGRATION_TESTS.md](./INTEGRATION_TESTS.md) - service-backed integration tests
- [CI/CD Pipeline](../../.github/workflows/ci.yml) - continuous integration config
