//! Low-level `SeaORM` CRUD macros shared by repository modules.

/// Insert a domain entity into the database.
///
/// Converts `$item` via `Into<ActiveModel>`, inserts, and maps errors.
///
/// ```rust,ignore
/// sea_repo_insert!(&self.db, project_phase, phase, "create project phase")
/// ```
macro_rules! sea_repo_insert {
    ($db:expr, $mod:ident, $item:expr, $ctx:expr) => {{
        let active: $mod::ActiveModel = $item.clone().into();
        $mod::Entity::insert(active)
            .exec($db)
            .await
            .map_err(crate::database::seaorm::repos::common::db_error($ctx))?;
        Ok(())
    }};
}

/// Find a single entity by primary key and return a required result.
///
/// Returns `Error::not_found` if the entity doesn't exist.
///
/// ```rust,ignore
/// sea_repo_get!(&self.db, project_phase, ProjectPhase, "ProjectPhase", id, "get project phase")
/// ```
macro_rules! sea_repo_get {
    ($db:expr, $mod:ident, $type:ty, $label:literal, $id:expr, $ctx:expr) => {{
        let model = $mod::Entity::find_by_id($id.to_owned())
            .one($db)
            .await
            .map_err(crate::database::seaorm::repos::common::db_error($ctx))?;
        Error::not_found_or(model.map(<$type>::from), $label, $id)
    }};
}

/// Find a single entity by primary key and return an optional result.
///
/// ```rust,ignore
/// sea_repo_get_opt!(&self.db, project_phase, ProjectPhase, id, "get project phase")
/// ```
macro_rules! sea_repo_get_opt {
    ($db:expr, $mod:ident, $type:ty, $id:expr, $ctx:expr) => {{
        let model = $mod::Entity::find_by_id($id.to_owned())
            .one($db)
            .await
            .map_err(crate::database::seaorm::repos::common::db_error($ctx))?;
        Ok(model.map(<$type>::from))
    }};
}

/// Find a single entity by primary key with additional column filters.
///
/// Returns `Error::not_found` if the entity doesn't exist.
///
/// ```rust,ignore
/// sea_repo_get_filtered!(&self.db, project_issue, ProjectIssue, "ProjectIssue", id, "get issue",
///     project_issue::Column::OrgId => org_id)
/// ```
macro_rules! sea_repo_get_filtered {
    ($db:expr, $mod:ident, $type:ty, $label:literal, $id:expr, $ctx:expr, $($col:expr => $val:expr),+) => {{
        let model = $mod::Entity::find_by_id($id.to_owned())
            $(.filter($col.eq($val)))+
            .one($db)
            .await
            .map_err(crate::database::seaorm::repos::common::db_error($ctx))?;
        Error::not_found_or(model.map(<$type>::from), $label, $id)
    }};
}

/// Update a domain entity by converting to `ActiveModel`.
///
/// ```rust,ignore
/// sea_repo_update!(&self.db, project_phase, phase, "update phase")
/// ```
macro_rules! sea_repo_update {
    ($db:expr, $mod:ident, $item:expr, $ctx:expr) => {{
        let active: $mod::ActiveModel = $item.clone().into();
        active
            .update($db)
            .await
            .map_err(crate::database::seaorm::repos::common::db_error($ctx))?;
        Ok(())
    }};
}

/// Delete an entity by primary key.
///
/// ```rust,ignore
/// sea_repo_delete!(&self.db, project_phase, id, "delete phase")
/// ```
macro_rules! sea_repo_delete {
    ($db:expr, $mod:ident, $id:expr, $ctx:expr) => {{
        $mod::Entity::delete_by_id($id.to_owned())
            .exec($db)
            .await
            .map_err(crate::database::seaorm::repos::common::db_error($ctx))?;
        Ok(())
    }};
}

/// Delete an entity by primary key with additional column filters.
///
/// ```rust,ignore
/// sea_repo_delete_filtered!(&self.db, project_issue, id, "delete issue", project_issue::Column::OrgId => org_id)
/// ```
macro_rules! sea_repo_delete_filtered {
    ($db:expr, $mod:ident, $id:expr, $ctx:expr, $($col:expr => $val:expr),+) => {{
        use sea_orm::ModelTrait;
        if let Some(m) = $mod::Entity::find_by_id($id.to_owned())
            $(.filter($col.eq($val)))+
            .one($db)
            .await
            .map_err(crate::database::seaorm::repos::common::db_error($ctx))?
        {
            m.delete($db).await.map_err(crate::database::seaorm::repos::common::db_error($ctx))?;
        }
        Ok(())
    }};
}

/// List entities with optional column filters.
///
/// ```rust,ignore
/// sea_repo_list!(&self.db, project_issue, ProjectIssue, "list issues", project_issue::Column::OrgId => org_id)
/// ```
macro_rules! sea_repo_list {
    ($db:expr, $mod:ident, $type:ty, $ctx:expr $(, $col:expr => $val:expr)*) => {{
        let models = $mod::Entity::find()
            $(.filter($col.eq($val)))*
            .all($db)
            .await
            .map_err(crate::database::seaorm::repos::common::db_error($ctx))?;
        Ok(models.into_iter().map(<$type>::from).collect())
    }};
}

/// Find-by-id, set a single field, and update.
///
/// Handles the common pattern of "find -> `not_found` -> set field -> save".
///
/// ```rust,ignore
/// sea_repo_set_field!(self.db(), api_key, id, "ApiKey", "revoke api key",
///     revoked_at = Some(revoked_at))
/// ```
macro_rules! sea_repo_set_field {
    ($db:expr, $mod:ident, $id:expr, $label:literal, $ctx:expr, $field:ident = $val:expr) => {{
        use sea_orm::ActiveValue;
        let model = $mod::Entity::find_by_id($id.to_owned())
            .one($db)
            .await
            .map_err(crate::database::seaorm::repos::common::db_error($ctx))?;
        let m = Error::not_found_or(model, $label, $id)?;
        let mut active: $mod::ActiveModel = m.into();
        active.$field = ActiveValue::Set($val);
        active
            .update($db)
            .await
            .map_err(crate::database::seaorm::repos::common::db_error($ctx))?;
        Ok(())
    }};
}

/// Find a single entity by a specific column and return a required result.
///
/// Returns `Error::not_found` if the entity doesn't exist.
///
/// ```rust,ignore
/// sea_repo_find_by_column!(&self.db, project, Project, "Project", name,
///     "get project by name", project::Column::OrgId => org_id, project::Column::Name => name)
/// ```
macro_rules! sea_repo_find_by_column {
    ($db:expr, $mod:ident, $type:ty, $label:literal, $key:expr, $ctx:expr, $($col:expr => $val:expr),+) => {{
        let model = $mod::Entity::find()
            $(.filter($col.eq($val.to_owned())))+
            .one($db)
            .await
            .map_err(crate::database::seaorm::repos::common::db_error($ctx))?;
        Error::not_found_or(model.map(<$type>::from), $label, $key)
    }};
}
