//!
//! **Documentation**: [docs/modules/domain.md](../../../../docs/modules/domain.md)
//!
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

/// Define multiple indexes for a table in one go.
/// Define multiple indexes for a single table with one macro call.
#[macro_export]
macro_rules! indexes_for_table {
    ($table:expr, { $($name:expr => [ $($col:expr),* $(,)? ]),+ $(,)? }) => {
        vec![
            $(
                $crate::index!($name, $table, [$($col),*]),
            )+
        ]
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

/// Implement [`HasTableSchema`] for an entity type using compact column specs.
///
/// Co-locate this invocation in the entity file so the schema lives next to
/// the struct it describes. The macro reuses the existing `col!`, `table!`,
/// `index!`, `fk!` and `unique!` helpers internally.
///
/// # Example
///
/// ```ignore
/// impl_table_schema!(Organization, "organizations",
///     columns: [
///         ("id", Text, pk),
///         ("name", Text),
///         ("slug", Text, unique),
///         ("settings_json", Text),
///         ("created_at", Integer),
///         ("updated_at", Integer),
///     ],
///     indexes: [
///         "idx_organizations_name" => ["name"],
///     ],
/// );
/// ```
#[macro_export]
macro_rules! impl_table_schema {
    // Full form: columns + indexes + foreign_keys + unique_constraints
    ($entity:ty, $table_name:expr,
        columns: [ $( ($col_name:expr, $col_type:ident $(, $flag:ident)?) ),* $(,)? ],
        indexes: [ $( $idx_name:expr => [ $($idx_col:expr),* $(,)? ] ),* $(,)? ],
        foreign_keys: [ $( ($fk_col:expr, $fk_table:expr, $fk_ref:expr) ),* $(,)? ],
        unique_constraints: [ $( [ $($uc_col:expr),* $(,)? ] ),* $(,)? ],
    ) => {
        impl $crate::schema::types::HasTableSchema for $entity {
            fn table_def() -> $crate::schema::types::TableDef {
                $crate::table!($table_name, [
                    $( $crate::col!($col_name, $col_type $(, $flag)?) ),*
                ])
            }

            fn indexes() -> Vec<$crate::schema::types::IndexDef> {
                vec![
                    $( $crate::index!($idx_name, $table_name, [ $($idx_col),* ]) ),*
                ]
            }

            fn foreign_keys() -> Vec<$crate::schema::types::ForeignKeyDef> {
                vec![
                    $( $crate::fk!($table_name, $fk_col, $fk_table, $fk_ref) ),*
                ]
            }

            fn unique_constraints() -> Vec<$crate::schema::types::UniqueConstraintDef> {
                vec![
                    $( $crate::unique!($table_name, [ $($uc_col),* ]) ),*
                ]
            }
        }
    };
    // Shorthand: columns only (no indexes, no FKs, no UCs)
    ($entity:ty, $table_name:expr,
        columns: [ $( ($col_name:expr, $col_type:ident $(, $flag:ident)?) ),* $(,)? ],
    ) => {
        $crate::impl_table_schema!($entity, $table_name,
            columns: [ $( ($col_name, $col_type $(, $flag)?) ),* ],
            indexes: [],
            foreign_keys: [],
            unique_constraints: [],
        );
    };
    // Shorthand: columns + indexes (no FKs, no UCs)
    ($entity:ty, $table_name:expr,
        columns: [ $( ($col_name:expr, $col_type:ident $(, $flag:ident)?) ),* $(,)? ],
        indexes: [ $( $idx_name:expr => [ $($idx_col:expr),* $(,)? ] ),* $(,)? ],
    ) => {
        $crate::impl_table_schema!($entity, $table_name,
            columns: [ $( ($col_name, $col_type $(, $flag)?) ),* ],
            indexes: [ $( $idx_name => [ $($idx_col),* ] ),* ],
            foreign_keys: [],
            unique_constraints: [],
        );
    };
    // Shorthand: columns + foreign_keys (no indexes, no UCs)
    ($entity:ty, $table_name:expr,
        columns: [ $( ($col_name:expr, $col_type:ident $(, $flag:ident)?) ),* $(,)? ],
        foreign_keys: [ $( ($fk_col:expr, $fk_table:expr, $fk_ref:expr) ),* $(,)? ],
    ) => {
        $crate::impl_table_schema!($entity, $table_name,
            columns: [ $( ($col_name, $col_type $(, $flag)?) ),* ],
            indexes: [],
            foreign_keys: [ $( ($fk_col, $fk_table, $fk_ref) ),* ],
            unique_constraints: [],
        );
    };
    // Shorthand: columns + indexes + foreign_keys (no UCs)
}
