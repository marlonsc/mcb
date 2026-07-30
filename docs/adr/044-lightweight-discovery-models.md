<!-- markdownlint-disable MD013 MD024 MD025 MD030 MD040 MD003 MD022 MD031 MD032 MD036 MD041 MD060 -->
---
adr: 44
title: Lightweight Discovery Models for Context Routing
status: PROPOSED
created:
updated: 2026-02-05
related: []
supersedes: []
superseded_by: []
implementation_status: Incomplete
---

<!-- markdownlint-disable MD013 MD024 MD025 MD060 -->

# ADR-044: Lightweight Discovery Models for Context Routing

**Status**: Proposed
**Date**: 2026-02-05
**Deciders**: MCB Architecture Team
**Related**: ADR-041 (Context), ADR-043 (Search)
**Context**: v0.4.0 MVP scope

## Context

ADR-043 hybrid search ranks results by BM25 + semantics + graph. But ranking is**global**: the same code is ranked the same regardless of**who's asking**or**what they're trying to do**.

Example:

- Query: "authentication"
- For a**security review task**: want cryptography libraries + auth policies
- For a**onboarding task**: want example login code + documentation
- Same query, different ideal results

**Solution**: Route queries based on**task context** without expensive ML training.

## Decision

### 1. Multi-Tier Routing: AST → Rules → (Optional: ML)

```text
┌──────────────────────────────┐
│ Task Context                 │  (From Beads: scope, priority, type)
└──────────────┬───────────────┘
               │
         ┌─────▼──────┐
         │ Stage 1    │  AST-based routing (100% reliable)
         │ (Always)   │
         └─────┬──────┘
               │
        ┌──────▼──────────┐
        │ Stage 2         │  Rule-based routing (90% cases)
        │ (Common cases)  │
        └──────┬──────────┘
               │
      ┌────────▼──────────┐
      │ Stage 3           │  ML-based (5% complex cases, v0.5.0)
      │ (Future, optional)│
      └───────────────────┘
```

### 2. Stage 1: AST-Based Routing (Zero ML)

```rust
pub struct AstBasedRouter {
    graph: Arc<CodeGraph>,
}

impl AstBasedRouter {
    pub async fn route(
        &self,
        task: &BeadsTask,
        search_results: &[ContextSearchResult],
    ) -> Result<RoutedResults> {
        // Route based purely on AST node types and structure

        let task_scope = match task.scope.as_str() {
            "feature" => {
                // Feature work: prioritize public APIs + tests
                search_results.iter()
                    .filter(|r| matches!(r.node.kind, CodeNodeKind::Function | CodeNodeKind::Struct))
                    .map(|r| (r.clone(), self.ast_score_for_feature(r)))
                    .collect()
            },
            "bug" => {
                // Bug fix: prioritize error handling + tests
                search_results.iter()
                    .filter(|r| r.node.name.contains("error") || r.node.kind == CodeNodeKind::TestModule)
                    .map(|r| (r.clone(), self.ast_score_for_bug(r)))
                    .collect()
            },
            "security" => {
                // Security review: prioritize crypto + auth + validation
                search_results.iter()
                    .filter(|r| self.is_security_critical(&r.node))
                    .map(|r| (r.clone(), self.ast_score_for_security(r)))
                    .collect()
            },
            _ => search_results.iter()
                .map(|r| (r.clone(), 1.0))
                .collect(),
        };

        Ok(RoutedResults { results: task_scope })
    }

    fn ast_score_for_feature(&self, result: &ContextSearchResult) -> f32 {
        let mut score = 1.0;

        // Boost public functions
        if result.node.is_public() { score *= 1.3; }

        // Boost if has tests (check callees for test functions)
        if self.has_tests(result.node.id) { score *= 1.2; }

        // Penalize implementation details
        if result.node.name.starts_with("_") { score *= 0.6; }

        score
    }

    fn ast_score_for_bug(&self, result: &ContextSearchResult) -> f32 {
        let mut score = 1.0;

        // Boost error/exception handling
        if result.node.name.contains("error") || result.node.name.contains("catch") {
            score *= 1.4;
        }

        // Boost tests
        if result.node.kind == CodeNodeKind::TestModule {
            score *= 1.3;
        }

        score
    }

    fn ast_score_for_security(&self, result: &ContextSearchResult) -> f32 {
        let security_keywords = [
            "crypto", "encrypt", "hash", "auth", "validate", "sanitize",
            "permission", "access", "secret", "token", "password",
        ];

        let mut score = 1.0;
        for keyword in &security_keywords {
            if result.node.name.to_lowercase().contains(keyword) {
                score *= 1.5;
            }
        }

        score
    }
}
```

**Cost**: <5ms per query (AST walk + scoring)
**Coverage**: 85% of real tasks (feature, bug, refactor, security, documentation)

### 3. Stage 2: Rule-Based Routing (rhai DSL)

For tasks that don't fit standard patterns, use**rhai scripting** for custom rules:

```rust
pub struct RuleBasedRouter {
    engine: rhai::Engine,
    rules: HashMap<String, String>,  // task_scope -> rhai script
}

impl RuleBasedRouter {
    pub async fn route(&self, task: &BeadsTask, results: &[ContextSearchResult]) -> Result<Vec<(ContextSearchResult, f32)>> {
        let rule_script = self.rules.get(&task.scope).ok_or(Error::NoRuleForScope)?;

        // Rhai script example (stored as string in config)
        // if node.kind == "Function" && node.name.contains("render") {
        //   score = base_score * 1.5;
        // }
        // if node.complexity > 10 {
        //   score = score * 0.8;  // penalize complex functions for UI work
        // }

        let mut scope = rhai::Scope::new();
        for (i, result) in results.iter().enumerate() {
            let node_map = self.node_to_rhai_map(&result.node);
            scope.push(format!("node_{}", i), node_map);
        }

        let result = self.engine.eval_with_scope(&mut scope, rule_script)?;

        Ok(result)  // Rhai returns Vec<(index, score)>
    }

    fn node_to_rhai_map(&self, node: &CodeNode) -> rhai::Map {
        let mut map = rhai::Map::new();
        map.insert("name".into(), node.name.clone().into());
        map.insert("kind".into(), format!("{:?}", node.kind).into());
        map.insert("complexity".into(), node.metrics.cyclomatic_complexity.into());
        map.insert("is_public".into(), node.is_public().into());
        map.insert("has_tests".into(), self.has_tests(node.id).into());
        map
    }
}
```

**Cost**: 5-20ms per query (rhai script execution)
**Coverage**: 90% of real tasks (custom per organization)
**Maintainability**: Non-engineers can write rules (no Rust needed)

### 4. Stage 3: ML-Based Routing (Deferred to v0.5.0)

```rust
// Placeholder for v0.5.0
pub struct MlBasedRouter {
    model: Option<Arc<OnnxModel>>,  // Small classifier (~10MB)
}

impl MlBasedRouter {
    pub async fn route(&self, task: &BeadsTask, results: &[ContextSearchResult]) -> Result<Vec<(ContextSearchResult, f32)>> {
        let model = self.model.as_ref().ok_or(Error::ModelNotLoaded)?;

        let features = results.iter().map(|r| self.extract_features(r)).collect::<Vec<_>>();

        // Run ONNX inference
        let scores = model.predict_batch(&features)?;

        Ok(results.iter().zip(scores).map(|(r, s)| (r.clone(), s)).collect())
    }
}
```

**Cost**: 10-50ms per query (inference)
**Coverage**: 95%+ (learns from feedback)
**Trade-off**: Requires training data + serving infrastructure (post-MVP)

## Configuration

```toml

# config/default.toml

> **v0.3.0 Migration Note:** Configuration is now Loco YAML (`config/development.yaml`, `config/test.yaml`), not Figment TOML (`config/default.toml`).

[routing]

# Which router to use: "ast" | "rules" | "ml"
enabled = ["ast", "rules"]  # Pipeline: AST first, fallback to rules

[routing.ast]

# Feature work: boost public APIs, tests
feature_scope.boost_public = 1.3
feature_scope.boost_tests = 1.2

# Bug fix: boost error handling
bug_scope.boost_error_handling = 1.4
bug_scope.boost_tests = 1.3

[routing.rules]

# Custom rules per task scope
security = """
if node.name.contains("crypto") || node.name.contains("auth") {
  score = base_score * 2.0;
}
if node.complexity > 15 {
  score = score * 0.7;
}
"""

onboarding = """
if node.has_documentation == true {
  score = base_score * 1.5;
}
if node.is_public == true && node.kind == "Function" {
  score = score * 1.2;
}
"""
```

## Integration with ADR-041-046

ADR-043 (Hybrid Search):

- After RRF fusion, apply routing to rerank

ADR-045 (Versioning):

- Store task-specific context snapshots (include routing rules used)

ADR-046 (Policies):

- Policies can gate routing (e.g., "Security policy requires high-confidence scoring")

## Testing

- **AST router tests** (8): Feature/bug/security task routing, scoring logic
- **Rule engine tests** (5): rhai script parsing, execution, error handling
- **ML router tests** (5, deferred): Model loading, inference, batching

**Target**: 18+ tests (AST + rules), 80%+ coverage

### Success Criteria

- ✅ AST router <5ms latency
- ✅ Rule router <20ms latency
- ✅ Routing improves top-1 Result relevance by 20% (A/B test)
- ✅ Custom rules work without code changes
- ✅ ML placeholder ready for v0.5.0

---

**Depends on**: ADR-041 (context), ADR-043 (hybrid search)
**Feeds**: ADR-046 (policy gating)
**Future**: ML models in v0.5.0 (Candle + ONNX)
