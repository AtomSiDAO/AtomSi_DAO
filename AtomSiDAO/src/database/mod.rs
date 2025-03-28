//! Database module for AtomSi DAO
//!
//! This module provides database connectivity and management functionality.

use async_trait::async_trait;
use sqlx::{
    postgres::{PgPool, PgPoolOptions},
    sqlite::{SqlitePool, SqlitePoolOptions},
    Pool, Postgres, Sqlite,
};
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::config::DatabaseConfig;
use crate::error::{Error, Result};

/// Database driver types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DatabaseType {
    /// SQLite database
    SQLite,
    /// PostgreSQL database
    Postgres,
}

/// Database connection manager
#[derive(Clone)]
pub struct DatabaseManager {
    /// Database type
    db_type: DatabaseType,
    /// SQLite pool
    sqlite_pool: Option<SqlitePool>,
    /// PostgreSQL pool
    pg_pool: Option<PgPool>,
    /// Configuration
    config: Arc<DatabaseConfig>,
    /// Mutex for database initialization
    init_mutex: Arc<Mutex<()>>,
}

impl DatabaseManager {
    /// Create a new database manager
    pub async fn new(config: &DatabaseConfig) -> Result<Self> {
        let db_type = match config.db_type.as_str() {
            "sqlite" => DatabaseType::SQLite,
            "postgres" => DatabaseType::Postgres,
            _ => {
                return Err(Error::DatabaseError(format!(
                    "Unsupported database type: {}",
                    config.db_type
                )))
            }
        };

        let mut manager = Self {
            db_type,
            sqlite_pool: None,
            pg_pool: None,
            config: Arc::new(config.clone()),
            init_mutex: Arc::new(Mutex::new(())),
        };

        // Connect to the database
        manager.connect().await?;

        Ok(manager)
    }

    /// Connect to the database
    async fn connect(&mut self) -> Result<()> {
        match self.db_type {
            DatabaseType::SQLite => {
                let sqlite_path = self
                    .config
                    .sqlite_path
                    .as_ref()
                    .ok_or_else(|| Error::DatabaseError("SQLite path not configured".to_string()))?;

                let pool = SqlitePoolOptions::new()
                    .max_connections(self.config.pool_size)
                    .connect(sqlite_path)
                    .await
                    .map_err(|e| Error::DatabaseError(format!("Failed to connect to SQLite: {}", e)))?;

                self.sqlite_pool = Some(pool);
            }
            DatabaseType::Postgres => {
                let connection_string = format!(
                    "postgres://{}:{}@{}:{}/{}",
                    self.config.username,
                    self.config.password,
                    self.config.host,
                    self.config.port,
                    self.config.name
                );

                let pool = PgPoolOptions::new()
                    .max_connections(self.config.pool_size)
                    .connect(&connection_string)
                    .await
                    .map_err(|e| Error::DatabaseError(format!("Failed to connect to PostgreSQL: {}", e)))?;

                self.pg_pool = Some(pool);
            }
        }

        Ok(())
    }

    /// Get the database type
    pub fn db_type(&self) -> DatabaseType {
        self.db_type
    }

    /// Get the SQLite pool
    pub fn sqlite_pool(&self) -> Result<&SqlitePool> {
        self.sqlite_pool
            .as_ref()
            .ok_or_else(|| Error::DatabaseError("SQLite pool not initialized".to_string()))
    }

    /// Get the PostgreSQL pool
    pub fn pg_pool(&self) -> Result<&PgPool> {
        self.pg_pool
            .as_ref()
            .ok_or_else(|| Error::DatabaseError("PostgreSQL pool not initialized".to_string()))
    }

    /// Initialize the database
    pub async fn init_db(&self) -> Result<()> {
        // Use a mutex to prevent multiple initialization attempts
        let _lock = self.init_mutex.lock().await;

        match self.db_type {
            DatabaseType::SQLite => {
                let pool = self.sqlite_pool()?;
                init_sqlite_db(pool).await?;
            }
            DatabaseType::Postgres => {
                let pool = self.pg_pool()?;
                init_postgres_db(pool).await?;
            }
        }

        Ok(())
    }

    /// Close the database connection
    pub async fn close(&self) -> Result<()> {
        match self.db_type {
            DatabaseType::SQLite => {
                if let Some(pool) = &self.sqlite_pool {
                    pool.close().await;
                }
            }
            DatabaseType::Postgres => {
                if let Some(pool) = &self.pg_pool {
                    pool.close().await;
                }
            }
        }

        Ok(())
    }

    /// Execute a database migration
    pub async fn migrate(&self) -> Result<()> {
        match self.db_type {
            DatabaseType::SQLite => {
                let pool = self.sqlite_pool()?;
                sqlx::migrate!("./migrations/sqlite")
                    .run(pool)
                    .await
                    .map_err(|e| Error::DatabaseError(format!("Failed to run SQLite migrations: {}", e)))?;
            }
            DatabaseType::Postgres => {
                let pool = self.pg_pool()?;
                sqlx::migrate!("./migrations/postgres")
                    .run(pool)
                    .await
                    .map_err(|e| Error::DatabaseError(format!("Failed to run PostgreSQL migrations: {}", e)))?;
            }
        }

        Ok(())
    }
}

/// Initialize SQLite database
async fn init_sqlite_db(pool: &SqlitePool) -> Result<()> {
    // Create tables if they don't exist
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS dao_info (
            id INTEGER PRIMARY KEY,
            name TEXT NOT NULL,
            description TEXT,
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL
        );

        CREATE TABLE IF NOT EXISTS members (
            id TEXT PRIMARY KEY,
            address TEXT NOT NULL UNIQUE,
            name TEXT,
            role TEXT NOT NULL,
            status TEXT NOT NULL,
            reputation INTEGER NOT NULL DEFAULT 0,
            joined_at INTEGER NOT NULL,
            last_active_at INTEGER NOT NULL,
            metadata TEXT
        );

        CREATE TABLE IF NOT EXISTS proposals (
            id TEXT PRIMARY KEY,
            title TEXT NOT NULL,
            description TEXT NOT NULL,
            proposer_id TEXT NOT NULL,
            proposal_type TEXT NOT NULL,
            status TEXT NOT NULL,
            created_at INTEGER NOT NULL,
            voting_starts_at INTEGER NOT NULL,
            voting_ends_at INTEGER NOT NULL,
            executed_at INTEGER,
            metadata TEXT,
            FOREIGN KEY (proposer_id) REFERENCES members(id)
        );

        CREATE TABLE IF NOT EXISTS votes (
            id TEXT PRIMARY KEY,
            proposal_id TEXT NOT NULL,
            voter_id TEXT NOT NULL,
            vote_type TEXT NOT NULL,
            vote_weight INTEGER NOT NULL,
            voted_at INTEGER NOT NULL,
            metadata TEXT,
            FOREIGN KEY (proposal_id) REFERENCES proposals(id),
            FOREIGN KEY (voter_id) REFERENCES members(id)
        );

        CREATE TABLE IF NOT EXISTS transactions (
            id TEXT PRIMARY KEY,
            description TEXT,
            recipient TEXT NOT NULL,
            token_symbol TEXT NOT NULL,
            amount TEXT NOT NULL,
            status TEXT NOT NULL,
            required_approvals INTEGER NOT NULL,
            created_at INTEGER NOT NULL,
            executed_at INTEGER,
            metadata TEXT
        );

        CREATE TABLE IF NOT EXISTS transaction_approvals (
            transaction_id TEXT NOT NULL,
            approver_id TEXT NOT NULL,
            approved_at INTEGER NOT NULL,
            PRIMARY KEY (transaction_id, approver_id),
            FOREIGN KEY (transaction_id) REFERENCES transactions(id),
            FOREIGN KEY (approver_id) REFERENCES members(id)
        );

        CREATE TABLE IF NOT EXISTS tokens (
            symbol TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            decimals INTEGER NOT NULL,
            total_supply TEXT NOT NULL,
            contract_address TEXT,
            created_at INTEGER NOT NULL,
            metadata TEXT
        );

        CREATE TABLE IF NOT EXISTS token_balances (
            holder_id TEXT NOT NULL,
            token_symbol TEXT NOT NULL,
            balance TEXT NOT NULL,
            updated_at INTEGER NOT NULL,
            PRIMARY KEY (holder_id, token_symbol),
            FOREIGN KEY (holder_id) REFERENCES members(id),
            FOREIGN KEY (token_symbol) REFERENCES tokens(symbol)
        );

        CREATE TABLE IF NOT EXISTS activities (
            id TEXT PRIMARY KEY,
            member_id TEXT NOT NULL,
            activity_type TEXT NOT NULL,
            related_id TEXT,
            timestamp INTEGER NOT NULL,
            description TEXT,
            reputation_change INTEGER NOT NULL DEFAULT 0,
            metadata TEXT,
            FOREIGN KEY (member_id) REFERENCES members(id)
        );
        "#,
    )
    .execute(pool)
    .await
    .map_err(|e| Error::DatabaseError(format!("Failed to initialize SQLite database: {}", e)))?;

    Ok(())
}

/// Initialize PostgreSQL database
async fn init_postgres_db(pool: &PgPool) -> Result<()> {
    // Create tables if they don't exist
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS dao_info (
            id SERIAL PRIMARY KEY,
            name TEXT NOT NULL,
            description TEXT,
            created_at BIGINT NOT NULL,
            updated_at BIGINT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS members (
            id TEXT PRIMARY KEY,
            address TEXT NOT NULL UNIQUE,
            name TEXT,
            role TEXT NOT NULL,
            status TEXT NOT NULL,
            reputation INTEGER NOT NULL DEFAULT 0,
            joined_at BIGINT NOT NULL,
            last_active_at BIGINT NOT NULL,
            metadata JSONB
        );

        CREATE TABLE IF NOT EXISTS proposals (
            id TEXT PRIMARY KEY,
            title TEXT NOT NULL,
            description TEXT NOT NULL,
            proposer_id TEXT NOT NULL,
            proposal_type TEXT NOT NULL,
            status TEXT NOT NULL,
            created_at BIGINT NOT NULL,
            voting_starts_at BIGINT NOT NULL,
            voting_ends_at BIGINT NOT NULL,
            executed_at BIGINT,
            metadata JSONB,
            FOREIGN KEY (proposer_id) REFERENCES members(id)
        );

        CREATE TABLE IF NOT EXISTS votes (
            id TEXT PRIMARY KEY,
            proposal_id TEXT NOT NULL,
            voter_id TEXT NOT NULL,
            vote_type TEXT NOT NULL,
            vote_weight BIGINT NOT NULL,
            voted_at BIGINT NOT NULL,
            metadata JSONB,
            FOREIGN KEY (proposal_id) REFERENCES proposals(id),
            FOREIGN KEY (voter_id) REFERENCES members(id)
        );

        CREATE TABLE IF NOT EXISTS transactions (
            id TEXT PRIMARY KEY,
            description TEXT,
            recipient TEXT NOT NULL,
            token_symbol TEXT NOT NULL,
            amount TEXT NOT NULL,
            status TEXT NOT NULL,
            required_approvals INTEGER NOT NULL,
            created_at BIGINT NOT NULL,
            executed_at BIGINT,
            metadata JSONB
        );

        CREATE TABLE IF NOT EXISTS transaction_approvals (
            transaction_id TEXT NOT NULL,
            approver_id TEXT NOT NULL,
            approved_at BIGINT NOT NULL,
            PRIMARY KEY (transaction_id, approver_id),
            FOREIGN KEY (transaction_id) REFERENCES transactions(id),
            FOREIGN KEY (approver_id) REFERENCES members(id)
        );

        CREATE TABLE IF NOT EXISTS tokens (
            symbol TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            decimals INTEGER NOT NULL,
            total_supply TEXT NOT NULL,
            contract_address TEXT,
            created_at BIGINT NOT NULL,
            metadata JSONB
        );

        CREATE TABLE IF NOT EXISTS token_balances (
            holder_id TEXT NOT NULL,
            token_symbol TEXT NOT NULL,
            balance TEXT NOT NULL,
            updated_at BIGINT NOT NULL,
            PRIMARY KEY (holder_id, token_symbol),
            FOREIGN KEY (holder_id) REFERENCES members(id),
            FOREIGN KEY (token_symbol) REFERENCES tokens(symbol)
        );

        CREATE TABLE IF NOT EXISTS activities (
            id TEXT PRIMARY KEY,
            member_id TEXT NOT NULL,
            activity_type TEXT NOT NULL,
            related_id TEXT,
            timestamp BIGINT NOT NULL,
            description TEXT,
            reputation_change INTEGER NOT NULL DEFAULT 0,
            metadata JSONB,
            FOREIGN KEY (member_id) REFERENCES members(id)
        );
        "#,
    )
    .execute(pool)
    .await
    .map_err(|e| Error::DatabaseError(format!("Failed to initialize PostgreSQL database: {}", e)))?;

    Ok(())
}

/// Database store trait
#[async_trait]
pub trait DatabaseStore: Send + Sync + 'static {
    /// Get the database manager
    fn get_db_manager(&self) -> &DatabaseManager;
}

/// Generic database store implementation
pub struct Store<T> {
    /// Database manager
    pub db_manager: DatabaseManager,
    /// Store-specific data
    pub data: T,
}

impl<T> Store<T> {
    /// Create a new store
    pub fn new(db_manager: DatabaseManager, data: T) -> Self {
        Self { db_manager, data }
    }
}

#[async_trait]
impl<T: Send + Sync + 'static> DatabaseStore for Store<T> {
    fn get_db_manager(&self) -> &DatabaseManager {
        &self.db_manager
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::DatabaseConfig;

    #[tokio::test]
    async fn test_sqlite_connection() {
        let config = DatabaseConfig {
            db_type: "sqlite".to_string(),
            host: "localhost".to_string(),
            port: 0,
            name: "test_db".to_string(),
            username: "".to_string(),
            password: "".to_string(),
            pool_size: 5,
            sqlite_path: Some("file::memory:".to_string()),
        };

        let db_manager = DatabaseManager::new(&config).await.unwrap();
        assert_eq!(db_manager.db_type(), DatabaseType::SQLite);

        // Initialize the database
        db_manager.init_db().await.unwrap();

        // Close the connection
        db_manager.close().await.unwrap();
    }
} 