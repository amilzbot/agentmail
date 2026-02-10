use agentmail_client::AGENTMAIL_ID;

use crate::utils::Address;

const AGENTMAIL_SEED: &[u8] = b"agentmail";

pub fn find_agent_registry_pda(authority: &Address) -> (Address, u8) {
    Address::find_program_address(&[AGENTMAIL_SEED, authority.as_ref()], &AGENTMAIL_ID)
}
