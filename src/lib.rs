#![doc = include_str!("../README.md")]

pub mod error;
pub mod gemini;
pub mod openai;
#[cfg(feature = "openai-compatible")]
pub mod openai_compatible;
#[cfg(feature = "stream")]
pub mod stream;

mod core;
pub(crate) mod utils;

pub mod prelude {
    pub use crate::error::{AiError, AiResponse, AiResult, ResponseMetadata};
    #[cfg(feature = "stream")]
    pub use crate::stream::{
        AiStream, AiStreamError, AiStreamErrorKind, SseEventMetadata, SseJsonEvent,
    };
}
