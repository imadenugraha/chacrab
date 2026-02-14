use anyhow::{Context, Result};
use sqlx::postgres::{PgConnectOptions, PgPool, PgPoolOptions};
use sqlx::sqlite::{SqliteConnectOptions, SqlitePool, SqlitePoolOptions};
use std::str::FromStr;

/// Database connection wrapper supporting both SQLite and PostgreSQL
#[derive(Clone)]
pub struct Database {
    pool: DatabasePool,
}

/// Internal enum to hold either SQLite or PostgreSQL pool
#[derive(Clone)]
pub(crate) enum DatabasePool {
    Sqlite(SqlitePool),
    Postgres(PgPool),
}

impl Database {
    /// Get reference to the underlying pool (for SQLite)
    #[allow(dead_code)]
    pub fn sqlite_pool(&self) -> Option<&SqlitePool> {
        match &self.pool {
            DatabasePool::Sqlite(pool) => Some(pool),
            DatabasePool::Postgres(_) => None,
        }
    }

    /// Get reference to the underlying pool (for PostgreSQL)
    #[allow(dead_code)]
    pub fn postgres_pool(&self) -> Option<&PgPool> {
        match &self.pool {
            DatabasePool::Sqlite(_) => None,
            DatabasePool::Postgres(pool) => Some(pool),
        }
    }

    /// Get reference to the pool enum (internal use)
    pub(crate) fn pool(&self) -> &DatabasePool {
        &self.pool
    }
}

/// Initialize database connection from DATABASE_URL
/// Supports both SQLite (sqlite://) and PostgreSQL (postgresql:// or postgres://)
pub async fn init_db(database_url: &str) -> Result<Database> {
    if database_url.starts_with("postgres://") || database_url.starts_with("postgresql://") {
        init_postgres(database_url).await
    } else {
        init_sqlite(database_url).await
    }
}

/// Initialize SQLite database
async fn init_sqlite(database_url: &str) -> Result<Database> {
    // Parse connection options
    let options = SqliteConnectOptions::from_str(database_url)
        .context(format!(
            "Failed to parse DATABASE_URL: '{}'\n   Check that the URL format is correct (e.g., sqlite://chacrab.db)",
            database_url
        ))?
        .create_if_missing(true);

    // Create connection pool
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(options)
        .await
        .context(format!(
            "Failed to connect to database: '{}'\n   Possible issues:\n   - Check file permissions\n   - Ensure the directory exists\n   - Verify DATABASE_URL environment variable",
            database_url
        ))?;

    // Run migrations
    sqlx::migrate!("./migrations/sqlite")
        .run(&pool)
        .await
        .context("Failed to run database migrations\n   The database schema may be corrupted. Consider backing up and reinitializing.")?;

    Ok(Database {
        pool: DatabasePool::Sqlite(pool),
    })
}

/// Initialize PostgreSQL database
async fn init_postgres(database_url: &str) -> Result<Database> {
    // Parse connection options
    let options = PgConnectOptions::from_str(database_url).context(format!(
        "Failed to parse DATABASE_URL: '{}'\n   Check that the URL format is correct (e.g., postgresql://user:pass@host/db)",
        database_url
    ))?;

    // Create connection pool
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect_with(options)
        .await
        .context(format!(
            "Failed to connect to database: '{}'\n   Possible issues:\n   - Check network connectivity\n   - Verify credentials\n   - Ensure PostgreSQL server is running\n   - Verify DATABASE_URL environment variable",
            database_url
        ))?;

    // Run migrations
    sqlx::migrate!("./migrations/postgres")
        .run(&pool)
        .await
        .context("Failed to run database migrations\n   The database schema may be corrupted. Contact your database administrator.")?;

    Ok(Database {
        pool: DatabasePool::Postgres(pool),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_init_db_in_memory() {
        let db = init_db("sqlite::memory:").await;
        assert!(db.is_ok());
    }
}
