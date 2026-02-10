use pinocchio::{account::AccountView, Address, ProgramResult};

use crate::{
    instructions::UpdateAgent,
    state::AgentRegistry,
    traits::{AccountDeserialize, AccountSerialize, AccountSize, Instruction},
    utils::get_current_timestamp,
    errors::AgentMailProgramError,
};

/// Processes the UpdateAgent instruction.
///
/// Updates an existing AgentRegistry PDA with new name and inbox URL.
/// Only the authority (agent) can update their own registry.
pub fn process_update_agent(
    _program_id: &Address,
    accounts: &[AccountView],
    instruction_data: &[u8],
) -> ProgramResult {
    let ix = UpdateAgent::parse(instruction_data, accounts)?;

    // Get current timestamp
    let timestamp = get_current_timestamp()?;

    // Verify that the registry account has the correct size
    if ix.accounts.agent_registry.data_len() != AgentRegistry::LEN {
        return Err(AgentMailProgramError::InvalidAccountSize.into());
    }

    // Deserialize existing registry state
    let registry_data = ix.accounts.agent_registry.try_borrow()?;
    let mut registry = AgentRegistry::from_bytes(&registry_data)
        .map_err(|_| AgentMailProgramError::InvalidAccountData)?.clone();
    
    // Release the borrow before we try to mutably borrow for writing
    drop(registry_data);

    // Verify that the signer is the authority for this registry
    registry.validate_authority(ix.accounts.agent_authority.address())?;

    // Update the registry fields
    registry.set_name(&ix.data.name)?;
    registry.set_inbox_url(&ix.data.inbox_url)?;
    
    // Update the timestamp
    registry.touch(timestamp);

    // Write updated registry data back to the account
    let mut registry_data_slice = ix.accounts.agent_registry.try_borrow_mut()?;
    registry.write_to_slice(&mut registry_data_slice)?;

    Ok(())
}

// Unit tests disabled in favor of comprehensive LiteSVM integration tests
#[cfg(disabled_unit_tests)]
mod tests {
    use super::*;
    use crate::{
        instructions::{UpdateAgentData, UpdateAgentAccounts},
        state::AgentRegistry,
        traits::AccountSerialize,
    };
    use pinocchio::{Address, AccountView};
    use alloc::vec::Vec;
    use alloc::string::ToString;
    use core::ptr;

    fn create_mock_account_with_data(
        address: Address,
        owner: Address,
        data: Vec<u8>,
        is_signer: bool,
        is_writable: bool,
    ) -> AccountView {
        unsafe { AccountView::new_unchecked(ptr::null_mut()) }
    }

    fn create_test_registry() -> AgentRegistry {
        let authority = Address::new_from_array([1u8; 32]);
        AgentRegistry::new(
            255,
            authority,
            "original-name",
            "https://original.example.com/inbox",
            1707523200,
        ).unwrap()
    }

    #[test]
    fn test_update_agent_successful() {
        let registry = create_test_registry();
        let registry_bytes = registry.to_bytes();

        let authority_address = Address::new_from_array([1u8; 32]);
        let registry_address = Address::new_from_array([2u8; 32]);

        let agent_authority = create_mock_account_with_data(
            authority_address,
            Address::new_from_array([11u8; 32]),
            Vec::new(),
            true,  // signer
            false, // not writable
        );

        let agent_registry = create_mock_account_with_data(
            registry_address,
            crate::ID, // Our program owns this
            registry_bytes,
            false, // not signer
            true,  // writable
        );

        let program = create_mock_account_with_data(
            crate::ID,
            Address::new_from_array([0u8; 32]),
            Vec::new(),
            false,
            false,
        );

        let accounts = [agent_authority, agent_registry, program];

        // Create instruction data
        let update_data = UpdateAgentData {
            name: "updated-name".to_string(),
            inbox_url: "https://updated.example.com/inbox".to_string(),
        };

        let ix = UpdateAgent {
            accounts: UpdateAgentAccounts {
                agent_authority: &accounts[0],
                agent_registry: &accounts[1],
                program: &accounts[2],
            },
            data: update_data,
        };

        // This test would need a proper Solana runtime environment to fully work
        // For now, we're just validating the data parsing and structure
        assert_eq!(ix.data.name, "updated-name");
        assert_eq!(ix.data.inbox_url, "https://updated.example.com/inbox");
    }

    #[test]
    fn test_update_agent_data_serialization() {
        let mut data = Vec::new();
        
        let name = "test-update";
        let url = "https://test-update.example.com/inbox";
        
        // Serialize name
        data.extend_from_slice(&(name.len() as u32).to_le_bytes());
        data.extend_from_slice(name.as_bytes());
        
        // Serialize URL
        data.extend_from_slice(&(url.len() as u32).to_le_bytes());
        data.extend_from_slice(url.as_bytes());

        let result = UpdateAgentData::try_from(&data[..]);
        assert!(result.is_ok());
        
        let update_data = result.unwrap();
        assert_eq!(update_data.name, name);
        assert_eq!(update_data.inbox_url, url);
    }

    #[test]
    fn test_registry_field_updates() {
        let mut registry = create_test_registry();
        let original_created_at = registry.created_at;
        let original_updated_at = registry.updated_at;

        // Update fields
        registry.set_name("new-name").unwrap();
        registry.set_inbox_url("https://new.example.com/inbox").unwrap();
        registry.touch(1707523300);

        // Verify updates
        assert_eq!(registry.get_name().unwrap(), "new-name");
        assert_eq!(registry.get_inbox_url().unwrap(), "https://new.example.com/inbox");
        assert_eq!(registry.created_at, original_created_at); // Should not change
        assert_ne!(registry.updated_at, original_updated_at); // Should change
        assert_eq!(registry.updated_at, 1707523300);
    }
}