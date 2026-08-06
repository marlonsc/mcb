# Triagem Semgrep — marlonsc/mcb

Gerado do dump da plataforma Semgrep (deployment `datacosmos`, 2026-08-06).

Bead de rastreio: `mcb-iboq.1`

## Resumo

**28 findings** — high 26, medium 2, low 0
Confiança: high 27, medium 0, low 1

| regra | achados |
|---|---|
| `rust.actix.path-traversal.tainted-path.tainted-path` | 25 |
| `package_managers.dependabot.dependabot-missing-cooldown.dependabot-missing-cooldown` | 2 |
| `generic.secrets.security.detected-generic-secret.detected-generic-secret` | 1 |

## Findings

Coluna **Decisão** a preencher: `corrigir` / `falso-positivo` / `risco-aceito`.

| # | sev | conf | regra | arquivo | linha | Decisão |
|---|---|---|---|---|---|---|
| 1 | high | high | `tainted-path` | `crates/mcb-infrastructure/src/config/loader.rs` | 51 | |
| 2 | high | high | `tainted-path` | `crates/mcb-infrastructure/src/services/indexing_service/processing.rs` | 252 | |
| 3 | high | high | `tainted-path` | `crates/mcb-infrastructure/src/validation/service.rs` | 227 | |
| 4 | high | high | `tainted-path` | `crates/mcb-providers/src/analysis/native.rs` | 30 | |
| 5 | high | high | `tainted-path` | `crates/mcb-providers/src/database/seaorm/repos/index.rs` | 459 | |
| 6 | high | high | `tainted-path` | `crates/mcb-providers/src/language/common/engine/chunker.rs` | 129 | |
| 7 | high | high | `tainted-path` | `crates/mcb-validate/build.rs` | 14 | |
| 8 | high | high | `tainted-path` | `crates/mcb-validate/src/ast/tree_sitter_query_executor.rs` | 90 | |
| 9 | high | high | `tainted-path` | `crates/mcb-validate/src/ast/unwrap_detector.rs` | 183 | |
| 10 | high | high | `tainted-path` | `crates/mcb-validate/src/ast/unwrap_detector.rs` | 209 | |
| 11 | high | high | `tainted-path` | `crates/mcb-validate/src/engines/rusty_rules_engine.rs` | 466 | |
| 12 | high | high | `tainted-path` | `crates/mcb-validate/src/extractor/rust_extractor.rs` | 40 | |
| 13 | high | high | `tainted-path` | `crates/mcb-validate/src/filters/dependency_parser.rs` | 194 | |
| 14 | high | high | `tainted-path` | `crates/mcb-validate/src/lib.rs` | 136 | |
| 15 | high | high | `tainted-path` | `crates/mcb-validate/src/linters/executor.rs` | 121 | |
| 16 | high | high | `tainted-path` | `crates/mcb-validate/src/metrics/rca_analyzer.rs` | 123 | |
| 17 | high | high | `tainted-path` | `crates/mcb-validate/src/metrics/rca_analyzer.rs` | 290 | |
| 18 | high | high | `tainted-path` | `crates/mcb-validate/src/pattern_registry/registry.rs` | 71 | |
| 19 | high | high | `tainted-path` | `crates/mcb-validate/src/rules/templates.rs` | 149 | |
| 20 | high | high | `tainted-path` | `crates/mcb-validate/src/rules/yaml_loader.rs` | 297 | |
| 21 | high | high | `tainted-path` | `crates/mcb-validate/src/run_context.rs` | 164 | |
| 22 | high | high | `tainted-path` | `crates/mcb-validate/src/run_context.rs` | 277 | |
| 23 | high | high | `tainted-path` | `crates/mcb-validate/src/validators/dependency/cargo.rs` | 29 | |
| 24 | high | high | `tainted-path` | `crates/mcb-validate/src/validators/dependency/cycles.rs` | 65 | |
| 25 | high | high | `tainted-path` | `crates/mcb-validate/src/validators/refactoring/tests.rs` | 152 | |
| 26 | high | low | `detected-generic-secret` | `k8s/kustomization.yaml` | 55 | |
| 27 | medium | high | `dependabot-missing-cooldown` | `.github/dependabot.yml` | 4 | |
| 28 | medium | high | `dependabot-missing-cooldown` | `.github/dependabot.yml` | 23 | |

## Como triar

1. Abrir `arquivo:linha` e seguir o fluxo até o sink.
2. Classificar: **corrigir** (entrada externa alcança o sink), **falso-positivo** (registrar via `nosemgrep` ou `.semgrepignore` com justificativa), **risco-aceito** (com prazo de revisão).
3. Priorizar findings high com confidence=high.

Dados brutos: `~/semgrep-violations/by-repo/marlonsc__mcb.json`

