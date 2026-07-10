//! Shared closed schemas used by Responses tool-call inputs and outputs.

mod common;
mod computer;
mod shell;
mod web;

pub use common::*;
pub use computer::*;
pub use shell::*;
pub use web::*;
