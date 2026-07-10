use super::{
    ExtendedReasoningEffort, Gpt5ReasoningEffort, Gpt5_1ReasoningEffort,
    Gpt5_5PromptCacheRetention, OpenAIResponsesModel, ProReasoningEffort, PromptCacheRetention,
    SupportsImageGenerationTool, SupportsPromptCacheKey, SupportsPromptCacheRetention,
    SupportsReasoning, SupportsSampling, SupportsStructuredOutput,
};

macro_rules! models {
    ($($name:ident => $id:literal),+ $(,)?) => {$ (
        #[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
        pub struct $name;
        impl OpenAIResponsesModel for $name { const ID: &'static str = $id; }
    )+ };
}

models! {
    Gpt4oMini => "gpt-4o-mini",
    Gpt4o => "gpt-4o",
    Gpt4_1 => "gpt-4.1",
    Gpt4_1Mini => "gpt-4.1-mini",
    Gpt4_1Nano => "gpt-4.1-nano",
    Gpt5_1 => "gpt-5.1",
    Gpt5 => "gpt-5",
    Gpt5Mini => "gpt-5-mini",
    Gpt5Nano => "gpt-5-nano",
    Gpt5_4 => "gpt-5.4",
    Gpt5_4Pro => "gpt-5.4-pro",
    Gpt5_4Mini => "gpt-5.4-mini",
    Gpt5_4Nano => "gpt-5.4-nano",
    Gpt5_5 => "gpt-5.5",
    Gpt5_5Pro => "gpt-5.5-pro",
}

pub const KNOWN_RESPONSE_MODEL_IDS: &[&str] = &[
    Gpt4oMini::ID,
    Gpt4o::ID,
    Gpt4_1::ID,
    Gpt4_1Mini::ID,
    Gpt4_1Nano::ID,
    Gpt5_1::ID,
    Gpt5::ID,
    Gpt5Mini::ID,
    Gpt5Nano::ID,
    Gpt5_4::ID,
    Gpt5_4Pro::ID,
    Gpt5_4Mini::ID,
    Gpt5_4Nano::ID,
    Gpt5_5::ID,
    Gpt5_5Pro::ID,
];

// Capability evidence reviewed 2026-07-10.
// Prompt caching: https://developers.openai.com/api/docs/guides/prompt-caching
// Model capabilities: https://developers.openai.com/api/docs/models
// The dedicated image-tool guide is the conservative compile-time authority.
// It omits gpt-5.4, gpt-5.4-pro, and gpt-5.5-pro despite conflicting model
// pages, so those models remain available only through the dynamic escape hatch:
// https://developers.openai.com/api/docs/guides/tools-image-generation

#[derive(Debug, Clone, Copy)]
pub(crate) struct ModelEvidence {
    pub id: &'static str,
    pub url: &'static str,
    pub reviewed: &'static str,
    pub sampling: bool,
    pub reasoning: &'static [&'static str],
    pub cache_retentions: &'static [&'static str],
    pub structured_output: bool,
    pub image_tool: bool,
}

macro_rules! evidence {
    ($id:literal, $sampling:literal, [$($reasoning:literal),*], [$($retention:literal),*], $structured:literal, $image:literal) => {
        ModelEvidence {
            id: $id,
            url: concat!("https://developers.openai.com/api/docs/models/", $id),
            reviewed: "2026-07-10",
            sampling: $sampling,
            reasoning: &[$($reasoning),*],
            cache_retentions: &[$($retention),*],
            structured_output: $structured,
            image_tool: $image,
        }
    };
}

pub(crate) const MODEL_EVIDENCE: &[ModelEvidence] = &[
    evidence!("gpt-4o-mini", true, [], [], true, true),
    evidence!("gpt-4o", true, [], [], true, true),
    evidence!("gpt-4.1", true, [], ["in_memory", "24h"], true, true),
    evidence!("gpt-4.1-mini", true, [], [], true, true),
    evidence!("gpt-4.1-nano", true, [], [], true, true),
    evidence!(
        "gpt-5.1",
        false,
        ["none", "low", "medium", "high"],
        ["in_memory", "24h"],
        true,
        false
    ),
    evidence!(
        "gpt-5",
        false,
        ["minimal", "low", "medium", "high"],
        ["in_memory", "24h"],
        true,
        true
    ),
    evidence!(
        "gpt-5-mini",
        false,
        ["minimal", "low", "medium", "high"],
        [],
        true,
        false
    ),
    evidence!(
        "gpt-5-nano",
        false,
        ["minimal", "low", "medium", "high"],
        [],
        true,
        true
    ),
    evidence!(
        "gpt-5.4",
        false,
        ["none", "low", "medium", "high", "xhigh"],
        ["in_memory", "24h"],
        true,
        false
    ),
    evidence!(
        "gpt-5.4-pro",
        false,
        ["medium", "high", "xhigh"],
        [],
        false,
        false
    ),
    evidence!(
        "gpt-5.4-mini",
        false,
        ["none", "low", "medium", "high", "xhigh"],
        [],
        true,
        true
    ),
    evidence!(
        "gpt-5.4-nano",
        false,
        ["none", "low", "medium", "high", "xhigh"],
        [],
        true,
        true
    ),
    evidence!(
        "gpt-5.5",
        false,
        ["none", "low", "medium", "high", "xhigh"],
        ["24h"],
        true,
        true
    ),
    evidence!(
        "gpt-5.5-pro",
        false,
        ["medium", "high", "xhigh"],
        ["24h"],
        true,
        false
    ),
];

macro_rules! impl_trait {
    ($trait:ident: $($model:ty),+ $(,)?) => { $(impl $trait for $model {})+ };
}

impl_trait!(SupportsSampling: Gpt4oMini, Gpt4o, Gpt4_1, Gpt4_1Mini, Gpt4_1Nano);
impl_trait!(SupportsPromptCacheKey: Gpt4oMini, Gpt4o, Gpt4_1, Gpt4_1Mini, Gpt4_1Nano, Gpt5_1, Gpt5, Gpt5Mini, Gpt5Nano, Gpt5_4, Gpt5_4Pro, Gpt5_4Mini, Gpt5_4Nano, Gpt5_5, Gpt5_5Pro);
impl_trait!(SupportsStructuredOutput: Gpt4oMini, Gpt4o, Gpt4_1, Gpt4_1Mini, Gpt4_1Nano, Gpt5_1, Gpt5, Gpt5Mini, Gpt5Nano, Gpt5_4, Gpt5_4Mini, Gpt5_4Nano, Gpt5_5, Gpt5_5Pro);
impl_trait!(SupportsImageGenerationTool: Gpt4oMini, Gpt4o, Gpt4_1, Gpt4_1Mini, Gpt4_1Nano, Gpt5, Gpt5Nano, Gpt5_4Mini, Gpt5_4Nano, Gpt5_5);

macro_rules! reasoning {
    ($effort:ty: $($model:ty),+ $(,)?) => { $(
        impl SupportsReasoning for $model { type Effort = $effort; }
    )+ };
}

reasoning!(Gpt5ReasoningEffort: Gpt5, Gpt5Mini, Gpt5Nano);
reasoning!(Gpt5_1ReasoningEffort: Gpt5_1);
reasoning!(ExtendedReasoningEffort: Gpt5_4, Gpt5_4Mini, Gpt5_4Nano, Gpt5_5);
reasoning!(ProReasoningEffort: Gpt5_4Pro, Gpt5_5Pro);

macro_rules! retention {
    ($retention:ty: $($model:ty),+ $(,)?) => { $(
        impl SupportsPromptCacheRetention for $model { type Retention = $retention; }
    )+ };
}

retention!(PromptCacheRetention: Gpt4_1, Gpt5_1, Gpt5, Gpt5_4);
retention!(Gpt5_5PromptCacheRetention: Gpt5_5, Gpt5_5Pro);

#[cfg(test)]
mod tests {
    use super::*;

    fn sampling<M: SupportsSampling>() {}
    fn reasoning<M: SupportsReasoning>() {}
    fn retention<M: SupportsPromptCacheRetention>() {}
    fn image_tool<M: SupportsImageGenerationTool>() {}

    #[test]
    fn representative_positive_capability_bounds_compile() {
        sampling::<Gpt4o>();
        reasoning::<Gpt5>();
        reasoning::<Gpt5_4Pro>();
        retention::<Gpt4_1>();
        retention::<Gpt5_5Pro>();
        image_tool::<Gpt4oMini>();
        image_tool::<Gpt5Nano>();
    }
}
