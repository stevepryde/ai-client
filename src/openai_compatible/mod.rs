//! Typed clients for APIs that implement selected OpenAI-shaped resources.

pub mod chat;
mod client;
mod dialect;

pub use client::*;
pub use dialect::*;
