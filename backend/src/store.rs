use anyhow::{Context, Result};
use chrono::Utc;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::SqlitePool;
use std::path::Path;
use std::str::FromStr;

#[derive(Clone)]
pub struct Store {
    pool: SqlitePool,
}

impl Store {
    pub async fn open(path: &str) -> Result<Self> {
        if let Some(parent) = Path::new(path).parent() {
            if !parent.as_os_str().is_empty() {
                std::fs::create_dir_all(parent).ok();
            }
        }
        let options = SqliteConnectOptions::from_str(&format!("sqlite://{}", path))
            .unwrap_or_else(|_| SqliteConnectOptions::new().filename(path))
            .create_if_missing(true)
            .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal)
            .busy_timeout(std::time::Duration::from_secs(5));
        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect_with(options)
            .await
            .context("abrir SQLite")?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS analyses (
                query_key TEXT PRIMARY KEY,
                query TEXT NOT NULL,
                created_at INTEGER NOT NULL,
                payload TEXT NOT NULL
            )
            "#,
        )
        .execute(&pool)
        .await?;

        Ok(Self { pool })
    }

    pub async fn get(&self, key: &str, ttl_secs: i64) -> Result<Option<String>> {
        let row: Option<(String, i64)> =
            sqlx::query_as("SELECT payload, created_at FROM analyses WHERE query_key = ?")
                .bind(key)
                .fetch_optional(&self.pool)
                .await?;
        let Some((payload, created_at)) = row else {
            return Ok(None);
        };
        if Utc::now().timestamp() - created_at > ttl_secs {
            return Ok(None);
        }
        Ok(Some(payload))
    }

    pub async fn put(&self, key: &str, query: &str, payload: &str) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO analyses (query_key, query, created_at, payload)
            VALUES (?, ?, ?, ?)
            ON CONFLICT(query_key) DO UPDATE SET
                query = excluded.query,
                created_at = excluded.created_at,
                payload = excluded.payload
            "#,
        )
        .bind(key)
        .bind(query)
        .bind(Utc::now().timestamp())
        .bind(payload)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn count(&self) -> Result<i64> {
        let row: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM analyses")
            .fetch_one(&self.pool)
            .await?;
        Ok(row.0)
    }
}
