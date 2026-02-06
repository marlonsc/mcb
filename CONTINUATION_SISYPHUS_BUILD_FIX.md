# Continuation Prompt — Sisyphus Build Fix (Wave 3/4 v0.2.0)

**Agent**: Sisyphus Build Fix  
**Branch**: `release/v0.2.0`  
**Date**: 2026-02-06  
**Last commit**: `32b3b621` (docs: Refine ADRs)

---

## Cole este prompt inteiro na próxima sessão

```
Contexto: Continuação da sessão "Sisyphus Build Fix" no release/v0.2.0.
Branch: release/v0.2.0
Foco: Rigor Arquitetural Máximo, Clean Architecture, Builds Determinísticos (ZERO features opcionais).

## Estado do Build

O `target/` foi deletado (cache corrupto). Precisa rebuild do zero.
Os arquivos no disco estão todos corretos — os erros de build originais (13 em 4 arquivos) JÁ ESTÃO CORRIGIDOS:
- memory_service.rs: importa MEMORY_COLLECTION_NAME (não mais ID2) ✅
- milvus.rs: todas as chamadas SDK usam collection.as_str() ✅
- milvus.rs:870: get_chunks_by_file recebe &CollectionId ✅
- index.rs: usa CollectionId em validate_request e clear branch ✅
- browse_tests.rs: usa info.id.as_str() (não .name) ✅

**⚠️ AVISO: O LSP pode reportar erros STALE (falsos) porque o `target/` foi deletado.**
A solução é ignorar os diagnósticos LSP até o primeiro `cargo check` completar.

**PRIMEIRO PASSO OBRIGATÓRIO:**
```bash
CARGO_BUILD_JOBS=2 cargo check --all 2>&1 | tail -40
```

Espere compilar do zero (~5-10min). Se passar limpo, commitar e prosseguir.
Se houver erros REAIS, verifique sempre o conteúdo do arquivo no disco antes de confiar no LSP.

## Mudanças não-commitadas

**Código (2 arquivos):**

1.  `crates/mcb-server/src/handlers/consolidated/search.rs` — removido import não-usado `CollectionId`
2.  `crates/mcb-providers/src/vector_store/milvus.rs:780` — fix: `CollectionId::new(&name)` → `CollectionId::new(name.clone())`

**Docs cleanup:** Vários .md deletados da raiz + 4 continuation prompts adicionados.

## Informações-chave sobre tipos

-   `CollectionId` é newtype via `define_id!` macro (mcb-domain/src/value_objects/ids.rs)
-   Métodos: `new(s)`, `as_str() -> &str`, `into_string() -> String`, `Display`, `From<String>`, `From<&str>`, `AsRef<str>`
-   **NÃO tem** `From<&CollectionId> for String` — use `collection.as_str()` nas chamadas de SDK
-   `CollectionInfo` struct: campos `id: CollectionId`, `vector_count: usize`, `file_count: usize`, `last_indexed: Option<i64>`, `provider: &'static str`

## Tarefas Pendentes (em ordem de prioridade)

### 0. Verificar Build e Commitar

-   Rodar `cargo check --all` até passar limpo (target/ foi deletado, rebuild do zero)
-   Commitar os 2 fixes de código + cleanup de docs
-   Push

### 1. Eliminar Null Providers

**Test fixtures (PRIORIDADE — escopo v0.2.0):**

-   `crates/mcb-server/tests/fixtures/sample_codebase/src/di.rs`:
    -   Linha 65, 93-95: `NullEmbeddingProvider` → substituir por FastEmbed real
    -   Linha 77, 99-101: `NullCacheProvider` → substituir por Moka real

**Infrastructure (SECUNDÁRIO — 4 nulls em bootstrap.rs/catalog.rs):**

-   `NullSyncProvider` — infrastructure/sync.rs:15, usado em bootstrap.rs:344, catalog.rs:216
-   `NullSnapshotProvider` — infrastructure/snapshot.rs:15, usado em bootstrap.rs:345, catalog.rs:217

**Domain (TERCIÁRIO — no-ops para métricas/validação, usados em testes):**

-   `NullMetricsObservabilityProvider` — ports/providers/metrics.rs:233
-   `NullMetricsProvider` — ports/providers/metrics_analysis.rs:268
-   `NullValidationProvider` — ports/providers/validation.rs:181

Total: 58 referências em 14 arquivos.

### 2. Migrar &str → &CollectionId (26+ locais)

Locais mapeados por camada:

-   **Domain:** hybrid_search.rs(3), chunk_repository.rs(6), search_repository.rs(3), hash.rs(4), metrics.rs(4), submodule.rs(1)
-   **Application:** vcs_indexing.rs(2), context_service.rs(3), indexing_service.rs(3)
-   **Infrastructure:** file_hash.rs(7)
-   **Providers:** hybrid_search/engine.rs(3), edgevec.rs(7), + ~42 .as_str()/.to_String() desnecessários
-   **Server:** formatter.rs(1), args.rs(1), session/manager.rs(1), admin/web/handlers.rs(2)

### 3. Docker Compose para testes

-   Criar `tests/docker-compose.yml` (Redis, Postgres, NATS)
-   Garantir que testes de integração usem containers com volumes gerenciados

### 4. Converter testes Mock → Real

-   SQLite in-memory + providers locais reais (FastEmbed, Moka, InMemory)

## Regras de Rigor

-   NUNCA `unwrap()` ou `expect()` em código de produção
-   ZERO `#[cfg(feature)]` no projeto (47 features já removidas)
-   Build DEVE passar `cargo check --all` antes de qualquer commit
-   Use `CARGO_BUILD_JOBS=2` se a máquina estiver lenta

```
