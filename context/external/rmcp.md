# RMCP Quick Reference

Purpose: fast operational notes for RMCP usage in this repository.

## What RMCP Is Used For Here

- Tool-exposed server handlers
- request/response transport behavior
- handler-level error mapping

## Core Patterns

- Register handlers centrally and keep routing explicit.
- Keep tool signatures stable and strongly typed.
- Return structured errors with actionable messages.
- Prefer deterministic behavior for index/search flows.

## Common Pitfalls

- Mismatched tool schema vs runtime payload shape
- Handler registration drift after refactors
- Silent fallback behavior hiding provider errors
- Unbounded retries on transport failures

## Verification Checklist

- Handler is registered and reachable.
- Input schema matches implementation expectations.
- Error path includes useful diagnostics.
- Trace/log output is sufficient for debugging.

## Full Detail

-> Full analysis: `docs/architecture/RMCP_PROTOCOL_ANALYSIS.md`

> Updated 2026-02-12
