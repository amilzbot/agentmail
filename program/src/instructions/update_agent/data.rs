use alloc::string::String;
use pinocchio::error::ProgramError;

use crate::{require_len, traits::InstructionData, errors::AgentMailProgramError};

/// Instruction data for UpdateAgent
///
/// # Layout
/// * `name_len` (u32, LE) - Length of agent name
/// * `name` (variable) - Agent name (UTF-8)
/// * `inbox_url_len` (u32, LE) - Length of inbox URL
/// * `inbox_url` (variable) - Inbox URL (UTF-8)
#[derive(Debug, PartialEq)]
pub struct UpdateAgentData {
    pub name: String,
    pub inbox_url: String,
}

impl<'a> TryFrom<&'a [u8]> for UpdateAgentData {
    type Error = ProgramError;

    #[inline(always)]
    fn try_from(data: &'a [u8]) -> Result<Self, Self::Error> {
        if data.is_empty() {
            return Err(ProgramError::InvalidInstructionData);
        }

        let mut offset = 0;

        // Read name length
        require_len!(data, offset + 4);
        let name_len = u32::from_le_bytes([
            data[offset],
            data[offset + 1],
            data[offset + 2],
            data[offset + 3],
        ]) as usize;
        offset += 4;

        // Validate name length
        if name_len > 64 {
            return Err(AgentMailProgramError::NameTooLong.into());
        }

        // Read name data
        require_len!(data, offset + name_len);
        let name_bytes = &data[offset..offset + name_len];
        let name = String::from_utf8(name_bytes.to_vec())
            .map_err(|_| AgentMailProgramError::InvalidUtf8)?;
        offset += name_len;

        // Read inbox URL length
        require_len!(data, offset + 4);
        let url_len = u32::from_le_bytes([
            data[offset],
            data[offset + 1],
            data[offset + 2],
            data[offset + 3],
        ]) as usize;
        offset += 4;

        // Validate URL length
        if url_len > 256 {
            return Err(AgentMailProgramError::InboxUrlTooLong.into());
        }

        // Read inbox URL data
        require_len!(data, offset + url_len);
        let url_bytes = &data[offset..offset + url_len];
        let inbox_url = String::from_utf8(url_bytes.to_vec())
            .map_err(|_| AgentMailProgramError::InvalidUtf8)?;

        Ok(Self {
            name,
            inbox_url,
        })
    }
}

impl<'a> InstructionData<'a> for UpdateAgentData {
    const LEN: usize = 0; // Variable length, so we override validation
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec::Vec;
    use alloc::borrow::ToOwned;

    fn create_test_data(name: &str, url: &str) -> Vec<u8> {
        let mut data = Vec::new();
        
        let name_bytes = name.as_bytes();
        data.extend_from_slice(&(name_bytes.len() as u32).to_le_bytes());
        data.extend_from_slice(name_bytes);
        
        let url_bytes = url.as_bytes();
        data.extend_from_slice(&(url_bytes.len() as u32).to_le_bytes());
        data.extend_from_slice(url_bytes);
        
        data
    }

    #[test]
    fn test_update_agent_data_try_from_valid() {
        let data = create_test_data("updated-agent", "https://updated.example.com/inbox");
        let result = UpdateAgentData::try_from(&data[..]);
        
        assert!(result.is_ok());
        let update_data = result.unwrap();
        assert_eq!(update_data.name, "updated-agent");
        assert_eq!(update_data.inbox_url, "https://updated.example.com/inbox");
    }

    #[test]
    fn test_update_agent_data_try_from_empty() {
        let data: [u8; 0] = [];
        let result = UpdateAgentData::try_from(&data[..]);
        assert!(matches!(result, Err(ProgramError::InvalidInstructionData)));
    }

    #[test]
    fn test_update_agent_data_name_too_long() {
        let long_name = "a".repeat(65);
        let data = create_test_data(&long_name, "https://test.example.com/inbox");
        let result = UpdateAgentData::try_from(&data[..]);
        assert_eq!(result, Err(AgentMailProgramError::NameTooLong.into()));
    }

    #[test]
    fn test_update_agent_data_url_too_long() {
        let long_url = "https://".to_owned() + &"a".repeat(250);
        let data = create_test_data("test", &long_url);
        let result = UpdateAgentData::try_from(&data[..]);
        assert_eq!(result, Err(AgentMailProgramError::InboxUrlTooLong.into()));
    }

    #[test]
    fn test_update_agent_data_invalid_utf8() {
        let mut data = Vec::new();
        data.extend_from_slice(&4u32.to_le_bytes()); // name length
        data.extend_from_slice(&[0xFF, 0xFE, 0xFD, 0xFC]); // invalid UTF-8
        data.extend_from_slice(&4u32.to_le_bytes()); // url length
        data.extend_from_slice(b"test"); // valid URL

        let result = UpdateAgentData::try_from(&data[..]);
        assert_eq!(result, Err(AgentMailProgramError::InvalidUtf8.into()));
    }

    #[test]
    fn test_update_agent_data_minimum_data() {
        let data = create_test_data("", "");
        let result = UpdateAgentData::try_from(&data[..]);
        
        assert!(result.is_ok());
        let update_data = result.unwrap();
        assert_eq!(update_data.name, "");
        assert_eq!(update_data.inbox_url, "");
    }
}