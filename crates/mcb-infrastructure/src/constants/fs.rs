/// Constant value for `DEFAULT_FILE_PERMISSIONS`.
pub const DEFAULT_FILE_PERMISSIONS: u32 = 0o644;
/// Constant value for `DEFAULT_DIR_PERMISSIONS`.
pub const DEFAULT_DIR_PERMISSIONS: u32 = 0o755;
/// Constant value for `MAX_SNAPSHOT_FILE_SIZE`.
pub const MAX_SNAPSHOT_FILE_SIZE: usize = 100 * 1024 * 1024;
/// Constant value for `BACKUP_FILE_EXTENSION`.
pub const BACKUP_FILE_EXTENSION: &str = ".backup";
/// Constant value for `TEMP_FILE_PREFIX`.
pub const TEMP_FILE_PREFIX: &str = "mcb_temp_";
/// Constant value for `FILESYSTEM_VECTOR_STORE_MAX_PER_SHARD`.
pub const FILESYSTEM_VECTOR_STORE_MAX_PER_SHARD: usize = 100_000;
/// Constant value for `FILESYSTEM_VECTOR_STORE_INDEX_CACHE_SIZE`.
pub const FILESYSTEM_VECTOR_STORE_INDEX_CACHE_SIZE: usize = 10_000;
/// Constant value for `FILESYSTEM_BYTES_PER_DIMENSION`.
pub const FILESYSTEM_BYTES_PER_DIMENSION: usize = 4;
