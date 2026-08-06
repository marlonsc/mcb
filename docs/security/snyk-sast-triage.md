# Triagem Snyk Code (SAST) — marlonsc/mcb

Gerado do scan Snyk da org Datacosmos (dump 2026-08-06).

**10 achados** — critical 0, high 0, medium 0, low 10

| categoria | achados |
|---|---|
| Path Traversal | 9 |
| Cleartext Transmission - HTTP Instead of HTTPS | 1 |

## Achados

Coluna **Decisão**: `corrigir` / `falso-positivo` / `risco-aceito`.

| # | sev | categoria | arquivo | linha | CWE | Decisão |
|---|---|---|---|---|---|---|
| 1 | low | Path Traversal | `scripts/codegen-post-process.py` | 39 | - | |
| 2 | low | Path Traversal | `scripts/codegen-post-process.py` | 54 | - | |
| 3 | low | Path Traversal | `scripts/docs/py/check_links.py` | 39 | - | |
| 4 | low | Path Traversal | `scripts/docs/py/check_links.py` | 46 | - | |
| 5 | low | Path Traversal | `scripts/docs/py/check_outdated.py` | 58 | - | |
| 6 | low | Path Traversal | `scripts/docs/py/check_outdated.py` | 67 | - | |
| 7 | low | Path Traversal | `scripts/docs/py/check_source_refs.py` | 16 | - | |
| 8 | low | Path Traversal | `scripts/docs/py/check_source_refs.py` | 23 | - | |
| 9 | low | Path Traversal | `scripts/extract-migration-sql.py` | 13 | - | |
| 10 | low | Cleartext Transmission - HTTP Instead of HTTPS | `tests/e2e/browse-ui.spec.ts` | 21 | - | |

## Como triar

1. Abrir `arquivo:linha` e seguir o fluxo de dados até o sink.
2. Classificar: **corrigir** (entrada externa alcança o sink sem sanitização), **falso-positivo** (credencial de fixture, path de constante — registrar em `.snyk` com justificativa), **risco-aceito** (com prazo de revisão).

Dados brutos: `~/snyk-violations/sast/marlonsc__mcb.sast.json`

