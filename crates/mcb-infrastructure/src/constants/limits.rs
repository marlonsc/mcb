//!
//! **Documentation**: [docs/modules/infrastructure.md](../../../../docs/modules/infrastructure.md)
//!
/// Constant value for `DEFAULT_MEMORY_LIMIT`.
pub const DEFAULT_MEMORY_LIMIT: usize = 1024 * 1024 * 1024;
/// Constant value for `DEFAULT_CPU_LIMIT`.
pub const DEFAULT_CPU_LIMIT: usize = 4;
/// Constant value for `DEFAULT_DISK_IO_LIMIT`.
pub const DEFAULT_DISK_IO_LIMIT: u64 = 100 * 1024 * 1024;
/// Constant value for `DEFAULT_MAX_CONNECTIONS`.
pub const DEFAULT_MAX_CONNECTIONS: u32 = 1000;
/// Constant value for `DEFAULT_MAX_REQUESTS_PER_CONNECTION`.
pub const DEFAULT_MAX_REQUESTS_PER_CONNECTION: u32 = 100;
