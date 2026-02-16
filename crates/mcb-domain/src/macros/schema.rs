//! DDL schema builder macros.
//!
//! Used by `schema/` modules for table, column, and index definitions.

/// Define a table schema with less boilerplate
#[macro_export]
macro_rules! table {
    ($name:expr, [ $($col:expr),* $(,)? ]) => {
        $crate::schema::memory::TableDef {
            name: $name.to_string(),
            columns: vec![ $($col),* ],
        }
    };
}

/// Define a column with less boilerplate
#[macro_export]
macro_rules! col {
    ($name:expr, $type:ident) => {
        $crate::schema::memory::ColumnDef {
            name: $name.to_string(),
            type_: $crate::schema::memory::ColumnType::$type,
            primary_key: false,
            unique: false,
            not_null: true,
            auto_increment: false,
        }
    };
    ($name:expr, $type:ident, pk) => {
        $crate::schema::memory::ColumnDef {
            name: $name.to_string(),
            type_: $crate::schema::memory::ColumnType::$type,
            primary_key: true,
            unique: false,
            not_null: true,
            auto_increment: false,
        }
    };
    ($name:expr, $type:ident, unique) => {
        $crate::schema::memory::ColumnDef {
            name: $name.to_string(),
            type_: $crate::schema::memory::ColumnType::$type,
            primary_key: false,
            unique: true,
            not_null: true,
            auto_increment: false,
        }
    };
    ($name:expr, $type:ident, nullable) => {
        $crate::schema::memory::ColumnDef {
            name: $name.to_string(),
            type_: $crate::schema::memory::ColumnType::$type,
            primary_key: false,
            unique: false,
            not_null: false,
            auto_increment: false,
        }
    };
    ($name:expr, $type:ident, auto) => {
        $crate::schema::memory::ColumnDef {
            name: $name.to_string(),
            type_: $crate::schema::memory::ColumnType::$type,
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
        $crate::schema::memory::IndexDef {
            name: $name.to_string(),
            table: $table.to_string(),
            columns: vec![ $($col.to_string()),* ],
        }
    };
}
