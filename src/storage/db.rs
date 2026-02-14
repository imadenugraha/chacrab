use anyhow::{Context, Result};
use sqlx::sqlite::{SqliteConnectOptions, SqlitePool, SqlitePoolOptions};
use std::str::FromStr;

/// Database connection wrapper
#[derive(Clone)]
pub struct Database {
    pool: SqlitePool,
}

impl Database {
    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }
}

/// Initialize database connection from DATABASE_URL
pub async fn init_db(database_url: &str) -> Result<Database> {
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
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .context("Failed to run database migrations\n   The database schema may be corrupted. Consider backing up and reinitializing.")?;

    Ok(Database { pool })
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
