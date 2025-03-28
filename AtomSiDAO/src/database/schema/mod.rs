//! Database schema module for AtomSi DAO
//!
//! This module contains schema definitions for the database tables.

/// Create script for PostgreSQL database
pub const POSTGRES_SCHEMA: &str = include_str!("postgres.sql");

/// Create script for SQLite database
pub const SQLITE_SCHEMA: &str = include_str!("sqlite.sql");

/// Get the appropriate schema based on database type
pub fn get_schema(db_type: &str) -> &'static str {
    match db_type {
        "postgres" => POSTGRES_SCHEMA,
        "sqlite" => SQLITE_SCHEMA,
        _ => SQLITE_SCHEMA, // Default to SQLite
    }
} 