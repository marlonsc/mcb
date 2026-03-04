//! Issue comment entity ↔ SeaORM model conversions via `impl_conversion!`.

use sea_orm::ActiveValue;

use crate::database::seaorm::entities::issue_comment;
use mcb_domain::entities::IssueComment;

crate::impl_conversion!(issue_comment, IssueComment,
    direct: [id, issue_id, author_id, content, created_at]
);
