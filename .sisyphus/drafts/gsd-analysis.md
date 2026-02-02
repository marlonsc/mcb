# Draft: Análise e Melhorias do Projeto GSD

## Visão Geral do Projeto

**Localização**: `~/.config/opencode/get-shit-done/`
**Versão**: 1.11.1
**Propósito**: Sistema de workflow para desenvolvimento solo com Claude Code

### Estatísticas
- **27 comandos** (/gsd-*)
- **11 agentes especializados**
- **53 arquivos** (templates, workflows, references)
- **~14.000 linhas** de markdown

## Arquitetura Atual

### Fluxo Principal
```
/gsd-new-project → PROJECT.md, REQUIREMENTS.md, ROADMAP.md, STATE.md
        ↓
/gsd-plan-phase N → PLAN.md (por fase)
        ↓
/gsd-execute-phase N → executa plans, cria SUMMARY.md
        ↓
verify-phase → VERIFICATION.md
        ↓
/gsd-complete-milestone → arquiva milestone
```

### Comandos por Categoria

**Inicialização (3)**:
- gsd-new-project, gsd-new-milestone, gsd-map-codebase

**Planejamento (5)**:
- gsd-discuss-phase, gsd-research-phase, gsd-list-phase-assumptions, gsd-plan-phase, gsd-plan-milestone-gaps

**Execução (2)**:
- gsd-execute-phase, gsd-quick

**Gerenciamento de Roadmap (3)**:
- gsd-add-phase, gsd-insert-phase, gsd-remove-phase

**Verificação/Qualidade (3)**:
- gsd-verify-work, gsd-audit-milestone, gsd-progress

**Sessão/Continuidade (2)**:
- gsd-resume-work, gsd-pause-work

**Debug/Todos (3)**:
- gsd-debug, gsd-add-todo, gsd-check-todos

**Configuração (3)**:
- gsd-settings, gsd-set-profile, gsd-complete-milestone

**Utilidade (3)**:
- gsd-help, gsd-update, gsd-join-discord

### Agentes Especializados

| Agente | Função | Linhas |
|--------|--------|--------|
| gsd-planner | Cria PLAN.md | ~42k bytes |
| gsd-executor | Executa planos | ~22k bytes |
| gsd-verifier | Verifica goals | ~22k bytes |
| gsd-roadmapper | Cria roadmap | ~16k bytes |
| gsd-project-researcher | Pesquisa domínio | ~22k bytes |
| gsd-phase-researcher | Pesquisa fase | ~18k bytes |
| gsd-plan-checker | Valida planos | ~23k bytes |
| gsd-codebase-mapper | Mapeia brownfield | ~15k bytes |
| gsd-debugger | Debug sistemático | ~36k bytes |
| gsd-research-synthesizer | Sintetiza pesquisa | ~7k bytes |
| gsd-integration-checker | Cross-phase wiring | ~12k bytes |

## Problemas Identificados

### 1. Complexidade Excessiva (CRÍTICO)

**Sintomas:**
- 27 comandos para memorizar
- execute-plan.md tem 1844 linhas
- checkpoints.md tem 1078 linhas
- Curva de aprendizado muito íngreme

**Evidência:**
```
Files with >500 lines:
- execute-plan.md: 1844 lines
- checkpoints.md: 1078 lines
- complete-milestone.md: 903 lines
- execute-phase.md: 671 lines
- verify-phase.md: 628 lines
```

### 2. Redundância de Conceitos (MAIOR)

**Sintomas:**
- Mesmos conceitos explicados em múltiplos lugares
- checkpoints.md e execute-plan.md ambos explicam checkpoints
- verify-phase.md e verify-work.md têm sobreposição

### 3. Falta de Feedback Loop (MAIOR)

**Sintomas:**
- Não há métricas de sucesso/falha coletadas
- Sem telemetria de uso dos comandos
- Sem identificação automática de gargalos no workflow

### 4. Context Window Pressure (MAIOR)

**Sintomas:**
- Arquivos muito grandes são carregados como @references
- Agentes podem degradar com context pressure
- Sem otimização de quais partes carregar

### 5. Documentação Fragmentada (MÉDIO)

**Sintomas:**
- References, templates, workflows em diretórios separados
- Difícil encontrar informação relevante
- gsd-help.md é a única visão consolidada

### 6. Rigidez do Workflow (MÉDIO)

**Sintomas:**
- Fluxo muito prescritivo
- Difícil pular etapas quando apropriado
- Modos (yolo/interactive) insuficientes

### 7. Sem Testes Automatizados (MÉDIO)

**Sintomas:**
- Nenhum teste para validar workflows
- Mudanças podem quebrar fluxos silenciosamente
- Validação depende de uso manual

### 8. State Management Frágil (MENOR)

**Sintomas:**
- STATE.md é arquivo markdown simples
- Sem validação de consistência
- Fácil ficar dessincronizado

## Oportunidades de Melhoria

### Quick Wins (Baixo Esforço, Alto Impacto)

1. **Refatorar arquivos gigantes**
   - Split execute-plan.md em módulos menores
   - Extract checkpoint handling para arquivo separado

2. **Criar "starter kit" simplificado**
   - Versão light com 5-7 comandos essenciais
   - Esconder complexidade atrás de defaults

3. **Melhorar gsd-help**
   - Adicionar categorização visual
   - Quick reference card

### Melhorias Estruturais (Médio Esforço)

4. **Context-aware loading**
   - Carregar apenas seções relevantes dos arquivos
   - Lazy loading de references

5. **Consolidar documentação**
   - Criar single source of truth por conceito
   - Eliminar redundância entre arquivos

6. **Adicionar métricas simples**
   - Tempo de execução por fase
   - Taxa de sucesso de verificações
   - Comandos mais usados

### Melhorias Avançadas (Alto Esforço)

7. **Schema-based state management**
   - Migrar STATE.md para formato estruturado
   - Validação automática de consistência

8. **Test framework para workflows**
   - Testes de integração para fluxos principais
   - CI para validar mudanças

9. **Adaptive complexity**
   - Detectar experiência do usuário
   - Ajustar verbosidade e guidance

## Decisões Tomadas

- [x] Scope: Melhoria abrangente (UX + técnico)
- [x] Prioridade: Impacto vs Esforço balanceado
- [x] Breaking changes: Permitidos (oportunidade para v2.0)

## Plano de Melhoria Proposto

### Fase 1: Quick Wins (Baixo Esforço, Alto Impacto)

**1.1 Refatorar Arquivos Gigantes**
- Split `execute-plan.md` (1844 linhas) em:
  - `execute-plan-core.md` (~400 linhas) - fluxo principal
  - `execute-plan-checkpoints.md` (~300 linhas) - handling de checkpoints
  - `execute-plan-segments.md` (~300 linhas) - segmentação
  - `execute-plan-git.md` (~200 linhas) - commits e state
  - `execute-plan-deviations.md` (~300 linhas) - regras de desvio
- Split `checkpoints.md` (1078 linhas) em:
  - `checkpoints-types.md` - tipos (human-verify, decision, human-action)
  - `checkpoints-protocol.md` - protocolo de execução
  - `checkpoints-automation.md` - referência de automação

**1.2 Criar Comando gsd-start (Unified Entry Point)**
- Novo comando que detecta contexto e roteia:
  - Sem .planning/ → sugere /gsd-new-project
  - Com .planning/ → mostra /gsd-progress summary
  - Mid-phase → oferece /gsd-resume-work
- Reduz de 27 comandos para 1 ponto de entrada

**1.3 Melhorar gsd-help com Categorização Visual**
```
/gsd-help

GETTING STARTED
  /gsd-start         - Intelligent entry point (NEW)
  /gsd-new-project   - Initialize new project
  /gsd-progress      - Check status and next steps

CORE WORKFLOW
  /gsd-plan-phase N  - Plan phase N
  /gsd-execute-phase N - Execute phase N

NEED MORE? /gsd-help full
```

### Fase 2: Consolidação de Documentação (Médio Esforço)

**2.1 Single Source of Truth por Conceito**
- Checkpoints: apenas em `references/checkpoints.md`
- TDD: apenas em `references/tdd.md`
- Git: apenas em `references/git-integration.md`
- Outros arquivos referenciam via `@reference`

**2.2 Criar Index de Conceitos**
- `references/INDEX.md` com mapa de todos os conceitos
- Links rápidos para cada tópico
- Facilita navegação e manutenção

**2.3 Eliminar Redundância**
- execute-plan.md referencia checkpoints.md em vez de duplicar
- verify-phase.md e verify-work.md consolidados

### Fase 3: Context Optimization (Médio Esforço)

**3.1 Modular @references**
- Arquivos grandes divididos permitem carregar só o necessário
- Executor carrega só `execute-plan-core.md` + módulos relevantes

**3.2 Progressive Disclosure nos Agentes**
- Agents começam com instruções mínimas
- Carregam detalhes conforme necessário

**3.3 Estimativas de Context Budget**
- Adicionar comentários estimando context usage por seção
- Helps agents decide what to load

### Fase 4: Robustez Técnica (Alto Esforço)

**4.1 State Validation Layer**
- Criar `state-validator.md` que valida STATE.md
- Detectar inconsistências automaticamente
- Rodar em /gsd-progress e /gsd-resume-work

**4.2 Métricas Simples**
- Adicionar timing tracking em STATE.md:
  ```markdown
  ## Metrics
  | Phase | Start | End | Duration |
  |-------|-------|-----|----------|
  | 1     | 2025-01-01 | 2025-01-02 | 4h |
  ```

**4.3 Test Scenarios**
- Criar `tests/` directory com cenários de teste
- Markdown files descrevendo expected behavior
- Pode ser validado manualmente ou via CI

### Fase 5: UX Simplificada (Alto Esforço)

**5.1 Modo Lite**
- Config option `mode: lite` em config.json
- Esconde comandos avançados de gsd-help
- Simplifica outputs e confirmations

**5.2 Adaptive Verbosity**
- Detectar se usuário é experienced (tem projects completos)
- Ajustar nível de explicação automaticamente

**5.3 Quick Command Aliases**
- /gsd-p → /gsd-progress
- /gsd-e N → /gsd-execute-phase N
- Documented in gsd-help

## Priorização por Impacto vs Esforço

| Melhoria | Impacto | Esforço | Prioridade |
|----------|---------|---------|------------|
| 1.1 Refatorar arquivos | Alto | Baixo | P0 |
| 1.2 gsd-start | Alto | Baixo | P0 |
| 1.3 gsd-help visual | Médio | Baixo | P0 |
| 2.1 Single source of truth | Alto | Médio | P1 |
| 2.3 Eliminar redundância | Alto | Médio | P1 |
| 3.1 Modular @references | Alto | Médio | P1 |
| 4.1 State validation | Médio | Médio | P2 |
| 4.2 Métricas | Baixo | Baixo | P2 |
| 5.1 Modo Lite | Médio | Alto | P3 |
| 5.2 Adaptive verbosity | Baixo | Alto | P3 |

## Estimativa de Escopo

**Fase 1 (P0)**: ~2-3 dias de trabalho
**Fase 2 (P1)**: ~3-4 dias de trabalho
**Fase 3 (P1)**: ~2-3 dias de trabalho
**Fase 4 (P2)**: ~3-4 dias de trabalho
**Fase 5 (P3)**: ~4-5 dias de trabalho

**Total estimado**: ~15-20 dias para melhoria completa
