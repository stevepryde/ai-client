use serde::{Deserialize, Serialize};

use super::{ResponseItemId, ResponseOperationError};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ListOrder {
    Asc,
    #[default]
    Desc,
}

impl ListOrder {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Asc => "asc",
            Self::Desc => "desc",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResponseInclude {
    #[serde(rename = "file_search_call.results")]
    FileSearchCallResults,
    #[serde(rename = "web_search_call.results")]
    WebSearchCallResults,
    #[serde(rename = "web_search_call.action.sources")]
    WebSearchCallActionSources,
    #[serde(rename = "message.input_image.image_url")]
    MessageInputImageUrl,
    #[serde(rename = "computer_call_output.output.image_url")]
    ComputerCallOutputImageUrl,
    #[serde(rename = "code_interpreter_call.outputs")]
    CodeInterpreterCallOutputs,
    #[serde(rename = "reasoning.encrypted_content")]
    ReasoningEncryptedContent,
    #[serde(rename = "message.output_text.logprobs")]
    MessageOutputTextLogprobs,
}

impl ResponseInclude {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::FileSearchCallResults => "file_search_call.results",
            Self::WebSearchCallResults => "web_search_call.results",
            Self::WebSearchCallActionSources => "web_search_call.action.sources",
            Self::MessageInputImageUrl => "message.input_image.image_url",
            Self::ComputerCallOutputImageUrl => "computer_call_output.output.image_url",
            Self::CodeInterpreterCallOutputs => "code_interpreter_call.outputs",
            Self::ReasoningEncryptedContent => "reasoning.encrypted_content",
            Self::MessageOutputTextLogprobs => "message.output_text.logprobs",
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RetrieveResponseOptions {
    include: Vec<ResponseInclude>,
}

impl RetrieveResponseOptions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn include(mut self, include: ResponseInclude) -> Self {
        self.include.push(include);
        self
    }

    pub fn includes(&self) -> &[ResponseInclude] {
        &self.include
    }

    pub(crate) fn query(&self) -> Vec<(String, String)> {
        include_query(&self.include)
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RetrieveResponseStreamOptions {
    include: Vec<ResponseInclude>,
    starting_after: Option<u64>,
    include_obfuscation: Option<bool>,
}

impl RetrieveResponseStreamOptions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn include(mut self, include: ResponseInclude) -> Self {
        self.include.push(include);
        self
    }

    pub fn starting_after(mut self, sequence_number: u64) -> Self {
        self.starting_after = Some(sequence_number);
        self
    }

    pub fn include_obfuscation(mut self, include: bool) -> Self {
        self.include_obfuscation = Some(include);
        self
    }

    pub fn includes(&self) -> &[ResponseInclude] {
        &self.include
    }

    #[cfg(feature = "stream")]
    pub(crate) fn query(&self) -> Vec<(String, String)> {
        let mut query = include_query(&self.include);
        query.push(("stream".into(), "true".into()));
        if let Some(starting_after) = self.starting_after {
            query.push(("starting_after".into(), starting_after.to_string()));
        }
        if let Some(include_obfuscation) = self.include_obfuscation {
            query.push((
                "include_obfuscation".into(),
                include_obfuscation.to_string(),
            ));
        }
        query
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ListResponseInputItemsOptions {
    limit: Option<u8>,
    order: Option<ListOrder>,
    after: Option<ResponseItemId>,
    include: Vec<ResponseInclude>,
}

impl ListResponseInputItemsOptions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn limit(mut self, limit: u8) -> Result<Self, ResponseOperationError> {
        if !(1..=100).contains(&limit) {
            return Err(ResponseOperationError::InvalidPageLimit);
        }
        self.limit = Some(limit);
        Ok(self)
    }

    pub fn order(mut self, order: ListOrder) -> Self {
        self.order = Some(order);
        self
    }

    pub fn after(mut self, after: ResponseItemId) -> Self {
        self.after = Some(after);
        self
    }

    pub fn include(mut self, include: ResponseInclude) -> Self {
        self.include.push(include);
        self
    }

    pub(crate) fn query(&self) -> Vec<(String, String)> {
        let mut query = Vec::new();
        if let Some(limit) = self.limit {
            query.push(("limit".into(), limit.to_string()));
        }
        if let Some(order) = self.order {
            query.push(("order".into(), order.as_str().into()));
        }
        if let Some(after) = &self.after {
            query.push(("after".into(), after.as_str().into()));
        }
        query.extend(include_query(&self.include));
        query
    }
}

fn include_query(include: &[ResponseInclude]) -> Vec<(String, String)> {
    include
        .iter()
        .map(|include| ("include[]".into(), include.as_str().into()))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(feature = "stream")]
    #[test]
    fn retrieve_stream_query_owns_stream_mode_and_repeated_includes() {
        let query = RetrieveResponseStreamOptions::new()
            .include(ResponseInclude::ReasoningEncryptedContent)
            .include(ResponseInclude::MessageOutputTextLogprobs)
            .starting_after(41)
            .include_obfuscation(false)
            .query();
        assert_eq!(
            query,
            vec![
                ("include[]".into(), "reasoning.encrypted_content".into()),
                ("include[]".into(), "message.output_text.logprobs".into()),
                ("stream".into(), "true".into()),
                ("starting_after".into(), "41".into()),
                ("include_obfuscation".into(), "false".into()),
            ]
        );
    }

    #[test]
    fn list_options_validate_limits_and_encode_pagination() {
        assert_eq!(
            ListResponseInputItemsOptions::new().limit(0).unwrap_err(),
            ResponseOperationError::InvalidPageLimit
        );
        let query = ListResponseInputItemsOptions::new()
            .limit(100)
            .unwrap()
            .order(ListOrder::Asc)
            .after(ResponseItemId::new("item_123").unwrap())
            .include(ResponseInclude::FileSearchCallResults)
            .query();
        assert_eq!(
            query,
            vec![
                ("limit".into(), "100".into()),
                ("order".into(), "asc".into()),
                ("after".into(), "item_123".into()),
                ("include[]".into(), "file_search_call.results".into()),
            ]
        );
    }
}
