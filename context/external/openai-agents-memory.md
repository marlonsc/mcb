# OpenAI Agents Memory: Patterns and MCB Relevance

Last updated: 2026-02-12  
Scope: memory architecture patterns from OpenAI Agents SDK with applicability to MCB's observation/context system.  
Cross-reference: `context/external/langgraph-memory.md`, `context/external/mcb-main-libraries-reference.md`.

---

## 1. Source and Evidence Basis

Primary source: OpenAI Agents SDK documentation and context engineering guidance.

This document captures transferable design patterns, not a direct dependency integration.

---

## 2. Core Memory Patterns from OpenAI Agents

### 2.1 Bounded session memory

Keep in-context history trimmed and bounded to preserve response quality and avoid noise accumulation.

### 2.2 Durable fact separation

Persist stable user/project facts independently from turn-by-turn conversation state. This enables cross-session continuity without replaying raw transcripts.

### 2.3 Session distillation

After task completion, distill reusable notes/learnings from session history. Re-inject selectively on future queries rather than replaying full logs.

### 2.4 Structured state over raw replay

Prefer typed/structured state objects over unstructured transcript replay for memory retrieval and injection.

---

## 3. Transferable Guidance for MCB

### 3.1 Observation storage model

MCB's observation system maps directly to the "durable fact" pattern:

- compact typed records with tags and metadata
- stored via `crates/mcb-providers/src/database/sqlite/memory_repository.rs`
- retrieved through hybrid search (FTS + vector)

### 3.2 Recall strategy

Split recall into distinct search channels before execution:

- profile/identity context
- project-level patterns and decisions
- session-specific recent observations

This aligns with MCB's multi-resource search model (`code`, `memory`, `context`).

### 3.3 Post-task learning capture

After significant operations, store compact observations with structured tags rather than raw tool output. This mirrors OpenAI's "distill and re-inject" pattern.

### 3.4 Context file synchronization

Stable learnings should eventually sync into markdown context files (`context/`) for combined human + agent reuse, not remain solely in database records.

---

## 4. Anti-Patterns to Avoid

- Injecting full session transcripts as memory context (noise, token waste).
- Storing observations without tags/metadata (makes retrieval imprecise).
- Treating all memory as equally relevant (need selection/ranking before injection).

---

## 5. Cross-Document Map

- Complementary memory architecture patterns: `context/external/langgraph-memory.md`
- MCB memory service implementation: `crates/mcb-application/src/use_cases/memory_service.rs`
- MCB memory repository: `crates/mcb-providers/src/database/sqlite/memory_repository.rs`

---

## 6. References

- OpenAI Agents SDK documentation
- OpenAI context engineering best practices
- MCB observation/memory architecture in `crates/mcb-domain/src/entities/observation.rs`
