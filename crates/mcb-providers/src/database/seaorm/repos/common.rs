use mcb_domain::error::Error;
use sea_orm::DbErr;

pub(crate) fn db_err(e: DbErr) -> Error {
    Error::database_with_source("Database error", e)
}

pub(crate) fn db_error(context: &str) -> impl FnOnce(DbErr) -> Error + '_ {
    move |e| Error::database_with_source(context, e)
}
