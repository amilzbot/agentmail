pub mod fixtures;
pub mod utils;

// AgentMail integration tests
#[cfg(test)]
mod test_register_agent;
#[cfg(test)]
mod test_update_agent;
#[cfg(test)]
mod test_deregister_agent;
