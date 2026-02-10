use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
};
use pinocchio_counter_client::PINOCCHIO_COUNTER_ID;

pub fn deregister_agent(
    agent_authority: &Pubkey,
    agent_registry: &Pubkey,
) -> Instruction {
    // DeregisterAgent instruction discriminator is 5
    let data = vec![5u8];

    Instruction {
        program_id: PINOCCHIO_COUNTER_ID,
        accounts: vec![
            AccountMeta::new(*agent_authority, true),
            AccountMeta::new(*agent_registry, false),
            AccountMeta::new_readonly(PINOCCHIO_COUNTER_ID, false),
        ],
        data,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deregister_agent_data_serialization() {
        let agent_authority = Pubkey::new_unique();
        let agent_registry = Pubkey::new_unique();
        
        let instruction = deregister_agent(&agent_authority, &agent_registry);

        // Check discriminator
        assert_eq!(instruction.data[0], 5);
        
        // DeregisterAgent has no additional data
        assert_eq!(instruction.data.len(), 1);
        
        // Check accounts
        assert_eq!(instruction.accounts.len(), 3);
        assert_eq!(instruction.accounts[0].pubkey, agent_authority);
        assert_eq!(instruction.accounts[1].pubkey, agent_registry);
        assert_eq!(instruction.accounts[2].pubkey, PINOCCHIO_COUNTER_ID);
    }
}