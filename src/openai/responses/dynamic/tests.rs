use std::collections::BTreeSet;

use super::*;
use crate::openai::responses::*;
use crate::openai::OpenAIReasoningEffort;

#[test]
fn dynamic_validation_warns_or_fails_without_coercing() {
    let model = DynamicOpenAIModel::new("gpt-4o").unwrap();
    let warned = DynamicResponseRequest::builder(model.clone())
        .input(OpenAIResponsesInput::Text("secret".into()))
        .reasoning(OpenAIReasoningEffort::High)
        .validation(ValidationMode::Warn)
        .builtin_catalog()
        .build()
        .unwrap();
    assert_eq!(warned.warnings().len(), 1);
    let value = serde_json::to_value(&warned).unwrap();
    assert_eq!(value["reasoning"]["effort"], "high");

    let error = DynamicResponseRequest::builder(model)
        .input(OpenAIResponsesInput::Text("secret".into()))
        .reasoning(OpenAIReasoningEffort::High)
        .validation(ValidationMode::Strict)
        .builtin_catalog()
        .build()
        .unwrap_err();
    assert!(matches!(
        error,
        DynamicRequestError::UnsupportedSetting { .. }
    ));
}

#[test]
fn builtin_catalog_covers_every_checked_in_marker() {
    let catalog = StaticResponseModelCapabilitiesCatalog::builtin();
    for evidence in MODEL_EVIDENCE {
        let capabilities = catalog
            .capabilities(evidence.id)
            .unwrap_or_else(|| panic!("missing {}", evidence.id));
        assert_eq!(
            capabilities.supports("temperature"),
            evidence.sampling,
            "{}",
            evidence.id
        );
        assert_eq!(
            capabilities.supports("top_p"),
            evidence.sampling,
            "{}",
            evidence.id
        );
        assert_eq!(
            capabilities.reasoning_efforts,
            evidence.reasoning.iter().map(ToString::to_string).collect(),
            "{}",
            evidence.id
        );
        assert_eq!(
            capabilities.prompt_cache_retentions,
            evidence
                .cache_retentions
                .iter()
                .map(ToString::to_string)
                .collect(),
            "{}",
            evidence.id
        );
        assert_eq!(
            capabilities.supports("structured_output"),
            evidence.structured_output,
            "{}",
            evidence.id
        );
        assert_eq!(
            capabilities.supports("image_generation_tool"),
            evidence.image_tool,
            "{}",
            evidence.id
        );
    }
}

#[test]
fn strict_rejects_unknown_retention_and_sampling_values_are_bounded() {
    assert!(Temperature::new(f64::NAN).is_err());
    assert!(Temperature::new(2.1).is_err());
    assert!(TopP::new(-0.1).is_err());

    let error = DynamicResponseRequest::builder(DynamicOpenAIModel::new("gpt-5.5").unwrap())
        .input(OpenAIResponsesInput::Text("secret".into()))
        .prompt_cache_retention("in_memory")
        .validation(ValidationMode::Strict)
        .builtin_catalog()
        .build()
        .unwrap_err();
    assert!(matches!(
        error,
        DynamicRequestError::UnsupportedSetting {
            setting: "prompt_cache_retention_value",
            ..
        }
    ));
}

#[test]
fn builtin_catalog_matches_representative_typed_capability_groups() {
    let catalog = StaticResponseModelCapabilitiesCatalog::builtin();
    let gpt4o = catalog.capabilities("gpt-4o").unwrap();
    assert!(gpt4o.supports("temperature"));
    assert!(gpt4o.supports("image_generation_tool"));
    assert!(!gpt4o.supports("reasoning"));
    assert!(catalog
        .capabilities("gpt-5-nano")
        .unwrap()
        .supports("image_generation_tool"));
    assert!(!catalog
        .capabilities("gpt-5.1")
        .unwrap()
        .supports("image_generation_tool"));

    let pro = catalog.capabilities("gpt-5.5-pro").unwrap();
    assert!(pro.reasoning_efforts.contains("xhigh"));
    assert!(!pro.reasoning_efforts.contains("low"));
    assert_eq!(
        pro.prompt_cache_retentions,
        BTreeSet::from(["24h".to_string()])
    );
}

#[test]
fn dynamic_plain_text_is_not_misclassified_as_structured_output() {
    let model = DynamicOpenAIModel::new("gpt-5.4-pro").unwrap();
    DynamicResponseRequest::builder(model.clone())
        .input(OpenAIResponsesInput::Text("hello".into()))
        .text_format(OpenAIResponsesTextFormat::Text(Default::default()))
        .validation(ValidationMode::Strict)
        .builtin_catalog()
        .build()
        .unwrap();

    let error = DynamicResponseRequest::builder(model)
        .input(OpenAIResponsesInput::Text("hello".into()))
        .text_format(OpenAIResponsesTextFormat::JsonSchema(
            crate::openai::OpenAIJsonSchema {
                name: "result".into(),
                description: "result".into(),
                schema: serde_json::json!({"type":"object"}),
                strict: Some(true),
            }
            .into(),
        ))
        .validation(ValidationMode::Strict)
        .builtin_catalog()
        .build()
        .unwrap_err();
    assert!(matches!(
        error,
        DynamicRequestError::UnsupportedSetting {
            setting: "structured_output",
            ..
        }
    ));
}

#[test]
fn dynamic_builder_reaches_every_non_transport_create_field() {
    let metadata = OpenAIResponseMetadata::new([("key", "value")]).unwrap();
    let request = DynamicResponseRequest::builder(DynamicOpenAIModel::new("custom-model").unwrap())
        .metadata(metadata)
        .top_logprobs(TopLogprobs::new(5).unwrap())
        .temperature(Temperature::new(0.5).unwrap())
        .top_p(TopP::new(0.9).unwrap())
        .user("legacy-user")
        .safety_identifier("safe-user")
        .prompt_cache_key("cache")
        .service_tier(OpenAIServiceTier::Default)
        .prompt_cache_retention("24h")
        .prompt_cache_options(OpenAIPromptCacheOptions::default())
        .previous_response_id(ResponseId::new("resp_previous").unwrap())
        .background(true)
        .max_tool_calls(4)
        .text_config(OpenAIResponsesTextConfig {
            format: Some(OpenAIResponsesTextFormat::Text(Default::default())),
            verbosity: Some(OpenAIResponseVerbosity::Low),
            extra: Default::default(),
        })
        .tool(OpenAIResponsesTool::image_generation())
        .tool_choice(OpenAIToolChoice::Mode(OpenAIToolChoiceMode::Auto))
        .prompt(OpenAIPromptTemplate {
            id: "pmpt_1".into(),
            version: Some("1".into()),
            variables: None,
        })
        .truncation(OpenAITruncation::Disabled)
        .reasoning_config(OpenAIResponsesReasoning {
            mode: Some("standard".into()),
            effort: Some(crate::openai::OpenAIReasoningEffort::High),
            summary: Some(OpenAIReasoningSummary::Concise),
            context: Some(OpenAIReasoningContext::CurrentTurn),
            generate_summary: None,
            extra: Default::default(),
        })
        .input(OpenAIResponsesInput::Text("hello".into()))
        .include(ResponseInclude::ReasoningEncryptedContent)
        .parallel_tool_calls(true)
        .store(true)
        .instructions("instructions")
        .moderation(OpenAIModerationConfig {
            model: "omni-moderation-latest".into(),
            policy: Some(OpenAIModerationPolicy {
                input: Some(OpenAIModerationRule {
                    mode: OpenAIModerationMode::Score,
                }),
                output: None,
            }),
        })
        .conversation(OpenAIConversationReference::Id(
            crate::openai::conversations::ConversationId::new("conv_1").unwrap(),
        ))
        .context_management(vec![OpenAIContextCompaction::new(Some(1000)).unwrap()])
        .max_output_tokens(64)
        .build()
        .unwrap();
    let object = serde_json::to_value(request)
        .unwrap()
        .as_object()
        .unwrap()
        .keys()
        .cloned()
        .collect::<std::collections::BTreeSet<_>>();
    let expected = [
        "metadata",
        "top_logprobs",
        "model",
        "input",
        "instructions",
        "max_output_tokens",
        "temperature",
        "top_p",
        "user",
        "safety_identifier",
        "prompt_cache_key",
        "prompt_cache_retention",
        "prompt_cache_options",
        "text",
        "previous_response_id",
        "service_tier",
        "background",
        "max_tool_calls",
        "store",
        "reasoning",
        "tools",
        "tool_choice",
        "prompt",
        "truncation",
        "include",
        "parallel_tool_calls",
        "moderation",
        "conversation",
        "context_management",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(object, expected);
}
