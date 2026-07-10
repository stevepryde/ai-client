#![doc = include_str!("../README.md")]

pub mod error;
pub mod gemini;
pub mod openai;

mod core;
pub(crate) mod utils;

pub mod prelude {
    pub use crate::error::{AiError, AiResponse, AiResult, ResponseMetadata};
}
