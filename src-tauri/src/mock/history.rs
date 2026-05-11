use std::sync::Mutex;
use std::sync::OnceLock;

use crate::core::duckdb::DuckDBManager;
use crate::mock::error::{MockError, MockResult};
use crate::mock::models::{MockConfig, MockHistoryRecord};

const HISTORY_TABLE: &str = "_system.mock_history";
const MAX_HISTORY_ROWS: usize = 200;

static HISTORY_STORE: OnceLock<Mutex<MockHistoryStore>> = OnceLock::new();

pub struct MockHistoryStore;

impl MockHistoryStore {
    pub fn global() -> &'static Mutex<MockHistoryStore> {
        HISTORY_STORE.get_or_init(|| Mutex::new(MockHistoryStore))
    }

    fn ensure_table() -> MockResult<()> {
        let duckdb = DuckDBManager::global().get_or_create_in_memory()?;
        let conn = duckdb
            .lock()
            .map_err(|e| MockError::Generation(format!("DuckDB lock error: {}", e)))?;

        conn.execute_batch(&format!(
            "CREATE TABLE IF NOT EXISTS \"{}\" (
                id VARCHAR PRIMARY KEY,
                table_name VARCHAR NOT NULL,
                row_count INTEGER NOT NULL,
                seed BIGINT,
                config_json VARCHAR NOT NULL,
                generated_at VARCHAR NOT NULL,
                status VARCHAR NOT NULL DEFAULT 'completed'
            )",
            HISTORY_TABLE
        ))?;

        Ok(())
    }

    pub fn save(config: &MockConfig, elapsed_ms: u64) -> MockResult<MockHistoryRecord> {
        Self::ensure_table()?;

        let duckdb = DuckDBManager::global().get_or_create_in_memory()?;
        let conn = duckdb
            .lock()
            .map_err(|e| MockError::Generation(format!("DuckDB lock error: {}", e)))?;

        let now = chrono::Utc::now()
            .format("%Y-%m-%dT%H:%M:%S%.3fZ")
            .to_string();
        let id = format!("mock_{}", fast_nano_id());
        let config_json = serde_json::to_string(config).unwrap_or_else(|_| "{}".to_string());

        let record = MockHistoryRecord {
            id: id.clone(),
            table_name: config.table_name.clone(),
            row_count: config.row_count,
            seed: config.seed,
            config_json: config_json.clone(),
            generated_at: now.clone(),
            status: format!("completed ({}ms)", elapsed_ms),
        };

        conn.execute(
            &format!(
                "INSERT INTO \"{}\" (id, table_name, row_count, seed, config_json, generated_at, status) VALUES (?, ?, ?, ?, ?, ?, ?)",
                HISTORY_TABLE
            ),
            duckdb::params![
                id,
                config.table_name,
                config.row_count as i64,
                config.seed.map(|s| s as i64),
                config_json,
                now,
                record.status,
            ],
        )?;

        let count: i64 = conn.query_row(
            &format!("SELECT COUNT(*) FROM \"{}\"", HISTORY_TABLE),
            [],
            |row| row.get(0),
        )?;

        if count > MAX_HISTORY_ROWS as i64 {
            let delete_count = count - MAX_HISTORY_ROWS as i64;
            conn.execute(
                &format!(
                    "DELETE FROM \"{}\" WHERE id IN (SELECT id FROM \"{}\" ORDER BY generated_at ASC LIMIT ?)",
                    HISTORY_TABLE, HISTORY_TABLE
                ),
                duckdb::params![delete_count],
            )?;
        }

        Ok(record)
    }

    pub fn list(limit: usize) -> MockResult<Vec<MockHistoryRecord>> {
        Self::ensure_table()?;

        let duckdb = DuckDBManager::global().get_or_create_in_memory()?;
        let conn = duckdb
            .lock()
            .map_err(|e| MockError::Generation(format!("DuckDB lock error: {}", e)))?;

        let sql = format!(
            "SELECT id, table_name, row_count, seed, config_json, generated_at, status FROM \"{}\" ORDER BY generated_at DESC LIMIT {}",
            HISTORY_TABLE,
            limit.clamp(1, 500)
        );

        let mut stmt = conn.prepare(&sql)?;
        let mut rows = stmt.query([])?;
        let mut records = Vec::new();

        while let Some(row) = rows.next()? {
            let seed_val: Option<i64> = row.get(3)?;
            records.push(MockHistoryRecord {
                id: row.get(0)?,
                table_name: row.get(1)?,
                row_count: row.get::<_, i64>(2)? as usize,
                seed: seed_val.map(|s| s as u64),
                config_json: row.get(4)?,
                generated_at: row.get(5)?,
                status: row.get(6)?,
            });
        }

        Ok(records)
    }

    pub fn get_by_id(id: &str) -> MockResult<Option<MockHistoryRecord>> {
        Self::ensure_table()?;

        let duckdb = DuckDBManager::global().get_or_create_in_memory()?;
        let conn = duckdb
            .lock()
            .map_err(|e| MockError::Generation(format!("DuckDB lock error: {}", e)))?;

        let sql = format!(
            "SELECT id, table_name, row_count, seed, config_json, generated_at, status FROM \"{}\" WHERE id = ?",
            HISTORY_TABLE
        );

        let mut stmt = conn.prepare(&sql)?;
        let mut rows = stmt.query(duckdb::params![id])?;

        if let Some(row) = rows.next()? {
            let seed_val: Option<i64> = row.get(3)?;
            Ok(Some(MockHistoryRecord {
                id: row.get(0)?,
                table_name: row.get(1)?,
                row_count: row.get::<_, i64>(2)? as usize,
                seed: seed_val.map(|s| s as u64),
                config_json: row.get(4)?,
                generated_at: row.get(5)?,
                status: row.get(6)?,
            }))
        } else {
            Ok(None)
        }
    }

    pub fn clear() -> MockResult<usize> {
        Self::ensure_table()?;

        let duckdb = DuckDBManager::global().get_or_create_in_memory()?;
        let conn = duckdb
            .lock()
            .map_err(|e| MockError::Generation(format!("DuckDB lock error: {}", e)))?;

        let count: i64 = conn.query_row(
            &format!("SELECT COUNT(*) FROM \"{}\"", HISTORY_TABLE),
            [],
            |row| row.get(0),
        )?;

        conn.execute_batch(&format!("DELETE FROM \"{}\"", HISTORY_TABLE))?;

        Ok(count as usize)
    }
}

fn fast_nano_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    format!("{:x}", ts)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fast_nano_id_is_hex() {
        let id = fast_nano_id();
        assert!(!id.is_empty());
        assert!(id.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_fast_nano_id_unique() {
        let id1 = fast_nano_id();
        std::thread::sleep(std::time::Duration::from_micros(10));
        let id2 = fast_nano_id();
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_history_record_construction() {
        let record = MockHistoryRecord {
            id: "mock_abc123".to_string(),
            table_name: "users".to_string(),
            row_count: 1000,
            seed: Some(42),
            config_json: r#"{"table_name":"users"}"#.to_string(),
            generated_at: "2026-05-10T10:00:00.000Z".to_string(),
            status: "completed (1500ms)".to_string(),
        };
        assert_eq!(record.id, "mock_abc123");
        assert_eq!(record.table_name, "users");
        assert_eq!(record.row_count, 1000);
        assert_eq!(record.seed, Some(42));
    }

    #[test]
    fn test_clamp_enforces_bounds() {
        assert_eq!(0usize.clamp(1, 500), 1);
        assert_eq!(1000usize.clamp(1, 500), 500);
        assert_eq!(100usize.clamp(1, 500), 100);
    }

    #[test]
    fn test_max_history_rows_constant() {
        assert_eq!(MAX_HISTORY_ROWS, 200);
    }
}
