#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use mcb_infrastructure::config::{AuthConfig, DatabaseConfigContainer};

    #[test]
    fn test_database_config_container_path() {
        let toml = r#"
        provider = "sqlite"

        [configs.default]
        provider = "sqlite"
        path = "/tmp/mcb-test.db"
        "#;

        let config: DatabaseConfigContainer = toml::from_str(toml).expect("parse database config");
        let default = config
            .configs
            .get("default")
            .expect("default database config");

        assert_eq!(config.provider, "sqlite");
        assert_eq!(default.provider, "sqlite");
        assert_eq!(default.path, Some(PathBuf::from("/tmp/mcb-test.db")));
    }

    #[test]
    fn test_auth_config_deserialization_without_user_db_path() {
        let toml = r#"
        enabled = false
        password_algorithm = "Argon2"

        [jwt]
        secret = ""
        expiration_secs = 86400
        refresh_expiration_secs = 604800

        [api_key]
        enabled = true
        header = "X-API-Key"

        [admin]
        enabled = false
        header = "X-Admin-Key"
        "#;

        let config: AuthConfig = toml::from_str(toml).expect("parse auth config");
        assert!(!config.enabled);
    }
}
