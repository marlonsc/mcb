# Tokio Quick Reference

## Use in This Repo

- async runtime orchestration
- task spawning and coordination
- timeouts and cancellation

## Preferred Patterns

- Keep async boundaries explicit.
- Use structured concurrency where possible.
- Bound background work with cancellation paths.
- Avoid blocking I/O on async executor threads.

## Validation Reminders

- Ensure graceful shutdown paths await task completion.
- Keep timeout values explicit near call sites.
- Guard fan-out workloads to avoid unbounded concurrency.

> Updated 2026-02-12
