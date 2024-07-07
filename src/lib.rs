pub mod error;
pub mod gemini;

pub(crate) mod utils;

pub mod prelude {
    pub use crate::error::{AiError, AiResult};
}
