# 🔧 Plano de Correção: Problemas Críticos de Compilação

## 📋 Visão Geral

Este plano aborda os **problemas críticos** identificados na revisão de código que impedem a compilação do MCP Context Browser v0.0.3.

**Status:** PENDING
**Prioridade:** CRITICAL
**Complexidade:** ALTA

## 🎯 Problemas Identificados

### P0 - CRÍTICO: Módulo Duplicado (Bloqueante)
- **Problema:** Módulo `factory` definido em dois locais (`factory.rs` e `factory/mod.rs`)
- **Impacto:** Compilação completamente bloqueada
- **Arquivo:** `src/lib.rs:6` + `src/factory.rs`

### P0 - CRÍTICO: Importação Inválida (Bloqueante)
- **Problema:** `PERFORMANCE_METRICS` não existe no módulo `metrics`
- **Impacto:** Falha de compilação
- **Arquivo:** `src/server/mod.rs:5`

### P1 - ALTO: Operações Bloqueantes em Async Context
- **Problema:** Comando `kill` executado de forma síncrona em contexto async
- **Impacto:** Performance degradada, potencial deadlock
- **Arquivo:** `src/sync/lockfile.rs:228-246`

### P1 - ALTO: Exposição de Dados Sensíveis
- **Problema:** PID e hostname expostos em metadata de lock
- **Impacto:** Informações sensíveis do sistema vazadas
- **Arquivo:** `src/sync/lockfile.rs:125-143`

## 📋 Feature Inventory

| Feature | Arquivo | Status Atual | Task # |
|---------|---------|--------------|--------|
| Módulo factory | `src/lib.rs:6` + `src/factory.rs` | CONFLITO | T1 |
| Importação PERFORMANCE_METRICS | `src/server/mod.rs:5` | AUSENTE | T2 |
| Comando kill síncrono | `src/sync/lockfile.rs:228-246` | BLOQUEANTE | T3 |
| Exposição PID/hostname | `src/sync/lockfile.rs:125-143` | SEGURANÇA | T4 |

## 🔄 Plano de Implementação

### **Tarefa 1: Resolver Conflito de Módulo Factory**
**Status:** `[x]` → `[x]`
**Tipo:** Correção crítica de compilação
**Arquivos:** `src/lib.rs`, `src/factory.rs`, `src/factory/mod.rs`

**Passos de Implementação:**
1. Remover arquivo duplicado `src/factory.rs`
2. Verificar que `src/factory/mod.rs` contém toda implementação necessária
3. Garantir que todas as importações no `src/lib.rs` funcionem
4. Testar compilação após remoção

**Definition of Done:**
- [ ] Arquivo duplicado removido
- [ ] Compilação bem-sucedida
- [ ] Todas as funcionalidades do factory preservadas
- [ ] Nenhum teste quebrado

---

### **Tarefa 2: Corrigir Importação PERFORMANCE_METRICS**
**Status:** `[ ]` → `[x]`
**Tipo:** Correção crítica de compilação
**Arquivos:** `src/server/mod.rs`, `src/metrics/mod.rs`

**Passos de Implementação:**
1. Verificar se `PERFORMANCE_METRICS` existe no módulo metrics
2. Se não existir, implementar ou remover a importação
3. Se existir em outro local, corrigir caminho de importação
4. Testar compilação após correção

**Definition of Done:**
- [ ] Importação corrigida ou removida
- [ ] Compilação bem-sucedida
- [ ] Funcionalidade relacionada preservada

---

### **Tarefa 3: Tornar Comando Kill Assíncrono**
**Status:** `[ ]` → `[x]`
**Tipo:** Correção de performance crítica
**Arquivos:** `src/sync/lockfile.rs`

**Passos de Implementação:**
1. Substituir `std::process::Command` por `tokio::process::Command`
2. Implementar verificação assíncrona de processo
3. Manter compatibilidade com sistemas não-Unix
4. Testar funcionalidade de limpeza de locks stale

**Definition of Done:**
- [ ] Comando kill executado de forma assíncrona
- [ ] Sem operações bloqueantes em contexto async
- [ ] Funcionalidade de limpeza preservada
- [ ] Testes de lock passando

---

### **Tarefa 4: Sanitizar Dados Sensíveis em Metadata**
**Status:** `[ ]` → `[x]`
**Tipo:** Correção de segurança crítica
**Arquivos:** `src/sync/lockfile.rs`

**Passos de Implementação:**
1. Remover exposição de PID e hostname da metadata
2. Manter apenas informações não-sensíveis (instance_id, timestamp)
3. Implementar hash ou ID anonimizado se necessário
4. Verificar que monitoramento ainda funciona

**Definition of Done:**
- [ ] PID e hostname não expostos
- [ ] Informações essenciais preservadas
- [ ] Monitoramento de locks funcional
- [ ] Sem vazamento de dados sensíveis

---

## 📊 Progress Tracking

**Completed:** 0 | **Remaining:** 4 | **Total:** 4

## ✅ Critérios de Aceitação

### **Geral:**
- [ ] Compilação bem-sucedida sem erros
- [ ] Todos os testes passando
- [ ] Nenhum warning de segurança
- [ ] Performance mantida

### **Por Tarefa:**
- [ ] Todas as Definition of Done cumpridas
- [ ] Código limpo e bem documentado
- [ ] Sem regressões introduzidas

## 🔍 Validação Final

Após completar todas as tarefas:
1. `make build` - Deve compilar sem erros
2. `make test` - Deve passar todos os testes
3. `make quality` - Deve passar verificações de qualidade
4. Verificar que todas as funcionalidades v0.0.3 ainda funcionam

## 📈 Resultado Esperado

- ✅ **Compilação funcionando** - Projeto compila sem erros
- ✅ **Segurança melhorada** - Dados sensíveis protegidos
- ✅ **Performance otimizada** - Sem operações bloqueantes
- ✅ **Código limpo** - Estrutura consistente e sem duplicatas

**Status Final:** PENDING → COMPLETE (após todas as tarefas concluídas)