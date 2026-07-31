/// A model accepted by OpenAI's embeddings endpoint.
pub trait OpenAIEmbeddingModel: Send + Sync + 'static {
    /// Provider model identifier serialized on the wire.
    const ID: &'static str;
    /// Largest supported output dimension for this model.
    const MAX_DIMENSIONS: u16;
}

/// OpenAI's most capable third-generation embedding model.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct TextEmbedding3Large;

impl OpenAIEmbeddingModel for TextEmbedding3Large {
    const ID: &'static str = "text-embedding-3-large";
    const MAX_DIMENSIONS: u16 = 3_072;
}
