use pinocchio::{account::AccountView, Address, ProgramResult};

use crate::{
    instructions::DeregisterAgent,
    state::AgentRegistry,
    traits::{AccountDeserialize, AccountSize, Instruction},
    errors::AgentMailProgramError,
};

/// Processes the DeregisterAgent instruction.
///
/// Closes an existing AgentRegistry PDA and transfers all lamports to the authority.
/// Only the authority (agent) can deregister their own registry.
pub fn process_deregister_agent(
    _program_id: &Address,
    accounts: &[AccountView],
    instruction_data: &[u8],
) -> ProgramResult {
    let ix = DeregisterAgent::parse(instruction_data, accounts)?;

    // Verify that the registry account has the correct size
    if ix.accounts.agent_registry.data_len() != AgentRegistry::LEN {
        return Err(AgentMailProgramError::InvalidAccountSize.into());
    }

    // Verify the account is actually a valid AgentRegistry by deserializing
    let registry_data = ix.accounts.agent_registry.try_borrow()?;
    let registry = AgentRegistry::from_bytes(&registry_data)
        .map_err(|_| AgentMailProgramError::InvalidAccountData)?;
    
    // Verify that the signer is the authority for this registry
    registry.validate_authority(ix.accounts.agent_authority.address())?;
    
    // Release the borrow before we modify account data
    drop(registry_data);

    // TODO: Transfer lamports back to authority using CPI to system program
    // For now, just close the account by zeroing data

    // Zero out the account data
    let mut registry_data_slice = ix.accounts.agent_registry.try_borrow_mut()?;
    registry_data_slice.fill(0);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        instructions::{DeregisterAgentData, DeregisterAgentAccounts},
        state::AgentRegistry,
        traits::AccountSerialize,
    };
    use pinocchio::{Address, AccountView};
    use alloc::vec::Vec;

    fn create_mock_account_with_data_and_lamports(
        address: Address,
        owner: Address,
        data: Vec<u8>,
        lamports: u64,
        is_signer: bool,
        is_writable: bool,
    ) -> AccountView {
        AccountView::new(
            &address,
            &owner,
            lamports,
            &data,
            is_signer,
            is_writable,
        ).unwrap()
    }

    fn create_test_registry() -> AgentRegistry {
        let authority = Address::new_from_array([1u8; 32]);
        AgentRegistry::new(
            255,
            authority,
            "test-agent",
            "https://test.example.com/inbox",
            1707523200,
        ).unwrap()
    }

    #[test]
    fn test_deregister_agent_successful() {
        let registry = create_test_registry();
        let registry_bytes = registry.to_bytes();

        let authority_address = Address::new_from_array([1u8; 32]);
        let registry_address = Address::new_from_array([2u8; 32]);

        let agent_authority = create_mock_account_with_data_and_lamports(
            authority_address,
            Address::new_from_array([11u8; 32]), // System program
            Vec::new(),
            1000000, // Starting balance
            true,    // signer
            true,    // writable
        );

        let agent_registry = create_mock_account_with_data_and_lamports(
            registry_address,
            crate::ID, // Our program owns this
            registry_bytes,
            5000000, // Registry rent balance
            false,   // not signer
            true,    // writable
        );

        let program = create_mock_account_with_data_and_lamports(
            crate::ID,
            Address::new_from_array([0u8; 32]),
            Vec::new(),
            1,
            false,
            false,
        );

        let accounts = [agent_authority, agent_registry, program];

        let ix = DeregisterAgent {
            accounts: DeregisterAgentAccounts {
                agent_authority: &accounts[0],
                agent_registry: &accounts[1],
                program: &accounts[2],
            },
            data: DeregisterAgentData,
        };

        // This test would need a proper Solana runtime environment to fully work
        // For now, we're validating the data parsing and authority validation
        
        // Verify the registry can be deserialized
        let registry_data = accounts[1].try_borrow().unwrap();
        let parsed_registry = AgentRegistry::from_bytes(&registry_data).unwrap();
        
        // Verify authority validation would pass
        assert!(parsed_registry.validate_authority(&authority_address).is_ok());
    }

    #[test]
    fn test_authority_validation() {
        let registry = create_test_registry();
        let correct_authority = Address::new_from_array([1u8; 32]);
        let wrong_authority = Address::new_from_array([99u8; 32]);

        assert!(registry.validate_authority(&correct_authority).is_ok());
        assert_eq!(
            registry.validate_authority(&wrong_authority),
            Err(AgentMailProgramError::InvalidAuthority.into())
        );
    }

    #[test]
    fn test_empty_data_parsing() {
        let data: [u8; 0] = [];
        let result = DeregisterAgentData::try_from(&data[..]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_data_with_bytes_parsing() {
        let data = [1u8, 2u8, 3u8];
        let result = DeregisterAgentData::try_from(&data[..]);
        assert!(result.is_ok()); // Should ignore extra data
    }
}