use rusqlite::{params, Connection, OptionalExtension, Result};
use std::path::Path;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct Store {
    conn: Arc<Mutex<Connection>>,
}

impl Store {
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let conn = Connection::open(path)?;
        let store = Self {
            conn: Arc::new(Mutex::new(conn)),
        };
        store.migrate()?;
        Ok(store)
    }

    pub fn memory() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        let store = Self {
            conn: Arc::new(Mutex::new(conn)),
        };
        store.migrate()?;
        Ok(store)
    }

    fn migrate(&self) -> Result<()> {
        let conn = self.conn.lock().expect("store mutex poisoned");
        conn.execute_batch(
            "
            PRAGMA foreign_keys = ON;
            CREATE TABLE IF NOT EXISTS users (
                id TEXT PRIMARY KEY,
                created_at INTEGER NOT NULL DEFAULT (unixepoch())
            );
            CREATE TABLE IF NOT EXISTS completed_puzzles (
                user_id TEXT NOT NULL,
                scenario_id TEXT NOT NULL,
                completed_at INTEGER NOT NULL DEFAULT (unixepoch()),
                PRIMARY KEY (user_id, scenario_id),
                FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
            );
            ",
        )?;
        Ok(())
    }

    pub fn ensure_user(&self, user_id: &str) -> Result<()> {
        let conn = self.conn.lock().expect("store mutex poisoned");
        conn.execute(
            "INSERT OR IGNORE INTO users (id) VALUES (?1)",
            params![user_id],
        )?;
        Ok(())
    }

    pub fn mark_completed(&self, user_id: &str, scenario_id: &str) -> Result<()> {
        self.ensure_user(user_id)?;
        let conn = self.conn.lock().expect("store mutex poisoned");
        conn.execute(
            "INSERT OR IGNORE INTO completed_puzzles (user_id, scenario_id) VALUES (?1, ?2)",
            params![user_id, scenario_id],
        )?;
        Ok(())
    }

    pub fn completed_ids(&self, user_id: &str) -> Result<Vec<String>> {
        let conn = self.conn.lock().expect("store mutex poisoned");
        let mut stmt = conn.prepare(
            "SELECT scenario_id FROM completed_puzzles WHERE user_id = ?1 ORDER BY scenario_id",
        )?;
        let ids = stmt
            .query_map(params![user_id], |row| row.get::<_, String>(0))?
            .collect();
        ids
    }

    pub fn user_exists(&self, user_id: &str) -> Result<bool> {
        let conn = self.conn.lock().expect("store mutex poisoned");
        conn.query_row(
            "SELECT 1 FROM users WHERE id = ?1",
            params![user_id],
            |_| Ok(()),
        )
        .optional()
        .map(|value| value.is_some())
    }
}
