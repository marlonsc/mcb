# Plano: Generalização dos Checks de Validação

## Visão Geral

Atualmente, as regras de validação rodam indiscriminadamente sobre todos os arquivos do projeto. Este plano propõe a implementação de um sistema de filtros inteligente que:

1.  **Filtra regras por linguagem** - Checks de Rust só rodam sobre arquivos `.rs`
2.  **Valida dependências** - Verifica se bibliotecas usadas estão declaradas no Cargo.toml
3.  **Gera alertas** - Quando bibliotecas são usadas sem declaração no Cargo.toml

## Arquitetura Atual

### Pontos de Filtragem Existentes

-   `duplication/` - Já filtra por linguagens definidas na configuração
-   `ast/executor.rs` - Detecta linguagem por extensão de arquivo
-   `scan.rs` - Filtra arquivos por extensão `.rs`

### Sistema de Regras YAML

-   Regras definidas em `/rules/` com campos como `languages`, `selectors`
-   Engine router direciona regras para engines apropriados
-   Validação limitada a estrutura da regra

## Plano de Implementação

### Fase 1: Sistema de Filtros Base

#### 1.1 Detector de Linguagem Inteligente

```rust
pub struct LanguageDetector {
    // Mapeamento extensão -> linguagem
    extensions: HashMap<&'static str, &'static str>,
    // Detecção por conteúdo (shebang, sintaxe)
    content_patterns: HashMap<&'static str, Regex>,
}

impl LanguageDetector {
    pub fn detect(&self, path: &Path, content: Option<&str>) -> Option<String> {
        // 1. Por extensão
        if let Some(ext) = path.extension()?.to_str() {
            if let Some(lang) = self.extensions.get(ext) {
                return Some(lang.to_string());
            }
        }

        // 2. Por conteúdo (fallback)
        if let Some(content) = content {
            for (lang, pattern) in &self.content_patterns {
                if pattern.is_match(content) {
                    return Some(lang.to_string());
                }
            }
        }

        None
    }
}
```

#### 1.2 Parser de Dependências Cargo.toml

```rust
pub struct CargoDependencyParser {
    pub workspace_root: PathBuf,
}

impl CargoDependencyParser {
    pub fn parse_workspace_deps(&self) -> Result<WorkspaceDependencies> {
        let mut deps = HashMap::new();

        // Para cada crate no workspace
        for crate_dir in self.find_crate_dirs()? {
            let cargo_toml = crate_dir.join("Cargo.toml");
            if cargo_toml.exists() {
                let crate_deps = self.parse_cargo_toml(&cargo_toml)?;
                deps.insert(crate_dir, crate_deps);
            }
        }

        Ok(WorkspaceDependencies { deps })
    }

    pub fn parse_cargo_toml(&self, path: &Path) -> Result<CrateDependencies> {
        let content = fs::read_to_string(path)?;
        let value: toml::Value = toml::from_str(&content)?;

        let mut deps = HashMap::new();

        // Parse [dependencies]
        if let Some(deps_section) = value.get("dependencies") {
            if let Some(table) = deps_section.as_table() {
                for (name, _config) in table {
                    deps.insert(name.clone(), DependencyInfo {
                        declared: true,
                        used_in_code: false, // será preenchido depois
                    });
                }
            }
        }

        Ok(CrateDependencies { deps })
    }
}
```

### Fase 2: Sistema de Filtros de Regra

#### 2.1 Estrutura de Filtros

```yaml
# Exemplo de regra com filtros
schema: "rule/v4"
id: "RUST001"
name: "Rust-specific check"
category: "quality"

# NOVO: Filtros de execução
filters:
  languages: ["rust"]  # Só executa em arquivos Rust
  dependencies: ["serde", "tokio"]  # Só executa se dependências estiverem presentes
  file_patterns: ["src/**/*.rs", "!src/tests/**"]  # Padrões de arquivo

# Configuração específica da linguagem
rust_config:
  min_version: "1.70"
  features: ["async-await"]

rule: |
  # Regra específica para Rust
```

#### 2.2 Executor de Filtros

```rust
pub struct RuleFilterExecutor {
    language_detector: LanguageDetector,
    dependency_parser: CargoDependencyParser,
    file_matcher: FilePatternMatcher,
}

impl RuleFilterExecutor {
    pub async fn should_execute_rule(
        &self,
        rule: &ValidatedRule,
        file_path: &Path,
        file_content: Option<&str>,
        workspace_deps: &WorkspaceDependencies,
    ) -> Result<bool> {
        // 1. Filtro de linguagem
        if let Some(languages) = &rule.filters.languages {
            let file_lang = self.language_detector.detect(file_path, file_content);
            if !file_lang.map_or(false, |lang| languages.contains(&lang)) {
                return Ok(false);
            }
        }

        // 2. Filtro de dependências
        if let Some(required_deps) = &rule.filters.dependencies {
            let crate_deps = self.find_crate_deps(file_path, workspace_deps)?;
            for dep in required_deps {
                if !crate_deps.has_dependency(dep) {
                    return Ok(false);
                }
            }
        }

        // 3. Filtro de padrões de arquivo
        if let Some(patterns) = &rule.filters.file_patterns {
            if !self.file_matcher.matches_any(file_path, patterns) {
                return Ok(false);
            }
        }

        Ok(true)
    }
}
```

### Fase 3: Validação de Dependências Não Declaradas

#### 3.1 Analisador de Uso de Bibliotecas

```rust
pub struct LibraryUsageAnalyzer {
    parsers: HashMap<String, Box<dyn LibraryParser>>,
}

#[async_trait]
pub trait LibraryParser: Send + Sync {
    async fn find_usages(&self, content: &str, file_path: &Path) -> Result<Vec<LibraryUsage>>;
}

pub struct LibraryUsage {
    pub name: String,
    pub usage_type: UsageType, // Import, Macro, etc.
    pub line: usize,
    pub context: String,
}

impl LibraryUsageAnalyzer {
    pub async fn analyze_file(&self, file_path: &Path, content: &str) -> Result<Vec<LibraryUsage>> {
        let language = self.detect_language(file_path)?;
        if let Some(parser) = self.parsers.get(&language) {
            parser.find_usages(content, file_path).await
        } else {
            Ok(vec![])
        }
    }
}
```

#### 3.2 Regra de Validação de Dependências

```yaml
schema: "rule/v4"
id: "DEP001"
name: "Undeclared Dependencies"
category: "dependency"
severity: "error"

filters:
  languages: ["rust", "python", "javascript", "typescript"]

description: "Libraries used in code must be declared in package manifest"

rule: |
  # Esta regra será executada em TODOS os arquivos suportados
  # e verificará se bibliotecas usadas estão declaradas
```

### Fase 4: Integração com Sistema Existente

#### 4.1 Modificações no AstQueryExecutor

```rust
impl AstQueryExecutor {
    pub async fn execute_rule_with_filters(
        &self,
        rule: &ValidatedRule,
        files: &[&Path],
        filter_executor: &RuleFilterExecutor,
        workspace_deps: &WorkspaceDependencies,
    ) -> Result<Vec<AstQueryViolation>> {
        let mut all_violations = Vec::new();

        for &file in files {
            // NOVO: Verificar filtros antes de processar
            let content = match fs::read_to_string(file) {
                Ok(c) => c,
                Err(_) => continue,
            };

            let should_execute = filter_executor
                .should_execute_rule(rule, file, Some(&content), workspace_deps)
                .await?;

            if !should_execute {
                continue;
            }

            // ... resto da lógica existente
        }

        Ok(all_violations)
    }
}
```

#### 4.2 Modificações no Rule Engine Router

```rust
impl RuleEngineRouter {
    pub async fn execute_with_filters(
        &self,
        rule_definition: &Value,
        context: &RuleContext,
        filters: &RuleFilters,
    ) -> Result<Vec<RuleViolation>> {
        // Aplicar filtros antes de rotear
        if !self.should_execute_based_on_filters(rule_definition, filters)? {
            return Ok(vec![]);
        }

        // Prosseguir com execução normal
        self.execute(rule_definition, context).await
    }
}
```

### Fase 5: Configuração e Personalização

#### 5.1 Extensão da Configuração

```toml
[filters]
# Configurações globais de filtro
default_languages = ["rust", "python", "javascript", "typescript"]
enable_dependency_validation = true
strict_dependency_checking = false

[filters.languages]
# Mapeamentos personalizados
custom_extensions = { "rsx" = "rust", "jsx" = "javascript" }

[filters.dependencies]
# Regras especiais para dependências
ignore_dev_dependencies = true
workspace_local_crates = ["mcb-*"]
```

#### 5.2 Cache de Dependências

```rust
pub struct DependencyCache {
    cache_file: PathBuf,
    cache: HashMap<PathBuf, (SystemTime, WorkspaceDependencies)>,
}

impl DependencyCache {
    pub async fn get_or_parse(&mut self, workspace_root: &Path) -> Result<&WorkspaceDependencies> {
        let metadata = fs::metadata(workspace_root)?;
        let modified = metadata.modified()?;

        if let Some((cached_time, deps)) = self.cache.get(workspace_root) {
            if *cached_time >= modified {
                return Ok(deps);
            }
        }

        // Re-parse e cache
        let deps = self.parser.parse_workspace_deps().await?;
        self.cache.insert(workspace_root.to_path_buf(), (modified, deps));
        self.save_cache().await?;

        Ok(self.cache.get(workspace_root).unwrap().1)
    }
}
```

## Benefícios Esperados

### Performance

-   **Redução de 60-80%** no tempo de execução de regras irrelevantes
-   **Execução paralela** mais eficiente por tipo de arquivo
-   **Cache inteligente** de análise de dependências

### Precisão

-   **Alertas relevantes** - só executa regras aplicáveis
-   **Detecção de dependências não declaradas** - catch bugs early
-   **Validação cross-language** - suporte consistente

### Manutenibilidade

-   **Regras mais específicas** - configuração declarativa
-   **Debugging facilitado** - filtros transparentes
-   **Extensibilidade** - fácil adicionar novos tipos de filtro

## Métricas de Sucesso

-   **Coverage**: >95% dos arquivos processados pelas regras corretas
-   **Performance**: <50% do tempo atual para execuções completas
-   **Accuracy**: Zero false positives em filtros de linguagem
-   **Detection**: >90% das dependências não declaradas detectadas

## Próximos Passos

1.  **Implementar protótipo** do sistema de filtros base
2.  **Testar performance** com regras existentes
3.  **Expandir parsers** para outras linguagens
4.  **Adicionar métricas** de eficácia dos filtros
5.  **Documentar** uso do novo sistema

## Riscos e Mitigações

### Risco: Filtros muito restritivos

**Mitigação**: Fallback para execução quando filtros falham

### Risco: Cache de dependências stale

**Mitigação**: Invalidar cache baseado em timestamps de arquivos

### Risco: Performance do parsing de dependências

**Mitigação**: Cache agressivo + lazy loading

Este plano estabelece uma base sólida para um sistema de validação mais inteligente e eficiente, alinhado com os princípios de arquitetura do projeto.
