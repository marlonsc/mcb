# Triagem Semgrep — marlonsc/mcb

Gerado do dump da plataforma Semgrep (deployment `datacosmos`, 2026-08-06).

Bead: `mcb-iboq.1`

## Resumo

**28 findings** — high 26, medium 2, low 0
Confiança: high 27, medium 0, low 1

| regra | achados |
|---|---|
| `rust.actix.path-traversal.tainted-path.tainted-path` | 25 |
| `package_managers.dependabot.dependabot-missing-cooldown.dependabot-missing-cooldown` | 2 |
| `generic.secrets.security.detected-generic-secret.detected-generic-secret` | 1 |

## Como usar

Cada finding traz a **mensagem completa da regra** (o Semgrep descreve o problema e frequentemente o fix), o **código real** (linha `>>>`), classe de vulnerabilidade, CWE/OWASP.
**Decisão**: `corrigir` / `falso-positivo` (`nosemgrep` ou `.semgrepignore` com justificativa) / `risco-aceito`. Priorizar high com confidence=high.

## Findings

### 1 · 🟠 HIGH · conf high · `rust.actix.path-traversal.tainted-path.tainted-path`
**Classe**: Path Traversal · **Local**: `crates/mcb-infrastructure/src/config/loader.rs:51`

> The application builds a file path from potentially untrusted data, which can lead to a path traversal vulnerability. An attacker can manipulate the path which the application uses to access files. If the application does not validate user input and sanitize file paths, sensitive files such as configuration or user data can be accessed, potentially creating or overwriting files. To prevent this vu

```rust
       47  }
       48  
       49  /// Load and deserialize `AppConfig` from a specific YAML file path.
       50  fn load_from_path(path: &Path) -> Result<AppConfig> {
>>>    51      let content = std::fs::read_to_string(path)
       52          .map_err(|e| Error::config_with_source(format!("Failed to read {}", path.display()), e))?;
       53  
       54      let yaml: serde_yaml::Value = serde_yaml::from_str(&content)
       55          .map_err(|e| Error::config_with_source("Failed to parse YAML", e))?;
```

**Decisão**: 

### 2 · 🟠 HIGH · conf high · `rust.actix.path-traversal.tainted-path.tainted-path`
**Classe**: Path Traversal · **Local**: `crates/mcb-infrastructure/src/services/indexing_service/processing.rs:252`

> The application builds a file path from potentially untrusted data, which can lead to a path traversal vulnerability. An attacker can manipulate the path which the application uses to access files. If the application does not validate user input and sanitize file paths, sensitive files such as configuration or user data can be accessed, potentially creating or overwriting files. To prevent this vu

```rust
      248  
      249          self.indexing_ops
      250              .update_progress(ctx.operation_id, Some(relative_path.clone()), index);
      251  
>>>   252          let content = std::fs::read_to_string(file_path)
      253              .map_err(|e| mcb_domain::error::Error::internal(format!("Failed to read file: {e}")))?;
      254  
      255          let current_hash = match self
      256              .check_incremental(ctx.collection, &relative_path, &content)
```

**Decisão**: 

### 3 · 🟠 HIGH · conf high · `rust.actix.path-traversal.tainted-path.tainted-path`
**Classe**: Path Traversal · **Local**: `crates/mcb-infrastructure/src/validation/service.rs:227`

> The application builds a file path from potentially untrusted data, which can lead to a path traversal vulnerability. An attacker can manipulate the path which the application uses to access files. If the application does not validate user input and sanitize file paths, sensitive files such as configuration or user data can be accessed, potentially creating or overwriting files. To prevent this vu

```rust
      223          .collect()
      224  }
      225  
      226  fn analyze_file_complexity(file_path: &Path, include_functions: bool) -> Result<ComplexityReport> {
>>>   227      let content = std::fs::read_to_string(file_path).map_err(|e| {
      228          mcb_domain::error::Error::io_with_source(
      229              format!("failed to read {}", file_path.display()),
      230              e,
      231          )
```

**Decisão**: 

### 4 · 🟠 HIGH · conf high · `rust.actix.path-traversal.tainted-path.tainted-path`
**Classe**: Path Traversal · **Local**: `crates/mcb-providers/src/analysis/native.rs:30`

> The application builds a file path from potentially untrusted data, which can lead to a path traversal vulnerability. An attacker can manipulate the path which the application uses to access files. If the application does not validate user input and sanitize file paths, sensitive files such as configuration or user data can be accessed, potentially creating or overwriting files. To prevent this vu

```rust
       26              if path.is_file()
       27                  && path.extension().is_some_and(|ext| ext == "rs")
       28                  && !path.to_str().is_some_and(|s| s.contains("/target/"))
       29              {
>>>    30                  let content = fs::read_to_string(path).map_err(|e| {
       31                      Error::io_with_source(format!("failed to read {}", path.display()), e)
       32                  })?;
       33                  files.push((path.to_path_buf(), content));
       34              }
```

**Decisão**: 

### 5 · 🟠 HIGH · conf high · `rust.actix.path-traversal.tainted-path.tainted-path`
**Classe**: Path Traversal · **Local**: `crates/mcb-providers/src/database/seaorm/repos/index.rs:459`

> The application builds a file path from potentially untrusted data, which can lead to a path traversal vulnerability. An attacker can manipulate the path which the application uses to access files. If the application does not validate user input and sanitize file paths, sensitive files such as configuration or user data can be accessed, potentially creating or overwriting files. To prevent this vu

```rust
      455      fn compute_hash(&self, path: &std::path::Path) -> Result<String> {
      456          use sha2::{Digest, Sha256};
      457          use std::io::{BufReader, Read};
      458  
>>>   459          let file = std::fs::File::open(path).map_err(|e| {
      460              Error::database_with_source(format!("open file for hashing: {}", path.display()), e)
      461          })?;
      462          let mut reader = BufReader::new(file);
      463          let mut hasher = Sha256::new();
```

**Decisão**: 

### 6 · 🟠 HIGH · conf high · `rust.actix.path-traversal.tainted-path.tainted-path`
**Classe**: Path Traversal · **Local**: `crates/mcb-providers/src/language/common/engine/chunker.rs:129`

> The application builds a file path from potentially untrusted data, which can lead to a path traversal vulnerability. An attacker can manipulate the path which the application uses to access files. If the application does not validate user input and sanitize file paths, sensitive files such as configuration or user data can be accessed, potentially creating or overwriting files. To prevent this vu

```rust
      125          &self,
      126          file_path: &std::path::Path,
      127          _options: ChunkingOptions,
      128      ) -> Result<ChunkingResult> {
>>>   129          let content = tokio::fs::read_to_string(file_path)
      130              .await
      131              .map_err(|e| Error::io(e.to_string()))?;
      132  
      133          let file_name = mcb_utils::utils::path::path_to_utf8_string(file_path)
```

**Decisão**: 

### 7 · 🟠 HIGH · conf high · `rust.actix.path-traversal.tainted-path.tainted-path`
**Classe**: Path Traversal · **Local**: `crates/mcb-validate/build.rs:14`

> The application builds a file path from potentially untrusted data, which can lead to a path traversal vulnerability. An attacker can manipulate the path which the application uses to access files. If the application does not validate user input and sanitize file paths, sensitive files such as configuration or user data can be accessed, potentially creating or overwriting files. To prevent this vu

```rust
       10  use std::fs;
       11  use std::path::{Path, PathBuf};
       12  
       13  fn collect_yaml_files(dir: &Path, root: &Path, acc: &mut Vec<PathBuf>) -> std::io::Result<()> {
>>>    14      for entry in fs::read_dir(dir)? {
       15          let entry = entry?;
       16          let path = entry.path();
       17          if path.is_dir() {
       18              collect_yaml_files(&path, root, acc)?;
```

**Decisão**: 

### 8 · 🟠 HIGH · conf high · `rust.actix.path-traversal.tainted-path.tainted-path`
**Classe**: Path Traversal · **Local**: `crates/mcb-validate/src/ast/tree_sitter_query_executor.rs:90`

> The application builds a file path from potentially untrusted data, which can lead to a path traversal vulnerability. An attacker can manipulate the path which the application uses to access files. If the application does not validate user input and sanitize file paths, sensitive files such as configuration or user data can be accessed, potentially creating or overwriting files. To prevent this vu

```rust
       86          let source = match crate::run_context::ValidationRunContext::active()
       87              .and_then(|ctx| ctx.read_cached(file).ok())
       88          {
       89              Some(cached) => cached.as_bytes().to_vec(),
>>>    90              None => std::fs::read(file).map_err(ValidationError::Io)?,
       91          };
       92          Self::execute_on_source(rule, file, &source)
       93      }
       94  
```

**Decisão**: 

### 9 · 🟠 HIGH · conf high · `rust.actix.path-traversal.tainted-path.tainted-path`
**Classe**: Path Traversal · **Local**: `crates/mcb-validate/src/ast/unwrap_detector.rs:183`

> The application builds a file path from potentially untrusted data, which can lead to a path traversal vulnerability. An attacker can manipulate the path which the application uses to access files. If the application does not validate user input and sanitize file paths, sensitive files such as configuration or user data can be accessed, potentially creating or overwriting files. To prevent this vu

```rust
      179  /// # Errors
      180  ///
      181  /// Returns an error if the language cannot be determined from the filename.
      182  pub fn detect_in_content(content: &str, filename: &str) -> Result<Vec<UnwrapDetection>> {
>>>   183      let path = Path::new(filename);
      184      let source = content.as_bytes().to_vec();
      185  
      186      let (lang, _) = guess_language(&source, path);
      187      let lang = lang.ok_or_else(|| {
```

**Decisão**: 

### 10 · 🟠 HIGH · conf high · `rust.actix.path-traversal.tainted-path.tainted-path`
**Classe**: Path Traversal · **Local**: `crates/mcb-validate/src/ast/unwrap_detector.rs:209`

> The application builds a file path from potentially untrusted data, which can lead to a path traversal vulnerability. An attacker can manipulate the path which the application uses to access files. If the application does not validate user input and sanitize file paths, sensitive files such as configuration or user data can be accessed, potentially creating or overwriting files. To prevent this vu

```rust
      205  /// # Errors
      206  ///
      207  /// Returns an error if the file cannot be read or its language is unsupported.
      208  pub fn detect_in_file(path: &Path) -> Result<Vec<UnwrapDetection>> {
>>>   209      let content = std::fs::read_to_string(path)?;
      210      let file_name = path
      211          .to_str()
      212          .ok_or_else(|| ValidationError::Config(format!("Non-UTF8 path: {}", path.display())))?;
      213      detect_in_content(&content, file_name)
```

**Decisão**: 

### 11 · 🟠 HIGH · conf high · `rust.actix.path-traversal.tainted-path.tainted-path`
**Classe**: Path Traversal · **Local**: `crates/mcb-validate/src/engines/rusty_rules_engine.rs:466`

> The application builds a file path from potentially untrusted data, which can lead to a path traversal vulnerability. An attacker can manipulate the path which the application uses to access files. If the application does not validate user input and sanitize file paths, sensitive files such as configuration or user data can be accessed, potentially creating or overwriting files. To prevent this vu

```rust
      462          ViolationCategory::Quality,
      463          Severity::Warning,
      464          format!("{message}: {line_count} lines (max: {max_lines})"),
      465      )
>>>   466      .with_file(std::path::PathBuf::from(file_path))
      467      .with_context(format!("File: {file_path}, Lines: {line_count}"))
      468  }
      469  
      470  fn forbidden_patterns(rule_definition: &Value) -> Vec<&str> {
```

**Decisão**: 

### 12 · 🟠 HIGH · conf high · `rust.actix.path-traversal.tainted-path.tainted-path`
**Classe**: Path Traversal · **Local**: `crates/mcb-validate/src/extractor/rust_extractor.rs:40`

> The application builds a file path from potentially untrusted data, which can lead to a path traversal vulnerability. An attacker can manipulate the path which the application uses to access files. If the application does not validate user input and sanitize file paths, sensitive files such as configuration or user data can be accessed, potentially creating or overwriting files. To prevent this vu

```rust
       36      /// # Errors
       37      ///
       38      /// Returns an error if the file cannot be read or parsed.
       39      pub fn extract_facts(&self, path: &Path) -> Result<Vec<Fact>> {
>>>    40          let code = fs::read(path)?;
       41          // Use RustParser which is a public type alias for Parser<RustCode>
       42          let parser = RustParser::new(code, path, None);
       43          let root = parser.get_root();
       44          let code_ref = parser.get_code();
```

**Decisão**: 

### 13 · 🟠 HIGH · conf high · `rust.actix.path-traversal.tainted-path.tainted-path`
**Classe**: Path Traversal · **Local**: `crates/mcb-validate/src/filters/dependency_parser.rs:194`

> The application builds a file path from potentially untrusted data, which can lead to a path traversal vulnerability. An attacker can manipulate the path which the application uses to access files. If the application does not validate user input and sanitize file paths, sensitive files such as configuration or user data can be accessed, potentially creating or overwriting files. To prevent this vu

```rust
      190      ///
      191      /// # Errors
      192      /// Returns a validation error if the file cannot be read or is invalid TOML.
      193      fn parse_cargo_toml(path: &Path) -> Result<CrateDependencies> {
>>>   194          let content = fs::read_to_string(path)?;
      195          let value: toml::Value =
      196              toml::from_str(&content).map_err(|e| crate::ValidationError::Parse {
      197                  file: path.to_path_buf(),
      198                  message: format!("Failed to parse Cargo.toml: {e}"),
```

**Decisão**: 

### 14 · 🟠 HIGH · conf high · `rust.actix.path-traversal.tainted-path.tainted-path`
**Classe**: Path Traversal · **Local**: `crates/mcb-validate/src/lib.rs:136`

> The application builds a file path from potentially untrusted data, which can lead to a path traversal vulnerability. An attacker can manipulate the path which the application uses to access files. If the application does not validate user input and sanitize file paths, sensitive files such as configuration or user data can be accessed, potentially creating or overwriting files. To prevent this vu

```rust
      132      crates_dir: &std::path::Path,
      133      file_config: &crate::config::FileConfig,
      134      dirs: &mut Vec<PathBuf>,
      135  ) -> Result<()> {
>>>   136      for entry in std::fs::read_dir(crates_dir)? {
      137          let path = entry?.path();
      138          let skipped = path
      139              .file_name()
      140              .and_then(|n| n.to_str())
```

**Decisão**: 

### 15 · 🟠 HIGH · conf high · `rust.actix.path-traversal.tainted-path.tainted-path`
**Classe**: Path Traversal · **Local**: `crates/mcb-validate/src/linters/executor.rs:121`

> The application builds a file path from potentially untrusted data, which can lead to a path traversal vulnerability. An attacker can manipulate the path which the application uses to access files. If the application does not validate user input and sanitize file paths, sensitive files such as configuration or user data can be accessed, potentially creating or overwriting files. To prevent this vu

```rust
      117      if !dir.exists() {
      118          return Ok(());
      119      }
      120  
>>>   121      for entry in std::fs::read_dir(dir).map_err(crate::ValidationError::Io)? {
      122          let entry = entry.map_err(crate::ValidationError::Io)?;
      123          let path = entry.path();
      124          if path.is_dir() {
      125              collect_matching_files(&path, linters, files)?;
```

**Decisão**: 

### 16 · 🟠 HIGH · conf high · `rust.actix.path-traversal.tainted-path.tainted-path`
**Classe**: Path Traversal · **Local**: `crates/mcb-validate/src/metrics/rca_analyzer.rs:123`

> The application builds a file path from potentially untrusted data, which can lead to a path traversal vulnerability. An attacker can manipulate the path which the application uses to access files. If the application does not validate user input and sanitize file paths, sensitive files such as configuration or user data can be accessed, potentially creating or overwriting files. To prevent this vu

```rust
      119          let lang = self.detect_language(path).ok_or_else(|| {
      120              ValidationError::Config(format!("Unsupported language for file: {}", path.display()))
      121          })?;
      122  
>>>   123          let code = std::fs::read(path).map_err(|e| {
      124              ValidationError::Io(std::io::Error::new(
      125                  e.kind(),
      126                  format!("Failed to read file {}: {}", path.display(), e),
      127              ))
```

**Decisão**: 

### 17 · 🟠 HIGH · conf high · `rust.actix.path-traversal.tainted-path.tainted-path`
**Classe**: Path Traversal · **Local**: `crates/mcb-validate/src/metrics/rca_analyzer.rs:290`

> The application builds a file path from potentially untrusted data, which can lead to a path traversal vulnerability. An attacker can manipulate the path which the application uses to access files. If the application does not validate user input and sanitize file paths, sensitive files such as configuration or user data can be accessed, potentially creating or overwriting files. To prevent this vu

```rust
      286          let lang = self.detect_language(path).ok_or_else(|| {
      287              ValidationError::Config(format!("Unsupported language for file: {}", path.display()))
      288          })?;
      289  
>>>   290          let code = std::fs::read(path).map_err(|e| {
      291              ValidationError::Io(std::io::Error::new(
      292                  e.kind(),
      293                  format!("Failed to read file {}: {}", path.display(), e),
      294              ))
```

**Decisão**: 

### 18 · 🟠 HIGH · conf high · `rust.actix.path-traversal.tainted-path.tainted-path`
**Classe**: Path Traversal · **Local**: `crates/mcb-validate/src/pattern_registry/registry.rs:71`

> The application builds a file path from potentially untrusted data, which can lead to a path traversal vulnerability. An attacker can manipulate the path which the application uses to access files. If the application does not validate user input and sanitize file paths, sensitive files such as configuration or user data can be accessed, potentially creating or overwriting files. To prevent this vu

```rust
       67          path: &Path,
       68          naming_config: &crate::config::NamingRulesConfig,
       69          project_prefix: &str,
       70      ) -> Result<()> {
>>>    71          let content = std::fs::read_to_string(path)?;
       72          let mut yaml: serde_yaml::Value = serde_yaml::from_str(&content)?;
       73  
       74          let variables_value = template_variables(naming_config, project_prefix);
       75          let engine = TemplateEngine::new();
```

**Decisão**: 

### 19 · 🟠 HIGH · conf high · `rust.actix.path-traversal.tainted-path.tainted-path`
**Classe**: Path Traversal · **Local**: `crates/mcb-validate/src/rules/templates.rs:149`

> The application builds a file path from potentially untrusted data, which can lead to a path traversal vulnerability. An attacker can manipulate the path which the application uses to access files. If the application does not validate user input and sanitize file paths, sensitive files such as configuration or user data can be accessed, potentially creating or overwriting files. To prevent this vu

```rust
      145              if !path.ends_with(".yml") || !path.contains("/templates/") {
      146                  continue;
      147              }
      148  
>>>   149              let template_name = Path::new(path)
      150                  .file_stem()
      151                  .and_then(|name| name.to_str())
      152                  .ok_or_else(|| {
      153                      crate::ValidationError::Config(format!("Invalid template filename: {path}"))
```

**Decisão**: 

### 20 · 🟠 HIGH · conf high · `rust.actix.path-traversal.tainted-path.tainted-path`
**Classe**: Path Traversal · **Local**: `crates/mcb-validate/src/rules/yaml_loader.rs:297`

> The application builds a file path from potentially untrusted data, which can lead to a path traversal vulnerability. An attacker can manipulate the path which the application uses to access files. If the application does not validate user input and sanitize file paths, sensitive files such as configuration or user data can be accessed, potentially creating or overwriting files. To prevent this vu

```rust
      293      /// # Errors
      294      ///
      295      /// Returns an error if the file cannot be read or YAML parsing fails.
      296      pub async fn load_rule_file(&self, path: &Path) -> Result<Vec<ValidatedRule>> {
>>>   297          let content = tokio::fs::read_to_string(path)
      298              .await
      299              .map_err(crate::ValidationError::Io)?;
      300  
      301          self.load_rule_from_str(path, &content)
```

**Decisão**: 

### 21 · 🟠 HIGH · conf high · `rust.actix.path-traversal.tainted-path.tainted-path`
**Classe**: Path Traversal · **Local**: `crates/mcb-validate/src/run_context.rs:164`

> The application builds a file path from potentially untrusted data, which can lead to a path traversal vulnerability. An attacker can manipulate the path which the application uses to access files. If the application does not validate user input and sanitize file paths, sensitive files such as configuration or user data can be accessed, potentially creating or overwriting files. To prevent this vu

```rust
      160          {
      161              return Ok(Arc::clone(content));
      162          }
      163  
>>>   164          let content = std::fs::read_to_string(&normalized)?;
      165          let value: Arc<str> = Arc::from(content);
      166  
      167          if let Ok(mut cache) = self.content_cache.lock() {
      168              cache.insert(normalized, Arc::clone(&value));
```

**Decisão**: 

### 22 · 🟠 HIGH · conf high · `rust.actix.path-traversal.tainted-path.tainted-path`
**Classe**: Path Traversal · **Local**: `crates/mcb-validate/src/run_context.rs:277`

> The application builds a file path from potentially untrusted data, which can lead to a path traversal vulnerability. An attacker can manipulate the path which the application uses to access files. If the application does not validate user input and sanitize file paths, sensitive files such as configuration or user data can be accessed, potentially creating or overwriting files. To prevent this vu

```rust
      273          if line.is_empty() {
      274              continue;
      275          }
      276  
>>>   277          let relative = PathBuf::from(line);
      278          let absolute = workspace_root.join(&relative);
      279  
      280          if !absolute.is_file() {
      281              continue;
```

**Decisão**: 

### 23 · 🟠 HIGH · conf high · `rust.actix.path-traversal.tainted-path.tainted-path`
**Classe**: Path Traversal · **Local**: `crates/mcb-validate/src/validators/dependency/cargo.rs:29`

> The application builds a file path from potentially untrusted data, which can lead to a path traversal vulnerability. An attacker can manipulate the path which the application uses to access files. If the application does not validate user input and sanitize file paths, sensitive files such as configuration or user data can be accessed, potentially creating or overwriting files. To prevent this vu

```rust
       25          if !cargo_toml.exists() {
       26              continue;
       27          }
       28  
>>>    29          let content = std::fs::read_to_string(&cargo_toml)?;
       30          let parsed: toml::Value = toml::from_str(&content)?;
       31  
       32          if let Some(deps) = parsed.get("dependencies").and_then(|d| d.as_table()) {
       33              violations.extend(
```

**Decisão**: 

### 24 · 🟠 HIGH · conf high · `rust.actix.path-traversal.tainted-path.tainted-path`
**Classe**: Path Traversal · **Local**: `crates/mcb-validate/src/validators/dependency/cycles.rs:65`

> The application builds a file path from potentially untrusted data, which can lead to a path traversal vulnerability. An attacker can manipulate the path which the application uses to access files. If the application does not validate user input and sanitize file paths, sensitive files such as configuration or user data can be accessed, potentially creating or overwriting files. To prevent this vu

```rust
       61  }
       62  
       63  /// Parse a crate's `Cargo.toml` and return its internal (`mcb-*`) dependency names.
       64  fn read_mcb_dependencies(cargo_toml: &std::path::Path) -> Result<HashSet<String>> {
>>>    65      let content = std::fs::read_to_string(cargo_toml)?;
       66      let parsed: toml::Value = toml::from_str(&content)?;
       67  
       68      let Some(dependencies) = parsed.get("dependencies").and_then(|d| d.as_table()) else {
       69          return Ok(HashSet::new());
```

**Decisão**: 

### 25 · 🟠 HIGH · conf high · `rust.actix.path-traversal.tainted-path.tainted-path`
**Classe**: Path Traversal · **Local**: `crates/mcb-validate/src/validators/refactoring/tests.rs:152`

> The application builds a file path from potentially untrusted data, which can lead to a path traversal vulnerability. An attacker can manipulate the path which the application uses to access files. If the application does not validate user input and sanitize file paths, sensitive files such as configuration or user data can be accessed, potentially creating or overwriting files. To prevent this vu

```rust
      148      {
      149          return Ok(None);
      150      }
      151  
>>>   152      let content = std::fs::read_to_string(path)?;
      153      if content.contains(CFG_TEST_MARKER) || has_test_coverage(relative, test_files, test_dirs) {
      154          return Ok(None);
      155      }
      156  
```

**Decisão**: 

### 26 · 🟠 HIGH · conf low · `generic.secrets.security.detected-generic-secret.detected-generic-secret`
**Classe**: Hard-coded Secrets · **Local**: `k8s/kustomization.yaml:55`

> Generic Secret detected

```yaml
       51        - master-key=0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef
       52      type: Opaque
       53    - name: mcp-auth-secret
       54      literals:
>>>    55        - jwt-secret=supersecretjwtkey12345678901234567890
       56      type: Opaque
       57    - name: mcp-providers-secret
       58      literals:
       59        - openai-api-key=sk-your-openai-key-here
```

**Decisão**: 

### 27 · 🟡 MEDIUM · conf high · `package_managers.dependabot.dependabot-missing-cooldown.dependabot-missing-cooldown`
**Classe**: Insecure Configuration · **Local**: `.github/dependabot.yml:4`

> This Dependabot configuration does not set a cooldown period. Newly published packages can be malicious or unstable. Add a `cooldown` block with `default-days: 7` to each `package-ecosystem` entry under `updates` to wait 7 days before proposing updates to newly published package versions. Reference: https://docs.github.com/en/code-security/dependabot/dependabot-version-updates/configuration-option

```yaml
        1  ---
        2  version: 2
        3  updates:
>>>     4    - package-ecosystem: "cargo"
        5      directory: "/"
        6      schedule:
        7        interval: "weekly"
        8      open-pull-requests-limit: 5
```

**Decisão**: 

### 28 · 🟡 MEDIUM · conf high · `package_managers.dependabot.dependabot-missing-cooldown.dependabot-missing-cooldown`
**Classe**: Insecure Configuration · **Local**: `.github/dependabot.yml:23`

> This Dependabot configuration does not set a cooldown period. Newly published packages can be malicious or unstable. Add a `cooldown` block with `default-days: 7` to each `package-ecosystem` entry under `updates` to wait 7 days before proposing updates to newly published package versions. Reference: https://docs.github.com/en/code-security/dependabot/dependabot-version-updates/configuration-option

```yaml
       19        cargo-major:
       20          update-types:
       21            - "major"
       22  
>>>    23    - package-ecosystem: "github-actions"
       24      directory: "/"
       25      schedule:
       26        interval: "weekly"
       27      open-pull-requests-limit: 3
```

**Decisão**: 

