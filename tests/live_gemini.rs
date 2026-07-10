#![cfg(feature = "live-tests")]

use std::collections::HashSet;

use ai_client::gemini::{
    Content, CountTokensRequest, GeminiAspectRatio, GeminiClient, GeminiImageConfig,
    GeminiImageSize, GeminiModel, GenerateContentRequest, GenerationConfig, GenerationMethod,
    HarmBlockThreshold, HarmCategory, Part, Role, SafetySetting,
};

fn client() -> GeminiClient {
    GeminiClient::builder()
        .api_key(required_env("GEMINI_API_KEY"))
        .build()
        .expect("GEMINI_API_KEY should build a Gemini client")
}

fn required_env(name: &str) -> String {
    std::env::var(name).unwrap_or_else(|_| {
        panic!("{name} is required because an explicitly ignored live-provider test was requested")
    })
}

fn text_request(generation_config: Option<GenerationConfig>) -> GenerateContentRequest {
    GenerateContentRequest {
        contents: vec![Content {
            parts: vec![Part::text("Reply with only OK.")],
            role: Some(Role::User),
        }],
        safety_settings: None,
        generation_config,
    }
}

#[tokio::test]
#[ignore = "live provider: requires GEMINI_API_KEY; metadata calls do not spend generation tokens"]
async fn live_gemini_core_catalog_covers_every_supported_model() {
    let client = client();
    let listed = client
        .list_models_with_params(ai_client::gemini::ModelsListRequest {
            page_token: None,
            page_size: Some(1_000),
        })
        .await
        .expect("Gemini models.list should succeed")
        .into_inner();

    let listed_names: HashSet<_> = listed
        .models
        .iter()
        .map(|model| model.name.as_str())
        .collect();
    for model in GeminiModel::ALL {
        let resource_name = format!("models/{model}");
        assert!(
            listed_names.contains(resource_name.as_str()),
            "supported Gemini model {model} was not returned by models.list"
        );
        let metadata = client
            .get_model(*model)
            .await
            .unwrap_or_else(|error| panic!("models.get failed for {model}: {error}"))
            .into_inner();
        assert_eq!(metadata.name, resource_name);
    }
}

#[tokio::test]
#[ignore = "live provider: requires GEMINI_API_KEY and spends a few tokens per supported text model"]
async fn live_gemini_model_matrix_generates_with_every_text_model() {
    let client = client();
    for model in GeminiModel::TEXT_GENERATION {
        let response = client
            .generate_content(
                *model,
                text_request(Some(GenerationConfig {
                    max_output_tokens: Some(16),
                    temperature: Some(0.0),
                    ..Default::default()
                })),
            )
            .await
            .unwrap_or_else(|error| panic!("generateContent failed for {model}: {error}"))
            .into_inner();
        assert!(
            !response.candidates.is_empty(),
            "{model} returned no candidates"
        );
    }
}

#[tokio::test]
#[ignore = "live provider: requires GEMINI_API_KEY and validates all inexpensive generation fields"]
async fn live_gemini_core_generation_options_are_accepted_together() {
    let safety_settings = [
        HarmCategory::Harassment,
        HarmCategory::HateSpeech,
        HarmCategory::SexuallyExplicit,
        HarmCategory::DangerousContent,
    ]
    .into_iter()
    .map(|category| SafetySetting::new(category, HarmBlockThreshold::Off))
    .collect();
    let request = GenerateContentRequest {
        contents: vec![Content {
            parts: vec![Part::text(
                "Return JSON with an ok field whose value is true.",
            )],
            role: Some(Role::User),
        }],
        safety_settings: Some(safety_settings),
        generation_config: Some(GenerationConfig {
            stop_sequences: Some(vec!["NEVER_STOP_ON_THIS".into()]),
            candidate_count: Some(1),
            max_output_tokens: Some(64),
            temperature: Some(0.0),
            top_p: Some(0.9),
            top_k: Some(10),
            response_mime_type: Some("application/json".into()),
            response_schema: Some(serde_json::json!({
                "type": "object",
                "properties": {"ok": {"type": "boolean"}},
                "required": ["ok"]
            })),
            response_modalities: Some(vec!["TEXT".into()]),
            image_config: None,
        }),
    };

    let response = client()
        .generate_content(GeminiModel::Gemini3_1FlashLite, request)
        .await
        .expect("Gemini should accept the complete inexpensive generation option set")
        .into_inner();
    assert!(!response.candidates.is_empty());
}

#[tokio::test]
#[ignore = "live provider: requires GEMINI_API_KEY and spends one tiny request per safety threshold"]
async fn live_gemini_option_matrix_accepts_every_safety_threshold() {
    let client = client();
    let thresholds = [
        HarmBlockThreshold::Unspecified,
        HarmBlockThreshold::LowAndAbove,
        HarmBlockThreshold::MediumAndAbove,
        HarmBlockThreshold::OnlyHigh,
        HarmBlockThreshold::None,
        HarmBlockThreshold::Off,
    ];
    for threshold in thresholds {
        let safety_settings = [
            HarmCategory::Harassment,
            HarmCategory::HateSpeech,
            HarmCategory::SexuallyExplicit,
            HarmCategory::DangerousContent,
        ]
        .into_iter()
        .map(|category| SafetySetting::new(category, threshold))
        .collect();
        let mut request = text_request(Some(GenerationConfig {
            max_output_tokens: Some(8),
            ..Default::default()
        }));
        request.safety_settings = Some(safety_settings);
        client
            .generate_content(GeminiModel::Gemini3_1FlashLite, request)
            .await
            .unwrap_or_else(|error| panic!("safety threshold {threshold:?} failed: {error}"));
    }
}

#[tokio::test]
#[ignore = "live provider: requires GEMINI_API_KEY; token counting does not generate output"]
async fn live_gemini_core_count_tokens_accepts_real_content() {
    let response = client()
        .count_tokens(
            GeminiModel::Gemini3_1FlashLite,
            CountTokensRequest::from_contents(vec![Content {
                parts: vec![Part::text("one two three")],
                role: Some(Role::User),
            }]),
        )
        .await
        .expect("Gemini countTokens should accept content")
        .into_inner();
    assert!(response.total_tokens() > 0);
}

#[cfg(feature = "stream")]
#[tokio::test]
#[ignore = "live provider: requires GEMINI_API_KEY and the stream feature"]
async fn live_gemini_core_streaming_decodes_provider_chunks() {
    use futures::StreamExt;

    let response = client()
        .generate_content_streamed(
            GeminiModel::Gemini3_1FlashLite,
            text_request(Some(GenerationConfig {
                max_output_tokens: Some(16),
                ..Default::default()
            })),
        )
        .await
        .expect("Gemini streaming handshake should succeed");
    let chunks = response
        .into_inner()
        .collect::<Vec<_>>()
        .await
        .into_iter()
        .collect::<Result<Vec<_>, _>>()
        .expect("Gemini streaming body should decode");
    assert!(!chunks.is_empty());
}

#[tokio::test]
#[ignore = "EXPENSIVE live provider: generates one image with every supported image model"]
async fn live_gemini_expensive_image_model_matrix() {
    let client = client();
    for model in GeminiModel::IMAGE_GENERATION {
        let response = client
            .generate_content(
                *model,
                GenerateContentRequest {
                    contents: vec![Content {
                        parts: vec![Part::text("A single small blue circle on white.")],
                        role: Some(Role::User),
                    }],
                    safety_settings: None,
                    generation_config: Some(GenerationConfig {
                        response_modalities: Some(vec!["IMAGE".into()]),
                        image_config: Some(GeminiImageConfig {
                            aspect_ratio: Some(GeminiAspectRatio::Square),
                            image_size: (*model != GeminiModel::Gemini2_5FlashImage)
                                .then_some(GeminiImageSize::OneK),
                        }),
                        ..Default::default()
                    }),
                },
            )
            .await
            .unwrap_or_else(|error| panic!("image generation failed for {model}: {error}"))
            .into_inner();
        assert!(!response.candidates.is_empty());
    }
}

#[tokio::test]
#[ignore = "EXPENSIVE live provider: generates images for every aspect-ratio and resolution option"]
async fn live_gemini_expensive_image_option_matrix() {
    let client = client();
    let aspect_ratios = [
        GeminiAspectRatio::Square,
        GeminiAspectRatio::Portrait2x3,
        GeminiAspectRatio::Landscape3x2,
        GeminiAspectRatio::Portrait3x4,
        GeminiAspectRatio::Landscape4x3,
        GeminiAspectRatio::Portrait4x5,
        GeminiAspectRatio::Landscape5x4,
        GeminiAspectRatio::Portrait9x16,
        GeminiAspectRatio::Landscape16x9,
        GeminiAspectRatio::Ultrawide21x9,
    ];
    for aspect_ratio in aspect_ratios {
        generate_image(
            &client,
            GeminiModel::Gemini3_1FlashLiteImage,
            aspect_ratio,
            GeminiImageSize::OneK,
        )
        .await;
    }
    for image_size in [
        GeminiImageSize::OneK,
        GeminiImageSize::TwoK,
        GeminiImageSize::FourK,
    ] {
        generate_image(
            &client,
            GeminiModel::Gemini3ProImage,
            GeminiAspectRatio::Square,
            image_size,
        )
        .await;
    }
}

async fn generate_image(
    client: &GeminiClient,
    model: GeminiModel,
    aspect_ratio: GeminiAspectRatio,
    image_size: GeminiImageSize,
) {
    client
        .generate_content(
            model,
            GenerateContentRequest {
                contents: vec![Content {
                    parts: vec![Part::text("A single small blue circle on white.")],
                    role: Some(Role::User),
                }],
                safety_settings: None,
                generation_config: Some(GenerationConfig {
                    response_modalities: Some(vec!["IMAGE".into()]),
                    image_config: Some(GeminiImageConfig {
                        aspect_ratio: Some(aspect_ratio),
                        image_size: Some(image_size),
                    }),
                    ..Default::default()
                }),
            },
        )
        .await
        .unwrap_or_else(|error| {
            panic!("image option {aspect_ratio:?}/{image_size:?} failed on {model}: {error}")
        });
}

#[test]
fn live_gemini_manifest_is_exhaustive_and_non_overlapping() {
    assert_eq!(
        GeminiModel::ALL.len(),
        GeminiModel::TEXT_GENERATION.len() + GeminiModel::IMAGE_GENERATION.len()
    );
    for model in GeminiModel::ALL {
        assert_eq!(
            GeminiModel::IMAGE_GENERATION.contains(model),
            model.supports_image_generation()
        );
    }
    assert!(GeminiModel::TEXT_GENERATION
        .iter()
        .all(|model| !model.supports_image_generation()));
    assert!(GeminiModel::ALL.iter().all(|model| {
        model.supports_image_input()
            && client_generation_method_is_supported(GenerationMethod::GenerateContent)
    }));
}

fn client_generation_method_is_supported(method: GenerationMethod) -> bool {
    matches!(method, GenerationMethod::GenerateContent)
}
