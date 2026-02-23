//!
//! SeaORM entities for SQLite tables, generated from domain schema.
//!
//! Column lists stay in sync with `mcb_domain::schema::*::table()`; tests enforce it.

#[allow(dead_code)]
pub mod organization;

#[cfg(test)]
mod schema_sync_tests {
    use mcb_domain::schema::Schema;
    use mcb_domain::schema::types::ColumnType;

    fn column_type_to_rust(t: &ColumnType) -> &'static str {
        match t {
            ColumnType::Text => "String",
            ColumnType::Integer => "i64",
            ColumnType::Real => "f64",
            ColumnType::Boolean => "bool",
            ColumnType::Blob => "Vec<u8>",
            ColumnType::Json => "String",
            ColumnType::Uuid => "String",
            ColumnType::Timestamp => "i64",
        }
    }

    #[test]
    fn organizations_entity_matches_domain_schema() {
        let schema = Schema::definition();
        let table = schema
            .tables
            .iter()
            .find(|t| t.name == "organizations")
            .expect("organizations table in schema");
        let expected: Vec<(&str, &str)> = table
            .columns
            .iter()
            .map(|c| (c.name.as_str(), column_type_to_rust(&c.type_)))
            .collect();
        let actual = super::organization::SCHEMA_COLUMNS;
        assert_eq!(
            expected.len(),
            actual.len(),
            "column count must match schema"
        );
        for (i, (name, ty)) in expected.iter().enumerate() {
            assert_eq!(
                (*name, *ty),
                (actual[i].0, actual[i].1),
                "column {} must match schema",
                i
            );
        }
    }
}
