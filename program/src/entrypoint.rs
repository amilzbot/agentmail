use pinocchio::{account::AccountView, entrypoint, error::ProgramError, Address, ProgramResult};

use crate::{
    instructions::{process_deregister_agent, process_register_agent, process_update_agent},
    traits::AgentMailInstructionDiscriminators,
};

entrypoint!(process_instruction);

pub fn process_instruction(
    program_id: &Address,
    accounts: &[AccountView],
    instruction_data: &[u8],
) -> ProgramResult {
    let (discriminator, instruction_data) = instruction_data
        .split_first()
        .ok_or(ProgramError::InvalidInstructionData)?;

    let ix_discriminator = AgentMailInstructionDiscriminators::try_from(*discriminator)?;

    match ix_discriminator {
        AgentMailInstructionDiscriminators::RegisterAgent => {
            process_register_agent(program_id, accounts, instruction_data)
        }
        AgentMailInstructionDiscriminators::UpdateAgent => {
            process_update_agent(program_id, accounts, instruction_data)
        }
        AgentMailInstructionDiscriminators::DeregisterAgent => {
            process_deregister_agent(program_id, accounts, instruction_data)
        }
    }
}
