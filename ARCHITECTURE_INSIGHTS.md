# WORKFLOW ORCHESTRATION ARCHITECTURE INSIGHTS

## Executive Summary

Comprehensive research of **5 reference orchestration systems** (Temporal, Argo, Dagster, Prefect, Airflow) reveals 7 core architectural patterns that enable scalable, reliable workflow systems.

---

## 1. THE MULTI-TIER EXECUTION MODEL

**Architecture:**

```
┌─────────────────────────────────────┐
│ User API Layer                      │
│ (Submit workflows, track status)    │
└────────────┬────────────────────────┘
             │
┌────────────▼────────────────────────┐
│ Orchestration Layer                 │
│ (Deterministic workflow state)      │
└────────────┬────────────────────────┘
             │
┌────────────▼────────────────────────┐
│ Activity/Operator Layer             │
│ (Non-deterministic work execution)  │
└────────────┬────────────────────────┘
             │
┌────────────▼────────────────────────┐
│ Task Routing / Queue Layer          │
│ (Activity delivery via queues)      │
└─────────────────────────────────────┘
```

**Key Principle:** Separate coordination logic from execution logic

**Why It Works:**

-   Workflows are deterministic and can be replayed from event history
-   Activities handle non-deterministic operations (API calls, DB writes)
-   Task queues decouple worker availability from workflow execution
-   Enables fault tolerance and recovery without complex state machines

**Reference:** Temporal, Cadence, Prefect

---

## 2. EVENT SOURCING + DETERMINISTIC REPLAY

**Pattern:**

```
Workflow Step N-1 → Event → Persisted → Recovery
                    ↓
                  History
                    ↓
        Replay from last checkpoint
        (Deterministic re-execution)
```

**Key Benefits:**

-   ✅ Automatic crash recovery (replay history)
-   ✅ Audit trail of all workflow decisions
-   ✅ No complex state machine logic needed
-   ✅ Testable: given same events, same Result

**Constraint:**

-   ❌ Workflow code must be deterministic
-   ❌ Cannot call external APIs directly
-   ✅ Must use activity abstraction for side effects

**Reference:** Temporal, Cadence

---

## 3. OPERATOR/PROVIDER PATTERN FOR ABSTRACTION

**Design:**

```rust
// Core abstraction
pub trait Database { ... }
pub trait VcsProvider { ... }
pub trait MessageQueue { ... }

// Implementations swapped at runtime or compile-time
pub struct PostgresProvider { ... }
pub struct SqliteProvider { ... }
pub struct GitHubProvider { ... }
pub struct GitLabProvider { ... }
```

**Why This Pattern:**

-   ✅ Multi-backend support (PostgreSQL, SQLite, MySQL)
-   ✅ Multi-VCS support (GitHub, GitLab, Gitea)
-   ✅ Multi-queue support (Redis, NATS, RabbitMQ, Kafka)
-   ✅ Testability with mock implementations

**Implementation Approaches:**

1.  **Trait-based (Static)**: Compile-time selection via generics
2.  **Dynamic dispatch**: Runtime selection via trait objects
3.  **Feature flags**: Different implementations per feature

**Gotcha:**

-   ❌ Compile-time query validation across multiple SQL dialects is impossible
-   ✅ Accept runtime validation via query builders (sqlx)

**Reference:** SQLx, Airflow providers, GitHub's gh-ost

---

## 4. ISOLATED CONCURRENCY WITH GIT WORKTREES

**Pattern:**

```
┌──────────────────────────────┐
│ Central Orchestrator         │
│ (Distribute work, track)     │
└──────────────────────────────┘
       │         │         │
       ▼         ▼         ▼
    Agent-1   Agent-2   Agent-3
   worktree   worktree   worktree
   /branch1   /branch2   /branch3
   (isolated) (isolated) (isolated)
       │         │         │
       └─────────┼─────────┘
               ▼
         Sequential Merge
        (prevent conflicts)
```

**Key Properties:**

-   Each agent has dedicated git worktree (checkout)
-   Agents work in parallel without interference
-   Central orchestrator tracks progress
-   Merges happen sequentially (not in parallel)

**Lessons from Field:**

-   ✅ Prevents branch conflicts and environment contamination
-   ✅ Each agent needs separate execution context (not just branches)
-   ❌ Don't run multiple agents on same branch
-   ❌ Don't spawn many sub-agents in single session (context exhaustion)

**Reference:** Worktrunk, Emdash, CCSwarm, Claude Code

---

## 5. EXPONENTIAL BACKOFF RETRY STRATEGY

**Pattern (from Svix webhook delivery):**

```
Retry Schedule:
  Attempt 1:  Immediate        (transient failure recovery)
  Attempt 2:  5 seconds        (fast feedback loop)
  Attempt 3:  5 minutes        (give service time to recover)
  Attempt 4:  30 minutes       (escalate wait times)
  Attempt 5:  2 hours
  Attempt 6:  5 hours
  Attempt 7-8: 10 hours each   (daily checks)
```

**Why This Works:**

-   ✅ Catches transient failures quickly (network hiccup)
-   ✅ Backs off aggressively (prevents retry storms)
-   ✅ Keeps service from being overwhelmed
-   ✅ Still attempts recovery for hours after initial failure

**Key Features:**

-   Configurable at task/workflow level
-   Support runtime override (operator intervention)
-   Conditional policies: OnFailure vs OnError vs OnTransientError
-   Dead-letter queue for exhausted retries

**Gotcha:**

-   ❌ Aggressive immediate retries cause cascading failures
-   ✅ Exponential backoff is essential for production

**Reference:** Svix, Argo Workflows, Temporal

---

## 6. POLICY ENFORCEMENT AT ADMISSION TIME

**Pattern:**

```
User Request
    ▼
┌─────────────────────────────┐
│ Admission Controller        │
│ (Policy evaluation point)   │
└─────────────────────────────┘
    │ (allow/deny)
    ├─ Allow ─→ Persist to system
    └─ Deny  ─→ Reject request
```

**Three Approaches:**

### Approach 1: OPA Gatekeeper (Kubernetes)

```rego
violation[{"msg": msg}] {
    resource := input.review.object
    not resource.metadata.labels.team
    msg := "Deployment must have team label"
}
```

-   Constraint templates for reusable policies
-   Rego policy language
-   Webhook-based admission control

### Approach 2: GitHub Branch Protection

```
REST API Endpoints:
  GET  /repos/{owner}/{repo}/branches/{branch}/protection
  PUT  /repos/{owner}/{repo}/branches/{branch}/protection

Policies:
  - Required status checks
  - Dismissible reviews
  - Admin override capability
```

### Approach 3: Fine-Grained Authorization (OpenFGA)

```
Relationships: (user, relation, resource)
Example: (alice, editor, document:123)

Query: Can alice view document:123?
→ Evaluate relationship graph → Yes/No
```

**Key Lesson:**

-   ✅ Enforce at admission (pre-merge) stage
-   ❌ Post-check enforcement causes rollback risk

**Reference:** OPA, Gatekeeper, GitHub, OpenFGA, Zanzibar

---

## 7. EVENT BROADCASTING WITH MULTIPLE CONSUMERS

**Pattern:**

```
Event Source
    ▼
┌─────────────────────┐
│ Message Queue       │
│ (Redis/NATS/Kafka)  │
└─────────────────────┘
    │ (distribute)
    ├─→ Consumer 1 (process)
    ├─→ Consumer 2 (log)
    └─→ Consumer 3 (alert)
```

**Queue Selection Guide:**

| System | Latency | Throughput | Persistence | Complexity | Use Case |
|--------|---------|-----------|-------------|-----------|----------|
| Redis Streams | Low | Medium | Memory | Low | Dev/staging, fast ephemeral events |
| NATS JetStream | Low | High | Configurable | Medium | Cloud-native, ordered delivery |
| RabbitMQ | Medium | Medium | Durable | High | Enterprise, complex routing |
| Kafka | Medium | Very High | Durable | High | Data warehouse, audit logs |

**Webhook Delivery Sub-Pattern:**

-   Each webhook is an event
-   Retries follow exponential backoff
-   Dead-letter queue captures failures
-   Manual replay capability via API/dashboard

**Reference:** Svix, Redis, NATS, RabbitMQ, Kafka

---

## INTEGRATION: HOW THESE PATTERNS WORK TOGETHER

```
┌──────────────────────────────────────────────────────────┐
│ WORKFLOW ORCHESTRATION SYSTEM                            │
├──────────────────────────────────────────────────────────┤
│                                                          │
│  User API → Multi-Tier Execution Model                  │
│               (Workflow | Activity | Queue)              │
│                      │                                   │
│  ┌──────────────────▼──────────────────┐                │
│  │ Event Sourcing + Deterministic      │                │
│  │ Replay (Recovery)                   │                │
│  └──────────────────┬──────────────────┘                │
│                     │                                   │
│  ┌──────────────────▼──────────────────┐                │
│  │ Provider Abstraction Layer          │                │
│  │ (Database, VCS, Queue traits)       │                │
│  └──────────────────┬──────────────────┘                │
│                     │                                   │
│  ┌──────────────────▼──────────────────┐                │
│  │ Multi-Agent Concurrency             │                │
│  │ (Git Worktrees, Isolation)          │                │
│  └──────────────────┬──────────────────┘                │
│                     │                                   │
│  ┌──────────────────▼──────────────────┐                │
│  │ Retry Strategy + Backoff            │                │
│  │ (Exponential, Configurable)         │                │
│  └──────────────────┬──────────────────┘                │
│                     │                                   │
│  ┌──────────────────▼──────────────────┐                │
│  │ Policy Enforcement (Admission)      │                │
│  │ (OPA, GitHub Protection, OpenFGA)   │                │
│  └──────────────────┬──────────────────┘                │
│                     │                                   │
│  ┌──────────────────▼──────────────────┐                │
│  │ Event Broadcasting + Queues         │                │
│  │ (Redis/NATS/Kafka, Webhooks, DLQ)   │                │
│  └──────────────────────────────────────┘                │
│                                                          │
└──────────────────────────────────────────────────────────┘
```

---

## CRITICAL DECISIONS TO MAKE

### 1. Execution Model

-   [ ] Sequential activities per workflow?
-   [ ] Parallel activities with fan-out/fan-in?
-   [ ] Dynamic workflow DAG (tasks generated at runtime)?

### 2. State Persistence

-   [ ] Event sourcing (full history)?
-   [ ] Snapshot + delta (for performance)?
-   [ ] Direct state storage (simpler, less recovery)?

### 3. Provider Support

-   [ ] PostgreSQL only (simplest)?
-   [ ] PostgreSQL + SQLite (dev/production)?
-   [ ] Multi-database abstraction?

### 4. Concurrency Model

-   [ ] Single workflow, sequential execution?
-   [ ] Parallel agents, isolated worktrees?
-   [ ] Both (adaptive)?

### 5. Message Queue

-   [ ] Redis (simple, fast)?
-   [ ] NATS (cloud-native)?
-   [ ] Pluggable (abstraction layer)?

---

## RECOMMENDATIONS FOR YOUR SYSTEM

### Must-Have (Foundation)

1.  **Multi-tier execution model** (Workflow → Activity → Queue)
2.  **Event sourcing + deterministic replay** (recovery)
3.  **Provider abstraction traits** (extensibility)
4.  **Exponential backoff retries** (production reliability)

### Should-Have (Production)

1.  **Git worktree orchestration** (multi-agent support)
2.  **Policy enforcement at admission** (compliance)
3.  **Event broadcasting + DLQ** (async processing)

### Nice-to-Have (Future)

-   Fine-grained authorization (OpenFGA)
-   Multiple message queue backends
-   Dynamic workflow DAGs
-   Real-time policy updates

---

## REFERENCES & FURTHER READING

See `WORKFLOW_ORCHESTRATION_RESEARCH.md` for complete documentation links covering:

-   Temporal/Cadence architecture
-   Argo Workflows operator patterns
-   Dagster asset lineage
-   Prefect dynamic workflows
-   Airflow plugin system
-   Git worktree orchestration
-   Database abstraction patterns
-   Policy enforcement systems
-   Message queue patterns
-   Webhook delivery patterns
