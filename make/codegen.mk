# =============================================================================
# Code Generation (SeaORM entities from migration DDL)
# =============================================================================
#
# Pipeline: migration.rs → SQLite DB → sea-orm-cli → entities/
#
# Single source of truth: the migration file defines the schema.
# sea-orm-cli generates entities from a temp SQLite DB built from that schema.
#
# Usage:
#   make codegen          # Full pipeline: DB → entities
#   make codegen-db       # Create temp SQLite DB from migration DDL
#   make codegen-entities # Generate SeaORM entities from DB
#   make codegen-cli      # Build sea-orm-cli from fork
#   make codegen-clean    # Remove temp DB

.PHONY: codegen codegen-db codegen-entities codegen-conversions codegen-cli codegen-clean

CODEGEN_DB       := /tmp/mcb_codegen.db
MIGRATION_RS     := crates/mcb-providers/src/database/seaorm/migration/m20260301_000001_initial_schema.rs
SEA_ORM_CLI      := third-party/sea-orm/sea-orm-cli/target/debug/sea-orm-cli
ENTITIES_DIR     := crates/mcb-providers/src/database/seaorm/entities
CONVERSIONS_DIR  := crates/mcb-providers/src/database/seaorm/conversions
CONVERSIONS_TOML := config/conversions.toml
CONVERSIONS_SCRIPT := scripts/codegen-conversions.py
EXTRACT_SCRIPT   := scripts/extract-migration-sql.py

##@ Code Generation

codegen: codegen-entities codegen-conversions ## Full codegen pipeline: migration → entities + conversions
	@echo "✓ Codegen complete. Entities in $(ENTITIES_DIR)/, conversions in $(CONVERSIONS_DIR)/"

codegen-cli: ## Build sea-orm-cli from fork
	@echo "Building sea-orm-cli from third-party/sea-orm..."
	@cargo build --manifest-path=third-party/sea-orm/sea-orm-cli/Cargo.toml
	@echo "✓ sea-orm-cli built at $(SEA_ORM_CLI)"

codegen-db: $(EXTRACT_SCRIPT) $(MIGRATION_RS) ## Create temp SQLite DB from migration DDL
	@rm -f $(CODEGEN_DB)
	@python3 $(EXTRACT_SCRIPT) $(MIGRATION_RS) | sqlite3 $(CODEGEN_DB)
	@TABLE_COUNT=$$(sqlite3 $(CODEGEN_DB) "SELECT count(*) FROM sqlite_master WHERE type='table'"); \
	echo "✓ Created codegen DB with $$TABLE_COUNT tables at $(CODEGEN_DB)"

codegen-entities: codegen-db $(SEA_ORM_CLI) ## Generate SeaORM entities from DB schema
	@echo "Generating entities from $(CODEGEN_DB)..."
	@$(SEA_ORM_CLI) generate entity \
		--database-url "sqlite://$(CODEGEN_DB)?mode=rwc" \
		--output-dir $(ENTITIES_DIR) \
		--with-serde both \
		--ignore-tables seaql_migrations \
		--date-time-crate time
	@python3 scripts/codegen-post-process.py $(ENTITIES_DIR)/mod.rs
	@echo "✓ Generated entities in $(ENTITIES_DIR)/"

codegen-clean: ## Remove temp codegen DB
	@rm -f $(CODEGEN_DB)
	@echo "✓ Cleaned codegen artifacts"

codegen-conversions: $(CONVERSIONS_SCRIPT) $(CONVERSIONS_TOML) ## Generate domain ↔ SeaORM conversions from TOML config
	@echo "Generating conversions from $(CONVERSIONS_TOML)..."
	@python3 $(CONVERSIONS_SCRIPT)
	@echo "✓ Generated conversions in $(CONVERSIONS_DIR)/"
