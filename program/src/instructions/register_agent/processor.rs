use alloc::vec::Vec;
use pinocchio::{account::AccountView, cpi::Seed, error::ProgramError, Address, ProgramResult};

use crate::{
    instructions::RegisterAgent,
    state::AgentRegistry,
    traits::{AccountSerialize, AccountSize, PdaSeeds},
    utils::{create_pda_account, get_current_timestamp},
    errors::AgentMailProgramError,
};

/// Processes the RegisterAgent instruction.
///
/// Creates an AgentRegistry PDA for the specified agent authority,
/// storing their name and inbox URL for the AgentMail protocol.
pub fn process_register_agent(
    program_id: &Address,
    accounts: &[AccountView],
    instruction_data: &[u8],
) -> ProgramResult {
    let ix = RegisterAgent::try_from((instruction_data, accounts))?;

    // Get current timestamp
    let timestamp = get_current_timestamp()?;

    // Create AgentRegistry state
    let agent_registry = AgentRegistry::new(
        ix.data.bump,
        *ix.accounts.agent_authority.address(),
        &ix.data.name,
        &ix.data.inbox_url,
        timestamp,
    )?;

    // Validate AgentRegistry PDA
    agent_registry.validate_pda(ix.accounts.agent_registry, program_id, ix.data.bump)?;

    // Ensure agent authority matches the PDA derivation
    agent_registry.validate_authority(ix.accounts.agent_authority.address())?;

    // Check that registry doesn't already exist (account should be empty)
    if ix.accounts.agent_registry.data_len() != 0 {
        return Err(AgentMailProgramError::RegistryAlreadyExists.into());
    }

    // Get seeds for AgentRegistry account creation
    let registry_bump_seed = [ix.data.bump];
    let registry_seeds: Vec<Seed> = agent_registry.seeds_with_bump(&registry_bump_seed);
    let registry_seeds_array: [Seed; 3] = registry_seeds
        .try_into()
        .map_err(|_| ProgramError::InvalidArgument)?;

    // Create the AgentRegistry account
    create_pda_account(
        ix.accounts.payer,
        AgentRegistry::LEN,
        program_id,
        ix.accounts.agent_registry,
        registry_seeds_array,
    )?;

    // Write serialized AgentRegistry data to the account
    let mut registry_data_slice = ix.accounts.agent_registry.try_borrow_mut()?;
    agent_registry.write_to_slice(&mut registry_data_slice)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::instructions::RegisterAgentData;

    // Helper function to create test instruction data
    fn create_test_instruction_data(bump: u8, name: &str, inbox_url: &str) -> Vec<u8> {
        let mut data = Vec::new();
        data.push(bump);
        
        let name_bytes = name.as_bytes();
        data.extend_from_slice(&(name_bytes.len() as u32).to_le_bytes());
        data.extend_from_slice(name_bytes);
        
        let url_bytes = inbox_url.as_bytes();
        data.extend_from_slice(&(url_bytes.len() as u32).to_le_bytes());
        data.extend_from_slice(url_bytes);
        
        data
    }

    #[test]
    fn test_register_agent_data_parsing() {
        let data = create_test_instruction_data(255, "nix", "https://nix.example.com/inbox");
        let parsed = RegisterAgentData::try_from(&data[..]).unwrap();
        
        assert_eq!(parsed.bump, 255);
        assert_eq!(parsed.name, "nix");
        assert_eq!(parsed.inbox_url, "https://nix.example.com/inbox");
    }

    #[test]
    fn test_agent_registry_creation() {
        let authority = Address::new_from_array([1u8; 32]);
        let result = AgentRegistry::new(
            200,
            authority,
            "test-agent",
            "https://test.example.com/inbox",
            1707523200,
        );
        
        assert!(result.is_ok());
        let registry = result.unwrap();
        assert_eq!(registry.bump, 200);
        assert_eq!(registry.authority, authority);
        assert_eq!(registry.get_name().unwrap(), "test-agent");
        assert_eq!(registry.get_inbox_url().unwrap(), "https://test.example.com/inbox");
    }
}