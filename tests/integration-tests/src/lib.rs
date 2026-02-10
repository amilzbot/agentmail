pub mod fixtures;
pub mod utils;

// Legacy counter tests (can be removed when cleaning up)
#[cfg(test)]
mod test_close_counter;
#[cfg(test)]
mod test_create_counter;
#[cfg(test)]
mod test_increment;

// AgentMail integration tests
#[cfg(test)]
mod test_register_agent;
#[cfg(test)]
mod test_update_agent;
#[cfg(test)]
mod test_deregister_agent;
