//! Permissions module for AtomSi DAO
//!
//! This module provides role-based access control for DAO resources.

use crate::identity::MemberRole;
use std::collections::{HashMap, HashSet};

/// Resource type alias
pub type Resource = String;

/// Permission type alias
pub type Permission = String;

/// Permission manager for role-based access control
pub struct PermissionManager {
    /// Permission map: role -> resource -> permissions
    permissions: HashMap<MemberRole, HashMap<Resource, HashSet<Permission>>>,
}

impl PermissionManager {
    /// Create a new permission manager
    pub fn new() -> Self {
        let mut manager = Self {
            permissions: HashMap::new(),
        };
        
        // Initialize default permissions
        manager.init_default_permissions();
        
        manager
    }
    
    /// Check if a role has a specific permission for a resource
    pub fn has_permission(&self, role: MemberRole, resource: &str, permission: &str) -> bool {
        // Admin role has all permissions
        if role == MemberRole::Admin {
            return true;
        }
        
        // Check if the role has permissions for the resource
        if let Some(resources) = self.permissions.get(&role) {
            if let Some(permissions) = resources.get(resource) {
                return permissions.contains(permission);
            }
        }
        
        false
    }
    
    /// Grant a permission to a role for a resource
    pub fn grant_permission(
        &mut self,
        role: MemberRole,
        resource: &str,
        permission: &str,
    ) {
        let resources = self
            .permissions
            .entry(role)
            .or_insert_with(HashMap::new);
        
        let permissions = resources
            .entry(resource.to_string())
            .or_insert_with(HashSet::new);
        
        permissions.insert(permission.to_string());
    }
    
    /// Revoke a permission from a role for a resource
    pub fn revoke_permission(
        &mut self,
        role: MemberRole,
        resource: &str,
        permission: &str,
    ) {
        if let Some(resources) = self.permissions.get_mut(&role) {
            if let Some(permissions) = resources.get_mut(resource) {
                permissions.remove(permission);
            }
        }
    }
    
    /// Grant all permissions to a role for a resource
    pub fn grant_all_permissions(
        &mut self,
        role: MemberRole,
        resource: &str,
        permissions: &[&str],
    ) {
        for permission in permissions {
            self.grant_permission(role, resource, permission);
        }
    }
    
    /// Initialize default permissions
    fn init_default_permissions(&mut self) {
        // Define resources
        let resources = [
            "proposal",
            "vote",
            "token",
            "treasury",
            "member",
            "settings",
        ];
        
        // Define permissions
        let crud_permissions = ["create", "read", "update", "delete"];
        
        // Grant permissions based on role
        
        // Member role
        let member_permissions = [
            ("proposal", &["create", "read"]),
            ("vote", &["create", "read"]),
            ("token", &["read"]),
            ("treasury", &["read"]),
            ("member", &["read"]),
            ("settings", &["read"]),
        ];
        
        for (resource, permissions) in &member_permissions {
            self.grant_all_permissions(MemberRole::Member, resource, permissions);
        }
        
        // Delegate role (inherits member permissions and adds more)
        let delegate_permissions = [
            ("proposal", &["create", "read", "update"]),
            ("vote", &["create", "read"]),
            ("token", &["read"]),
            ("treasury", &["read"]),
            ("member", &["read"]),
            ("settings", &["read"]),
        ];
        
        for (resource, permissions) in &delegate_permissions {
            self.grant_all_permissions(MemberRole::Delegate, resource, permissions);
        }
        
        // Council role (inherits delegate permissions and adds more)
        let council_permissions = [
            ("proposal", &["create", "read", "update", "delete"]),
            ("vote", &["create", "read"]),
            ("token", &["read", "create"]),
            ("treasury", &["read", "create"]),
            ("member", &["read", "update"]),
            ("settings", &["read", "update"]),
        ];
        
        for (resource, permissions) in &council_permissions {
            self.grant_all_permissions(MemberRole::Council, resource, permissions);
        }
        
        // Admin role has all permissions for all resources
        for resource in &resources {
            self.grant_all_permissions(MemberRole::Admin, resource, &crud_permissions);
        }
    }
    
    /// Get all permissions for a role
    pub fn get_permissions(&self, role: MemberRole) -> HashMap<Resource, HashSet<Permission>> {
        self.permissions.get(&role).cloned().unwrap_or_default()
    }
    
    /// Get all resources
    pub fn get_resources(&self) -> HashSet<Resource> {
        let mut resources = HashSet::new();
        
        for role_permissions in self.permissions.values() {
            for resource in role_permissions.keys() {
                resources.insert(resource.clone());
            }
        }
        
        resources
    }
    
    /// Get all permissions for a resource
    pub fn get_resource_permissions(&self, resource: &str) -> HashSet<Permission> {
        let mut permissions = HashSet::new();
        
        for role_permissions in self.permissions.values() {
            if let Some(resource_permissions) = role_permissions.get(resource) {
                permissions.extend(resource_permissions.iter().cloned());
            }
        }
        
        permissions
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_role_permissions() {
        let manager = PermissionManager::new();
        
        // Test member permissions
        assert!(manager.has_permission(MemberRole::Member, "proposal", "create"));
        assert!(manager.has_permission(MemberRole::Member, "proposal", "read"));
        assert!(!manager.has_permission(MemberRole::Member, "proposal", "update"));
        assert!(!manager.has_permission(MemberRole::Member, "proposal", "delete"));
        
        // Test delegate permissions
        assert!(manager.has_permission(MemberRole::Delegate, "proposal", "create"));
        assert!(manager.has_permission(MemberRole::Delegate, "proposal", "read"));
        assert!(manager.has_permission(MemberRole::Delegate, "proposal", "update"));
        assert!(!manager.has_permission(MemberRole::Delegate, "proposal", "delete"));
        
        // Test council permissions
        assert!(manager.has_permission(MemberRole::Council, "proposal", "create"));
        assert!(manager.has_permission(MemberRole::Council, "proposal", "read"));
        assert!(manager.has_permission(MemberRole::Council, "proposal", "update"));
        assert!(manager.has_permission(MemberRole::Council, "proposal", "delete"));
        
        // Test admin permissions
        assert!(manager.has_permission(MemberRole::Admin, "proposal", "create"));
        assert!(manager.has_permission(MemberRole::Admin, "proposal", "read"));
        assert!(manager.has_permission(MemberRole::Admin, "proposal", "update"));
        assert!(manager.has_permission(MemberRole::Admin, "proposal", "delete"));
    }
    
    #[test]
    fn test_admin_has_all_permissions() {
        let manager = PermissionManager::new();
        
        // Test some arbitrary permissions
        assert!(manager.has_permission(MemberRole::Admin, "unknown_resource", "unknown_permission"));
        assert!(manager.has_permission(MemberRole::Admin, "token", "transfer"));
        assert!(manager.has_permission(MemberRole::Admin, "treasury", "withdraw"));
    }
    
    #[test]
    fn test_grant_and_revoke_permission() {
        let mut manager = PermissionManager::new();
        
        // Test granting a new permission
        assert!(!manager.has_permission(MemberRole::Member, "token", "transfer"));
        manager.grant_permission(MemberRole::Member, "token", "transfer");
        assert!(manager.has_permission(MemberRole::Member, "token", "transfer"));
        
        // Test revoking a permission
        manager.revoke_permission(MemberRole::Member, "token", "transfer");
        assert!(!manager.has_permission(MemberRole::Member, "token", "transfer"));
    }
} 