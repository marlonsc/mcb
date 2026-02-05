# 🔥 VALIDAÇÃO EXTREMAMENTE AGRESSIVA v0.2.0

## Teste TODOS os Verbs, Sub-Verbs, Tabelas e Dados na Tela

**Data**: 2026-02-05  
**Status**: PLAN para validação final antes de release  
**Confiança Target**: 100% (após todos os testes passarem)  

---

## 📋 ESTRATÉGIA

Isto é uma validação **behavioral E2E**, não só code review. Testa:

1.  **Cada MCP verb** funciona com dados reais
2.  **Cada sub-verb/ação** dentro dos verbs
3.  **Dados aparecem correto na tela** (Admin UI)
4.  **Relacionamentos entre dados** estão corretos
5.  **Fluxos completos end-to-end** funcionam

---

## 🎯 VALIDAÇÃO VERB-BY-VERB (8 MCP Tools)

### ✅ VERB 1: `index` (Indexing Operations)

#### Sub-verbs/Actions

-   `start_index` - Iniciar indexação de codebase
-   `get_status` - Obter status de indexação (percentual, arquivo atual, tempo)
-   `clear_index` - Limpar índice completamente

#### Dados Esperados (Tabela)

```
INDEX_OPERATIONS {
  operation_id: UUID
  status: "pending" | "running" | "completed" | "failed"
  files_scanned: int
  files_indexed: int
  total_files: int
  progress_percent: 0-100
  started_at: DateTime
  completed_at: DateTime?
  error_message: String?
  codebase_path: String
}
```

#### Testes com Dados Reais

**TEST 1.1**: Iniciar indexação

```rust
// Action: POST /mcp/tools/index
// Input: { action: "start_index", codebase_path: "/home/marlonsc/mcb" }
// Expected Response: { operation_id: "op-123", status: "running" }
// Verify: 
//   ✓ operation_id retornado
//   ✓ status é "running" ou "completed"
//   ✓ dados no banco (INDEX_OPERATIONS tabela)
```

**TEST 1.2**: Obter status durante indexação

```rust
// Action: GET /mcp/tools/index?operation_id=op-123
// Expected Response: { 
//   status: "running|completed",
//   files_scanned: 150,
//   files_indexed: 45,
//   progress_percent: 30
// }
// Verify:
//   ✓ progress_percent aumenta a cada poll
//   ✓ files_scanned ≥ files_indexed
//   ✓ timestamps consistentes
```

**TEST 1.3**: Dados aparecem no Admin UI

```
Admin Dashboard → Indexing Tab
Expected Visible:
  ✓ Progress bar (0-100%)
  ✓ "150 files total"
  ✓ "45 files indexed"
  ✓ "Elapsed: 2min 34sec"
  ✓ Status: "running"
```

**TEST 1.4**: Limpar índice

```rust
// Action: DELETE /mcp/tools/index
// Expected: All data wiped from vector store
// Verify:
//   ✓ INDEX_OPERATIONS.status = "cleared"
//   ✓ Vector store is empty
//   ✓ Admin UI shows "No index"
```

#### Relacionamentos

```
INDEX_OPERATIONS → EMBEDDINGS (foreign key: operation_id)
INDEX_OPERATIONS → CODE_CHUNKS (foreign key: operation_id)
```

---

### ✅ VERB 2: `search` (Code Search)

#### Sub-verbs/Actions

-   `semantic_search` - Buscar por semântica (via embeddings)
-   `keyword_search` - Busca literal por keywords
-   `hybrid_search` - Combinar semântico + keyword

#### Dados Esperados (Tabelas)

```
SEARCHES {
  search_id: UUID
  query_text: String
  search_type: "semantic" | "keyword" | "hybrid"
  results_count: int
  execution_time_ms: int
  timestamp: DateTime
  user_context: String?
}

SEARCH_RESULTS {
  result_id: UUID
  search_id: UUID (FK)
  code_chunk_id: UUID
  file_path: String
  line_number: int
  similarity_score: 0.0-1.0
  rank: int
  highlighted_code: String
}
```

#### Testes com Dados Reais

**TEST 2.1**: Busca Semântica

```rust
// Action: POST /mcp/tools/search
// Input: {
//   query: "how do I get configuration from environment variables?",
//   search_type: "semantic"
// }
// Expected: Array de código relacionado a config + env

// Verify in Admin UI → Search Tab:
//   ✓ Query field mostra "how do I get configuration..."
//   ✓ Results appear como tabela com 5-10 linhas
//   ✓ Cada resultado tem: file_path, line, score (0.85), snippet código
//   ✓ Results ordenados por score (desc)
//   ✓ "Execution: 234ms" no footer
```

**TEST 2.2**: Busca por Keyword

```rust
// Action: POST /mcp/tools/search
// Input: {
//   query: "async fn",
//   search_type: "keyword"
// }
// Expected: Todos `async fn` declarations no código

// Verify:
//   ✓ Results mostra EXATO número de matches
//   ✓ Cada resultado highlighted mostra "async fn" em **bold**
//   ✓ Line numbers corretos
```

**TEST 2.3**: Busca Híbrida

```rust
// Action: POST /mcp/tools/search
// Input: {
//   query: "error handling best practice",
//   search_type: "hybrid",
//   weights: { semantic: 0.7, keyword: 0.3 }
// }

// Verify:
//   ✓ Scores combinam semântica + keyword (0.7 * sem_score + 0.3 * kw_score)
//   ✓ Resultados rankeados corretamente (scores desc)
//   ✓ Ambos semântico ("error handling") e keyword ("error", "handling") matches aparecem
```

**TEST 2.4**: Dados Relacionados Aparecem

```
Admin UI → Search Results Table
Columns visíveis:
  ✓ File Path    | /home/marlonsc/mcb/crates/mcb-application/src/error.rs
  ✓ Line         | 42
  ✓ Score        | 0.89 (verde se >0.7, amarelo se 0.5-0.7, vermelho <0.5)
  ✓ Code Snippet | pub enum Error { ... (first 60 chars)
  ✓ Click → full file view
```

#### Relacionamentos

```
SEARCHES → SEARCH_RESULTS (search_id FK)
SEARCH_RESULTS → CODE_CHUNKS (code_chunk_id FK)
CODE_CHUNKS → INDEX_OPERATIONS (operation_id FK)
```

---

### ✅ VERB 3: `validate` (Code Quality & Architecture Validation)

#### Sub-verbs/Actions

-   `lint` - Verificar estilo (clippy, fmt)
-   `architecture` - Verificar ADR compliance
-   `performance` - Verificar performance metrics

#### Dados Esperados

```
VALIDATIONS {
  validation_id: UUID
  validation_type: "lint" | "architecture" | "performance"
  status: "passed" | "failed" | "warnings"
  severity_count: { critical: int, warning: int, info: int }
  timestamp: DateTime
}

VALIDATION_ISSUES {
  issue_id: UUID
  validation_id: UUID (FK)
  file_path: String
  line_number: int
  severity: "critical" | "warning" | "info"
  message: String
  rule_id: String (e.g., "E0001")
  suggestion: String?
}
```

#### Testes com Dados Reais

**TEST 3.1**: Lint Validation

```rust
// Action: POST /mcp/tools/validate
// Input: { action: "lint", codebase_path: "/home/marlonsc/mcb" }

// Verify in Admin UI → Validation Tab → Lint:
//   ✓ "Clippy: 0 warnings" aparece
//   ✓ "Format: 100% compliant" aparece
//   ✓ Status badge: GREEN (passing)
//   ✓ No issues in VALIDATION_ISSUES table
```

**TEST 3.2**: Architecture Validation

```rust
// Action: POST /mcp/tools/validate
// Input: { action: "architecture" }

// Expected: Check ADR-001 até ADR-046
// Verify in Admin UI:
//   ✓ "31 ADRs verified" aparece
//   ✓ Table mostra: ADR ID | Status | Last Checked | Issues
//   ✓ Todos status "✓ COMPLIANT"
//   ✓ 0 architecture violations
```

**TEST 3.3**: Performance Validation

```rust
// Action: POST /mcp/tools/validate
// Input: { action: "performance" }

// Verify:
//   ✓ "Index speed: 1500 files/sec" (metrics)
//   ✓ "Search latency: <100ms (p99)" (actual benchmark)
//   ✓ "Memory: 82 MB (baseline)" (comparison)
//   ✓ Alertas se degradação > 10% vs baseline
```

#### Relacionamentos

```
VALIDATIONS → VALIDATION_ISSUES (validation_id FK)
VALIDATION_ISSUES → CODE_CHUNKS (file_path FK)
```

---

### ✅ VERB 4: `memory` (Session Memory & Context)

#### Sub-verbs/Actions

-   `store` - Guardar item em memória
-   `retrieve` - Recuperar por ID
-   `search` - Buscar por semântica
-   `timeline` - Ver histórico temporal
-   `inject` - Injetar contexto em sessão

#### Dados Esperados

```
MEMORY_ITEMS {
  item_id: UUID
  session_id: UUID
  type: "observation" | "decision" | "pattern" | "error" | "context"
  content: String (up to 64KB)
  tags: String[] (["auth", "v0.2.0", "bug-fix"])
  created_at: DateTime
  updated_at: DateTime
  ttl_seconds: int?
}

MEMORY_TIMELINE {
  timeline_id: UUID
  session_id: UUID
  events: MEMORY_ITEMS[]
  total_events: int
}
```

#### Testes com Dados Reais

**TEST 4.1**: Store Memory Item

```rust
// Action: POST /mcp/tools/memory
// Input: {
//   action: "store",
//   type: "observation",
//   content: "Found bug in cache invalidation logic",
//   tags: ["bug", "cache", "critical"]
// }

// Verify:
//   ✓ item_id retornado
//   ✓ Dados salvos em MEMORY_ITEMS tabela
//   ✓ created_at timestamp existe
//   ✓ Tags indexadas para search
```

**TEST 4.2**: Retrieve Memory Item

```rust
// Action: GET /mcp/tools/memory?item_id=item-123

// Verify:
//   ✓ Retorna: { type, content, tags, created_at, updated_at }
//   ✓ Conteúdo exatamente como foi armazenado
```

**TEST 4.3**: Search Memory

```rust
// Action: POST /mcp/tools/memory
// Input: { action: "search", query: "cache bug fix" }

// Verify in Admin UI → Memory Tab:
//   ✓ Results show items matching query semantically
//   ✓ Table columns: Created | Type | Tags | Snippet
//   ✓ Pode filtrar por tag ("cache", "bug")
//   ✓ Pode filtrar por type ("observation", "decision")
```

**TEST 4.4**: Timeline View

```rust
// Action: GET /mcp/tools/memory?action=timeline

// Verify:
//   ✓ Events ordenados por created_at (desc)
//   ✓ Timeline visual com dots + lines
//   ✓ Cada dot clickable → mostra full item
//   ✓ "42 events stored" aparece no header
```

**TEST 4.5**: Inject into Session

```rust
// Action: POST /mcp/tools/memory
// Input: { action: "inject", item_ids: ["item-1", "item-2"] }

// Verify:
//   ✓ Items injetados para context do session
//   ✓ Aparecem em próximas chamadas (context + relevance)
//   ✓ Session receives injected items
```

#### Relacionamentos

```
MEMORY_ITEMS → MEMORY_TIMELINE (session_id FK)
MEMORY_ITEMS → SESSION (session_id FK)
MEMORY_ITEMS (tags) → TAG (many-to-many)
```

---

### ✅ VERB 5: `session` (Session Lifecycle)

#### Sub-verbs/Actions

-   `create` - Nova sessão
-   `get` - Recuperar sessão por ID
-   `list` - Listar todas as sessões
-   `summarize` - Gerar summary de sessão
-   `close` - Finalizar sessão

#### Dados Esperados

```
SESSIONS {
  session_id: UUID
  start_time: DateTime
  end_time: DateTime?
  status: "active" | "completed" | "error"
  user_id: String?
  context: JSON
  messages_count: int
  memory_items_count: int
  last_activity: DateTime
}

SESSION_METADATA {
  session_id: UUID (FK)
  title: String?
  description: String?
  project: String?
  branch: String?
  tags: String[]
}
```

#### Testes com Dados Reais

**TEST 5.1**: Create Session

```rust
// Action: POST /mcp/tools/session
// Input: { action: "create", project: "MCB", branch: "release/v0.2.0" }

// Verify in Admin UI → Sessions Tab:
//   ✓ Nova sessão aparece em lista
//   ✓ start_time ≈ agora
//   ✓ status = "active"
//   ✓ messages_count = 0
```

**TEST 5.2**: Get Session Details

```rust
// Action: GET /mcp/tools/session?session_id=sess-123

// Verify:
//   ✓ Retorna session com: start_time, user_id, context, messages_count
//   ✓ last_activity timestamp recente (< 5 min atrás)
//   ✓ memory_items_count correto (relacionado a MEMORY_ITEMS)
```

**TEST 5.3**: List Sessions

```rust
// Action: GET /mcp/tools/session?action=list

// Verify in Admin UI:
//   ✓ Table com todas as sessions
//   ✓ Columns: Created | Status | Project | Messages | Last Activity
//   ✓ Pode sort por Created (newest first)
//   ✓ Pode filter por Status (active/completed)
//   ✓ Pagination se >20 sessions
```

**TEST 5.4**: Summarize Session

```rust
// Action: POST /mcp/tools/session
// Input: { action: "summarize", session_id: "sess-123" }

// Verify:
//   ✓ Summary retorna: {
//       title: "Phase 8 Planning",
//       summary: "Estimated 9 hours for FSM + Scout implementation...",
//       key_decisions: ["Start with ADR-035", "Parallelize ADR-034"],
//       memory_extracted: 5
//     }
```

**TEST 5.5**: Close Session

```rust
// Action: POST /mcp/tools/session
// Input: { action: "close", session_id: "sess-123" }

// Verify:
//   ✓ status changes from "active" → "completed"
//   ✓ end_time set to now
//   ✓ Session still queryable (not deleted)
//   ✓ Admin UI shows "Completed 2min ago"
```

#### Relacionamentos

```
SESSIONS → SESSION_METADATA (session_id FK)
SESSIONS → MEMORY_ITEMS (session_id FK)
SESSIONS → TOOL_EXECUTIONS (session_id FK)
```

---

### ✅ VERB 6: `agent` (Agent Activity & Tracking)

#### Sub-verbs/Actions

-   `log` - Log atividade do agent
-   `status` - Obter status current do agent
-   `history` - Histórico de atividades

#### Dados Esperados

```
AGENT_LOGS {
  log_id: UUID
  agent_id: String (e.g., "sisyphus-001")
  agent_type: String (e.g., "sisyphus", "oracle", "explore")
  action: String
  status: "started" | "running" | "completed" | "failed"
  result_summary: String?
  metadata: JSON
  timestamp: DateTime
  session_id: UUID (FK)
}
```

#### Testes com Dados Reais

**TEST 6.1**: Log Agent Activity

```rust
// Action: POST /mcp/tools/agent
// Input: {
//   action: "log",
//   agent_id: "sisyphus-001",
//   agent_type: "sisyphus",
//   activity: "Completed Phase 8 planning",
//   status: "completed",
//   metadata: { tasks: 12, duration_sec: 300 }
// }

// Verify:
//   ✓ Log entrada criada em AGENT_LOGS
//   ✓ Timestamp ≈ agora
//   ✓ metadata armazenado como JSON
```

**TEST 6.2**: Get Agent Status

```rust
// Action: GET /mcp/tools/agent?agent_id=sisyphus-001

// Verify in Admin UI → Agents Tab:
//   ✓ "sisyphus-001: Running"
//   ✓ "Last activity: 2min ago"
//   ✓ "Tasks completed: 12"
//   ✓ "Current task: Phase 8 analysis"
```

**TEST 6.3**: View Agent History

```rust
// Action: GET /mcp/tools/agent?action=history&agent_id=sisyphus-001

// Verify:
//   ✓ Timeline de todas as atividades
//   ✓ Ordenadas por timestamp (newest first)
//   ✓ Cada entry mostra: timestamp, action, status, duration
//   ✓ Pode filtrar por status (completed/failed)
```

#### Relacionamentos

```
AGENT_LOGS → SESSION (session_id FK)
AGENT_LOGS (agent_id) → AGENT (many-to-many tracking)
```

---

### ✅ VERB 7: `project` (Project Management) ⚠️ PARTIAL

#### Sub-verbs/Actions

-   `create` - Criar novo projeto
-   `update` - Atualizar projeto
-   `list` - Listar projetos
-   `add_dependency` - Adicionar dependência

#### ⚠️ **CRITICAL ISSUE**: Todas essas ações retornam "not implemented yet"

```
PROJECT_ISSUES {
  project_id: UUID
  handler_status: "stub_not_implemented" ✗
  blocking_release: YES
  impact: "HIGH"
  fix_effort: ~2 hours
}
```

#### Opções

**OPÇÃO A: Remover de v0.2.0**

```
Remove project verb completamente
Update MCP schema (7 tools instead of 8)
Docs note: "Project management planned for v0.3.0"
```

**OPÇÃO B: Implementar agora**

```
1. Implement full project handler (1.5h)
2. Create PROJECT, PROJECT_DEPENDENCY tables (30min)
3. Add E2E tests (30min)
4. Document (15min)
Total: ~2.5 hours
```

---

### ✅ VERB 8: `vcs` (Version Control Operations)

#### Sub-verbs/Actions

-   `commit` - Commit com mensagem
-   `branch` - Criar branch
-   `log` - Ver commit history
-   `status` - Git status atual
-   `diff` - Ver diferenças

#### Dados Esperados

```
VCS_OPERATIONS {
  operation_id: UUID
  operation_type: "commit" | "branch" | "log" | "status" | "diff"
  repository_path: String
  result: String (git output)
  timestamp: DateTime
  status: "success" | "failed"
}
```

#### Testes com Dados Reais

**TEST 8.1**: Git Status

```rust
// Action: POST /mcp/tools/vcs
// Input: { action: "status", repo_path: "/home/marlonsc/mcb" }

// Verify in Admin UI → VCS Tab:
//   ✓ "Branch: release/v0.2.0" aparece
//   ✓ "On branch release/v0.2.0"
//   ✓ "2 files modified" (ou número correto)
//   ✓ Files list mostra arquivos alterados
```

**TEST 8.2**: Git Log

```rust
// Action: GET /mcp/tools/vcs?action=log&limit=5

// Verify:
//   ✓ Últimos 5 commits aparecem
//   ✓ Columns: Commit Hash | Author | Message | Date
//   ✓ Hashes são links (clickable → diff view)
//   ✓ Dates em human-readable format ("2min ago")
```

**TEST 8.3**: Commit Operation

```rust
// Action: POST /mcp/tools/vcs
// Input: {
//   action: "commit",
//   message: "Fix admin API startup and project handler",
//   files: ["AUDIT_FINAL_v0.2.0.md", "src/init.rs"]
// }

// Verify:
//   ✓ Commit ID retornado
//   ✓ Git log updated
//   ✓ Admin UI → VCS Tab mostra novo commit no topo
```

#### Relacionamentos

```
VCS_OPERATIONS → SESSION (session_id FK)
VCS_OPERATIONS → CODE_CHUNKS (através do file path)
```

---

## 🎬 PLANO DE EXECUÇÃO

### FASE 1: Setup + Admin UI Wiring (1 hora)

**TEST ENV SETUP**:

```bash
# 1. Build projeto
make build-release

# 2. Start admin server (if not running)
# Fix: Wire AdminApi into init.rs startup

# 3. Open Admin UI
open http://localhost:8080/admin

# 4. Verify 5 screens visíveis:
#    ✓ Dashboard
#    ✓ Configuration
#    ✓ Health Status
#    ✓ Indexing
#    ✓ Browse
```

---

### FASE 2: Testar VERB 1-2 (Index + Search) - 2 horas

```bash
# TEST 1: Index operations
cd /home/marlonsc/mcb
# POST /mcp/tools/index with real codebase
# Watch Admin UI → Indexing tab
# Verify files_scanned increases

# TEST 2: Search operations
# POST /mcp/tools/search with semantic query
# Verify results appear in Admin UI
# Check scores > 0.7 for relevant results
```

---

### FASE 3: Testar VERB 3-5 (Validate + Memory + Session) - 2 horas

```bash
# TEST 3: Validation
# POST /mcp/tools/validate (lint, architecture, performance)
# Check Admin UI → Validation tab

# TEST 4: Memory
# Store 5 items, search, timeline
# Verify all operations work

# TEST 5: Sessions
# Create, list, summarize sessions
# Verify relationships correct
```

---

### FASE 4: Testar VERB 6, 7, 8 (Agents + Project + VCS) - 2 horas

```bash
# TEST 6: Agent tracking
# Log activities, check history

# TEST 7: Project management
# DECISION: Fix or remove?

# TEST 8: VCS operations
# Status, log, commit with test data
```

---

### FASE 5: E2E Workflows (1.5 horas)

**WORKFLOW 1**: Index → Search → Find Result → View File

```
1. Index codebase
2. Search for "how do I handle errors"
3. Click result → file view in Admin UI
4. Verify line 42 highlighted + code correct
```

**WORKFLOW 2**: Session → Memory → Inject → Decision

```
1. Create session
2. Store observation + decision items
3. Search memory by tag
4. Inject into context
5. Verify in session details
```

**WORKFLOW 3**: Validate → Check Issues → VCS Commit

```
1. Run validation
2. See issues found
3. Commit fix via VCS verb
4. Re-validate
5. Verify 0 issues now
```

---

### FASE 6: Final Checks (1 hora)

```bash
# All tests passing?
make test

# All clippy/fmt clean?
make lint

# Architecture valid?
make validate

# Admin UI responsive?
# Dark mode works?
# All tables render correct?
```

---

## 📊 RESULTADO ESPERADO

Após completar TODAS as fases (9 horas total):

| Componente | Status | Evidência |
|-----------|--------|-----------|
| **Admin UI** | ✅ | 5 screens funcionando com dados reais |
| **Index Verb** | ✅ | Dados aparecem na tela, progress funciona |
| **Search Verb** | ✅ | Semantic + Keyword + Hybrid testados |
| **Validate Verb** | ✅ | 0 warnings, 31 ADRs compliant |
| **Memory Verb** | ✅ | Store/Search/Timeline/Inject funcionam |
| **Session Verb** | ✅ | Create/List/Summarize/Close funcionam |
| **Agent Verb** | ✅ | Tracking de atividades funciona |
| **Project Verb** | ⚠️ | DECISION: Fixed or Removed |
| **VCS Verb** | ✅ | Git operations funcionam |
| **Relacionamentos** | ✅ | Todas as FKs apontam correto |
| **E2E Workflows** | ✅ | 3 workflows completos testados |
| **Confiança** | 100% | Ready for production release |

---

## 🎯 DECISÕES NECESSÁRIAS ANTES DE COMEÇAR

**Pergunta 1**: Você quer começar AGORA com este plano de 9 horas?

**Pergunta 2**: VERB 7 (Project) - Corrigir ou Remover de v0.2.0?

-   A) Remover completamente (mais rápido)
-   B) Implementar agora (mais completo)

**Pergunta 3**: Qual vai ser a primeira fase a executar?

-   Fase 1 (Setup) → Fase 2 (Index+Search) → ...?

---

**Status**: 🔴 AWAITING YOUR DECISION  
**Effort Estimate**: 9-12 horas (com Fase 7 decision)  
**Timeline**: ~1-1.5 dias de trabalho intenso  
**Confidence After**: 100% production-ready  
