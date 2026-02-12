# PROJECT MANAGEMENT IMPLEMENTATION RESEARCH

## Real-World Patterns from Rust OSS (Feb 2026)

### EXECUTIVE SUMMARY

Pesquisei 5 projetos Rust em produção (Sway, Shuttle, Vibe-Kanban, Rocket, Ockam) para extrair padrões reais de project management. Resultado: **implementação viável em 2-3 semanas** no MCB com Clean Architecture.

---

## 1. DOMAIN MODEL (MÍNIMO VIÁVEL)

### 1.1 Core Entities

```rust
// mcb-domain/src/entities/project.rs

use uuid::Uuid;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Minimal viable Project entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: Uuid,
    pub name: String,
    pub version: String,           // Cargo.toml version
    pub path: String,              // Filesystem path
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Dependency relationship
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectDependency {
    pub id: Uuid,
    pub from_project_id: Uuid,     // Dependent project
    pub to_project_id: Uuid,       // Dependency target
    pub version_requirement: String, // "^1.0", "1.2.3", etc
    pub is_dev: bool,              // dev-dependency vs regular
    pub created_at: DateTime<Utc>,
}

/// Parsed Cargo.toml metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectMetadata {
    pub project_id: Uuid,
    pub cargo_name: String,
    pub cargo_version: String,
    pub description: Option<String>,
    pub license: Option<String>,
    pub repository: Option<String>,
    pub raw_toml: String,          // Store original for re-parsing
}
```

**Why this design?**

- **Sway** (FuelLabs): Uses `PackageDependencyIdentifier { name, version }` + `Source` enum (Git, Path, Registry)
- **Vibe-Kanban**: Minimal `Project { id, name, created_at, updated_at }` + separate `ProjectRepo` junction table
- **Shuttle**: `ProjectResponse { id, name, user_id, team_id, created_at, ... }`

**Key insight**: Keep Project simple, use junction tables for relationships.

---

## 2. DATABASE SCHEMA (SQLite/PostgreSQL)

### 2.1 Core Tables

```sql
-- projects table
CREATE TABLE projects (
    id UUID PRIMARY KEY,
    name TEXT NOT NULL,
    version TEXT NOT NULL,
    path TEXT NOT NULL UNIQUE,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- project_dependencies table (junction)
CREATE TABLE project_dependencies (
    id UUID PRIMARY KEY,
    from_project_id UUID NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    to_project_id UUID NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    version_requirement TEXT NOT NULL,
    is_dev BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(from_project_id, to_project_id, is_dev)
);

-- project_metadata table (denormalized Cargo.toml data)
CREATE TABLE project_metadata (
    id UUID PRIMARY KEY,
    project_id UUID NOT NULL UNIQUE REFERENCES projects(id) ON DELETE CASCADE,
    cargo_name TEXT NOT NULL,
    cargo_version TEXT NOT NULL,
    description TEXT,
    license TEXT,
    repository TEXT,
    raw_toml TEXT NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Indices for common queries
CREATE INDEX idx_projects_name ON projects(name);
CREATE INDEX idx_projects_path ON projects(path);
CREATE INDEX idx_deps_from ON project_dependencies(from_project_id);
CREATE INDEX idx_deps_to ON project_dependencies(to_project_id);
CREATE INDEX idx_metadata_cargo_name ON project_metadata(cargo_name);
```

**Why this structure?**

- **Vibe-Kanban** uses: `projects (id, name, created_at, updated_at)` + `project_repos (id, project_id, repo_id)` junction
- **Sway** stores: `PackageEntry { name, version, source_cid, dependencies: Vec<PackageDependencyIdentifier> }`
- **Shuttle** has: `projects (id, name, user_id, team_id, created_at, deployment_state, uris)`

**Optimization**: Denormalize `project_metadata` to avoid repeated Cargo.toml parsing.

---

## 3. Rust TRAIT-BASED ARCHITECTURE

### 3.1 Port Definitions (mcb-domain)

```rust
// mcb-domain/src/ports/project_provider.rs

use crate::entities::{Project, ProjectDependency, ProjectMetadata};
use uuid::Uuid;
use async_trait::async_trait;

#[async_trait]
pub trait ProjectProvider: Send + Sync {
    /// List all indexed projects
    async fn list_projects(&self) -> Result<Vec<Project>, ProjectError>;

    /// Get project by ID
    async fn get_project(&self, id: Uuid) -> Result<Option<Project>, ProjectError>;

    /// Get project by filesystem path
    async fn get_project_by_path(&self, path: &str) -> Result<Option<Project>, ProjectError>;

    /// Create new project from Cargo.toml
    async fn create_project(&self, path: &str) -> Result<Project, ProjectError>;

    /// Update project metadata
    async fn update_project(&self, project: &Project) -> Result<(), ProjectError>;

    /// Delete project
    async fn delete_project(&self, id: Uuid) -> Result<(), ProjectError>;
}

#[async_trait]
pub trait DependencyProvider: Send + Sync {
    /// Get all dependencies of a project
    async fn get_dependencies(&self, project_id: Uuid) -> Result<Vec<ProjectDependency>, ProjectError>;

    /// Get projects that depend on this one
    async fn get_dependents(&self, project_id: Uuid) -> Result<Vec<ProjectDependency>, ProjectError>;

    /// Add dependency relationship
    async fn add_dependency(
        &self,
        from: Uuid,
        to: Uuid,
        version_req: &str,
        is_dev: bool,
    ) -> Result<ProjectDependency, ProjectError>;

    /// Remove dependency
    async fn remove_dependency(&self, from: Uuid, to: Uuid) -> Result<(), ProjectError>;

    /// Check for circular dependencies
    async fn has_circular_dependency(&self, from: Uuid, to: Uuid) -> Result<bool, ProjectError>;
}

#[derive(Debug, thiserror::Error)]
pub enum ProjectError {
    #[error("Project not found: {0}")]
    NotFound(String),

    #[error("Invalid Cargo.toml: {0}")]
    InvalidManifest(String),

    #[error("Circular dependency detected: {0} -> {1}")]
    CircularDependency(String, String),

    #[error("Database error: {0}")]
    Database(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}
```

### 3.2 Service Implementation (mcb-application)

```rust
// mcb-application/src/services/project_service.rs

use crate::ports::{ProjectProvider, DependencyProvider};
use mcb_domain::entities::{Project, ProjectDependency};
use uuid::Uuid;
use std::sync::Arc;

pub struct ProjectService {
    project_provider: Arc<dyn ProjectProvider>,
    dependency_provider: Arc<dyn DependencyProvider>,
}

impl ProjectService {
    pub fn new(
        project_provider: Arc<dyn ProjectProvider>,
        dependency_provider: Arc<dyn DependencyProvider>,
    ) -> Self {
        Self {
            project_provider,
            dependency_provider,
        }
    }

    /// Get project with all its dependencies
    pub async fn get_project_with_deps(&self, id: Uuid) -> Result<ProjectWithDeps, ProjectError> {
        let project = self.project_provider
            .get_project(id)
            .await?
            .ok_or(ProjectError::NotFound(id.to_string()))?;

        let dependencies = self.dependency_provider
            .get_dependencies(id)
            .await?;

        let dependents = self.dependency_provider
            .get_dependents(id)
            .await?;

        Ok(ProjectWithDeps {
            project,
            dependencies,
            dependents,
        })
    }

    /// Analyze dependency graph for issues
    pub async fn analyze_dependencies(&self, project_id: Uuid) -> Result<DependencyAnalysis, ProjectError> {
        let deps = self.dependency_provider.get_dependencies(project_id).await?;

        let mut analysis = DependencyAnalysis::default();
        analysis.total_dependencies = deps.len();
        analysis.dev_dependencies = deps.iter().filter(|d| d.is_dev).count();

        // Check for circular dependencies
        for dep in &deps {
            if self.dependency_provider.has_circular_dependency(project_id, dep.to_project_id).await? {
                analysis.circular_dependencies.push((project_id, dep.to_project_id));
            }
        }

        Ok(analysis)
    }
}

#[derive(Debug)]
pub struct ProjectWithDeps {
    pub project: Project,
    pub dependencies: Vec<ProjectDependency>,
    pub dependents: Vec<ProjectDependency>,
}

#[derive(Debug, Default)]
pub struct DependencyAnalysis {
    pub total_dependencies: usize,
    pub dev_dependencies: usize,
    pub circular_dependencies: Vec<(Uuid, Uuid)>,
}
```

### 3.3 Provider Implementation (mcb-providers)

```rust
// mcb-providers/src/project_provider_impl.rs

use async_trait::async_trait;
use sqlx::SqlitePool;
use uuid::Uuid;

pub struct SqliteProjectProvider {
    pool: SqlitePool,
}

#[async_trait]
impl ProjectProvider for SqliteProjectProvider {
    async fn list_projects(&self) -> Result<Vec<Project>, ProjectError> {
        sqlx::query_as::<_, Project>(
            r#"SELECT id, name, version, path, created_at, updated_at
               FROM projects
               ORDER BY created_at DESC"#
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| ProjectError::Database(e.to_string()))
    }

    async fn get_project(&self, id: Uuid) -> Result<Option<Project>, ProjectError> {
        sqlx::query_as::<_, Project>(
            r#"SELECT id, name, version, path, created_at, updated_at
               FROM projects
               WHERE id = $1"#
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| ProjectError::Database(e.to_string()))
    }

    async fn create_project(&self, path: &str) -> Result<Project, ProjectError> {
        // Parse Cargo.toml
        let manifest = parse_cargo_toml(path)?;
        let id = Uuid::new_v4();

        sqlx::query_as::<_, Project>(
            r#"INSERT INTO projects (id, name, version, path, created_at, updated_at)
               VALUES ($1, $2, $3, $4, NOW(), NOW())
               RETURNING id, name, version, path, created_at, updated_at"#
        )
        .bind(id)
        .bind(&manifest.name)
        .bind(&manifest.version)
        .bind(path)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| ProjectError::Database(e.to_string()))
    }
}

#[async_trait]
impl DependencyProvider for SqliteProjectProvider {
    async fn get_dependencies(&self, project_id: Uuid) -> Result<Vec<ProjectDependency>, ProjectError> {
        sqlx::query_as::<_, ProjectDependency>(
            r#"SELECT id, from_project_id, to_project_id, version_requirement, is_dev, created_at
               FROM project_dependencies
               WHERE from_project_id = $1
               ORDER BY created_at DESC"#
        )
        .bind(project_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| ProjectError::Database(e.to_string()))
    }

    async fn add_dependency(
        &self,
        from: Uuid,
        to: Uuid,
        version_req: &str,
        is_dev: bool,
    ) -> Result<ProjectDependency, ProjectError> {
        // Check for circular dependency
        if self.has_circular_dependency(from, to).await? {
            return Err(ProjectError::CircularDependency(
                from.to_string(),
                to.to_string(),
            ));
        }

        let id = Uuid::new_v4();
        sqlx::query_as::<_, ProjectDependency>(
            r#"INSERT INTO project_dependencies
               (id, from_project_id, to_project_id, version_requirement, is_dev, created_at)
               VALUES ($1, $2, $3, $4, $5, NOW())
               RETURNING id, from_project_id, to_project_id, version_requirement, is_dev, created_at"#
        )
        .bind(id)
        .bind(from)
        .bind(to)
        .bind(version_req)
        .bind(is_dev)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| ProjectError::Database(e.to_string()))
    }

    async fn has_circular_dependency(&self, from: Uuid, to: Uuid) -> Result<bool, ProjectError> {
        // Recursive check: if `to` depends on `from`, it's circular
        let count: (i64,) = sqlx::query_as(
            r#"WITH RECURSIVE dep_chain AS (
                 SELECT from_project_id, to_project_id FROM project_dependencies WHERE from_project_id = $2
                 UNION ALL
                 SELECT d.from_project_id, d.to_project_id
                 FROM project_dependencies d
                 JOIN dep_chain ON d.from_project_id = dep_chain.to_project_id
               )
               SELECT COUNT(*) FROM dep_chain WHERE to_project_id = $1"#
        )
        .bind(from)
        .bind(to)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| ProjectError::Database(e.to_string()))?;

        Ok(count.0 > 0)
    }
}
```

---

## 4. API PATTERNS (MCP TOOLS)

### 4.1 MCP Tool Handlers (mcb-server)

```rust
// mcb-server/src/handlers/project_handlers.rs

use serde_json::json;
use crate::services::ProjectService;

pub async fn index_projects(
    service: Arc<ProjectService>,
    _params: serde_json::Value,
) -> Result<CallToolResult, String> {
    let projects = service.list_projects()
        .await
        .map_err(|e| e.to_string())?;

    Ok(CallToolResult {
        content: vec![Content {
            type_: "text".to_string(),
            text: serde_json::to_string_pretty(&projects)
                .unwrap_or_default(),
        }],
        is_error: false,
    })
}

pub async fn get_project_details(
    service: Arc<ProjectService>,
    params: serde_json::Value,
) -> Result<CallToolResult, String> {
    let project_id = params["project_id"]
        .as_str()
        .ok_or("Missing project_id")?;

    let uuid = uuid::Uuid::parse_str(project_id)
        .map_err(|e| e.to_string())?;

    let project_with_deps = service.get_project_with_deps(uuid)
        .await
        .map_err(|e| e.to_string())?;

    Ok(CallToolResult {
        content: vec![Content {
            type_: "text".to_string(),
            text: serde_json::to_string_pretty(&project_with_deps)
                .unwrap_or_default(),
        }],
        is_error: false,
    })
}

pub async fn add_project_dependency(
    service: Arc<ProjectService>,
    params: serde_json::Value,
) -> Result<CallToolResult, String> {
    let from_id = params["from_project_id"].as_str().ok_or("Missing from_project_id")?;
    let to_id = params["to_project_id"].as_str().ok_or("Missing to_project_id")?;
    let version_req = params["version_requirement"].as_str().unwrap_or("*");
    let is_dev = params["is_dev"].as_bool().unwrap_or(false);

    let from = uuid::Uuid::parse_str(from_id).map_err(|e| e.to_string())?;
    let to = uuid::Uuid::parse_str(to_id).map_err(|e| e.to_string())?;

    let dep = service.add_dependency(from, to, version_req, is_dev)
        .await
        .map_err(|e| e.to_string())?;

    Ok(CallToolResult {
        content: vec![Content {
            type_: "text".to_string(),
            text: serde_json::to_string_pretty(&dep)
                .unwrap_or_default(),
        }],
        is_error: false,
    })
}

pub async fn analyze_project_dependencies(
    service: Arc<ProjectService>,
    params: serde_json::Value,
) -> Result<CallToolResult, String> {
    let project_id = params["project_id"].as_str().ok_or("Missing project_id")?;
    let uuid = uuid::Uuid::parse_str(project_id).map_err(|e| e.to_string())?;

    let analysis = service.analyze_dependencies(uuid)
        .await
        .map_err(|e| e.to_string())?;

    Ok(CallToolResult {
        content: vec![Content {
            type_: "text".to_string(),
            text: serde_json::to_string_pretty(&analysis)
                .unwrap_or_default(),
        }],
        is_error: false,
    })
}
```

### 4.2 Tool Definitions

```json
{
  "tools": [
    {
      "name": "index_projects",
      "description": "List all indexed projects with their metadata",
      "inputSchema": {
        "type": "object",
        "properties": {}
      }
    },
    {
      "name": "get_project_details",
      "description": "Get detailed information about a project including dependencies",
      "inputSchema": {
        "type": "object",
        "properties": {
          "project_id": {
            "type": "string",
            "description": "UUID of the project"
          }
        },
        "required": ["project_id"]
      }
    },
    {
      "name": "add_project_dependency",
      "description": "Add a dependency relationship between two projects",
      "inputSchema": {
        "type": "object",
        "properties": {
          "from_project_id": {
            "type": "string",
            "description": "UUID of the dependent project"
          },
          "to_project_id": {
            "type": "string",
            "description": "UUID of the dependency target"
          },
          "version_requirement": {
            "type": "string",
            "description": "Version requirement (e.g., '^1.0', '1.2.3')"
          },
          "is_dev": {
            "type": "boolean",
            "description": "Whether this is a dev dependency"
          }
        },
        "required": ["from_project_id", "to_project_id"]
      }
    },
    {
      "name": "analyze_project_dependencies",
      "description": "Analyze project dependencies for issues (circular deps, etc)",
      "inputSchema": {
        "type": "object",
        "properties": {
          "project_id": {
            "type": "string",
            "description": "UUID of the project to analyze"
          }
        },
        "required": ["project_id"]
      }
    }
  ]
}
```

---

## 5. TESTING PATTERNS

### 5.1 Unit Tests

```rust
// mcb-application/src/services/tests.rs

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    struct MockProjectProvider {
        projects: Vec<Project>,
    }

    #[async_trait]
    impl ProjectProvider for MockProjectProvider {
        async fn list_projects(&self) -> Result<Vec<Project>, ProjectError> {
            Ok(self.projects.clone())
        }

        async fn get_project(&self, id: Uuid) -> Result<Option<Project>, ProjectError> {
            Ok(self.projects.iter().find(|p| p.id == id).cloned())
        }

        // ... other methods
    }

    #[tokio::test]
    async fn test_get_project_with_deps() {
        let project = Project {
            id: Uuid::new_v4(),
            name: "test-project".to_string(),
            version: "1.0.0".to_string(),
            path: "/test".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let provider = Arc::new(MockProjectProvider {
            projects: vec![project.clone()],
        });

        let service = ProjectService::new(provider, Arc::new(MockDependencyProvider::new()));
        let result = service.get_project_with_deps(project.id).await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap().project.id, project.id);
    }

    #[tokio::test]
    async fn test_circular_dependency_detection() {
        // Create projects A -> B -> C -> A (circular)
        let a_id = Uuid::new_v4();
        let b_id = Uuid::new_v4();
        let c_id = Uuid::new_v4();

        let provider = Arc::new(MockDependencyProvider::with_deps(vec![
            (a_id, b_id, false),
            (b_id, c_id, false),
            (c_id, a_id, false),
        ]));

        let result = provider.has_circular_dependency(a_id, b_id).await;
        assert!(result.is_ok());
        assert!(result.unwrap());
    }
}
```

---

## 6. IMPLEMENTATION TIMELINE (MCB)

### Phase 1: Domain & Ports (3-4 days)

- [ ] Define `Project`, `ProjectDependency`, `ProjectMetadata` entities
- [ ] Create `ProjectProvider` and `DependencyProvider` traits
- [ ] Add error types with `thiserror`
- [ ] Write unit tests for domain logic

### Phase 2: Database & Providers (4-5 days)

- [ ] Create SQLite/PostgreSQL migrations
- [ ] Implement `SqliteProjectProvider`
- [ ] Implement `SqliteDependencyProvider`
- [ ] Add circular dependency detection (recursive SQL)
- [ ] Integration tests with real database

### Phase 3: Application Services (2-3 days)

- [ ] Implement `ProjectService`
- [ ] Add dependency analysis logic
- [ ] Create service-level error handling
- [ ] Unit tests for services

### Phase 4: MCP Integration (2-3 days)

- [ ] Create tool handlers
- [ ] Register tools in MCP server
- [ ] Add request/response serialization
- [ ] E2E tests

### Phase 5: Testing & Polish (2-3 days)

- [ ] Full test coverage (>90%)
- [ ] Performance testing (large dependency graphs)
- [ ] Documentation
- [ ] Code review & cleanup

### Total: 13-18 days (2-3 weeks)

---

## 7. REAL-WORLD EXAMPLES

### From Sway (FuelLabs)

```rust
// Dependency model
pub struct PackageDependencyIdentifier {
    package_name: String,
    version: String,
}

pub enum Source {
    Member(member::Source),
    Git(git::Source),
    Path(path::Source),
    Ipfs(ipfs::Source),
    Registry(reg::Source),
}

pub enum Pinned {
    Member(member::Pinned),
    Git(git::Pinned),
    Path(path::Pinned),
    Ipfs(ipfs::Pinned),
    Registry(reg::Pinned),
}
```

**Lesson**: Support multiple source types (git, path, registry) for flexibility.

### From Vibe-Kanban

```rust
// Minimal project model
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct Project {
    pub id: Uuid,
    pub name: String,
    pub default_agent_working_dir: Option<String>,
    pub remote_project_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// Junction table for relationships
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct ProjectRepo {
    pub id: Uuid,
    pub project_id: Uuid,
    pub repo_id: Uuid,
}
```

**Lesson**: Keep entities small, use junction tables for M:N relationships.

### From Shuttle

```rust
// Rich project response
pub struct ProjectResponse {
    pub id: String,
    pub name: String,
    pub user_id: String,
    pub team_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub compute_tier: Option<ComputeTier>,
    pub deployment_state: Option<DeploymentState>,
    pub uris: Vec<String>,
    pub repo_link: Option<GithubRepoLink>,
}
```

**Lesson**: API responses can be richer than domain entities (DTO pattern).

---

## 8. ESTIMATED EFFORT FOR MCB

| Component | Effort | Notes |
| ----------- | -------- | ------- |
| Domain entities | 1 day | Simple structs + error types |
| Database schema | 1 day | 3 tables + indices |
| Providers (SQLite) | 3 days | CRUD + circular dep detection |
| Services | 2 days | Business logic layer |
| MCP handlers | 2 days | Tool registration + serialization |
| Tests | 3 days | Unit + integration + E2E |
| Documentation | 1 day | ADR + README |
| **TOTAL** | **13 days** | **2 weeks realistic** |

---

## 9. RISKS & MITIGATIONS

| Risk | Mitigation |
| ------ | ----------- |
| Circular dependency detection performance | Use recursive CTE with depth limit |
| Large dependency graphs (1000+ projects) | Add pagination + caching |
| Cargo.toml parsing errors | Fallback to raw TOML storage |
| Concurrent updates to dependencies | Use database transactions + unique constraints |
| Version requirement parsing | Use `semver` crate for validation |

---

## 10. NEXT STEPS

1. **Create ADR-030**: "Project Management Domain Model"
2. **Create ADR-031**: "Project Dependency Tracking via Linkme Registry"
3. **Create feature branch**: `feature/project-management`
4. **Start Phase 1**: Domain entities in `mcb-domain`
5. **Create beads issues**: One per phase (5 issues total)

---

## REFERENCES

- **Sway**: <https://github.com/FuelLabs/sway/blob/master/forc-pkg/src/manifest/mod.rs>
- **Vibe-Kanban**: <https://github.com/BloopAI/vibe-kanban/blob/main/crates/db/src/models/project.rs>
- **Shuttle**: <https://github.com/shuttle-hq/shuttle/blob/main/common/src/models/project.rs>
- **Rocket**: <https://github.com/SergioBenitez/Rocket/blob/master/examples/databases/db/sqlx/migrations/>
- **Ockam**: <https://github.com/build-trust/ockam/blob/develop/implementations/rust/ockam/ockam_api/src/orchestrator/project/>
