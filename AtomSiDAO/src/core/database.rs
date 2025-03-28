//! Database module for AtomSi DAO
//!
//! This module provides database connectivity and operations for AtomSi DAO.

use crate::config::DatabaseConfig;
use crate::core::error::{DaoError, Result};
use deadpool_postgres::{Config, Pool, PoolConfig, Runtime};
use std::sync::Arc;
use tokio_postgres::NoTls;

/// Database connection pool
#[derive(Clone)]
pub struct Database {
    /// Connection pool
    pool: Arc<Pool>,
}

impl Database {
    /// Connect to the database
    pub async fn connect(config: &DatabaseConfig) -> Result<Self> {
        let mut pg_config = Config::new();
        
        // Parse database URL and set connection parameters
        let db_url = &config.url;
        let parsed_url = url::Url::parse(db_url)
            .map_err(|e| DaoError::DatabaseError(format!("Invalid database URL: {}", e)))?;
        
        let host = parsed_url.host_str()
            .ok_or_else(|| DaoError::DatabaseError("Missing host in database URL".to_string()))?;
        
        let port = parsed_url.port().unwrap_or(5432);
        
        let path = parsed_url.path();
        let db_name = if path.len() > 1 {
            &path[1..]
        } else {
            "postgres"
        };
        
        let username = parsed_url.username();
        let password = parsed_url.password().unwrap_or("");
        
        pg_config.host = Some(host.to_string());
        pg_config.port = Some(port);
        pg_config.dbname = Some(db_name.to_string());
        pg_config.user = Some(username.to_string());
        pg_config.password = Some(password.to_string());
        
        // Configure connection pool
        let pool_config = PoolConfig::new(config.max_connections as usize);
        pg_config.pool = pool_config;
        
        // Create connection pool
        let pool = pg_config
            .create_pool(Some(Runtime::Tokio1), NoTls)
            .map_err(|e| DaoError::DatabaseError(format!("Failed to create connection pool: {}", e)))?;
        
        // Test connection
        let client = pool
            .get()
            .await
            .map_err(|e| DaoError::DatabaseError(format!("Failed to connect to database: {}", e)))?;
        
        client
            .query("SELECT 1", &[])
            .await
            .map_err(|e| DaoError::DatabaseError(format!("Failed to query database: {}", e)))?;
        
        Ok(Self {
            pool: Arc::new(pool),
        })
    }
    
    /// Get a connection from the pool
    pub async fn get_client(&self) -> Result<deadpool_postgres::Client> {
        self.pool
            .get()
            .await
            .map_err(|e| DaoError::DatabaseError(format!("Failed to get database connection: {}", e)))
    }
    
    /// Execute a query and return the number of rows affected
    pub async fn execute(&self, query: &str, params: &[&(dyn tokio_postgres::types::ToSql + Sync)]) -> Result<u64> {
        let client = self.get_client().await?;
        
        client
            .execute(query, params)
            .await
            .map_err(|e| DaoError::DatabaseError(format!("Failed to execute query: {}", e)))
    }
    
    /// Execute a query and return the rows
    pub async fn query(&self, query: &str, params: &[&(dyn tokio_postgres::types::ToSql + Sync)]) -> Result<Vec<tokio_postgres::Row>> {
        let client = self.get_client().await?;
        
        client
            .query(query, params)
            .await
            .map_err(|e| DaoError::DatabaseError(format!("Failed to execute query: {}", e)))
    }
    
    /// Execute a query and return the first row, if any
    pub async fn query_opt(&self, query: &str, params: &[&(dyn tokio_postgres::types::ToSql + Sync)]) -> Result<Option<tokio_postgres::Row>> {
        let client = self.get_client().await?;
        
        client
            .query_opt(query, params)
            .await
            .map_err(|e| DaoError::DatabaseError(format!("Failed to execute query: {}", e)))
    }
    
    /// Execute a query and return the first row
    pub async fn query_one(&self, query: &str, params: &[&(dyn tokio_postgres::types::ToSql + Sync)]) -> Result<tokio_postgres::Row> {
        let client = self.get_client().await?;
        
        client
            .query_one(query, params)
            .await
            .map_err(|e| DaoError::DatabaseError(format!("Failed to execute query: {}", e)))
    }
    
    /// Execute a transaction
    pub async fn transaction<F, R>(&self, f: F) -> Result<R>
    where
        F: for<'a> FnOnce(&'a mut deadpool_postgres::Transaction<'_>) -> futures_util::future::BoxFuture<'a, Result<R>>,
    {
        let client = self.get_client().await?;
        
        let mut tx = client
            .transaction()
            .await
            .map_err(|e| DaoError::DatabaseError(format!("Failed to start transaction: {}", e)))?;
        
        let result = f(&mut tx).await?;
        
        tx.commit()
            .await
            .map_err(|e| DaoError::DatabaseError(format!("Failed to commit transaction: {}", e)))?;
        
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config;
    
    #[tokio::test]
    async fn test_database_url_parsing() {
        let mut db_config = config::default().database;
        
        // Test valid URL
        db_config.url = "postgres://user:pass@localhost:5432/atomsi_dao".to_string();
        let result = Database::connect(&db_config).await;
        
        // This will fail because we can't actually connect, but it should parse correctly
        match result {
            Err(DaoError::DatabaseError(e)) => {
                assert!(e.contains("Failed to connect to database") || e.contains("connection refused"),
                    "Unexpected error: {}", e);
            }
            _ => panic!("Expected DatabaseError"),
        }
        
        // Test invalid URL
        db_config.url = "not-a-valid-url".to_string();
        let result = Database::connect(&db_config).await;
        match result {
            Err(DaoError::DatabaseError(e)) => {
                assert!(e.contains("Invalid database URL"), "Unexpected error: {}", e);
            }
            _ => panic!("Expected DatabaseError for invalid URL"),
        }
    }
} 