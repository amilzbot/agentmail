use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
};
use pinocchio_counter_client::PINOCCHIO_COUNTER_ID;

pub fn update_agent(
    agent_authority: &Pubkey,
    agent_registry: &Pubkey,
    name: String,
    inbox_url: String,
) -> Instruction {
    // UpdateAgent instruction discriminator is 4
    let mut data = vec![4u8];
    
    // Add name (length-prefixed string)
    let name_bytes = name.as_bytes();
    data.extend_from_slice(&(name_bytes.len() as u32).to_le_bytes());
    data.extend_from_slice(name_bytes);
    
    // Add inbox_url (length-prefixed string)
    let inbox_url_bytes = inbox_url.as_bytes();
    data.extend_from_slice(&(inbox_url_bytes.len() as u32).to_le_bytes());
    data.extend_from_slice(inbox_url_bytes);

    Instruction {
        program_id: PINOCCHIO_COUNTER_ID,
        accounts: vec![
            AccountMeta::new_readonly(*agent_authority, true),
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
    fn test_update_agent_data_serialization() {
        let agent_authority = Pubkey::new_unique();
        let agent_registry = Pubkey::new_unique();
        
        let instruction = update_agent(
            &agent_authority,
            &agent_registry,
            "updated_agent".to_string(),
            "https://newurl.com/inbox".to_string(),
        );

        // Check discriminator
        assert_eq!(instruction.data[0], 4);
        
        // Check name serialization (length + data)
        let name_len = u32::from_le_bytes([
            instruction.data[1],
            instruction.data[2], 
            instruction.data[3],
            instruction.data[4],
        ]);
        assert_eq!(name_len, 13);
        
        let name_bytes = &instruction.data[5..18];
        assert_eq!(name_bytes, b"updated_agent");
    }
}