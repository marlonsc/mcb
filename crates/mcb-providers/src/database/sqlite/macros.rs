//!
//! Macros for SQLite provider (row conversion, SeaORM entities).
//!
//! **Documentation**: [docs/modules/providers.md](../../../../../../docs/modules/providers.md#database)

/// Generates a `FromRow` impl from a list of (field, extractor) pairs.
///
/// Column name is the field name. Extractors: `req_str`, `req_i64`, `req_parsed`, `opt_str`, `opt_i64`.
/// Use manual `impl FromRow` for custom logic (e.g. computed fields, JSON, custom types).
#[macro_export]
macro_rules! from_row_simple {
    ($type:ty, { $($field:ident : $ext:ident),* $(,)? }) => {
        impl $crate::database::sqlite::row_convert::FromRow for $type {
            fn from_row(row: &dyn mcb_domain::ports::SqlRow) -> mcb_domain::error::Result<Self> {
                Ok(Self {
                    $($field: $ext(row, stringify!($field))?),*
                })
            }
        }
    };
}

/// Generates a SeaORM entity from a table name and column list.
///
/// First column is the primary key. List must match `mcb_domain::schema::<table>::table()`.
/// Emits `Model`, `Relation`, `ActiveModelBehavior` and `SCHEMA_COLUMNS` (for tests).
#[macro_export]
macro_rules! sea_entity {
    ($table:expr, [ ($first:ident : $first_ty:ty), $( ($f:ident : $ty:ty) ),* $(,)? ]) => {
        #[allow(dead_code)]
        #[derive(Clone, Debug, PartialEq, Eq, sea_orm::DeriveEntityModel)]
        #[sea_orm(table_name = $table)]
        pub struct Model {
            #[sea_orm(primary_key)]
            pub $first: $first_ty,
            $( pub $f: $ty ),*
        }

        #[allow(dead_code)]
        #[derive(Copy, Clone, Debug, sea_orm::EnumIter, sea_orm::DeriveRelation)]
        pub enum Relation {}

        impl sea_orm::ActiveModelBehavior for ActiveModel {}

        #[cfg(test)]
        pub const SCHEMA_COLUMNS: &[(&str, &str)] = &[
            (stringify!($first), stringify!($first_ty)),
            $( (stringify!($f), stringify!($ty)) ),*
        ];
    };
}
