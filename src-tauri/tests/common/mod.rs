/// Common test utilities and helpers for integration tests
use mellilex_lib::db::Database;
use tempfile::TempDir;

/// Creates a temporary database for testing
#[allow(dead_code)]
pub fn create_test_db() -> (Database, TempDir) {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let db_path = temp_dir.path().join("test.db");
    let db = Database::new(db_path).expect("Failed to create test database");
    (db, temp_dir)
}
