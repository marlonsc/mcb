use mcb_domain::schema::{Schema, SchemaDdlGenerator};
use mcb_providers::database::SqliteSchemaDdlGenerator;
use rstest::rstest;
use rstest::*;

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

#[fixture]
fn schema_def() -> Schema {
    Schema::definition()
}

#[fixture]
fn schema_ddl(schema_def: Schema) -> Vec<String> {
    let generator = SqliteSchemaDdlGenerator;
    generator.generate_ddl(&schema_def)
}

fn find_table_ddl<'a>(ddl: &'a [String], table: &str) -> &'a str {
    ddl.iter()
        .find(|stmt| {
            stmt.starts_with("CREATE TABLE") && stmt.contains(&format!("IF NOT EXISTS {table}"))
        })
        .map_or_else(
            || unreachable!("Table {table} not found in DDL — schema definition changed?"),
            String::as_str,
        )
}

#[rstest]
fn sqlite_ddl_generators_produce_statements(schema_def: Schema) {
    let generator = SqliteSchemaDdlGenerator;
    let ddl = generator.generate_ddl(&schema_def);

    assert!(!ddl.is_empty());
    assert!(ddl.iter().any(|s| s.contains("CREATE TABLE")));
    assert!(ddl.iter().any(|s| s.contains("fts5")));
    assert!(ddl.iter().any(|s| s.contains("AUTOINCREMENT")));

    for table in &schema_def.tables {
        assert!(
            ddl.iter()
                .any(|s| s.contains("CREATE TABLE") && s.contains(&table.name)),
            "Missing table DDL for {}",
            table.name
        );
    }
}

#[rstest]
fn schema_create_table_count_matches_schema(schema_ddl: Vec<String>, schema_def: Schema) {
    let create_count = schema_ddl
        .iter()
        .filter(|s| s.starts_with("CREATE TABLE IF NOT EXISTS") && !s.contains("VIRTUAL TABLE"))
        .count();
    assert_eq!(
        create_count,
        schema_def.tables.len(),
        "Expected {} CREATE TABLE statements, got {create_count}",
        schema_def.tables.len()
    );
}

#[rstest]
fn schema_contains_all_tables(schema_ddl: Vec<String>, schema_def: Schema) {
    let joined = schema_ddl.join("\n");
    for table in &schema_def.tables {
        let pattern = format!("CREATE TABLE IF NOT EXISTS {}", table.name);
        assert!(
            joined.contains(&pattern),
            "Missing table in DDL: {}",
            table.name
        );
    }
}

#[rstest]
fn schema_contains_all_fk_references(schema_ddl: Vec<String>, schema_def: Schema) {
    for fk in &schema_def.foreign_keys {
        let table_ddl = find_table_ddl(&schema_ddl, &fk.from_table);
        let fk_ref = format!("REFERENCES {}({})", fk.to_table, fk.to_column);
        let fk_col = format!("{} ", fk.from_column);
        assert!(
            table_ddl.contains(&fk_ref) && table_ddl.contains(&fk_col),
            "Missing FK in {}.{} -> {}.{}\nActual DDL: {}",
            fk.from_table,
            fk.from_column,
            fk.to_table,
            fk.to_column,
            table_ddl
        );
    }

    let fk_count_in_tables: usize = schema_def
        .tables
        .iter()
        .map(|table| {
            let table_ddl = find_table_ddl(&schema_ddl, &table.name);
            table_ddl.matches(" REFERENCES ").count()
        })
        .sum();

    assert_eq!(
        fk_count_in_tables,
        schema_def.foreign_keys.len(),
        "Expected {} FK references in DDL, got {fk_count_in_tables}",
        schema_def.foreign_keys.len()
    );
}

#[rstest]
fn schema_contains_all_indexes(schema_ddl: Vec<String>, schema_def: Schema) {
    let joined = schema_ddl.join("\n");
    let index_count = schema_ddl
        .iter()
        .filter(|s| s.starts_with("CREATE INDEX IF NOT EXISTS"))
        .count();

    assert_eq!(
        index_count,
        schema_def.indexes.len(),
        "Expected {} indexes, got {index_count}",
        schema_def.indexes.len()
    );

    for index in &schema_def.indexes {
        assert!(
            joined.contains(&index.name),
            "Missing index in DDL: {}",
            index.name
        );
    }
}

#[rstest]
fn schema_contains_unique_constraints(schema_ddl: Vec<String>, schema_def: Schema) {
    for unique in &schema_def.unique_constraints {
        let table_ddl = find_table_ddl(&schema_ddl, &unique.table);
        let unique_sql = format!("UNIQUE({})", unique.columns.join(", "));
        assert!(
            table_ddl.contains(&unique_sql),
            "Missing unique constraint in {}: {}\nActual DDL: {}",
            unique.table,
            unique_sql,
            table_ddl
        );
    }
}

#[tokio::test]
async fn test_ddl_executes_against_fresh_sqlite_db() -> TestResult {
    use mcb_providers::database::create_memory_repository_with_executor;
    let schema_def = Schema::definition();

    let temp_dir = tempfile::tempdir()?;
    let db_path = temp_dir.path().join("ddl_test.db");
    let (_repo, executor) = create_memory_repository_with_executor(db_path).await?;

    let rows = executor
        .query_all(
            "SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%' ORDER BY name",
            &[],
        )
        .await?;

    let table_names: Vec<String> = rows
        .iter()
        .filter_map(|r| r.try_get_string("name").ok().flatten())
        .collect();

    let expected_tables: Vec<String> = schema_def
        .tables
        .iter()
        .map(|table| table.name.clone())
        .chain(std::iter::once("observations_fts".to_owned()))
        .collect();

    assert!(
        table_names.len() >= expected_tables.len(),
        "Expected at least {} tables, found {}: {:?}",
        expected_tables.len(),
        table_names.len(),
        table_names
    );

    for expected in &expected_tables {
        assert!(
            table_names.contains(expected),
            "Table {expected} not created in SQLite DB. Found: {table_names:?}"
        );
    }

    let index_rows = executor
        .query_all(
            "SELECT name FROM sqlite_master WHERE type='index' AND name NOT LIKE 'sqlite_%'",
            &[],
        )
        .await?;

    assert!(
        index_rows.len() >= schema_def.indexes.len(),
        "Expected at least {} indexes, found {}",
        schema_def.indexes.len(),
        index_rows.len()
    );

    Ok(())
}
