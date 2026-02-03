use mcb_domain::schema::{
    MemorySchema, MemorySchemaDdlGenerator, ProjectSchema, SchemaDdlGenerator,
};
use mcb_providers::database::{SqliteMemoryDdlGenerator, SqliteSchemaDdlGenerator};

#[test]
fn test_sqlite_memory_ddl_generator_produces_statements() {
    let generator = SqliteMemoryDdlGenerator;
    let schema = MemorySchema::definition();
    let ddl: Vec<String> = generator.generate_ddl(&schema);
    assert!(!ddl.is_empty());
    assert!(ddl.iter().any(|s| s.contains("CREATE TABLE")));
    assert!(ddl.iter().any(|s| s.contains("fts5")));
}

#[test]
fn test_sqlite_project_schema_ddl_includes_all_entities() {
    let generator = SqliteSchemaDdlGenerator;
    let schema = ProjectSchema::definition();
    let ddl: Vec<String> = generator.generate_ddl(&schema);
    assert!(!ddl.is_empty());
    assert!(
        ddl.iter()
            .any(|s| s.contains("CREATE TABLE") && s.contains("collections"))
    );
    assert!(ddl.iter().any(|s| s.contains("observations")));
    assert!(ddl.iter().any(|s| s.contains("session_summaries")));
    assert!(ddl.iter().any(|s| s.contains("file_hashes")));
    assert!(
        ddl.iter()
            .any(|s| s.contains("UNIQUE(project_id, collection, file_path)"))
    );
    assert!(ddl.iter().any(|s| s.contains("AUTOINCREMENT")));
}
