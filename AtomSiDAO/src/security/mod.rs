//! Security module for AtomSi DAO
//!
//! This module provides functionality for authentication, authorization,
//! and cryptographic operations for the DAO.

mod permissions;

pub use permissions::{Permission, PermissionManager, Resource};

use crate::{
    blockchain::BlockchainAdapter,
    config::Config,
    core::{Database, DaoError, Result},
    crypto,
    identity::{IdentityManager, MemberRole},
};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

/// Session ID type
pub type SessionId = String;

/// Session structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    /// Session ID
    pub id: SessionId,
    /// Member address
    pub address: String,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// Expiration timestamp
    pub expires_at: DateTime<Utc>,
    /// Last activity timestamp
    pub last_active_at: DateTime<Utc>,
    /// IP address
    pub ip_address: Option<String>,
    /// User agent
    pub user_agent: Option<String>,
    /// Is session active
    pub is_active: bool,
}

/// Authentication type enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AuthenticationType {
    /// Signature-based authentication
    Signature,
    /// Token-based authentication
    Token,
    /// OAuth-based authentication
    OAuth,
}

/// Authentication manager
pub struct AuthManager {
    /// Configuration
    config: Arc<Config>,
    /// Blockchain adapter
    blockchain: Arc<dyn BlockchainAdapter>,
    /// Database
    database: Database,
    /// Identity manager reference
    identity_manager: Arc<IdentityManager>,
    /// Permission manager
    permission_manager: Arc<PermissionManager>,
}

impl AuthManager {
    /// Create a new authentication manager
    pub fn new(
        config: &Config,
        blockchain: impl BlockchainAdapter + 'static,
        database: Database,
        identity_manager: &IdentityManager,
    ) -> Result<Self> {
        let permission_manager = Arc::new(PermissionManager::new());
        
        Ok(Self {
            config: Arc::new(config.clone()),
            blockchain: Arc::new(blockchain),
            database,
            identity_manager: Arc::new(identity_manager.clone()),
            permission_manager,
        })
    }
    
    /// Authenticate a user with signature
    pub async fn authenticate_with_signature(
        &self,
        address: &str,
        message: &str,
        signature: &str,
    ) -> Result<Session> {
        // Verify the signature
        let is_valid = crypto::verify_signature(address, message, signature)
            .map_err(|e| DaoError::SecurityError(e.to_string()))?;
        
        if !is_valid {
            return Err(DaoError::Unauthorized);
        }
        
        // Check if the address is a member
        let is_member = self.identity_manager.member_exists(address).await?;
        
        if !is_member {
            // Register as a new member if not already registered
            self.identity_manager
                .register_member(address, None, MemberRole::Member)
                .await?;
        }
        
        // Create a new session
        self.create_session(address, None, None).await
    }
    
    /// Authenticate a user with token
    pub async fn authenticate_with_token(&self, token: &str) -> Result<Session> {
        // Load the session from the database
        let query = "SELECT * FROM sessions WHERE id = $1 AND is_active = true";
        let row = self
            .database
            .query_one(query, &[&token])
            .await
            .map_err(|_| DaoError::Unauthorized)?;
        
        // Parse the session from the row
        let mut session: Session = serde_json::from_value(row.get("data"))
            .map_err(|e| DaoError::DatabaseError(format!("Failed to parse session: {}", e)))?;
        
        // Check if the session has expired
        let now = Utc::now();
        if now > session.expires_at {
            session.is_active = false;
            self.save_session(&session).await?;
            return Err(DaoError::Unauthorized);
        }
        
        // Update the session's last activity timestamp
        session.last_active_at = now;
        self.save_session(&session).await?;
        
        Ok(session)
    }
    
    /// Logout a user
    pub async fn logout(&self, session_id: &SessionId) -> Result<()> {
        // Load the session
        let query = "SELECT * FROM sessions WHERE id = $1";
        let row = self.database.query_one(query, &[&session_id]).await?;
        
        // Parse the session from the row
        let mut session: Session = serde_json::from_value(row.get("data"))
            .map_err(|e| DaoError::DatabaseError(format!("Failed to parse session: {}", e)))?;
        
        // Mark the session as inactive
        session.is_active = false;
        
        // Save the session
        self.save_session(&session).await?;
        
        Ok(())
    }
    
    /// Check if a user has permission to access a resource
    pub async fn check_permission(
        &self,
        address: &str,
        resource: &str,
        permission: &str,
    ) -> Result<bool> {
        // Get the member's role
        let member = self.identity_manager.get_member_by_address(address).await?;
        let role = member.role;
        
        // Check permission
        Ok(self
            .permission_manager
            .has_permission(role, resource, permission))
    }
    
    /// Create a new session
    async fn create_session(
        &self,
        address: &str,
        ip_address: Option<String>,
        user_agent: Option<String>,
    ) -> Result<Session> {
        // Create a session expiration time (24 hours from now)
        let now = Utc::now();
        let expires_at = now + Duration::hours(24);
        
        // Create a new session
        let session = Session {
            id: Uuid::new_v4().to_string(),
            address: address.to_string(),
            created_at: now,
            expires_at,
            last_active_at: now,
            ip_address,
            user_agent,
            is_active: true,
        };
        
        // Save the session to the database
        self.save_session(&session).await?;
        
        Ok(session)
    }
    
    /// Save a session to the database
    async fn save_session(&self, session: &Session) -> Result<()> {
        // Serialize the session
        let data = serde_json::to_value(session)
            .map_err(|e| DaoError::DatabaseError(format!("Failed to serialize session: {}", e)))?;
        
        // Check if the session already exists
        let exists = self
            .database
            .query_opt("SELECT 1 FROM sessions WHERE id = $1", &[&session.id])
            .await?
            .is_some();
        
        if exists {
            // Update the session
            self.database
                .execute(
                    "UPDATE sessions SET data = $1, is_active = $2, last_active_at = $3 WHERE id = $4",
                    &[&data, &session.is_active, &session.last_active_at, &session.id],
                )
                .await?;
        } else {
            // Insert the session
            self.database
                .execute(
                    "INSERT INTO sessions (id, address, data, created_at, expires_at, is_active) VALUES ($1, $2, $3, $4, $5, $6)",
                    &[
                        &session.id,
                        &session.address,
                        &data,
                        &session.created_at,
                        &session.expires_at,
                        &session.is_active,
                    ],
                )
                .await?;
        }
        
        Ok(())
    }
} 