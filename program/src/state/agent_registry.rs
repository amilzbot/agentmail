use alloc::vec;
use alloc::vec::Vec;
use codama::CodamaAccount;
use pinocchio::{account::AccountView, cpi::Seed, error::ProgramError, Address};

use crate::assert_no_padding;
use crate::errors::AgentMailProgramError;
use crate::traits::{
    AccountDeserialize, AccountSerialize, AccountSize, Discriminator, PdaSeeds,
    AgentMailAccountDiscriminators, Versioned,
};

/// Agent registry account state
///
/// Stores an agent's public information for the AgentMail protocol.
/// Each agent gets one registry entry per Solana keypair.
///
/// # PDA Seeds
/// `[b"agentmail", agent_authority.as_ref()]`
///
/// # Layout (334 bytes)
/// - bump: 1 byte
/// - version: 1 byte  
/// - _padding: 6 bytes (reserved for future use / alignment)
/// - authority: 32 bytes (agent's pubkey - owner)
/// - name: 4 + 64 bytes (length-prefixed string, max 64 chars)
/// - inbox_url: 4 + 256 bytes (length-prefixed string, max 256 chars)
/// - created_at: 8 bytes (i64 unix timestamp)
/// - updated_at: 8 bytes (i64 unix timestamp)
#[derive(Clone, Debug, PartialEq, CodamaAccount)]
#[repr(C)]
pub struct AgentRegistry {
    pub bump: u8,
    pub version: u8,
    pub _padding: [u8; 6],
    pub authority: Address,
    pub name: [u8; 68],        // 4 bytes length + 64 bytes data
    pub inbox_url: [u8; 260],  // 4 bytes length + 256 bytes data
    pub created_at: i64,
    pub updated_at: i64,
}

assert_no_padding!(AgentRegistry, 1 + 1 + 6 + 32 + 68 + 260 + 8 + 8);

impl Discriminator for AgentRegistry {
    const DISCRIMINATOR: u8 = AgentMailAccountDiscriminators::AgentRegistryDiscriminator as u8;
}

impl Versioned for AgentRegistry {
    const VERSION: u8 = 1;
}

impl AccountSize for AgentRegistry {
    const DATA_LEN: usize = 1 + 1 + 6 + 32 + 68 + 260 + 8 + 8; // 384 bytes total
}

impl AccountDeserialize for AgentRegistry {}

impl AccountSerialize for AgentRegistry {
    #[inline(always)]
    fn to_bytes_inner(&self) -> Vec<u8> {
        let mut data = Vec::with_capacity(Self::DATA_LEN);
        data.push(self.bump);
        data.push(self.version);
        data.extend_from_slice(&self._padding);
        data.extend_from_slice(self.authority.as_ref());
        data.extend_from_slice(&self.name);
        data.extend_from_slice(&self.inbox_url);
        data.extend_from_slice(&self.created_at.to_le_bytes());
        data.extend_from_slice(&self.updated_at.to_le_bytes());
        data
    }
}

impl PdaSeeds for AgentRegistry {
    const PREFIX: &'static [u8] = b"agentmail";

    #[inline(always)]
    fn seeds(&self) -> Vec<&[u8]> {
        vec![Self::PREFIX, self.authority.as_ref()]
    }

    #[inline(always)]
    fn seeds_with_bump<'a>(&'a self, bump: &'a [u8; 1]) -> Vec<Seed<'a>> {
        vec![
            Seed::from(Self::PREFIX),
            Seed::from(self.authority.as_ref()),
            Seed::from(bump.as_slice()),
        ]
    }
}

impl AgentRegistry {
    /// Maximum length for agent name (UTF-8 bytes)
    pub const MAX_NAME_LEN: usize = 64;
    
    /// Maximum length for inbox URL (UTF-8 bytes)
    pub const MAX_INBOX_URL_LEN: usize = 256;

    /// Create a new AgentRegistry instance
    #[inline(always)]
    pub fn new(
        bump: u8,
        authority: Address,
        name: &str,
        inbox_url: &str,
        timestamp: i64,
    ) -> Result<Self, ProgramError> {
        let mut registry = Self {
            bump,
            version: Self::VERSION,
            _padding: [0u8; 6],
            authority,
            name: [0u8; 68],
            inbox_url: [0u8; 260],
            created_at: timestamp,
            updated_at: timestamp,
        };

        registry.set_name(name)?;
        registry.set_inbox_url(inbox_url)?;

        Ok(registry)
    }

    /// Create AgentRegistry from account data with validation
    #[inline(always)]
    pub fn from_account<'a>(
        data: &'a [u8],
        account: &AccountView,
        program_id: &Address,
    ) -> Result<&'a Self, ProgramError> {
        let state = Self::from_bytes(data)?;
        state.validate_pda(account, program_id, state.bump)?;
        Ok(state)
    }

    /// Validate that the provided authority matches the account's authority
    #[inline(always)]
    pub fn validate_authority(&self, provided_authority: &Address) -> Result<(), ProgramError> {
        if self.authority != *provided_authority {
            return Err(AgentMailProgramError::InvalidAuthority.into());
        }
        Ok(())
    }

    /// Update the agent's name
    #[inline(always)]
    pub fn set_name(&mut self, name: &str) -> Result<(), ProgramError> {
        let name_bytes = name.as_bytes();
        
        if name_bytes.len() > Self::MAX_NAME_LEN {
            return Err(AgentMailProgramError::NameTooLong.into());
        }

        // Clear the name field
        self.name = [0u8; 68];
        
        // Set length prefix (4 bytes, little-endian)
        let len_bytes = (name_bytes.len() as u32).to_le_bytes();
        self.name[..4].copy_from_slice(&len_bytes);
        
        // Copy name data
        self.name[4..4 + name_bytes.len()].copy_from_slice(name_bytes);
        
        Ok(())
    }

    /// Update the agent's inbox URL
    #[inline(always)]
    pub fn set_inbox_url(&mut self, inbox_url: &str) -> Result<(), ProgramError> {
        let url_bytes = inbox_url.as_bytes();
        
        if url_bytes.len() > Self::MAX_INBOX_URL_LEN {
            return Err(AgentMailProgramError::InboxUrlTooLong.into());
        }

        // Clear the inbox_url field
        self.inbox_url = [0u8; 260];
        
        // Set length prefix (4 bytes, little-endian)
        let len_bytes = (url_bytes.len() as u32).to_le_bytes();
        self.inbox_url[..4].copy_from_slice(&len_bytes);
        
        // Copy URL data
        self.inbox_url[4..4 + url_bytes.len()].copy_from_slice(url_bytes);
        
        Ok(())
    }

    /// Get the agent's name as a string
    #[inline(always)]
    pub fn get_name(&self) -> Result<alloc::string::String, ProgramError> {
        let len = u32::from_le_bytes([self.name[0], self.name[1], self.name[2], self.name[3]]) as usize;
        
        if len > Self::MAX_NAME_LEN {
            return Err(AgentMailProgramError::InvalidNameLength.into());
        }

        let name_bytes = &self.name[4..4 + len];
        alloc::string::String::from_utf8(name_bytes.to_vec())
            .map_err(|_| AgentMailProgramError::InvalidUtf8.into())
    }

    /// Get the agent's inbox URL as a string
    #[inline(always)]
    pub fn get_inbox_url(&self) -> Result<alloc::string::String, ProgramError> {
        let len = u32::from_le_bytes([self.inbox_url[0], self.inbox_url[1], self.inbox_url[2], self.inbox_url[3]]) as usize;
        
        if len > Self::MAX_INBOX_URL_LEN {
            return Err(AgentMailProgramError::InvalidInboxUrlLength.into());
        }

        let url_bytes = &self.inbox_url[4..4 + len];
        alloc::string::String::from_utf8(url_bytes.to_vec())
            .map_err(|_| AgentMailProgramError::InvalidUtf8.into())
    }

    /// Update the updated_at timestamp
    #[inline(always)]
    pub fn touch(&mut self, timestamp: i64) {
        self.updated_at = timestamp;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::string::ToString;

    fn create_test_registry() -> AgentRegistry {
        let authority = Address::new_from_array([1u8; 32]);
        AgentRegistry::new(
            255,
            authority,
            "nix",
            "https://nix.example.com/inbox",
            1707523200, // 2026-02-10 timestamp
        ).unwrap()
    }

    #[test]
    fn test_agent_registry_new() {
        let authority = Address::new_from_array([1u8; 32]);
        let registry = AgentRegistry::new(
            200,
            authority,
            "test-agent",
            "https://test.example.com/inbox",
            1707523200,
        ).unwrap();

        assert_eq!(registry.bump, 200);
        assert_eq!(registry.version, 1);
        assert_eq!(registry.authority, authority);
        assert_eq!(registry.created_at, 1707523200);
        assert_eq!(registry.updated_at, 1707523200);
        assert_eq!(registry.get_name().unwrap(), "test-agent".to_string());
        assert_eq!(registry.get_inbox_url().unwrap(), "https://test.example.com/inbox".to_string());
    }

    #[test]
    fn test_agent_registry_validate_authority_success() {
        let registry = create_test_registry();
        let valid_authority = Address::new_from_array([1u8; 32]);

        assert!(registry.validate_authority(&valid_authority).is_ok());
    }

    #[test]
    fn test_agent_registry_validate_authority_invalid() {
        let registry = create_test_registry();
        let invalid_authority = Address::new_from_array([99u8; 32]);

        let result = registry.validate_authority(&invalid_authority);
        assert_eq!(
            result,
            Err(AgentMailProgramError::InvalidAuthority.into())
        );
    }

    #[test]
    fn test_agent_registry_name_too_long() {
        let authority = Address::new_from_array([1u8; 32]);
        let long_name = "a".repeat(65); // Exceeds MAX_NAME_LEN
        
        let result = AgentRegistry::new(
            200,
            authority,
            &long_name,
            "https://test.example.com/inbox",
            1707523200,
        );
        
        assert_eq!(result, Err(AgentMailProgramError::NameTooLong.into()));
    }

    #[test]
    fn test_agent_registry_inbox_url_too_long() {
        let authority = Address::new_from_array([1u8; 32]);
        let long_url = "https://".to_string() + &"a".repeat(250); // Exceeds MAX_INBOX_URL_LEN
        
        let result = AgentRegistry::new(
            200,
            authority,
            "test",
            &long_url,
            1707523200,
        );
        
        assert_eq!(result, Err(AgentMailProgramError::InboxUrlTooLong.into()));
    }

    #[test]
    fn test_agent_registry_update_fields() {
        let mut registry = create_test_registry();
        let original_updated_at = registry.updated_at;

        // Update name
        registry.set_name("new-name").unwrap();
        assert_eq!(registry.get_name().unwrap(), "new-name".to_string());

        // Update inbox URL
        registry.set_inbox_url("https://new.example.com/inbox").unwrap();
        assert_eq!(registry.get_inbox_url().unwrap(), "https://new.example.com/inbox".to_string());

        // Touch timestamp
        registry.touch(1707523300);
        assert_eq!(registry.updated_at, 1707523300);
        assert!(registry.updated_at > original_updated_at);
    }

    #[test]
    fn test_agent_registry_serialization() {
        let registry = create_test_registry();
        let bytes = registry.to_bytes_inner();

        assert_eq!(bytes.len(), AgentRegistry::DATA_LEN);
        assert_eq!(bytes[0], 255); // bump
        assert_eq!(bytes[1], 1);   // version
        assert_eq!(&bytes[2..8], &[0u8; 6]); // padding
        assert_eq!(&bytes[8..40], &[1u8; 32]); // authority
    }

    #[test]
    fn test_agent_registry_to_bytes() {
        let registry = create_test_registry();
        let bytes = registry.to_bytes();

        assert_eq!(bytes.len(), AgentRegistry::LEN);
        assert_eq!(bytes[0], AgentRegistry::DISCRIMINATOR);
        assert_eq!(bytes[1], AgentRegistry::VERSION);
        assert_eq!(bytes[2], 255); // bump
    }

    #[test]
    fn test_agent_registry_from_bytes() {
        let registry = create_test_registry();
        let bytes = registry.to_bytes();

        let deserialized = AgentRegistry::from_bytes(&bytes).unwrap();

        assert_eq!(deserialized.bump, registry.bump);
        assert_eq!(deserialized.version, registry.version);
        assert_eq!(deserialized.authority, registry.authority);
        assert_eq!(deserialized.created_at, registry.created_at);
        assert_eq!(deserialized.updated_at, registry.updated_at);
        assert_eq!(deserialized.get_name().unwrap(), registry.get_name().unwrap());
        assert_eq!(deserialized.get_inbox_url().unwrap(), registry.get_inbox_url().unwrap());
    }

    #[test]
    fn test_agent_registry_seeds() {
        let registry = create_test_registry();
        let seeds = registry.seeds();

        assert_eq!(seeds.len(), 2);
        assert_eq!(seeds[0], AgentRegistry::PREFIX);
        assert_eq!(seeds[1], registry.authority.as_ref());
    }
}