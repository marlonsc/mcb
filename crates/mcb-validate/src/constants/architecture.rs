//! Architecture path fragment constants.
//!
//! Shared path patterns used by organization, naming, and documentation
//! validators to identify architectural layers and directory structure.

/// Path fragment identifying the ports directory.
pub const ARCH_PATH_PORTS: &str = "/ports/";

/// Path fragment identifying the ports/providers directory.
pub const ARCH_PATH_PORTS_PROVIDERS: &str = "/ports/providers/";

/// Path fragment identifying the DI directory.
pub const ARCH_PATH_DI: &str = "/di/";

/// Path fragment identifying the DI modules directory.
pub const ARCH_PATH_DI_MODULES: &str = "/di/modules/";

/// Path fragment identifying the handlers directory.
pub const ARCH_PATH_HANDLERS: &str = "/handlers/";

/// Path fragment identifying the admin directory.
pub const ARCH_PATH_ADMIN: &str = "/admin/";

/// Path fragment identifying the server layer.
pub const ARCH_PATH_SERVER: &str = "/server/";

/// Path fragment identifying the application layer.
pub const ARCH_PATH_APPLICATION: &str = "/application/";

/// Path fragment identifying the infrastructure layer.
pub const ARCH_PATH_INFRASTRUCTURE: &str = "/infrastructure/";
