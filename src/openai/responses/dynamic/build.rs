use crate::openai::responses::{OpenAIResponsesInput, PreparedResponseRequest};

use super::{builder::DynamicResponseRequestBuilder, catalog::DynamicRequestError};

impl DynamicResponseRequestBuilder {
    pub fn input_text(mut self, input: impl Into<String>) -> Self {
        self.wire.input = Some(OpenAIResponsesInput::Text(input.into()));
        self
    }

    pub fn input(mut self, input: impl Into<OpenAIResponsesInput>) -> Self {
        self.wire.input = Some(input.into());
        self
    }

    pub fn build(self) -> Result<PreparedResponseRequest, DynamicRequestError> {
        let warnings = self.validate()?;
        Ok(PreparedResponseRequest::new(self.wire, warnings))
    }
}
