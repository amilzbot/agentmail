use super::{RegisterAgentAccounts, RegisterAgentData};
use crate::{impl_instruction, traits::Instruction};

/// RegisterAgent instruction combining accounts and data
pub struct RegisterAgent<'a> {
    pub accounts: RegisterAgentAccounts<'a>,
    pub data: RegisterAgentData,
}

impl_instruction!(RegisterAgent, RegisterAgentAccounts, RegisterAgentData);

impl<'a> Instruction<'a> for RegisterAgent<'a> {
    type Accounts = RegisterAgentAccounts<'a>;
    type Data = RegisterAgentData;

    #[inline(always)]
    fn accounts(&self) -> &Self::Accounts {
        &self.accounts
    }

    #[inline(always)]
    fn data(&self) -> &Self::Data {
        &self.data
    }
}