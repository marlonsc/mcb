# PROJECT MANAGEMENT RESEARCH - EXECUTIVE SUMMARY

**Data**: Feb 5, 2026  
**Pesquisador**: Claude Code (Librarian Mode)  
**Status**: ✅ COMPLETO - Pronto para implementação

---

## 🎯 ACHADOS PRINCIPAIS

### 1. **Domain Model é Simples**
- **Project**: `{ id, name, version, path, created_at, updated_at }`
- **ProjectDependency**: `{ id, from_id, to_id, version_req, is_dev, created_at }`
- **ProjectMetadata**: Denormalized Cargo.toml data

**Fonte**: Vibe-Kanban (BloopAI), Shuttle, Sway (FuelLabs)

### 2. **Database Schema é Straightforward**
```sql
-- 3 tabelas principais
projects (id, name, version, path, created_at, updated_at)
project_dependencies (id, from_id, to_id, version_req, is_dev, created_at)
project_metadata (id, project_id, cargo_name, cargo_version, raw_toml, ...)

-- Índices críticos
idx_projects_name, idx_projects_path
idx_deps_from, idx_deps_to
```

**Otimização**: Circular dependency detection via recursive CTE (SQL)

### 3. **Rust Architecture Segue Clean Architecture**
```
mcb-domain/
  ├── entities/ (Project, ProjectDependency, ProjectMetadata)
  └── ports/ (ProjectProvider, DependencyProvider traits)

mcb-application/
  └── services/ (ProjectService com business logic)

mcb-providers/
  └── sqlite_project_provider.rs (CRUD + circular dep check)

mcb-server/
  └── handlers/project_handlers.rs (MCP tool handlers)
```

### 4. **API Patterns são Consistentes**
- **GET /projects** → List all
- **GET /projects/{id}** → Get with dependencies
- **POST /projects/{id}/dependencies** → Add dependency
- **GET /projects/{id}/dependencies** → List dependencies
- **POST /projects/{id}/analyze** → Analyze for issues

**Fonte**: Vibe-Kanban routes, Shuttle API, Ockam orchestrator

### 5. **Testing é Essencial**
- Mock providers para unit tests
- Real database para integration tests
- Circular dependency detection tests
- Performance tests para large graphs (1000+ projects)

---

## 📊 ESTIMATIVA DE ESFORÇO

| Fase | Componente | Dias | Status |
|------|-----------|------|--------|
| 1 | Domain entities + ports | 3-4 | 📋 Planejado |
| 2 | Database + providers | 4-5 | 📋 Planejado |
| 3 | Services | 2-3 | 📋 Planejado |
| 4 | MCP integration | 2-3 | 📋 Planejado |
| 5 | Testing + polish | 2-3 | 📋 Planejado |
| **TOTAL** | **13-18 dias** | **2-3 semanas** | ✅ Viável |

---

## 🔍 PADRÕES REAIS ENCONTRADOS

### Sway (FuelLabs) - Manifest Management
```rust
pub enum Source {
    Member(member::Source),
    Git(git::Source),
    Path(path::Source),
    Ipfs(ipfs::Source),
    Registry(reg::Source),
}
```
**Lição**: Suportar múltiplas fontes de dependência (git, path, registry)

### Vibe-Kanban - Minimal Model
```rust
pub struct Project {
    pub id: Uuid,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub struct ProjectRepo {
    pub id: Uuid,
    pub project_id: Uuid,
    pub repo_id: Uuid,
}
```
**Lição**: Keep entities small, use junction tables for M:N relationships

### Shuttle - Rich API Response
```rust
pub struct ProjectResponse {
    pub id: String,
    pub name: String,
    pub user_id: String,
    pub team_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub deployment_state: Option<DeploymentState>,
    pub uris: Vec<String>,
}
```
**Lição**: API responses podem ser richer que domain entities (DTO pattern)

---

## ⚠️ RISCOS & MITIGAÇÕES

| Risco | Mitigação |
|-------|-----------|
| Circular dep detection performance | Recursive CTE com depth limit |
| Large graphs (1000+ projects) | Pagination + caching |
| Cargo.toml parsing errors | Fallback to raw TOML storage |
| Concurrent updates | Database transactions + unique constraints |
| Version requirement parsing | Use `semver` crate |

---

## 📚 DOCUMENTAÇÃO COMPLETA

**Arquivo**: `/home/marlonsc/mcb/docs/research/PROJECT_MANAGEMENT_RESEARCH.md`

Contém:
- ✅ Domain model completo (código Rust)
- ✅ Database schema (SQL)
- ✅ Trait definitions (ports)
- ✅ Service implementation
- ✅ Provider implementation
- ✅ MCP tool handlers
- ✅ Testing patterns
- ✅ Implementation timeline
- ✅ Real-world examples com links GitHub

---

## 🚀 PRÓXIMOS PASSOS

1. **Criar ADR-030**: "Project Management Domain Model"
2. **Criar ADR-031**: "Project Dependency Tracking"
3. **Criar feature branch**: `feature/project-management`
4. **Criar beads issues**: 5 issues (um por fase)
5. **Start Phase 1**: Domain entities em `mcb-domain`

---

## 📖 REFERÊNCIAS

- **Sway**: https://github.com/FuelLabs/sway/blob/master/forc-pkg/src/manifest/mod.rs
- **Vibe-Kanban**: https://github.com/BloopAI/vibe-kanban/blob/main/crates/db/src/models/project.rs
- **Shuttle**: https://github.com/shuttle-hq/shuttle/blob/main/common/src/models/project.rs
- **Rocket**: https://github.com/SergioBenitez/Rocket/blob/master/examples/databases/db/sqlx/migrations/
- **Ockam**: https://github.com/build-trust/ockam/blob/develop/implementations/rust/ockam/ockam_api/src/orchestrator/project/

---

**Conclusão**: Implementação é **viável e bem-definida**. Padrões reais de 5 projetos Rust em produção confirmam a abordagem. Recomenda-se começar com Phase 1 (domain entities) imediatamente.
