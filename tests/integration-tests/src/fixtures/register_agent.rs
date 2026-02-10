use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    system_program,
};
use pinocchio_counter_client::PINOCCHIO_COUNTER_ID;

pub fn register_agent(
    payer: &Pubkey,
    agent_authority: &Pubkey,
    agent_registry: &Pubkey,
    bump: u8,
    name: String,
    inbox_url: String,
) -> Instruction {
    // RegisterAgent instruction discriminator is 3
    let mut data = vec![3u8];
    
    // Add bump
    data.push(bump);
    
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
            AccountMeta::new(*payer, true),
            AccountMeta::new_readonly(*agent_authority, true),
            AccountMeta::new(*agent_registry, false),
            AccountMeta::new_readonly(system_program::id(), false),
            AccountMeta::new_readonly(PINOCCHIO_COUNTER_ID, false),
        ],
        data,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_agent_data_serialization() {
        let payer = Pubkey::new_unique();
        let agent_authority = Pubkey::new_unique();
        let agent_registry = Pubkey::new_unique();
        
        let instruction = register_agent(
            &payer,
            &agent_authority,
            &agent_registry,
            255, // bump
            "test_agent".to_string(),
            "https://example.com/inbox".to_string(),
        );

        // Check discriminator
        assert_eq!(instruction.data[0], 3);
        
        // Check bump
        assert_eq!(instruction.data[1], 255);
        
        // Check name serialization (length + data)
        let name_len = u32::from_le_bytes([
            instruction.data[2],
            instruction.data[3], 
            instruction.data[4],
            instruction.data[5],
        ]);
        assert_eq!(name_len, 10);
        
        let name_bytes = &instruction.data[6..16];
        assert_eq!(name_bytes, b"test_agent");
    }
}