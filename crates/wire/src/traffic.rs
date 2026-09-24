//! `Traffic`, what went through one port: the transport's counts and
//! the server's.

/// what went through one port since it opened, counted where it happens:
/// the transport counts connections, messages and bytes (a line as
/// written on the socket, its `\n` included), the server counts what the
/// messages were. `errors` adds the lines refused before the app (not
/// JSON, another version) to the errors the server answered.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Traffic {
    pub conns: u64,
    pub msgs_in: u64,
    pub msgs_out: u64,
    pub bytes_in: u64,
    pub bytes_out: u64,
    pub observes_snapshot: u64,
    pub observes_delta: u64,
    pub acts: u64,
    /// acts by what they rest on: `from_revision`, or `observed` given
    /// explicitly. an act with both or neither is in `acts` only.
    pub acts_from_revision: u64,
    pub acts_observed: u64,
    pub stages_out: u64,
    pub rejected_stale: u64,
    pub errors: u64,
}
