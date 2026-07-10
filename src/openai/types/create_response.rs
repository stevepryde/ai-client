//! Migration reexports for the pre-0.4 Responses protocol module.
//!
//! New code should import these types from [`crate::openai::responses`].

pub use crate::openai::responses::{events::*, input::*, output::*, tools::*};
