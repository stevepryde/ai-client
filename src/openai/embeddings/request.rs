use std::marker::PhantomData;

use serde::Serialize;

use super::OpenAIEmbeddingModel;

/// Invalid client-side embedding request configuration.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum EmbeddingRequestError {
    #[error("embedding input must contain at least one text")]
    EmptyInput,
    #[error("embedding input text must not be empty")]
    EmptyText,
    #[error("embedding dimensions must be between 1 and {maximum}, got {actual}")]
    InvalidDimensions { actual: u16, maximum: u16 },
}

/// One text or a non-empty batch of texts to embed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(transparent)]
pub struct EmbeddingInput(EmbeddingInputWire);

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(untagged)]
enum EmbeddingInputWire {
    Text(String),
    Batch(Vec<String>),
}

impl EmbeddingInput {
    /// Construct a validated single-text input.
    pub fn text(value: impl Into<String>) -> Result<Self, EmbeddingRequestError> {
        let value = value.into();
        validate_text(&value)?;
        Ok(Self(EmbeddingInputWire::Text(value)))
    }

    /// Construct a validated batch input while preserving provider output order.
    pub fn batch<I, S>(values: I) -> Result<Self, EmbeddingRequestError>
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let values = values.into_iter().map(Into::into).collect::<Vec<_>>();
        if values.is_empty() {
            return Err(EmbeddingRequestError::EmptyInput);
        }
        for value in &values {
            validate_text(value)?;
        }
        Ok(Self(EmbeddingInputWire::Batch(values)))
    }

    /// Number of embeddings requested.
    pub fn len(&self) -> usize {
        match &self.0 {
            EmbeddingInputWire::Text(_) => 1,
            EmbeddingInputWire::Batch(values) => values.len(),
        }
    }

    /// Embedding input is always non-empty after successful construction.
    pub fn is_empty(&self) -> bool {
        false
    }
}

fn validate_text(value: &str) -> Result<(), EmbeddingRequestError> {
    if value.trim().is_empty() {
        Err(EmbeddingRequestError::EmptyText)
    } else {
        Ok(())
    }
}

/// A model-bound embedding output dimension.
#[derive(Debug, PartialEq, Eq, Hash)]
pub struct EmbeddingDimensions<M: OpenAIEmbeddingModel> {
    value: u16,
    model: PhantomData<fn() -> M>,
}

impl<M: OpenAIEmbeddingModel> Clone for EmbeddingDimensions<M> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<M: OpenAIEmbeddingModel> Copy for EmbeddingDimensions<M> {}

impl<M: OpenAIEmbeddingModel> EmbeddingDimensions<M> {
    /// Validate a dimension against the selected model's documented maximum.
    pub fn new(value: u16) -> Result<Self, EmbeddingRequestError> {
        if value == 0 || value > M::MAX_DIMENSIONS {
            return Err(EmbeddingRequestError::InvalidDimensions {
                actual: value,
                maximum: M::MAX_DIMENSIONS,
            });
        }
        Ok(Self {
            value,
            model: PhantomData,
        })
    }

    pub fn get(self) -> u16 {
        self.value
    }
}

/// Typed request for `POST /v1/embeddings`.
#[derive(Debug, Clone, Serialize)]
pub struct CreateEmbeddingRequest<M: OpenAIEmbeddingModel> {
    model: &'static str,
    input: EmbeddingInput,
    encoding_format: EmbeddingEncodingFormat,
    #[serde(skip_serializing_if = "Option::is_none")]
    dimensions: Option<u16>,
    #[serde(skip)]
    model_marker: PhantomData<fn() -> M>,
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
enum EmbeddingEncodingFormat {
    Float,
}

impl<M: OpenAIEmbeddingModel> CreateEmbeddingRequest<M> {
    pub fn new(input: EmbeddingInput) -> Self {
        Self {
            model: M::ID,
            input,
            encoding_format: EmbeddingEncodingFormat::Float,
            dimensions: None,
            model_marker: PhantomData,
        }
    }

    /// Request a shortened embedding supported by this model.
    pub fn dimensions(mut self, dimensions: EmbeddingDimensions<M>) -> Self {
        self.dimensions = Some(dimensions.get());
        self
    }

    pub fn input(&self) -> &EmbeddingInput {
        &self.input
    }

    pub fn requested_dimensions(&self) -> Option<u16> {
        self.dimensions
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::openai::embeddings::TextEmbedding3Large;

    #[test]
    fn serializes_single_and_batch_requests_with_typed_dimensions() {
        let single = CreateEmbeddingRequest::<TextEmbedding3Large>::new(
            EmbeddingInput::text("one fact").unwrap(),
        );
        assert_eq!(
            serde_json::to_value(single).unwrap(),
            serde_json::json!({
                "model": "text-embedding-3-large",
                "input": "one fact",
                "encoding_format": "float"
            })
        );

        let batch = CreateEmbeddingRequest::<TextEmbedding3Large>::new(
            EmbeddingInput::batch(["first", "second"]).unwrap(),
        )
        .dimensions(EmbeddingDimensions::new(1_024).unwrap());
        assert_eq!(batch.input().len(), 2);
        assert_eq!(batch.requested_dimensions(), Some(1_024));
        assert_eq!(
            serde_json::to_value(batch).unwrap(),
            serde_json::json!({
                "model": "text-embedding-3-large",
                "input": ["first", "second"],
                "encoding_format": "float",
                "dimensions": 1024
            })
        );
    }

    #[test]
    fn rejects_empty_input_and_out_of_range_dimensions() {
        assert_eq!(
            EmbeddingInput::batch(Vec::<String>::new()).unwrap_err(),
            EmbeddingRequestError::EmptyInput
        );
        assert_eq!(
            EmbeddingInput::text(" \n ").unwrap_err(),
            EmbeddingRequestError::EmptyText
        );
        assert_eq!(
            EmbeddingDimensions::<TextEmbedding3Large>::new(0).unwrap_err(),
            EmbeddingRequestError::InvalidDimensions {
                actual: 0,
                maximum: 3_072
            }
        );
        assert_eq!(
            EmbeddingDimensions::<TextEmbedding3Large>::new(3_073).unwrap_err(),
            EmbeddingRequestError::InvalidDimensions {
                actual: 3_073,
                maximum: 3_072
            }
        );
    }
}
