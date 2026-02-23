//! Schema-Entity synchronization tests.
//!
//! Detects divergences between entity struct fields and their corresponding schema definitions.
//! Uses the canonical Schema::definition() to access table metadata.

#[cfg(test)]
mod tests {
    use mcb_domain::schema::Schema;
    use std::collections::HashMap;

    #[test]
    fn print_full_schema_report() {
        let schema = Schema::definition();
        println!("\n=== FULL SCHEMA ANALYSIS ===");
        println!("Total tables: {}", schema.tables.len());
        println!("Total indexes: {}", schema.indexes.len());
        println!("Total foreign keys: {}", schema.foreign_keys.len());
        println!("\nTable summary:");

        for table in &schema.tables {
            println!(
                "  {} ({} cols): {}",
                table.name,
                table.columns.len(),
                table
                    .columns
                    .iter()
                    .map(|c| &c.name)
                    .cloned()
                    .collect::<Vec<_>>()
                    .join(", ")
            );
        }
    }

    #[test]
    fn detect_schema_only_fields() {
        let schema = Schema::definition();

        // Build a map of table_name -> column_names
        let mut tables_by_name: HashMap<String, Vec<String>> = HashMap::new();
        for table in &schema.tables {
            let cols: Vec<String> = table.columns.iter().map(|c| c.name.clone()).collect();
            tables_by_name.insert(table.name.clone(), cols);
        }

        println!("\n=== SCHEMA-ONLY FIELDS (Not in entities) ===");

        // Check repositories
        if let Some(repo_cols) = tables_by_name.get("repositories") {
            if repo_cols.contains(&"origin_context".to_string()) {
                println!("  ⚠ Repository.origin_context - schema-only, needs entity field");
            }
        }

        // Check worktrees
        if let Some(wt_cols) = tables_by_name.get("worktrees") {
            let mut extra = Vec::new();
            if wt_cols.contains(&"org_id".to_string()) {
                extra.push("org_id");
            }
            if wt_cols.contains(&"project_id".to_string()) {
                extra.push("project_id");
            }
            if wt_cols.contains(&"origin_context".to_string()) {
                extra.push("origin_context");
            }
            if !extra.is_empty() {
                println!("  ⚠ Worktree has schema-only fields: {}", extra.join(", "));
            }
        }

        // Check branches
        if let Some(br_cols) = tables_by_name.get("branches") {
            if br_cols.contains(&"origin_context".to_string()) {
                println!("  ⚠ Branch.origin_context - schema-only, needs entity field");
            }
        }
    }

    #[test]
    fn test_organizations_table() {
        let schema = Schema::definition();
        let org_table = schema
            .tables
            .iter()
            .find(|t| t.name == "organizations")
            .expect("organizations table not found");

        assert_eq!(org_table.columns.len(), 6);
        let col_names: Vec<&str> = org_table.columns.iter().map(|c| c.name.as_str()).collect();
        assert_eq!(
            col_names,
            vec!["id", "name", "slug", "settings_json", "created_at", "updated_at"]
        );
        println!("✓ organizations table has correct schema");
    }

    #[test]
    fn test_users_table() {
        let schema = Schema::definition();
        let users_table = schema
            .tables
            .iter()
            .find(|t| t.name == "users")
            .expect("users table not found");

        let col_names: Vec<&str> = users_table.columns.iter().map(|c| c.name.as_str()).collect();
        assert!(col_names.contains(&"id"));
        assert!(col_names.contains(&"org_id"));
        assert!(col_names.contains(&"email"));
        assert!(col_names.contains(&"display_name"));
        assert!(col_names.contains(&"role"));
        println!("✓ users table has correct columns");
    }

    #[test]
    fn test_plans_table() {
        let schema = Schema::definition();
        let plans_table = schema
            .tables
            .iter()
            .find(|t| t.name == "plans")
            .expect("plans table not found");

        let col_names: Vec<&str> = plans_table.columns.iter().map(|c| c.name.as_str()).collect();
        assert!(col_names.contains(&"id"));
        assert!(col_names.contains(&"org_id"));
        assert!(col_names.contains(&"project_id"));
        assert!(col_names.contains(&"title"));
        assert!(col_names.contains(&"status"));
        assert!(col_names.contains(&"created_by"));
        println!("✓ plans table has correct columns");
    }

    #[test]
    fn test_project_issues_table() {
        let schema = Schema::definition();
        let issues_table = schema
            .tables
            .iter()
            .find(|t| t.name == "project_issues")
            .expect("project_issues table not found");

        let col_names: Vec<&str> = issues_table.columns.iter().map(|c| c.name.as_str()).collect();
        assert!(col_names.contains(&"id"));
        assert!(col_names.contains(&"labels"), "labels column should be Text (JSON)");
        assert!(col_names.contains(&"issue_type"));
        assert!(col_names.contains(&"status"));
        println!("✓ project_issues table has correct columns");
    }

    #[test]
    fn report_divergence_mitigation() {
        println!("\n=== DIVERGENCE MITIGATION STRATEGY ===");
        println!("\nFor each schema-only field:");
        println!("  1. Add to entity struct: pub field_name: Type");
        println!("  2. Add to derive(TableSchema) anotation");
        println!("  3. OR use extra_columns for temporary escape hatch");
        println!("\nFor each entity field not in schema:");
        println!("  1. Review if field should be persisted");
        println!("  2. If yes, add to schema definition");
        println!("  3. If no, mark with #[schema(skip)]");
    }
}
