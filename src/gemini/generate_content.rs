use std::{collections::HashSet, fmt::Display};

use serde::{Deserialize, Serialize};

use crate::utils::{base64_decode, base64_encode};

use super::Model;

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Content {
    pub parts: Vec<Part>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<Role>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    User,
    Model,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Part {
    Text(String),
    #[serde(rename = "inlineData")]
    Blob {
        mime_type: String,
        /// Base64 encoded data.
        data: String,
    },
}

impl Part {
    /// Create a new text part.
    pub fn text(text: impl Into<String>) -> Self {
        Part::Text(text.into())
    }

    /// Create a new blob part.
    pub fn blob(mime_type: &str, data: Vec<u8>) -> Self {
        Part::Blob {
            mime_type: mime_type.to_string(),
            data: base64_encode(&data),
        }
    }

    /// Create a new blob part with base64 encoded data.
    pub fn blob_base64(mime_type: &str, data: &str) -> Self {
        Part::Blob {
            mime_type: mime_type.to_string(),
            data: data.to_string(),
        }
    }

    /// Get the text data if this part is a text part.
    pub fn as_text(&self) -> Option<&str> {
        match &self {
            Part::Text(text) => Some(text),
            _ => None,
        }
    }

    /// Get the blob data if this part is a blob part.
    pub fn as_blob(&self) -> Option<(String, Vec<u8>)> {
        match &self {
            Part::Blob { mime_type, data } => base64_decode(data)
                .ok()
                .map(|data| (mime_type.clone(), data)),
            _ => None,
        }
    }

    /// Get the base64 encoded blob data if this part is a blob part.
    pub fn as_blob_base64(&self) -> Option<(&str, &str)> {
        match &self {
            Part::Blob { mime_type, data } => Some((mime_type, data)),
            _ => None,
        }
    }
}

/// Request type used in the `countTokens` endpoint.
///
/// NOTE: The countTokens endpoint includes the `model`
///       field in the GenerateContentRequest type.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CountTokensGenerateContentRequest {
    pub model: Model,
    #[serde(flatten)]
    pub request: GenerateContentRequest,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GenerateContentRequest {
    pub contents: Vec<Content>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub safety_settings: Option<HashSet<SafetySetting>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub generation_config: Option<GenerationConfig>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SafetySetting {
    pub category: HarmCategory,
    pub threshold: HarmBlockThreshold,
}

// NOTE: There should only be one SafetySetting per category.
impl std::hash::Hash for SafetySetting {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.category.hash(state);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum HarmCategory {
    #[serde(rename = "HARM_CATEGORY_UNSPECIFIED")]
    Unspecified,
    #[serde(rename = "HARM_CATEGORY_DEROGATORY")]
    Derogatory,
    #[serde(rename = "HARM_CATEGORY_TOXICITY")]
    Toxicity,
    #[serde(rename = "HARM_CATEGORY_VIOLENCE")]
    Violence,
    #[serde(rename = "HARM_CATEGORY_SEXUAL")]
    Sexual,
    #[serde(rename = "HARM_CATEGORY_MEDICAL")]
    Medical,
    #[serde(rename = "HARM_CATEGORY_DANGEROUS")]
    Dangerous,
    #[serde(rename = "HARM_CATEGORY_HARASSMENT")]
    Harassment,
    #[serde(rename = "HARM_CATEGORY_HATE_SPEECH")]
    HateSpeech,
    #[serde(rename = "HARM_CATEGORY_SEXUALLY_EXPLICIT")]
    SexuallyExplicit,
    #[serde(rename = "HARM_CATEGORY_DANGEROUS_CONTENT")]
    DangerousContent,
}

impl Display for HarmCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Descriptions are taken from the API documentation.
        let desc = match self {
            HarmCategory::Unspecified => "Category is unspecified",
            HarmCategory::Derogatory => "Negative or harmful comments targeting identity and/or protected attribute",
            HarmCategory::Toxicity => "Content that is rude, disrespectful, or profane",
            HarmCategory::Violence => "Describes scenarios depicting violence against an individual or group, or general descriptions of gore",
            HarmCategory::Sexual => "Contains references to sexual acts or other lewd content",
            HarmCategory::Medical => "Promotes unchecked medical advice",
            HarmCategory::Dangerous => "Dangerous content that promotes, facilitates, or encourages harmful acts",
            HarmCategory::Harassment => "Harassment content",
            HarmCategory::HateSpeech => "Hate speech and content",
            HarmCategory::SexuallyExplicit => "Sexually explicit content",
            HarmCategory::DangerousContent => "Dangerous content",
        };
        write!(f, "{desc}")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum HarmBlockThreshold {
    #[serde(rename = "HARM_BLOCK_THRESHOLD_UNSPECIFIED")]
    Unspecified,
    #[serde(rename = "BLOCK_LOW_AND_ABOVE")]
    LowAndAbove,
    #[serde(rename = "BLOCK_MEDIUM_AND_ABOVE")]
    MediumAndAbove,
    #[serde(rename = "BLOCK_ONLY_HIGH")]
    OnlyHigh,
    #[serde(rename = "BLOCK_NONE")]
    None,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GenerationConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_sequences: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub candidate_count: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_output_tokens: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_k: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GenerateContentResponse {
    pub candidates: Vec<Candidate>,
    pub prompt_feedback: Option<PromptFeedback>,
    pub usage_metadata: UsageMetadata,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Candidate {
    pub content: Content,
    pub finish_reason: FinishReason,
    pub safety_ratings: Vec<SafetyRating>,
    pub citation_metadata: Option<CitationMetadata>,
    pub token_count: Option<u64>,
    pub index: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum FinishReason {
    #[serde(rename = "FINISH_REASON_UNSPECIFIED")]
    Unspecified,
    Stop,
    MaxTokens,
    Safety,
    Recitation,
    Other,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SafetyRating {
    pub category: HarmCategory,
    pub probability: HarmProbability,
    pub blocked: Option<bool>,
}

impl SafetyRating {
    pub fn blocked(&self) -> bool {
        self.blocked.unwrap_or(false)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum HarmProbability {
    #[serde(rename = "HARM_PROBABILITY_UNSPECIFIED")]
    Unspecified,
    Negligible,
    Low,
    Medium,
    High,
}

impl Display for HarmProbability {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Descriptions are taken from the API documentation.
        let desc = match self {
            HarmProbability::Unspecified => "Probability is unspecified",
            HarmProbability::Negligible => "Content has a negligible chance of being unsafe",
            HarmProbability::Low => "Content has a low chance of being unsafe",
            HarmProbability::Medium => "Content has a medium chance of being unsafe",
            HarmProbability::High => "Content has a high chance of being unsafe",
        };
        write!(f, "{desc}")
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CitationMetadata {
    pub citation_sources: Vec<CitationSource>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CitationSource {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_index: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_index: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uri: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub license: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PromptFeedback {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub block_reason: Option<BlockReason>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub safety_ratings: Option<Vec<SafetyRating>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum BlockReason {
    #[serde(rename = "BLOCK_REASON_UNSPECIFIED")]
    Unspecified,
    Safety,
    Other,
}

impl Display for BlockReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Descriptions are taken from the API documentation.
        let desc = match self {
            BlockReason::Unspecified => "Block reason is unspecified",
            BlockReason::Safety => "Prompt was blocked due to safety reasons. You can inspect safetyRatings to understand which safety category blocked it",
            BlockReason::Other => "Prompt was blocked due to unknown reasons",
        };
        write!(f, "{desc}")
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UsageMetadata {
    pub prompt_token_count: u64,
    pub candidates_token_count: u64,
    pub total_token_count: u64,
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn test_content_round_trip() {
        let content = Content {
            parts: vec![Part::text("Hello, World!")],
            role: Some(Role::User),
        };

        let serialized = serde_json::to_string(&content).unwrap();
        let deserialized: Content = serde_json::from_str(&serialized).unwrap();

        assert_eq!(content, deserialized);
    }

    #[test]
    fn test_content_deserialize() {
        let content_json = json!({
            "parts": [{"text": "Hello, World!"}],
            "role": "user"
        });

        let deserialized: Content = serde_json::from_value(content_json).unwrap();
        assert_eq!(
            deserialized,
            Content {
                parts: vec![Part::text("Hello, World!")],
                role: Some(Role::User),
            }
        );
    }

    #[test]
    fn test_content_serialize() {
        let content = Content {
            parts: vec![Part::text("Hello, World!")],
            role: Some(Role::User),
        };

        let serialized = serde_json::to_value(&content).unwrap();
        assert_eq!(
            serialized,
            json!({
                "parts": [{"text": "Hello, World!"}],
                "role": "user"
            })
        );
    }

    #[test]
    fn test_request_round_trip() {
        let request = GenerateContentRequest {
            contents: vec![Content {
                parts: vec![Part::text("Hello, World!")],
                role: Some(Role::User),
            }],
            safety_settings: Some(HashSet::from([SafetySetting {
                category: HarmCategory::Unspecified,
                threshold: HarmBlockThreshold::Unspecified,
            }])),
            generation_config: Some(GenerationConfig {
                stop_sequences: Some(vec![".".to_string()]),
                candidate_count: Some(1),
                max_output_tokens: Some(10),
                temperature: Some(0.5),
                top_p: Some(0.9),
                top_k: Some(100),
            }),
        };

        let serialized = serde_json::to_string(&request).unwrap();
        let deserialized: GenerateContentRequest = serde_json::from_str(&serialized).unwrap();

        assert_eq!(request, deserialized);
    }

    #[test]
    fn test_request_deserialize() {
        let request_json = json!({
            "contents": [{
                "parts": [{"text": "Hello, World!"}],
                "role": "user"
            }],
            "safetySettings": [{
                "category": "HARM_CATEGORY_UNSPECIFIED",
                "threshold": "HARM_BLOCK_THRESHOLD_UNSPECIFIED"
            }],
            "generationConfig": {
                "stopSequences": ["."],
                "candidateCount": 1,
                "maxOutputTokens": 10,
                "temperature": 0.5,
                "topP": 0.9,
                "topK": 100
            }
        });

        let deserialized: GenerateContentRequest = serde_json::from_value(request_json).unwrap();
        assert_eq!(
            deserialized,
            GenerateContentRequest {
                contents: vec![Content {
                    parts: vec![Part::text("Hello, World!")],
                    role: Some(Role::User),
                }],
                safety_settings: Some(HashSet::from([SafetySetting {
                    category: HarmCategory::Unspecified,
                    threshold: HarmBlockThreshold::Unspecified,
                }])),
                generation_config: Some(GenerationConfig {
                    stop_sequences: Some(vec![".".to_string()]),
                    candidate_count: Some(1),
                    max_output_tokens: Some(10),
                    temperature: Some(0.5),
                    top_p: Some(0.9),
                    top_k: Some(100),
                }),
            }
        );
    }

    #[test]
    fn test_request_serialize() {
        let request = GenerateContentRequest {
            contents: vec![Content {
                parts: vec![Part::text("Hello, World!")],
                role: Some(Role::User),
            }],
            safety_settings: Some(HashSet::from([SafetySetting {
                category: HarmCategory::Unspecified,
                threshold: HarmBlockThreshold::Unspecified,
            }])),
            generation_config: Some(GenerationConfig {
                stop_sequences: Some(vec![".".to_string()]),
                candidate_count: Some(1),
                max_output_tokens: Some(10),
                temperature: Some(0.5),
                top_p: Some(0.9),
                top_k: Some(100),
            }),
        };

        let serialized = serde_json::to_value(&request).unwrap();
        assert_eq!(
            serialized,
            json!({
                "contents": [{
                    "parts": [{"text": "Hello, World!"}],
                    "role": "user"
                }],
                "safetySettings": [{
                    "category": "HARM_CATEGORY_UNSPECIFIED",
                    "threshold": "HARM_BLOCK_THRESHOLD_UNSPECIFIED"
                }],
                "generationConfig": {
                    "stopSequences": ["."],
                    "candidateCount": 1,
                    "maxOutputTokens": 10,
                    "temperature": 0.5,
                    "topP": 0.9,
                    "topK": 100
                }
            })
        );
    }

    #[test]
    fn test_response_deserialize() {
        let response_json = json!({
          "candidates": [
            {
              "content": {
                "parts": [
                  {
                    "text": "Hello, World!"
                  }
                ],
                "role": "model"
              },
              "finishReason": "STOP",
              "index": 0,
              "safetyRatings": [
                {
                  "category": "HARM_CATEGORY_SEXUALLY_EXPLICIT",
                  "probability": "NEGLIGIBLE"
                },
                {
                  "category": "HARM_CATEGORY_HATE_SPEECH",
                  "probability": "NEGLIGIBLE"
                },
                {
                  "category": "HARM_CATEGORY_HARASSMENT",
                  "probability": "NEGLIGIBLE"
                },
                {
                  "category": "HARM_CATEGORY_DANGEROUS_CONTENT",
                  "probability": "NEGLIGIBLE"
                }
              ]
            }
          ],
          "usageMetadata": {
            "promptTokenCount": 21,
            "candidatesTokenCount": 8,
            "totalTokenCount": 29
          }
        });

        let deserialized: GenerateContentResponse = serde_json::from_value(response_json).unwrap();
        assert_eq!(
            deserialized,
            GenerateContentResponse {
                candidates: vec![Candidate {
                    content: Content {
                        parts: vec![Part::text("Hello, World!")],
                        role: Some(Role::Model),
                    },
                    finish_reason: FinishReason::Stop,
                    safety_ratings: vec![
                        SafetyRating {
                            category: HarmCategory::SexuallyExplicit,
                            probability: HarmProbability::Negligible,
                            blocked: None,
                        },
                        SafetyRating {
                            category: HarmCategory::HateSpeech,
                            probability: HarmProbability::Negligible,
                            blocked: None,
                        },
                        SafetyRating {
                            category: HarmCategory::Harassment,
                            probability: HarmProbability::Negligible,
                            blocked: None,
                        },
                        SafetyRating {
                            category: HarmCategory::DangerousContent,
                            probability: HarmProbability::Negligible,
                            blocked: None,
                        }
                    ],
                    citation_metadata: None,
                    token_count: None,
                    index: 0,
                }],
                prompt_feedback: None,
                usage_metadata: UsageMetadata {
                    prompt_token_count: 21,
                    candidates_token_count: 8,
                    total_token_count: 29,
                },
            }
        );
    }
}
