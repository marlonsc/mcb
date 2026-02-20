//! DDL schema builder macros.
//!
//! Used by `schema/` modules for table, column, and index definitions.

/// Define a table schema with less boilerplate
#[macro_export]
macro_rules! table {
    ($name:expr, [ $($col:expr),* $(,)? ]) => {
        $crate::schema::types::TableDef {
            name: $name.to_string(),
            columns: vec![ $($col),* ],
        }
    };
}

/// Define a column with less boilerplate
#[macro_export]
macro_rules! col {
    ($name:expr, $type:ident) => {
        $crate::schema::types::ColumnDef {
            name: $name.to_string(),
            type_: $crate::schema::types::ColumnType::$type,
            primary_key: false,
            unique: false,
            not_null: true,
            auto_increment: false,
        }
    };
    ($name:expr, $type:ident, pk) => {
        $crate::schema::types::ColumnDef {
            name: $name.to_string(),
            type_: $crate::schema::types::ColumnType::$type,
            primary_key: true,
            unique: false,
            not_null: true,
            auto_increment: false,
        }
    };
    ($name:expr, $type:ident, unique) => {
        $crate::schema::types::ColumnDef {
            name: $name.to_string(),
            type_: $crate::schema::types::ColumnType::$type,
            primary_key: false,
            unique: true,
            not_null: true,
            auto_increment: false,
        }
    };
    ($name:expr, $type:ident, nullable) => {
        $crate::schema::types::ColumnDef {
            name: $name.to_string(),
            type_: $crate::schema::types::ColumnType::$type,
            primary_key: false,
            unique: false,
            not_null: false,
            auto_increment: false,
        }
    };
    ($name:expr, $type:ident, auto) => {
        $crate::schema::types::ColumnDef {
            name: $name.to_string(),
            type_: $crate::schema::types::ColumnType::$type,
            primary_key: true,
            unique: false,
            not_null: true,
            auto_increment: true,
        }
    };
}

/// Define an index with less boilerplate
#[macro_export]
macro_rules! index {
    ($name:expr, $table:expr, [ $($col:expr),* $(,)? ]) => {
        $crate::schema::types::IndexDef {
            name: $name.to_string(),
            table: $table.to_string(),
            columns: vec![ $($col.to_string()),* ],
        }
    };
}

/// Define a foreign key with less boilerplate.
#[macro_export]
macro_rules! fk {
    ($from_table:expr, $from_col:expr, $to_table:expr, $to_col:expr) => {
        $crate::schema::types::ForeignKeyDef {
            from_table: $from_table.to_string(),
            from_column: $from_col.to_string(),
            to_table: $to_table.to_string(),
            to_column: $to_col.to_string(),
        }
    };
}

/// Define a table-level unique constraint with less boilerplate.
#[macro_export]
macro_rules! unique {
    ($table:expr, [ $($col:expr),* $(,)? ]) => {
        $crate::schema::types::UniqueConstraintDef {
            table: $table.to_string(),
            columns: vec![ $($col.to_string()),* ],
        }
    };
}
