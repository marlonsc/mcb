# WORKFLOW ORCHESTRATION RESEARCH REPORT

## Reference Implementations & Best Practices

---

## 1. WORKFLOW ORCHESTRATION PATTERNS

### A. Temporal.io & Cadence

**Architecture: Multi-Tier Execution Model**

-   **Tier 1: Workflow Layer** - Deterministic, fault-oblivious state machine
-   **Tier 2: Activity Layer** - Non-deterministic execution of actual work
-   **Tier 3: Task Routing** - Activity delivery via task queues/task lists

**Key Characteristics:**

-   Workflows are stateful yet resilient to process/service failures
-   State is persisted in event history; execution resumes from history on recovery
-   Activities are invoked asynchronously through task lists (essentially queues)
-   No direct API calls from workflow; must use activity abstraction

**Compensating Transactions (Saga Pattern):**

```
Saga Pattern: Orchestrated → Choreographed
- Orchestrated: Central workflow coordinates compensations
- Choreographed: Services publish events triggering compensation
- Temporal uses orchestrated sagas with explicit compensating activities
```

**Example Pattern:**

```
Workflow:
  1. Execute Activity1
  2. Execute Activity2
  3. On failure of Activity2:
     - Execute CompensationActivity1 (undo Activity1)
     - Throw error to workflow
```

**References:**

-   Temporal Blog: "Compensating Actions, part of a complete breakfast" (2023)
-   Saga Pattern Mastery Guide: temporal.io/blog/mastering-saga-patterns
-   Cadence Docs: cadenceworkflow.io/docs/concepts/

### B. Operator Override & Conditional Execution

**Argo Workflows Pattern:**

-   Override retry strategies at runtime with parameter substitution
-   `retryStrategy` defined in WorkflowTemplate can be overridden per submission
-   Fallback/recover patterns for failing activities
-   Support for `retryPolicy` combinations (OnFailure, OnError, OnTransientError)

**Key Learning:**

-   Make retry behavior configurable, not hardcoded
-   Support conditional retries based on failure type
-   Allow parameter override at execution time

---

## 2. MULTI-AGENT CONCURRENCY PATTERNS

### A. Git Worktree Orchestration

**Systems:**

1.  **Worktrunk** - Worktree management for parallel AI agents
2.  **Emdash** - Open-source agentic development orchestration with git worktrees
3.  **CCSwarm** - Multi-agent orchestration using Claude Code + git worktrees

**Key Patterns:**

-   Each agent gets isolated git worktree (branch)
-   Prevents merge conflicts, environment contamination
-   Standardized templates for environment setup
-   Progress tracking per worktree
-   Validation scripts verify PR readiness

**Best Practice:**

```
For N agents working in parallel:
1. Create N git worktrees from main branch
2. Each agent gets dedicated worktree + isolation
3. Hooks automate local workflows (setup, validation, cleanup)
4. Central orchestrator tracks progress across all agents
5. Merges happen sequentially (not in parallel) to avoid conflicts
```

**References:**

-   Worktrunk.dev: Git worktree management CLI
-   Mastering Git Worktrees (Medium, 2025)
-   Claude Code Multi-Agent Orchestration Guide

### B. Multi-Agent Task Distribution

**Claude Code Patterns:**

-   **Parallel**: Independent tasks run simultaneously in different worktrees
-   **Sequential**: Tasks with dependencies run one after another
-   **Background**: Low-priority tasks run while main agent works
-   **Sub-Agent Delegation**: Main agent spawns specialized sub-agents

**Work Queue Pattern:**

```
Central Task Queue → Distribute to N agents → Track status → Aggregate results
- Agents pull work from queue
- Status tracked in central store (database/file)
- On agent failure, work returns to queue
- Supports prioritization and dependency tracking
```

---

## 3. DATABASE PROVIDER ABSTRACTION PATTERNS

### A. Rust Trait-Based Abstraction

**Pattern: Multiple Backend Support via Traits**

```rust
// Core abstraction trait
pub trait Database: Send + Sync {
    async fn query<T>(&self, sql: &str) -> Result<T>;
    async fn execute(&self, sql: &str) -> Result<u64>;
    async fn transaction<F>(&self, f: F) -> Result<T>;
}

// Backend implementations
pub struct PostgresProvider {
    pool: PgPool,
}

pub struct SqliteProvider {
    pool: SqlitePool,
}

impl Database for PostgresProvider { ... }
impl Database for SqliteProvider { ... }
```

**Key Challenges:**

-   SQLx compile-time checking difficult across backends
-   SQL dialect differences require abstraction layer
-   Connection pooling differs per backend

**Solutions in Field:**

1.  **Runtime SQL abstraction**: Use query builder (sqlx, Tokio-postgres)
2.  **Macro-based abstraction**: Compile-time checks against all backends
3.  **Feature flags**: Compile different implementations based on feature

**References:**

-   SQLx Docs: Multi-backend support with compile-time checking
-   Medium: "Ports and Adapters in Rust" (2025)
-   Rust Forum: SQLx generic functions over specific backends

### B. Online Schema Migration Pattern (gh-ost)

**GitHub's Approach for Live Database Changes:**

-   Create shadow table with new schema
-   Copy data incrementally from original → shadow
-   Capture ongoing changes (INSERT/UPDATE/DELETE) via binlog
-   Atomic table swap with zero downtime
-   **Key**: Minimal production impact, pausable/resumable

**Implications for Abstraction:**

-   Abstractions must support schema versioning
-   Need capability to support multiple schemas in transition
-   Migration strategy must be database-agnostic where possible

---

## 4. VCS PROVIDER ABSTRACTION PATTERNS

### A. Git Provider Trait Pattern

**Multi-VCS Support (GitHub, GitLab, Gitea, etc.):**

```rust
pub trait VcsProvider {
    async fn create_branch(&self, name: &str) -> Result<Branch>;
    async fn create_worktree(&self, path: &str, branch: &str) -> Result<Worktree>;
    async fn create_pull_request(&self, opts: PrOptions) -> Result<Pr>;
    async fn get_branch_protection(&self, branch: &str) -> Result<Protection>;
    async fn enforce_status_checks(&self, branch: &str) -> Result<()>;
}

pub struct GitHubProvider { ... }
pub struct GitLabProvider { ... }
```

**Worktree Management:**

-   Abstract git worktree operations behind trait
-   Support for cleanup, switching, merging
-   Status tracking per worktree
-   Hooks for pre/post-worktree operations

**Branch Protection API:**

-   GitHub REST API: `/repos/{owner}/{repo}/branches/{branch}/protection`
-   Configurable protections: required checks, dismissible reviews, admin override
-   Automatic enforcement of policies

---

## 5. POLICY ENFORCEMENT PATTERNS

### A. Open Policy Agent (OPA) + Gatekeeper

**Architecture:**

-   Kubernetes admission webhook intercepts resource creation/update/delete
-   Evaluates against Rego policy language
-   Returns allow/deny decision
-   Supports constraint templates for reusable policies

**Pattern for Workflow Systems:**

```
Admission Controller
  ↓
Gatekeeper evaluates policy
  ↓
ConstraintTemplate defines rules
  ↓
Constraint (specific policy instance)
```

**Example: Require labels on deployments**

```rego
violation[{"msg": msg}] {
    resource := input.review.object
    not resource.metadata.labels.team
    msg := "Deployment must have team label"
}
```

**References:**

-   OPA Kubernetes Tutorial: openpolicyagent.org/docs/kubernetes
-   Gatekeeper Integration: GitHub.com/open-policy-agent/gatekeeper

### B. GitHub Branch Protection Rules

**REST API Endpoints:**

-   `GET /repos/{owner}/{repo}/branches/{branch}/protection` - Read rules
-   `PUT /repos/{owner}/{repo}/branches/{branch}/protection` - Set rules
-   Rules enable: required status checks, dismissible reviews, admin override, enforcement

**Key Pattern:**

-   Status checks = required passes before merge
-   Can auto-enforce on PR creation
-   Supports organization-wide ruleset enforcement

### C. Fine-Grained Authorization: OpenFGA/Zanzibar

**Graph-Based Authorization Model:**

```
Relationships: (user, relation, resource)
Example: (alice, editor, document:123)

Check: Can alice view document:123?
→ Query relationship graph → Yes/No
```

**Pattern Benefits:**

-   Scales to complex permission hierarchies
-   Supports delegation: (alice, delegated-to, bob)
-   Can model cross-tenant access
-   Caching strategies for performance

**References:**

-   OpenFGA.dev: Fine-grained authorization system
-   Zanzibar paper: Google's authorization engine
-   SpiceDB: Open-source Zanzibar implementation

---

## 6. EVENT BROADCASTING & MESSAGE QUEUE PATTERNS

### A. Message Queue Selection

| System | Use Case | Strengths | Complexity |
|--------|----------|-----------|-----------|
| **Redis Streams** | Simple pub/sub, message replay | Fast, simple, in-memory | Low |
| **NATS JetStream** | Event streaming, work queues | Lightweight, powerful | Medium |
| **RabbitMQ** | Complex routing, guaranteed delivery | Enterprise-grade, rich features | High |
| **Kafka** | High-throughput event log | Durable, partitioned, scalable | High |

**Selection Criteria:**

-   **Redis**: Dev/staging, low latency requirements, ephemeral events
-   **NATS**: Cloud-native, multiple consumers, ordered delivery needed
-   **RabbitMQ**: Enterprise, complex routing rules, critical deliverability
-   **Kafka**: Data warehouse, audit logs, replay capability essential

### B. Webhook Delivery Pattern

**Industry Standard (Svix):**

```
Retry Schedule with Exponential Backoff:
Attempt 1: Immediate
Attempt 2: 5 seconds
Attempt 3: 5 minutes
Attempt 4: 30 minutes
Attempt 5: 2 hours
Attempt 6: 5 hours
Attempt 7-8: 10 hours each
```

**Key Lessons:**

-   Start fast (transient errors), back off aggressively
-   Mark as failed after all retries exhausted
-   Support manual replay via dashboard/API
-   Track delivery status per endpoint separately
-   Separate queue for failed webhooks (dead-letter queue)

**Success Indicator:**

-   HTTP 2xx (200-299) response = success
-   Timeout = failure (treat like 5xx)
-   Supports idempotency via webhook ID

### C. Event Broadcasting Architecture

```
Event Source
  ↓
Message Queue (Redis/NATS/Kafka)
  ↓
Multiple Subscribers (Workers)
  ↓
Persistent State (if needed)
```

**Patterns:**

-   **Pub/Sub**: Ephemeral events, no persistence
-   **Streams**: Durable log, multiple consumers, replay capability
-   **Job Queues**: Task execution with guaranteed delivery
-   **Webhook**: External service notification

---

## 7. REFERENCE SYSTEMS SUMMARY

### System 1: Temporal.io

-   **Focus**: Durable, deterministic workflow execution
-   **Abstraction**: Workflow/Activity separation
-   **Concurrency**: Sequential workflows, parallel activities
-   **Compensation**: Explicit Saga pattern with compensating activities
-   **GitHub**: GitHub.com/temporalio/temporal

### System 2: Argo Workflows

-   **Focus**: Kubernetes-native DAG orchestration
-   **Abstraction**: Operators, sensors, retry strategies
-   **Concurrency**: DAG-based parallelism with worktree support
-   **Compensation**: Manual recovery patterns, retry overrides
-   **GitHub**: GitHub.com/argoproj/argo-workflows

### System 3: Dagster

-   **Focus**: Data orchestration, asset lineage
-   **Abstraction**: Op/Asset model, executor abstraction
-   **Concurrency**: In-process, multi-process, containerized
-   **Compensation**: Op retries, run retries, backoff policies
-   **GitHub**: GitHub.com/dagster-io/dagster

### System 4: Prefect

-   **Focus**: Dataflow orchestration, dynamic workflows
-   **Abstraction**: Flow/Task model, task runners
-   **Concurrency**: Task runners (Dask, Ray, Kubernetes)
-   **Compensation**: State hooks, transactional workflows
-   **GitHub**: GitHub.com/PrefectHQ/prefect

### System 5: Apache Airflow

-   **Focus**: Batch data processing orchestration
-   **Abstraction**: Operator/Hook/Sensor model
-   **Concurrency**: Multiple executors (Sequential, Celery, Kubernetes)
-   **Compensation**: Plugin/provider pattern for extensibility
-   **GitHub**: GitHub.com/apache/airflow

---

## KEY PATTERNS TO ADOPT

### 1. **Multi-Tier Abstraction**

```
User API Layer
  ↓
Orchestration Layer (workflows, DAGs, jobs)
  ↓
Provider Abstraction Layer (database, VCS, messaging)
  ↓
Implementation Layer (PostgreSQL, GitHub, Redis)
```

### 2. **Operator/Provider Pattern**

-   Define core abstraction as trait/interface
-   Implement per-provider (PostgreSQL, SQLite, GitHub, GitLab)
-   Use feature flags or runtime selection for activation
-   Support multiple implementations simultaneously

### 3. **Deterministic Execution + Event Sourcing**

-   Workflow state = event history
-   Recovery = replay events from last checkpoint
-   Enables fault tolerance without state machine complexity

### 4. **Isolated Concurrency with Git Worktrees**

-   N parallel agents in separate worktrees
-   No branch conflicts, environment contamination
-   Centralized progress tracking
-   Sequential merge (no parallel integration)

### 5. **Exponential Backoff Retry Strategy**

-   Fast initial retries (5s, 5m)
-   Exponential backoff (30m, 2h, 5h, 10h)
-   Configurable at task/workflow level
-   Support manual override (operator intervention)

### 6. **Configurable Policies**

-   Policy-as-code (OPA/Rego for complex rules)
-   API enforcement points (GitHub Branch Protection)
-   Graph-based authorization (OpenFGA for relationships)
-   Runtime override capability for exceptional cases

### 7. **Event Broadcast with Multiple Consumers**

-   Queue system abstracts transport (Redis/NATS/Kafka)
-   Pub/Sub for ephemeral, Streams for durable events
-   Dead-letter queue for failed deliveries
-   Exponential backoff with manual replay support

---

## GOTCHAS & LESSONS LEARNED

### 1. **Deterministic Execution Constraints**

-   ❌ Cannot call external APIs directly from workflow
-   ✅ Must use activity abstraction for non-deterministic operations
-   Impact: Increased code complexity, but enables recovery

### 2. **SQL Abstraction Challenges**

-   ❌ Compile-time query checking across multiple backends impossible
-   ✅ Use query builders or feature-gated compile-time checks
-   Lesson: Accept some runtime validation cost for multi-backend support

### 3. **Git Worktree Isolation Failures**

-   ❌ Don't run parallel agents on same branch
-   ✅ Each agent must have dedicated worktree/branch
-   Lesson: Enforce isolation at orchestration level, not developer level

### 4. **Webhook Retry Storms**

-   ❌ Aggressive immediate retries can overwhelm failing services
-   ✅ Exponential backoff essential; start with jitter
-   Lesson: Svix-style schedule prevents cascading failures

### 5. **Multi-Agent Context Window Depletion**

-   ❌ Running many sub-agents in same session exhausts context
-   ✅ Use separate worktrees/sessions for parallel agents
-   Lesson: Isolation != just git branches; need separate execution contexts

### 6. **Policy Enforcement Ordering**

-   ❌ Policies checked after merge causes rollback risk
-   ✅ Enforce at admission (pre-merge) stage
-   Lesson: Gatekeeper-style admission better than post-check remediation

### 7. **Dead Letter Queue Neglect**

-   ❌ Failed webhooks silently dropped without alerting
-   ✅ Implement DLQ, monitoring, manual replay capability
-   Lesson: Failure visibility essential for production reliability

---

## IMPLEMENTATION ROADMAP

### Phase 1: Core Workflow Abstraction

```
1. Define Workflow trait (sequence of steps)
2. Implement Activity abstraction (non-deterministic work)
3. Add event history for recovery
4. Support retry policies
```

### Phase 2: Provider Abstraction

```
1. Define Database trait (query, execute, transaction)
2. Implement PostgreSQL + SQLite providers
3. Define VcsProvider trait (branch, worktree, PR)
4. Implement GitHub + fallback (git CLI)
```

### Phase 3: Concurrency & Isolation

```
1. Implement git worktree orchestration
2. Add per-agent progress tracking
3. Support parallel agent dispatch
4. Implement sequential merge aggregation
```

### Phase 4: Event Broadcasting

```
1. Abstract message queue (Redis/NATS)
2. Implement webhook delivery with retries
3. Add dead-letter queue handling
4. Support multiple subscribers per event
```

### Phase 5: Policy & Authorization

```
1. Support policy override patterns
2. Integrate GitHub branch protection checks
3. Add fine-grained authorization (graph-based)
4. Implement admission webhook pattern
```

---

## DOCUMENTATION REFERENCES

### Temporal

-   <https://temporal.io/blog/mastering-saga-patterns-for-distributed-transactions-in-microservices>
-   <https://temporal.io/blog/compensating-actions-part-of-a-complete-breakfast-with-sagas>
-   <https://cadenceworkflow.io/docs/concepts/>

### Git Worktree Orchestration

-   <https://worktrunk.dev/>
-   <https://github.com/generalaction/emdash>
-   <https://github.com/coderabbitai/git-worktree-runner>

### Database Abstraction

-   <https://github.com/launchbadge/sqlx>
-   <https://github.com/github/gh-ost>
-   Medium: "Ports and Adapters in Rust" (2025)

### Policy Enforcement

-   <https://openpolicyagent.org/docs/kubernetes/>
-   <https://github.com/open-policy-agent/gatekeeper>
-   <https://docs.github.com/en/rest/branches/branch-protection>
-   <https://openfga.dev/>

### Webhook Delivery

-   <https://docs.svix.com/retries>
-   <https://hookdeck.com/blog/building-reliable-outbound-webhooks>
-   <https://github.com/ConnectEverything/nats-by-example> (JetStream patterns)

### Message Queues

-   <https://redis.io/solutions/message-broker-pattern-for-microservices-interservice-communication>
-   <https://docs.nats.io/>
-   <https://www.rabbitmq.com/>

### Orchestration Systems

-   Dagster: <https://github.com/dagster-io/dagster>
-   Prefect: <https://github.com/PrefectHQ/prefect>
-   Airflow: <https://github.com/apache/airflow>
-   Argo Workflows: <https://github.com/argoproj/argo-workflows>
