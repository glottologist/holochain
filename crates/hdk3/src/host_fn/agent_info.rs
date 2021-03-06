use crate::prelude::*;

/// Trivial wrapper for __agent_info host function.
/// Agent info input struct is `()` so the function call simply looks like this:
///
/// ```ignore
/// let agent_info = agent_info()?;
/// ```
///
/// the AgentInfo is the current agent's original pubkey/address that they joined the network with
/// and their most recent pubkey/address.
pub fn agent_info() -> ExternResult<AgentInfo> {
    host_call::<(), AgentInfo>(__agent_info, ())
}
