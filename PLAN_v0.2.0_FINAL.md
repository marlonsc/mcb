# PLANO FINAL: v0.2.0 Release (Project Handler + Admin API + E2E Validation)

**Status**: PLAN READY FOR EXECUTION  
**Criado**: 2026-02-05  
**Estimate**: 6-8 horas  
**Goal**: v0.2.0 release-ready com Project handler implementado + Admin API rodando + E2E validation passing

---

## 🎯 BLOCKERS IDENTIFICADOS

| Blocker | Impact | Fix Effort | P |
|---------|--------|-----------|---|
| Project handler é HashMap in-memory (sem BD) | 🔴 CRITICAL | 3-4h | 0 |
| AdminApi não wired em init.rs | 🔴 CRITICAL | 30 min | 0 |
| 6/8 MCP tools SEM testes | 🟡 HIGH | 3-4h | 0 |
| Project handler: 0 tests | 🟡 HIGH | 1-2h | 0 |

**Total Blockers**: 6-8 horas

---

## 📋 PLANO ESTRUTURADO (4 WAVES)

### WAVE 1: Admin API Wiring (30 min) — CRÍTICA

**Objetivo**: AdminApi server starting na porta 9090

**Tasks**:

1.  **Task 1.1**: Wire AdminApi em `crates/mcb-server/src/init.rs`
   -   Arquivo: `init.rs` linha 277-302 (criar_MCP_server)
   -   Ação: Adicionar `AppContext` field `admin_api_handle: Arc<AdminApi>`
   -   Ação: Spawn `tokio::spawn(admin_api_handle.start())` antes de return
   -   Verify: Admin server listening on port 9090
   -   Done: `/admin` endpoint responde 200

2.  **Task 1.2**: Add integration test for Admin API startup
   -   Arquivo: `crates/mcb-server/tests/handlers/admin_integration.rs` (NEW)
   -   Ação: Test GET /admin/provider/current
   -   Ação: Test POST /admin/provider/switch
   -   Verify: Endpoints retornam JSON válido
   -   Done: 2 testes passando

**Beads Issues**:

-   mcb-1wa: Wire AdminApi in init.rs (30 min, P0)
-   mcb-2wa: Add Admin API integration test (20 min, P0)

---

### WAVE 2: Project Handler DB Persistence (3-4 hours) — PARALELO COM WAVE 1

**Objetivo**: ProjectRepository com SQLite + tests

**Architecture Alignment** (Clean Architecture v0.2.0):

```
mcb-domain/src/
├─ ports/
│  └─ repositories/
│     └─ project_repository.rs (PORT TRAIT)
│
mcb-providers/src/
├─ repositories/
│  └─ sqlite_project_repository.rs (IMPL)
│  └─ postgres_project_repository.rs (IMPL)
│
mcb-infrastructure/src/
├─ di/
│  └─ catalog.rs (wiring)
└─ handles/
   └─ project_repository_handle.rs (Arc<RwLock<>>)
```

**Tasks**:

1.  **Task 2.1**: Create ProjectRepository PORT in mcb-domain
   -   Arquivo: `crates/mcb-domain/src/ports/repositories/project_repository.rs`
   -   Entidade: Project, ProjectPhase, ProjectIssue, ProjectDependency (usar entities já existentes)
   -   Métodos (trait):

     ```rust
     async fn create_project(&self, project: &Project) -> Result<()>;
     async fn get_project(&self, id: &str) -> Result<Option<Project>>;
     async fn list_projects(&self) -> Result<Vec<Project>>;
     async fn create_phase(&self, phase: &ProjectPhase) -> Result<()>;
     async fn update_phase(&self, phase: &ProjectPhase) -> Result<()>;
     async fn list_phases(&self, project_id: &str) -> Result<Vec<ProjectPhase>>;
     async fn create_issue(&self, issue: &ProjectIssue) -> Result<()>;
     async fn update_issue(&self, issue: &ProjectIssue) -> Result<()>;
     async fn list_issues(&self, project_id: &str, filters: IssueFilter) -> Result<Vec<ProjectIssue>>;
     async fn add_dependency(&self, dep: &ProjectDependency) -> Result<()>;
     async fn list_dependencies(&self, project_id: &str) -> Result<Vec<ProjectDependency>>;
     ```

   -   Verify: Trait compila sem erros
   -   Done: PORT definida em mcb-domain

2.  **Task 2.2**: Implement SQLite ProjectRepository in mcb-providers
   -   Arquivo: `crates/mcb-providers/src/repositories/sqlite_project_repository.rs`
   -   Schema: Usar `schema::project::{projects, collections, observations, session_summaries, file_hashes}` já definido
   -   DDL: Usar `ProjectSchema::definition().generate_ddl()` (implementar para SQLite)
   -   Métodos: Implementar trait inteiro com queries
   -   Verify: `cargo build --release` sem erros
   -   Done: 20+ SQLite queries implementadas

3.  **Task 2.3**: Add #[linkme::distributed_slice] registration (Provider Registry)
   -   Arquivo: `crates/mcb-application/src/registry.rs`
   -   Ação: Adicionar nova distributed_slice `PROJECT_REPOSITORIES`
   -   Arquivo: `crates/mcb-providers/src/repositories/mod.rs`
   -   Ação: Registrar `SqliteProjectRepository` na slice
   -   Verify: `cargo build` sucesso
   -   Done: Provider auto-registered

4.  **Task 2.4**: Wire ProjectRepositoryHandle em DI Catalog
   -   Arquivo: `crates/mcb-infrastructure/src/di/catalog.rs`
   -   Ação: Adicionar `ProjectRepositoryHandle` com `RwLock<Arc<dyn ProjectRepository>>`
   -   Ação: Atualizar `AppContext` para incluir `project_repo_handle`
   -   Verify: DI container builds
   -   Done: ProjectRepository acessível via AppContext

5.  **Task 2.5**: Implement PostgreSQL ProjectRepository (bonus, se tempo permitir)
   -   Arquivo: `crates/mcb-providers/src/repositories/postgres_project_repository.rs`
   -   Replicar SQLite impl com sqlx::PgPool
   -   Register na slice

**Beads Issues**:

-   mcb-3wa: Create ProjectRepository PORT trait (45 min, P0)
-   mcb-4wa: Implement SQLite ProjectRepository (90 min, P0)
-   mcb-5wa: Add linkme registration for ProjectRepository (20 min, P0)
-   mcb-6wa: Wire ProjectRepositoryHandle in DI catalog (30 min, P0)
-   mcb-7wa: Implement PostgreSQL ProjectRepository (OPTIONAL, 90 min)

---

### WAVE 3: Update Project Handler + Tests (1.5-2 hours) — PARALELO COM WAVE 2

**Objetivo**: ProjectHandler usa ProjectRepository real + 30+ tests

**Tasks**:

1.  **Task 3.1**: Refactor ProjectHandler para usar ProjectRepositoryHandle
   -   Arquivo: `crates/mcb-server/src/handlers/consolidated/project.rs`
   -   Ação: Remover `ProjectStore` (HashMap)
   -   Ação: Injetar `Arc<ProjectRepositoryHandle>` via AppContext
   -   Métodos: Atualizar `create()`, `update()`, `list_resources()`, `add_dependency()` para usar repository
   -   Verify: Handler compila sem errors
   -   Done: 410 linhas → 350 linhas (menos boilerplate in-memory)

2.  **Task 3.2**: Add ProjectRepository unit tests
   -   Arquivo: `crates/mcb-domain/tests/unit/project_repository_tests.rs` (NEW)
   -   Testes:

     ```rust
     #[tokio::test] async fn test_create_project { }
     #[tokio::test] async fn test_get_project_by_id { }
     #[tokio::test] async fn test_list_projects { }
     #[tokio::test] async fn test_create_phase { }
     #[tokio::test] async fn test_update_phase { }
     #[tokio::test] async fn test_list_phases_by_project { }
     #[tokio::test] async fn test_create_issue { }
     #[tokio::test] async fn test_filter_issues_by_status { }
     #[tokio::test] async fn test_add_dependency { }
     #[tokio::test] async fn test_list_dependencies { }
     #[tokio::test] async fn test_unique_constraint_project_name { }
     #[tokio::test] async fn test_foreign_key_projects { }
     ```

   -   Verify: 12+ testes passing
   -   Done: 100% coverage da ProjectRepository

3.  **Task 3.3**: Add ProjectHandler integration tests
   -   Arquivo: `crates/mcb-server/tests/handlers/project_handler_integration.rs` (NEW)
   -   Testes E2E:

     ```rust
     #[tokio::test] async fn test_create_project_verb { }
     #[tokio::test] async fn test_update_project_verb { }
     #[tokio::test] async fn test_list_phases { }
     #[tokio::test] async fn test_add_issue { }
     #[tokio::test] async fn test_add_dependency { }
     #[tokio::test] async fn test_filter_issues { }
     ```

   -   Verify: 6+ testes passing
   -   Done: E2E project flows tested

**Beads Issues**:

-   mcb-8wa: Refactor ProjectHandler to use repository (45 min, P0)
-   mcb-9wa: Add ProjectRepository unit tests (60 min, P0)
-   mcb-10wa: Add ProjectHandler integration tests (45 min, P0)

---

### WAVE 4: E2E Validation + Final Checks (1-1.5 hours) — SEQUENTIAL

**Objetivo**: v0.2.0 passar ALL quality gates + Admin UI showing data real

**Tasks**:

1.  **Task 4.1**: Wire ProjectHandler em MCP server (if not already)
   -   Arquivo: `crates/mcb-server/src/handlers/mod.rs`
   -   Verify: ProjectHandler is registered in MCP tool registry
   -   Done: Project verb accessible via MCP

2.  **Task 4.2**: E2E Test: All 8 MCP Verbs
   -   **Test**: POST /MCP/tools/index (start indexing)
   -   **Test**: GET /MCP/tools/search (semantic search)
   -   **Test**: POST /MCP/tools/validate (lint + architecture)
   -   **Test**: POST /MCP/tools/memory (store + retrieve)
   -   **Test**: GET /MCP/tools/session (list sessions)
   -   **Test**: POST /MCP/tools/agent (log activity)
   -   **Test**: POST /MCP/tools/project (create + list) ← NEW
   -   **Test**: GET /MCP/tools/vcs (git status)
   -   Verify: 8/8 verbs respond 200
   -   Done: All verbs working

3.  **Task 4.3**: Admin UI: Project data appearing
   -   Browser: Open <http://localhost:9090/admin>
   -   Click: "Projects" tab
   -   Verify: Projects list showing
   -   Verify: Phases sub-table showing
   -   Verify: Issues sub-table showing with priority colors
   -   Done: Admin UI rendering data real

4.  **Task 4.4**: Run `make quality`
   -   `make fmt` → 0 warnings
   -   `make lint` → 0 clippy errors
   -   `make test` → 100+ tests passing (59 handlers + 30 project + 6 E2E)
   -   `make validate` → 0 architecture violations
   -   `make docs-lint` → Markdown clean
   -   Verify: All checks ✅
   -   Done: Quality gate passed

5.  **Task 4.5**: Prepare v0.2.0 Release
   -   Arquivo: CHANGELOG.md
   -   Ação: Add v0.2.0 entry with features (Project handler, Admin API, E2E validation)
   -   Arquivo: docs/operations/CHANGELOG.md
   -   Ação: Add v0.2.0 summary
   -   Verify: CHANGELOG updated
   -   Done: Release notes ready

**Beads Issues**:

-   mcb-11wa: Wire ProjectHandler in MCP registry (10 min, P0)
-   mcb-12wa: E2E test all 8 MCP verbs (30 min, P0)
-   mcb-13wa: Admin UI project data validation (20 min, P0)
-   mcb-14wa: Run make quality checks (15 min, P0)
-   mcb-15wa: Prepare v0.2.0 release notes (10 min, P0)

---

## 📊 RESUMO DE BEADS ISSUES

**Total Issues**: 15 (P0 todas)

| Wave | Issues | Effort | Status |
|------|--------|--------|--------|
| Wave 1 (Admin API) | mcb-1wa, mcb-2wa | 50 min | READY |
| Wave 2 (ProjectRepo) | mcb-3wa...mcb-7wa | 245 min + opt | READY |
| Wave 3 (Handler+Tests) | mcb-8wa...mcb-10wa | 150 min | READY |
| Wave 4 (E2E+Release) | mcb-11wa...mcb-15wa | 85 min | READY |

**TOTAL ESTIMATE**: 530 min = **~8.5 horas**

---

## ✅ QUALITY GATES FINAL

**Before v0.2.0 tag**:

-   [ ] Admin API running (port 9090)
-   [ ] ProjectRepository wired
-   [ ] All 8 MCP verbs working
-   [ ] 30+ project tests passing
-   [ ] 59 handler tests passing
-   [ ] Admin UI showing project data
-   [ ] `make quality` all green
-   [ ] CHANGELOG updated
-   [ ] Beads issues all closed

---

## 🚀 PRÓXIMOS PASSOS

**Ordem de Execução**:

1.  Crie Beads issues (executar bash commands abaixo)
2.  Execute `Wave 1 + Wave 2 paralelo + Wave 3 paralelo` (parallel work)
3.  Execute `Wave 4` (final checks + release)
4.  Tag v0.2.0 e push

---

## 🔗 BEADS CREATION COMMANDS

```bash
# Wave 1: Admin API
bd create "Wire AdminApi in init.rs startup" -t task -p 0
bd create "Add integration test for Admin API" -t task -p 0

# Wave 2: ProjectRepository
bd create "Create ProjectRepository PORT trait" -t task -p 0
bd create "Implement SQLite ProjectRepository" -t task -p 0
bd create "Add linkme registration for ProjectRepository" -t task -p 0
bd create "Wire ProjectRepositoryHandle in DI catalog" -t task -p 0
bd create "Implement PostgreSQL ProjectRepository (optional)" -t task -p 0

# Wave 3: Handler + Tests
bd create "Refactor ProjectHandler to use repository" -t task -p 0
bd create "Add ProjectRepository unit tests" -t task -p 0
bd create "Add ProjectHandler integration tests" -t task -p 0

# Wave 4: E2E + Release
bd create "Wire ProjectHandler in MCP registry" -t task -p 0
bd create "E2E test all 8 MCP verbs" -t task -p 0
bd create "Admin UI project data validation" -t task -p 0
bd create "Run make quality checks" -t task -p 0
bd create "Prepare v0.2.0 release notes" -t task -p 0
```

---

**Status**: READY FOR EXECUTION  
**Blocker**: NONE (todas as correções identificadas estão no plano)  
**Confidence**: 95% (após testes passing)  
