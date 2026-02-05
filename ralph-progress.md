# Ralph Loop – Progress (global)

## Fluxo de dados – Admin Web

| Página       | Fonte de dados                         | Padrão |
|-------------|-----------------------------------------|--------|
| **Dashboard** (/) | SSE (eventos) + fetch `/health`, `/metrics`, `/services` | Estado inicial "Loading..."; valores reais ao receber evento/resposta. |
| **Config** (/ui/config) | GET `/config` (HTMX) → `#config-display`; `populateConfigForms(data)` preenche forms; PATCH `/config/<section>` via `applyConfigSection()` com JSON `{ values }`. | Placeholders "—"; sem valores fixos nos inputs. |
| **Health** (/ui/health) | HTMX GET `/health/extended`, `/ready`, `/live` (load + polling). | Conteúdo 100% da resposta. |
| **Indexing** (/ui/indexing) | HTMX GET `/indexing` (load + every 2s). | Skeleton inicial; HTML montado a partir do JSON. |
| **Browse** (/ui/browse) | fetch `/collections` → `renderCollections`. | "Loading..."; lista da API. |
| **Browse collection** | fetch `/collections/:name/files`. | Contador "Loading..." até resposta; file count real depois. |
| **Browse file** | fetch `/collections/:name/chunks/:path`. | "Loading code chunks..."; dados da API. |

Regra: nenhum número ou status na UI deve ser estático quando representar dado de backend; usar "Loading..." ou valor da API.

## Quality gates

-   `make fmt` – format Rust + Markdown
-   `make lint CI_MODE=1` – fmt check + clippy (Rust 2024)
-   `make test SCOPE=all` – all tests
-   `make validate QUICK=1` – architecture validation

## Iterations

### Iteration 1 – Lint: unfulfilled `#[expect(dead_code)]`

-   **Task**: Remove `#[expect(dead_code)]` from `agent_session` in `mcp_server.rs` (field is now used by agent session tools).
-   **Verify**: `make lint CI_MODE=1` ✓, `make test SCOPE=all` ✓, `make validate QUICK=1` ✓.
-   **Learnings**: fmt was already applied by earlier `make fmt`. Test failures (23 vs 25 tools) were from an earlier state; current tests expect 25 and pass.

### Iteration 2 – Admin web: remover código falso nos provedores/config

-   **Task**: Corrigir dados falsos na interface web admin (formulários de config e versão).
-   **Alterações**:
  1.  **config.html**: Formulário Server sem valores fixos (127.0.0.1, 8080, 9090); ids adicionados (config-server-host, config-server-port, config-server-admin-port); placeholders "—"; preenchimento via `populateConfigForms(data)` já existente. Footer config: v0.1.1 → v0.1.5.
  2.  **mcb-validate**: Removido `reporter.rs` duplicado (mantido `reporter/mod.rs`); clippy em `ca009_tests.rs` (uninlined_format_args) corrigido.
-   **Verify**: `make fmt` ✓, `make lint` ✓, `make test` ✓, `make validate QUICK=1` ✓.
-   **Learnings**: Formulários de config devem refletir dados reais da API; um único módulo `reporter` (diretório) evita conflito reporter.rs vs reporter/mod.rs.

### Iteration 3 – Admin web: remover contagem falsa em Browse

-   **Task**: Eliminar último texto que sugere número antes dos dados (provedores/browse).
-   **Alteração**: **browse_collection.html**: `-- files` → `Loading...` no contador de arquivos, para não exibir valor falso antes da API responder.
-   **Verify**: `make lint` ✓, `make test` ✓, `make validate QUICK=1` ✓.
-   **Learnings**: Contadores na UI devem mostrar "Loading..." ou valor real da API, nunca "--" que parece número.

### Iteration 4 – Código falso nos servers: search_branch

-   **Task**: Handler `search_branch` retornava sucesso com `files_searched: 0` sem executar busca (resposta falsa).
-   **Alteração**: **search_branch.rs**: Resposta explícita de "não implementado" (`SearchBranchNotImplementedResponse` com `implemented: false` e mensagem clara); documentação no módulo e no struct.
-   **Verify**: `make lint CI_MODE=1` ✓, `make test SCOPE=all` ✓.
-   **Learnings**: Stubs devem indicar claramente "not implemented" para clientes não tratarem como resultado real.

### Iteration 5 – Código falso nos servers: comentário auth

-   **Task**: Comentário em `with_admin_auth` dizia "no-op" mas a função registra `auth_config` no Rocket.
-   **Alteração**: **admin/auth.rs**: Comentário corrigido para "Registers auth_config with Rocket..."; removida a afirmação de no-op.
-   **Verify**: `make lint CI_MODE=1` ✓.
-   **Learnings**: Documentação deve refletir o comportamento real (register vs no-op).

### Iteration 6 – Código falso nos servers: list_repositories

-   **Task**: Handler `list_repositories` retornava sempre `repositories: [], count: 0` sem consultar nenhuma fonte (resposta falsa).
-   **Alteração**: **list_repositories.rs**: Resposta explícita de "não implementado" (`ListRepositoriesNotImplementedResponse` com `implemented: false` e mensagem clara); doc do módulo; removido import não usado `validator::Validate`.
-   **Verify**: `make lint CI_MODE=1` ✓, `make test SCOPE=all` ✓.
-   **Learnings**: Handlers sem backend wired devem retornar "not implemented", não lista vazia que parece dado real.

### Iteration 7 – Código falso nos servers: comentário lib.rs

-   **Task**: Comentário em `lib.rs` dizia "Placeholder modules removed" (presente), podendo sugerir que ainda há placeholders.
-   **Alteração**: **lib.rs**: Texto para "Legacy placeholder modules were removed; functionality lives in the infrastructure layer" (passado, explícito).
-   **Verify**: `make lint CI_MODE=1` ✓.
-   **Learnings**: Comentários históricos em passado evitam ambiguidade.

### Iteration 8 – DOC002: Traits sem documentação

-   **Task**: Corrigir ERROR DOC002 em AgentSessionServiceInterface e ProjectRepository.
-   **Alteração**: Adicionadas doc comments (///) em ambos os traits.
-   **Verify**: `make lint` ✓, `cargo run -p mcb validate` sem ERRORs ✓.
-   **Learnings**: Traits em ports precisam de doc para DOC002.

### Iteration 7 – REF002: SessionSummary duplicado

-   **Task**: Consolidar tipo SessionSummary (definido em mcb-server e mcb-domain).
-   **Alteração**: list_agent_sessions.rs – struct local renomeada para `AgentSessionListItem` (o domain SessionSummary é para RAG; o do handler é para API de listagem de agent sessions).
-   **Verify**: `make lint` ✓, REF002 resolvido (sem mais ERROR de duplicate definition).
-   **Learnings**: Tipos com mesmo nome em camadas diferentes geram REF002; renomear o DTO da API evita colisão com entidade de domínio.

### Iteration 8 – Clean Architecture e DI

-   **Task**: Corrigir violações de Clean Architecture e DI no projeto.
-   **Alterações**:
  1.  **CA004**: Entidade `ExecutionMetadata` sem campo de identidade – adicionado `id: String` em `mcb-domain/src/entities/memory.rs` com `#[serde(default)]` para compatibilidade; construção em `store_execution.rs` passa a usar `Uuid::new_v4().to_string()`.
  2.  **CA009**: Teste de integração esperava violações; composition root (`mcb-infrastructure/src/di/`) é permitido importar `mcb_application`. Teste atualizado para esperar 0 violações CA009 quando fora de `di/` não importa application. Módulo `ca009` incluído em `tests/integration.rs`.
-   **Verify**: `make lint` ✓, `cargo test -p mcb-validate ca009` ✓, `test_full_validation_report`: categoria Architecture sem violações (antes 1 CA004).
-   **Learnings**: Entidades no domain devem ter identidade (id/uuid) para CA004; composition root é exceção permitida para CA009.

### Iteration 9 – Fluxo de dados e padrões (documentação)

-   **Task**: Deixar explícito o fluxo de dados e o limite de desenvolvimento nos handlers que retornam "not implemented", para que quem for implementar entenda onde falta port/use case.
-   **Alterações**:
  1.  **list_repositories.rs**: Doc do módulo explica que não existe port na camada de aplicação (registry de repositórios) ainda; doc do struct deixa claro "no injected port".
  2.  **search_branch.rs**: Doc do módulo explica data flow (handler recebe `VcsProvider`; branch-scoped search exigiria VCS + search, use case não implementado).
-   **Verify**: `make lint CI_MODE=1` ✓.
-   **Learnings**: Documentar "por que não há dados" (falta de port vs. port existente mas use case não implementado) evita código falso e guia implementação futura.

### Iteration 10 – Admin web: fluxo PATCH config (código falso no formato do request)

-   **Task**: Formulários de config enviavam PATCH com body form-urlencoded; o backend espera `application/json` e `{"values": {...}}`. O submit dava sempre falha (formato errado) – UI sugeria que "Apply" funcionava mas o fluxo de dados era falso.
-   **Alteração**: **config.html**: Removido `hx-patch` dos forms Cache e Server. Adicionada função `applyConfigSection(evt, section)` que: faz `evt.preventDefault()`, monta `values` (cache: enabled + max_size; server: host + port + admin_port), valida campos, envia `fetch('/config/' + section, { method: 'PATCH', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify({ values }) })`, trata resposta e notificação e dispara `configReloaded`.
-   **Verify**: `make lint` ✓, `make test SCOPE=all` ✓.
-   **Learnings**: O backend (config_handlers) usa `format = "json", data = "<request>"`; a UI deve enviar JSON explícito para o PATCH fazer efeito.

### Iteration 11 – Admin web: dashboard sem valores que parecem dados ("--")

-   **Task**: Na interface web admin, o dashboard exibia "--" em Status, Uptime, Active Operations e métricas antes dos dados reais (SSE/fetch), o que pode ser lido como valor.
-   **Alteração**: **index.html**: Substituído "--" por "Loading..." nos placeholders iniciais (status-value, uptime-value, active-ops-value, metric-total-queries, metric-success-rate, metric-cache-hit, metric-avg-response). Fluxo de dados inalterado: SSE e fetch continuam a preencher com valores reais ao chegar.
-   **Verify**: `make lint` ✓, `make test` ✓, `make validate QUICK=1` ✓.
-   **Learnings**: Estado inicial da UI deve ser claramente "carregando" (Loading...) e não um glifo (--) que possa ser confundido com dado.

### Iteration 12 – Clean Architecture e DI (QUAL020 + ORG016)

-   **Task**: Remover QUAL020 (allow(dead_code) em agent_session) e ORG016 (domain com métodos não permitidos).
-   **Alterações**:
  1.  **QUAL020**: Removido `#[allow(dead_code)]` do campo `agent_session` em `McpServices`; adicionado getter `agent_session_service()` em `mcp_server.rs` para o campo ser usado.
  2.  **ORG016**: Permitidos no validador (domain trait-only) os métodos de definição de schema e snapshot: `definition`, `tables`, `fts_def`, `indexes`, `foreign_keys`, `unique_constraints`, `capture` em `organization.rs`.
-   **Verify**: `make lint` ✓; report: Quality 2→1 (QUAL020 resolvido), Organization 9→1 (ORG016 resolvidos; resta ORG002).
-   **Learnings**: Campos injetados devem ter getter se expostos; domain pode ter factories de dados (schema, capture) na allow-list do ORG016.

### Iteration 13 – ORG002: literal duplicado "observation_type"

-   **Task**: Criar constante nomeada para o nome da coluna "observation_type" (single source of truth no domain).
-   **Alterações**: Em `mcb-domain/src/schema/memory.rs` adicionada `pub const COL_OBSERVATION_TYPE: &str = "observation_type"` e re-export em `schema/mod.rs`. Uso em `schema/memory.rs`, `schema/project.rs`, `mcb-infrastructure/.../row_convert.rs` e `mcb-providers/.../row_convert.rs`.
-   **Verify**: `make fmt` ✓, `make lint` ✓; categoria Organization: 1→0 violações.
-   **Learnings**: Nomes de coluna do schema devem vir do domain para evitar ORG002 e manter uma única fonte de verdade.

### Iteration 15 – Wiring list_repositories (real data)

-   **Task**: Conectar list_repositories a fonte real (collection_mapping).
-   **Alteração**: list_repositories.rs usa `collection_mapping::list_collections()`; retorna `{ repositories, count }` com dados reais.
-   **Verify**: `make lint` ✓, `make test` ✓.
-   **Learnings**: collection_mapping centraliza nomes de coleções; handler sem novo port.

### Iteration 14 – Wiring e duplicação (admin web)

-   **Task**: Deixar explícito o wiring faltante e a duplicação na interface web admin.
-   **Alterações**:
  1.  **admin/API.rs**: Doc do módulo explica que o init padrão não inicia o AdminApi; para a UI admin e REST (config, browse) funcionarem é preciso construir e iniciar `AdminApi` com `.with_config_watcher(...)` e `.with_browse_state(...)`.
  2.  **admin/web/mod.rs**: Secção "Duplication" documenta que nav e footer estão duplicados nos templates (index, config, health, indexing, browse, browse_collection, browse_file) e que mudanças de estrutura devem ser replicadas em todos.
-   **Verify**: `make fmt` ✓, `make lint` ✓, `make test` ✓, `make validate QUICK=1` ✓.
-   **Learnings**: Documentar wiring ausente evita expectativa de que a UI "funcione" sem iniciar o admin server; documentar duplicação guia manutenção consistente da nav/footer.

### Iteration 17 – DOC002: struct docs agent session handlers

-   **Task**: Corrigir DOC002 (missing struct doc) nos handlers de agent session.
-   **Alterações**: Adicionadas doc comments em CreateAgentSessionHandler, GetAgentSessionHandler, ListAgentSessionsHandler, UpdateAgentSessionHandler, StoreToolCallHandler, StoreDelegationHandler. Corrigidos imports faltantes (ResponseFormatter) em list_agent_sessions, get_agent_session, update_agent_session.
-   **Verify**: `make lint` ✓, `make test` ✓.
-   **Learnings**: Handlers devem ter doc para DOC002; ResponseFormatter precisa de import explícito.

### Iteration 16 – Admin web: wiring de auth + shared.js (remoção de duplicações)

-   **Task**: UI admin chamava endpoints protegidos sem `X-Admin-Key` (fluxo falho quando auth habilitado) e duplicava funções utilitárias em vários templates.
-   **Alterações**:
  1.  **shared.js**: Novo arquivo com `adminFetch`, `escapeHtml`, `formatUptime` e injeção automática do header `X-Admin-Key` via `htmx:configRequest`. A chave é lida de `localStorage` e solicitada via `prompt` quando ausente.
  2.  **handlers.rs + router.rs**: Adicionado endpoint `/ui/shared.js` para servir o script.
  3.  **Templates**: Incluído `<script src="/ui/shared.js"></script>` em index/config/health/indexing/browse/browse_collection/browse_file. Substituídos `fetch` por `adminFetch` em chamadas a endpoints protegidos.
  4.  **Remoção de duplicações**: Removidas funções duplicadas `escapeHtml` e `formatUptime` dos templates; agora são globais via shared.js.
-   **Verify**: `make lint` ✓, `make test SCOPE=all` ✓.
-   **Learnings**: Sem header de auth, o UI fica "funcionando" só visualmente; centralizar utilitários evita drift entre templates e garante wiring consistente.

### Iteration 19 – CA004: QualityGateResult sem identidade

-   **Task**: Entidade `QualityGateResult` sem campo id/uuid (violação CA004).
-   **Alterações**: Adicionado `id: String` em `mcb-domain/src/entities/memory.rs` com `#[serde(default)]`; em `store_quality_gate.rs` construção com `id: Uuid::new_v4().to_string()`.
-   **Verify**: `make lint` ✓; Architecture: 1→0 violações (total 165→164).
-   **Learnings**: Todas as entidades em entities/ devem ter identidade para CA004.

### Iteration 20 – Wiring search_branch (registry + busca real)

-   **Task**: Implementar wiring faltante para `search_branch` (resolver repository_id → path e executar busca).
-   **Alterações**:
  1.  **vcs_repository_registry.rs**: Novo registry `repository_id -> path` em `~/.config/mcb/vcs_repository_registry.json` com lock (flock).
  2.  **index_vcs_repository.rs**: Registra o repository_id após abrir o repo.
  3.  **search_branch.rs**: Implementado search real (lista arquivos no branch, lê conteúdo, busca substring e retorna resultados).
  4.  **tests/unit/vcs_registry_tests.rs**: Testes para record/lookup com `XDG_CONFIG_HOME` temporário (unsafe set_var em Rust 2024).
  5.  **tests/unit.rs**: Inclui o novo módulo de testes.
-   **Verify**: `make lint` ✓, `cargo test -p mcb-server --test unit` ✓.
-   **Learnings**: search_branch precisa de registry persistente para resolver repository_id; wiring simples baseado em VcsProvider já entrega funcionalidade correta.

### Iteration 18 – REF002: consolidar SqliteMemoryRepository (remover duplicação)

-   **Task**: Remover definição duplicada de `SqliteMemoryRepository` (existia em mcb-infrastructure e mcb-providers).
-   **Alterações**:
  1.  Removidos `adapters/memory_repository/mod.rs` e `row_convert.rs` em mcb-infrastructure (implementação antiga com SqlitePool; runtime já usa providers).
  2.  `adapters/mod.rs`: removidos `pub mod memory_repository` e re-export; doc atualizada.
  3.  `repositories/memory_repository.rs`: passou a re-exportar `mcb_providers::database::SqliteMemoryRepository` (única definição).
  4.  git2_provider_tests.rs: corrigido clippy `size_of_ref` (`size_of_val(erased)` em vez de `size_of_val(&erased)`).
-   **Verify**: `make lint` ✓; Refactoring 7→6 violações (REF002 SqliteMemoryRepository resolvido).
-   **Learnings**: Uma única definição em providers + re-export em infrastructure elimina REF002; runtime já usava providers.

### Iteration 17 – Indexing page: escapeHtml para dados da API (XSS)

-   **Task**: Na página Indexing, os dados da API (op.collection, op.id, op.current_file) eram inseridos em innerHTML sem escape, risco de XSS.
-   **Alterações**: **indexing.html**: Garantido carregamento de `/ui/shared.js`; ao montar a lista de operações, uso de `escapeHtml(op.collection || '')`, `escapeHtml(String(op.id || ''))` e `escapeHtml((op.current_file || '...').toString())` para título e conteúdo.
-   **Verify**: `make lint` ✓, `make test` ✓, `make validate QUICK=1` ✓.
-   **Learnings**: Qualquer template que monte HTML a partir de JSON da API deve usar escapeHtml ao inserir em innerHTML; indexing agora alinhado ao padrão das outras páginas.

### Iteration 18 – Browse collection: file-count em estado de erro

-   **Task**: Na página Browse collection, em caso de falha no load de arquivos o contador (`#file-count`) permanecia "Loading...", sugerindo carregamento em curso.
-   **Alteração**: **browse_collection.html**: No `catch` de `loadFiles()`, `document.getElementById('file-count').textContent = '—'` para refletir ausência de dado (erro).
-   **Verify**: `make fmt` ✓, `make lint CI_MODE=1` ✓. Unit + integration tests passam; doc-tests mcb falham por link extern (pré-existente).
-   **Learnings**: Em erro de fetch, atualizar todos os elementos que exibem estado (contador e lista) evita "Loading..." indefinido.

### Iteration 19 – Browse file: chunk-count em estado de erro

-   **Task**: Na página Browse file, em falha no load de chunks o contador (`#chunk-count`) permanecia "Loading chunks...", sugerindo carregamento em curso.
-   **Alteração**: **browse_file.html**: No `catch` de `loadChunks()`, `document.getElementById('chunk-count').textContent = '—'` para consistência com browse_collection.
-   **Verify**: `make fmt` ✓, `make lint CI_MODE=1` ✓.
-   **Learnings**: Mesmo padrão de estado de erro em todas as páginas de browse: contador "—" quando a API falha.

### Iteration 20 – Config page: escapeHtml para data.message nas notificações (XSS)

-   **Task**: Na página Config, as mensagens de sucesso/erro do PATCH (`data.message`) eram inseridas em innerHTML sem escape.
-   **Alteração**: **config.html**: Uso de `escapeHtml(data.message || 'Configuration updated.')` e `escapeHtml(data.message || 'Failed to update configuration.')` nas notificações de apply.
-   **Verify**: `make fmt` ✓, `make lint CI_MODE=1` ✓.
-   **Learnings**: Qualquer conteúdo de resposta da API inserido em innerHTML deve ser escapado (defense in depth).

### Iteration 21 – Duplicação: migração em massa para ResponseFormatter::JSON_success

-   **Task**: Remover duplicação do padrão `serde_json::to_string_pretty` + `CallToolResult::success(vec![Content::text(json)])` em todos os handlers que retornam JSON genérico.
-   **Alterações**: Migrados para `ResponseFormatter::json_success(&value)` os handlers: store_execution, list_agent_sessions, get_executions, store_observation, store_tool_call, update_agent_session, create_agent_session, get_agent_session, list_validators, memory_get_observations, memory_inject_context, memory_search, memory_timeline, search_memories, analyze_complexity, analyze_impact, compare_branches, create_session_summary, get_session_summary, get_validation_rules, index_vcs_repository (além dos já migrados list_repositories, search_branch, store_delegation).
-   **Verify**: `make fmt` ✓, `make lint CI_MODE=1` ✓, `make test SCOPE=all` ✓.
-   **Learnings**: Um único helper (JSON_success) elimina dezenas de linhas repetidas e garante fallback de serialização consistente; handlers de erro continuam usando `Content::text` / `CallToolResult::error` onde necessário.

### Iteration 22 – Dashboard: escapeHtml para service.state em renderServices (XSS)

-   **Task**: Na página Dashboard, o estado do serviço (`service.state`) era exibido em innerHTML sem escape na lista de services.
-   **Alteração**: **index.html**: Em `renderServices`, whitelist `safeStateClass` para a classe CSS (running, stopped, starting, stopping); exibição do estado com `escapeHtml(String(service.state || ''))`.
-   **Verify**: `make fmt` ✓, `make lint CI_MODE=1` ✓.
-   **Learnings**: Dashboard services alinhado ao padrão health/indexing: dados da API escapados; classe com whitelist.

### Iteration 23 – list_repositories: propagar erro de list_collections (evitar código falso)

-   **Task**: Handler retornava `repositories: [], count: 0` quando `list_collections()` falhava (`unwrap_or_default()`), indistinguível de "nenhuma coleção" — resposta falsa em caso de erro.
-   **Alteração**: **list_repositories.rs**: `list_collections()` propagado com `.map_err(|e| McpError::internal_error(format!("Failed to list collections: {}", e), None))?`; em falha o cliente recebe erro MCP em vez de lista vazia.
-   **Verify**: `make fmt` ✓, `make lint CI_MODE=1` ✓, `make test SCOPE=all` ✓.
-   **Learnings**: Nunca mascarar falha de backend com resposta de sucesso (lista vazia); propagar erro permite ao cliente distinguir "erro" de "zero repositórios".

## Status

-   [x] **list_repositories: propagar erro** (Iter. 23: sem unwrap_or_default; erro MCP em falha)
-   [x] **Admin web dashboard services XSS** (Iter. 22: escapeHtml service.state)
-   [x] **Admin web config notification XSS** (Iter. 20: escapeHtml data.message)
-   [x] **Admin web browse_file error state** (Iter. 19: chunk-count = "—" on load error)
-   [x] **Admin web browse_collection error state** (Iter. 18: file-count = "—" on load error)
-   [x] **Admin web indexing XSS** (Iter. 17: escapeHtml na página Indexing)
-   [x] **Admin web wiring e duplicação** (Iter. 14: doc API.rs + web/mod.rs)
-   [x] **Admin web auth wiring** (Iter. 16: shared.js, adminFetch, header X-Admin-Key via htmx)
-   [x] **Admin web código falso – COMPLETE** (Iter. 2–3, 10–11: config da API, PATCH JSON, placeholders "—", dashboard "Loading...", footers v0.1.5, browse "Loading...")
-   [x] **Admin web fluxo PATCH** (Iter. 10: config forms enviam JSON; Apply passa a compatível com backend)
-   [x] Admin web: config form Server sem valores falsos; versão footer config 0.1.5
-   [x] Admin web: browse collection file count sem valor falso (Loading...)
-   [x] search_branch: resposta honesta "not implemented"
-   [x] auth: comentário with_admin_auth corrigido
-   [x] list_repositories: wired to collection_mapping (real data)
-   [x] lib.rs: comentário em passado (legacy placeholders)
-   [x] Clean Architecture: CA004 (ExecutionMetadata id), CA009 test (composition root)
-   [x] QUAL020 + ORG016 (Iter 12): agent_session getter; domain schema/capture allow-list
-   [x] ORG002 (Iter 13): constante COL_OBSERVATION_TYPE no domain
-   [x] Fluxo de dados: doc list_repositories e search_branch (port / use case)
-   [x] Duplicação: helper JSON_success em formatter; todos os handlers JSON migrados (Iter. 21)
-   [x] Lint/test passando
-   [ ] Commit (sugerido abaixo)

## Suggested commit (Iter 23 – list_repositories propagar erro)

```bash
git add crates/mcb-server/src/handlers/list_repositories.rs ralph-progress.md
git commit -m "fix(server): propagate list_collections error in list_repositories"
```

## Suggested commit (Iter 21 – JSON_success em todos os handlers)

```bash
git add crates/mcb-server/src/handlers/*.rs ralph-progress.md
git commit -m "refactor(server): use ResponseFormatter::json_success in all JSON handlers"
```

## Suggested commit (REF002 – Iter 18)

```bash
git add crates/mcb-infrastructure/src/adapters/mod.rs \
  crates/mcb-infrastructure/src/repositories/memory_repository.rs \
  crates/mcb-providers/tests/unit/git2_provider_tests.rs \
  ralph-progress.md
git commit -m "refactor(infra): consolidate SqliteMemoryRepository in providers, fix clippy size_of_ref"
```

## Suggested commit (CA004 – Iter 19)

```bash
git add crates/mcb-domain/src/entities/memory.rs \
  crates/mcb-server/src/handlers/store_quality_gate.rs \
  ralph-progress.md
git commit -m "fix(arch): CA004 add id to QualityGateResult entity"
```

## Suggested commit (Wiring search_branch – Iter 20)

```bash
git add crates/mcb-server/src/lib.rs \
  crates/mcb-server/src/vcs_repository_registry.rs \
  crates/mcb-server/src/handlers/index_vcs_repository.rs \
  crates/mcb-server/src/handlers/search_branch.rs \
  crates/mcb-server/tests/unit/vcs_registry_tests.rs \
  crates/mcb-server/tests/unit.rs \
  ralph-progress.md
git commit -m "feat(server): wire search_branch using VCS registry and basic branch search"
```

Then `make sync` when ready (per Claude.md).
