//! Compile-time checked and dynamic-ready Responses request wire protocol.

mod builder;
mod options;
mod wire;

pub use builder::*;
pub use options::*;
pub(crate) use wire::OpenAIResponsesWireRequest;
pub use wire::PreparedResponseRequest;
