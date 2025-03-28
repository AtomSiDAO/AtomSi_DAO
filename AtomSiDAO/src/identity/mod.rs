//! Identity module for AtomSi DAO
//!
//! This module provides functionality for managing identities and
//! reputation within the DAO.

use crate::{
    blockchain::BlockchainAdapter,
    config::Config,
    core::{Database, DaoError, Result},
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

/// Member ID type
pub type MemberId = String;

/// Member role enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MemberRole {
    /// Regular member
    Member,
    /// Delegate member
    Delegate,
    /// Council member
    Council,
    /// Admin member
    Admin,
}

/// Member status enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MemberStatus {
    /// Active member
    Active,
    /// Inactive member
    Inactive,
    /// Suspended member
    Suspended,
}

/// Member structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Member {
    /// Member ID
    pub id: MemberId,
    /// Member address
    pub address: String,
    /// Member name
    pub name: Option<String>,
    /// Member role
    pub role: MemberRole,
    /// Member status
    pub status: MemberStatus,
    /// Member reputation
    pub reputation: u64,
    /// Join timestamp
    pub joined_at: DateTime<Utc>,
    /// Last activity timestamp
    pub last_active_at: DateTime<Utc>,
    /// Additional metadata
    pub metadata: serde_json::Value,
}

/// Activity type enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ActivityType {
    /// Proposal submission
    ProposalSubmission,
    /// Voting
    Voting,
    /// Comment
    Comment,
    /// Delegation
    Delegation,
    /// Treasury transaction
    TreasuryTransaction,
    /// Other activity
    Other,
}

/// Activity structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Activity {
    /// Activity ID
    pub id: String,
    /// Member ID
    pub member_id: MemberId,
    /// Activity type
    pub activity_type: ActivityType,
    /// Related object ID
    pub related_id: Option<String>,
    /// Activity timestamp
    pub timestamp: DateTime<Utc>,
    /// Activity description
    pub description: String,
    /// Reputation change
    pub reputation_change: i64,
    /// Additional metadata
    pub metadata: serde_json::Value,
}

/// Identity manager
pub struct IdentityManager {
    /// Configuration
    config: Arc<Config>,
    /// Blockchain adapter
    blockchain: Arc<dyn BlockchainAdapter>,
    /// Database
    database: Database,
}

impl IdentityManager {
    /// Create a new identity manager
    pub fn new(
        config: &Config,
        blockchain: impl BlockchainAdapter + 'static,
        database: Database,
    ) -> Result<Self> {
        Ok(Self {
            config: Arc::new(config.clone()),
            blockchain: Arc::new(blockchain),
            database,
        })
    }
    
    /// Register a new member
    pub async fn register_member(
        &self,
        address: &str,
        name: Option<String>,
        role: MemberRole,
    ) -> Result<MemberId> {
        // Check if the address is valid
        if !self.blockchain.is_valid_address(address) {
            return Err(DaoError::InvalidParameter("Invalid address".to_string()));
        }
        
        // Check if the member already exists
        if self.member_exists(address).await? {
            return Err(DaoError::InvalidParameter(
                "Member with this address already exists".to_string(),
            ));
        }
        
        // Create a new member
        let member = Member {
            id: Uuid::new_v4().to_string(),
            address: address.to_string(),
            name,
            role,
            status: MemberStatus::Active,
            reputation: 0,
            joined_at: Utc::now(),
            last_active_at: Utc::now(),
            metadata: serde_json::Value::Null,
        };
        
        // Save the member to the database
        self.save_member(&member).await?;
        
        Ok(member.id)
    }
    
    /// Get a member by ID
    pub async fn get_member_by_id(&self, id: &MemberId) -> Result<Member> {
        // Load the member from the database
        let query = "SELECT * FROM members WHERE id = $1";
        let row = self.database.query_one(query, &[&id]).await?;
        
        // Parse the member from the row
        let member: Member = serde_json::from_value(row.get("data"))
            .map_err(|e| DaoError::DatabaseError(format!("Failed to parse member: {}", e)))?;
        
        Ok(member)
    }
    
    /// Get a member by address
    pub async fn get_member_by_address(&self, address: &str) -> Result<Member> {
        // Load the member from the database
        let query = "SELECT * FROM members WHERE address = $1";
        let row = self.database.query_one(query, &[&address]).await?;
        
        // Parse the member from the row
        let member: Member = serde_json::from_value(row.get("data"))
            .map_err(|e| DaoError::DatabaseError(format!("Failed to parse member: {}", e)))?;
        
        Ok(member)
    }
    
    /// Check if a member exists
    pub async fn member_exists(&self, address: &str) -> Result<bool> {
        // Query the database for the member
        let query = "SELECT 1 FROM members WHERE address = $1";
        let result = self.database.query_opt(query, &[&address]).await?;
        
        Ok(result.is_some())
    }
    
    /// Update a member
    pub async fn update_member(&self, member: &Member) -> Result<()> {
        // Check if the member exists
        let exists = self
            .database
            .query_opt("SELECT 1 FROM members WHERE id = $1", &[&member.id])
            .await?
            .is_some();
        
        if !exists {
            return Err(DaoError::InvalidParameter("Member not found".to_string()));
        }
        
        // Save the member to the database
        self.save_member(member).await?;
        
        Ok(())
    }
    
    /// Update member status
    pub async fn update_member_status(
        &self,
        id: &MemberId,
        status: MemberStatus,
    ) -> Result<()> {
        // Load the member
        let mut member = self.get_member_by_id(id).await?;
        
        // Update the status
        member.status = status;
        member.last_active_at = Utc::now();
        
        // Save the member
        self.save_member(&member).await?;
        
        Ok(())
    }
    
    /// Update member role
    pub async fn update_member_role(
        &self,
        id: &MemberId,
        role: MemberRole,
    ) -> Result<()> {
        // Load the member
        let mut member = self.get_member_by_id(id).await?;
        
        // Update the role
        member.role = role;
        member.last_active_at = Utc::now();
        
        // Save the member
        self.save_member(&member).await?;
        
        Ok(())
    }
    
    /// Record an activity
    pub async fn record_activity(
        &self,
        member_id: &MemberId,
        activity_type: ActivityType,
        related_id: Option<String>,
        description: &str,
        reputation_change: i64,
        metadata: serde_json::Value,
    ) -> Result<String> {
        // Create a new activity
        let activity = Activity {
            id: Uuid::new_v4().to_string(),
            member_id: member_id.to_string(),
            activity_type,
            related_id,
            timestamp: Utc::now(),
            description: description.to_string(),
            reputation_change,
            metadata,
        };
        
        // Save the activity to the database
        self.save_activity(&activity).await?;
        
        // Update the member's reputation
        if reputation_change != 0 {
            let mut member = self.get_member_by_id(member_id).await?;
            
            // Apply reputation change, ensuring it doesn't go below 0
            if reputation_change < 0 && member.reputation < (-reputation_change) as u64 {
                member.reputation = 0;
            } else {
                member.reputation = (member.reputation as i64 + reputation_change) as u64;
            }
            
            member.last_active_at = Utc::now();
            self.save_member(&member).await?;
        }
        
        Ok(activity.id)
    }
    
    /// Get member activities
    pub async fn get_member_activities(
        &self,
        member_id: &MemberId,
        limit: Option<u32>,
    ) -> Result<Vec<Activity>> {
        // Construct the query
        let query = "SELECT * FROM activities WHERE member_id = $1 ORDER BY timestamp DESC LIMIT $2";
        let limit_value = limit.unwrap_or(100) as i64;
        
        // Load the activities from the database
        let rows = self.database.query(query, &[&member_id, &limit_value]).await?;
        
        // Parse the activities from the rows
        let activities = rows
            .into_iter()
            .map(|row| {
                serde_json::from_value(row.get("data"))
                    .map_err(|e| DaoError::DatabaseError(format!("Failed to parse activity: {}", e)))
            })
            .collect::<Result<Vec<Activity>>>()?;
        
        Ok(activities)
    }
    
    /// Get members by role
    pub async fn get_members_by_role(&self, role: MemberRole) -> Result<Vec<Member>> {
        // Convert role to string for the query
        let role_str = serde_json::to_string(&role)
            .map_err(|e| DaoError::DatabaseError(format!("Failed to serialize role: {}", e)))?;
        
        // Construct the query
        let query = "SELECT * FROM members WHERE data->>'role' = $1 AND data->>'status' = $2";
        let status_str = serde_json::to_string(&MemberStatus::Active)
            .map_err(|e| DaoError::DatabaseError(format!("Failed to serialize status: {}", e)))?;
        
        // Load the members from the database
        let rows = self.database.query(query, &[&role_str, &status_str]).await?;
        
        // Parse the members from the rows
        let members = rows
            .into_iter()
            .map(|row| {
                serde_json::from_value(row.get("data"))
                    .map_err(|e| DaoError::DatabaseError(format!("Failed to parse member: {}", e)))
            })
            .collect::<Result<Vec<Member>>>()?;
        
        Ok(members)
    }
    
    /// Get active members
    pub async fn get_active_members(&self) -> Result<Vec<Member>> {
        // Convert status to string for the query
        let status_str = serde_json::to_string(&MemberStatus::Active)
            .map_err(|e| DaoError::DatabaseError(format!("Failed to serialize status: {}", e)))?;
        
        // Construct the query
        let query = "SELECT * FROM members WHERE data->>'status' = $1";
        
        // Load the members from the database
        let rows = self.database.query(query, &[&status_str]).await?;
        
        // Parse the members from the rows
        let members = rows
            .into_iter()
            .map(|row| {
                serde_json::from_value(row.get("data"))
                    .map_err(|e| DaoError::DatabaseError(format!("Failed to parse member: {}", e)))
            })
            .collect::<Result<Vec<Member>>>()?;
        
        Ok(members)
    }
    
    // Private methods
    
    /// Save a member to the database
    async fn save_member(&self, member: &Member) -> Result<()> {
        // Serialize the member
        let data = serde_json::to_value(member)
            .map_err(|e| DaoError::DatabaseError(format!("Failed to serialize member: {}", e)))?;
        
        // Check if the member already exists
        let exists = self
            .database
            .query_opt("SELECT 1 FROM members WHERE id = $1", &[&member.id])
            .await?
            .is_some();
        
        if exists {
            // Update the member
            self.database
                .execute(
                    "UPDATE members SET data = $1, address = $2, updated_at = $3 WHERE id = $4",
                    &[&data, &member.address, &Utc::now(), &member.id],
                )
                .await?;
        } else {
            // Insert the member
            self.database
                .execute(
                    "INSERT INTO members (id, address, data, created_at, updated_at) VALUES ($1, $2, $3, $4, $5)",
                    &[
                        &member.id,
                        &member.address,
                        &data,
                        &member.joined_at,
                        &Utc::now(),
                    ],
                )
                .await?;
        }
        
        Ok(())
    }
    
    /// Save an activity to the database
    async fn save_activity(&self, activity: &Activity) -> Result<()> {
        // Serialize the activity
        let data = serde_json::to_value(activity)
            .map_err(|e| DaoError::DatabaseError(format!("Failed to serialize activity: {}", e)))?;
        
        // Insert the activity
        self.database
            .execute(
                "INSERT INTO activities (id, member_id, data, activity_type, timestamp) VALUES ($1, $2, $3, $4, $5)",
                &[
                    &activity.id,
                    &activity.member_id,
                    &data,
                    &serde_json::to_string(&activity.activity_type).unwrap(),
                    &activity.timestamp,
                ],
            )
            .await?;
        
        Ok(())
    }
} 