use pinocchio::error::ProgramError;

use crate::traits::InstructionData;

/// Instruction data for DeregisterAgent
///
/// This instruction takes no additional data beyond the accounts.
/// The authority closes their registry account and reclaims rent.
pub struct DeregisterAgentData;

impl<'a> TryFrom<&'a [u8]> for DeregisterAgentData {
    type Error = ProgramError;

    #[inline(always)]
    fn try_from(_data: &'a [u8]) -> Result<Self, Self::Error> {
        // Deregister takes no instruction data - empty is expected
        Ok(DeregisterAgentData)
    }
}

impl<'a> InstructionData<'a> for DeregisterAgentData {
    const LEN: usize = 0; // No data required
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deregister_agent_data_empty() {
        let data: [u8; 0] = [];
        let result = DeregisterAgentData::try_from(&data[..]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_deregister_agent_data_with_bytes() {
        let data = [1u8, 2u8, 3u8];
        let result = DeregisterAgentData::try_from(&data[..]);
        // Should still work - we ignore any extra data
        assert!(result.is_ok());
    }
}