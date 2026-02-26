use mcb_domain::error::Error;
use sea_orm::DbErr;

pub(crate) fn db_err(e: DbErr) -> Error {
    Error::Database {
        message: "Database error".into(),
        source: Some(Box::new(e)),
    }
}

pub(crate) fn db_error(context: &str) -> impl FnOnce(DbErr) -> Error + '_ {
    move |e| Error::Database {
        message: context.to_owned(),
        source: Some(Box::new(e)),
    }
}
