pub mod error;
pub mod gemini;
pub mod openai;

pub(crate) mod utils;

pub mod prelude {
    pub use crate::error::{AiError, AiResult};
}
