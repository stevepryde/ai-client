use std::{fmt, marker::PhantomData, sync::Arc};

use crate::openai::OpenAIJsonSchema;

use super::{
    request::OpenAIResponsesWireRequest, IntoResponsesTool, OpenAIContextCompaction,
    OpenAIConversationReference, OpenAIModerationConfig, OpenAIPromptCacheOptions,
    OpenAIPromptTemplate, OpenAIResponseMetadata, OpenAIResponsesInput, OpenAIResponsesInputItem,
    OpenAIResponsesModel, OpenAIResponsesTextConfig, OpenAIResponsesTextFormat, OpenAIServiceTier,
    OpenAIToolChoice, OpenAITruncation, PreparedResponseRequest, ResponseId, ResponseInclude,
    ResponseModelConfig, SupportsItemInput, SupportsPromptCacheKey, SupportsStructuredOutput,
    SupportsTool, SupportsTools,
};

type RequestState<Input, Output, Tools, ToolControls, Cache> =
    fn() -> (Input, Output, Tools, ToolControls, Cache);

/// Model-independent content and endpoint options for creating a Response.
///
/// Model-specific settings live in [`ResponseModelConfig`]. The type parameters
/// record capabilities used by this request so the selected model is checked at
/// the `ResponsesResource::create` call.
pub struct CreateResponseRequest<
    Input = CommonInput,
    Output = PlainTextOutput,
    Tools = NoTools,
    ToolControls = NoToolControls,
    Cache = NoPromptCacheKey,
> {
    wire: OpenAIResponsesWireRequest,
    state: PhantomData<RequestState<Input, Output, Tools, ToolControls, Cache>>,
}

impl CreateResponseRequest {
    pub fn builder() -> Self {
        Self {
            wire: OpenAIResponsesWireRequest::new(String::new()),
            state: PhantomData,
        }
    }
}

impl<I, O, T, TC, C> Clone for CreateResponseRequest<I, O, T, TC, C> {
    fn clone(&self) -> Self {
        Self {
            wire: self.wire.clone(),
            state: PhantomData,
        }
    }
}

impl<I, O, T, TC, C> fmt::Debug for CreateResponseRequest<I, O, T, TC, C> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CreateResponseRequest")
            .field("request", &"[redacted]")
            .finish()
    }
}

impl<I, O, T, TC, C> CreateResponseRequest<I, O, T, TC, C> {
    fn with_state<I2, O2, T2, TC2, C2>(self) -> CreateResponseRequest<I2, O2, T2, TC2, C2> {
        CreateResponseRequest {
            wire: self.wire,
            state: PhantomData,
        }
    }

    pub fn input_text(
        mut self,
        input: impl Into<String>,
    ) -> CreateResponseRequest<CommonInput, O, T, TC, C> {
        self.wire.input = Some(OpenAIResponsesInput::Text(input.into()));
        self.with_state()
    }

    pub fn input_items(
        mut self,
        items: Vec<OpenAIResponsesInputItem>,
    ) -> CreateResponseRequest<ItemInput, O, T, TC, C> {
        self.wire.input = Some(OpenAIResponsesInput::Items(items));
        self.with_state()
    }

    pub fn instructions(mut self, instructions: impl Into<String>) -> Self {
        self.wire.instructions = Some(instructions.into());
        self
    }

    pub fn max_output_tokens(mut self, max_output_tokens: u64) -> Self {
        self.wire.max_output_tokens = Some(max_output_tokens);
        self
    }

    pub fn previous_response_id(mut self, id: ResponseId) -> Self {
        self.wire.previous_response_id = Some(id);
        self
    }

    pub fn store(mut self, store: bool) -> Self {
        self.wire.store = Some(store);
        self
    }

    pub fn metadata(mut self, metadata: OpenAIResponseMetadata) -> Self {
        self.wire.metadata = Some(metadata);
        self
    }

    pub fn safety_identifier(mut self, identifier: impl Into<String>) -> Self {
        self.wire.safety_identifier = Some(identifier.into());
        self
    }

    pub fn service_tier(mut self, tier: OpenAIServiceTier) -> Self {
        self.wire.service_tier = Some(tier);
        self
    }

    pub fn background(mut self, background: bool) -> Self {
        self.wire.background = Some(background);
        self
    }

    pub fn max_tool_calls(
        mut self,
        maximum: u64,
    ) -> CreateResponseRequest<I, O, T, UsesToolControls, C> {
        self.wire.max_tool_calls = Some(maximum);
        self.with_state()
    }

    pub fn parallel_tool_calls(
        mut self,
        parallel: bool,
    ) -> CreateResponseRequest<I, O, T, UsesToolControls, C> {
        self.wire.parallel_tool_calls = Some(parallel);
        self.with_state()
    }

    pub fn tool<Tool>(mut self, tool: Tool) -> CreateResponseRequest<I, O, ToolList<Tool, T>, TC, C>
    where
        Tool: IntoResponsesTool,
    {
        self.wire
            .tools
            .get_or_insert_with(Vec::new)
            .push(tool.into_responses_tool());
        self.with_state()
    }

    pub fn image_generation_tool(
        self,
        tool: super::OpenAIImageGenerationTool,
    ) -> CreateResponseRequest<I, O, ToolList<super::OpenAIImageGenerationTool, T>, TC, C> {
        self.tool(tool)
    }

    pub fn tool_choice(
        mut self,
        choice: OpenAIToolChoice,
    ) -> CreateResponseRequest<I, O, T, UsesToolControls, C> {
        self.wire.tool_choice = Some(choice);
        self.with_state()
    }

    pub fn include(mut self, include: ResponseInclude) -> Self {
        self.wire.include.get_or_insert_with(Vec::new).push(include);
        self
    }

    pub fn truncation(mut self, truncation: OpenAITruncation) -> Self {
        self.wire.truncation = Some(truncation);
        self
    }

    pub fn prompt_cache_options(mut self, options: OpenAIPromptCacheOptions) -> Self {
        self.wire.prompt_cache_options = Some(options);
        self
    }

    pub fn conversation(mut self, conversation: OpenAIConversationReference) -> Self {
        self.wire.conversation = Some(conversation);
        self
    }

    pub fn prompt(mut self, prompt: OpenAIPromptTemplate) -> Self {
        self.wire.prompt = Some(prompt);
        self
    }

    pub fn moderation(mut self, moderation: OpenAIModerationConfig) -> Self {
        self.wire.moderation = Some(moderation);
        self
    }

    pub fn context_management(mut self, entries: Vec<OpenAIContextCompaction>) -> Self {
        self.wire.context_management = Some(entries);
        self
    }

    pub fn prompt_cache_key(
        mut self,
        key: impl Into<String>,
    ) -> CreateResponseRequest<I, O, T, TC, UsesPromptCacheKey> {
        self.wire.prompt_cache_key = Some(key.into());
        self.with_state()
    }

    pub fn json_schema(
        mut self,
        schema: OpenAIJsonSchema,
    ) -> CreateResponseRequest<I, StructuredOutput, T, TC, C> {
        self.wire.text = Some(OpenAIResponsesTextConfig {
            format: Some(OpenAIResponsesTextFormat::JsonSchema(schema.into())),
            verbosity: None,
            extra: Default::default(),
        });
        self.with_state()
    }

    pub fn text_config(
        mut self,
        text: OpenAIResponsesTextConfig,
    ) -> CreateResponseRequest<I, StructuredOutput, T, TC, C> {
        self.wire.text = Some(text);
        self.with_state()
    }

    /// Finish building the reusable request.
    pub fn build(self) -> Self {
        self
    }
}

mod compatibility {
    use super::*;

    pub trait InputCompatible<M: OpenAIResponsesModel> {}
    impl<M: OpenAIResponsesModel> InputCompatible<M> for CommonInput {}
    impl<M: OpenAIResponsesModel + SupportsItemInput> InputCompatible<M> for ItemInput {}

    pub trait OutputCompatible<M: OpenAIResponsesModel> {}
    impl<M: OpenAIResponsesModel> OutputCompatible<M> for PlainTextOutput {}
    impl<M: OpenAIResponsesModel + SupportsStructuredOutput> OutputCompatible<M> for StructuredOutput {}

    pub trait ToolListCompatible<M: OpenAIResponsesModel> {}
    impl<M: OpenAIResponsesModel> ToolListCompatible<M> for NoTools {}
    impl<M, Tool, Tail> ToolListCompatible<M> for ToolList<Tool, Tail>
    where
        M: OpenAIResponsesModel + SupportsTool<Tool>,
        Tool: IntoResponsesTool,
        Tail: ToolListCompatible<M>,
    {
    }

    pub trait ToolControlsCompatible<M: OpenAIResponsesModel> {}
    impl<M: OpenAIResponsesModel> ToolControlsCompatible<M> for NoToolControls {}
    impl<M: OpenAIResponsesModel + SupportsTools> ToolControlsCompatible<M> for UsesToolControls {}

    pub trait CacheCompatible<M: OpenAIResponsesModel> {}
    impl<M: OpenAIResponsesModel> CacheCompatible<M> for NoPromptCacheKey {}
    impl<M: OpenAIResponsesModel + SupportsPromptCacheKey> CacheCompatible<M> for UsesPromptCacheKey {}
}

mod sealed {
    pub trait Sealed {}
}

mod model_sealed {
    pub trait Sealed {}
}

/// Implemented only when every capability used by a request is supported by M.
#[doc(hidden)]
pub trait CompatibleResponseRequest<M: OpenAIResponsesModel>: sealed::Sealed {
    fn prepare<State>(self, model: ResponseModelConfig<M, State>) -> PreparedResponseRequest;
}

impl<I, O, T, TC, C> sealed::Sealed for CreateResponseRequest<I, O, T, TC, C> {}

/// A model configuration that is compile-time compatible with `Request`.
///
/// This is object-safe so applications can erase different model markers and
/// typestate modes into `Box<dyn ResponseModelFor<_>>` or
/// `Arc<dyn ResponseModelFor<_>>` after every model/request capability has been
/// checked by the compiler.
pub trait ResponseModelFor<Request>: model_sealed::Sealed + Send + Sync {
    #[doc(hidden)]
    fn prepare(&self, request: Request) -> PreparedResponseRequest;
}

impl<M: OpenAIResponsesModel, State> model_sealed::Sealed for ResponseModelConfig<M, State> {}

impl<M, State, I, O, T, TC, C> ResponseModelFor<CreateResponseRequest<I, O, T, TC, C>>
    for ResponseModelConfig<M, State>
where
    M: OpenAIResponsesModel,
    CreateResponseRequest<I, O, T, TC, C>: CompatibleResponseRequest<M>,
{
    fn prepare(&self, request: CreateResponseRequest<I, O, T, TC, C>) -> PreparedResponseRequest {
        CompatibleResponseRequest::prepare(request, self.clone())
    }
}

impl<T: ?Sized + model_sealed::Sealed> model_sealed::Sealed for Box<T> {}

impl<Request, T> ResponseModelFor<Request> for Box<T>
where
    T: ?Sized + ResponseModelFor<Request>,
{
    fn prepare(&self, request: Request) -> PreparedResponseRequest {
        T::prepare(self, request)
    }
}

impl<T: ?Sized + model_sealed::Sealed> model_sealed::Sealed for &T {}

impl<Request, T> ResponseModelFor<Request> for &T
where
    T: ?Sized + ResponseModelFor<Request>,
{
    fn prepare(&self, request: Request) -> PreparedResponseRequest {
        T::prepare(self, request)
    }
}

impl<T: ?Sized + model_sealed::Sealed> model_sealed::Sealed for Arc<T> {}

impl<Request, T> ResponseModelFor<Request> for Arc<T>
where
    T: ?Sized + ResponseModelFor<Request>,
{
    fn prepare(&self, request: Request) -> PreparedResponseRequest {
        T::prepare(self, request)
    }
}

impl<M, I, O, T, TC, C> CompatibleResponseRequest<M> for CreateResponseRequest<I, O, T, TC, C>
where
    M: OpenAIResponsesModel,
    I: compatibility::InputCompatible<M>,
    O: compatibility::OutputCompatible<M>,
    T: compatibility::ToolListCompatible<M>,
    TC: compatibility::ToolControlsCompatible<M>,
    C: compatibility::CacheCompatible<M>,
{
    fn prepare<State>(mut self, model: ResponseModelConfig<M, State>) -> PreparedResponseRequest {
        model.apply(&mut self.wire);
        PreparedResponseRequest::new(self.wire)
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct CommonInput;
#[derive(Debug, Clone, Copy, Default)]
pub struct ItemInput;
#[derive(Debug, Clone, Copy, Default)]
pub struct PlainTextOutput;
#[derive(Debug, Clone, Copy, Default)]
pub struct StructuredOutput;
#[derive(Debug, Clone, Copy, Default)]
pub struct NoTools;
#[derive(Debug, Clone, Copy, Default)]
pub struct ToolList<Tool, Tail>(PhantomData<fn() -> (Tool, Tail)>);
#[derive(Debug, Clone, Copy, Default)]
pub struct NoToolControls;
#[derive(Debug, Clone, Copy, Default)]
pub struct UsesToolControls;
#[derive(Debug, Clone, Copy, Default)]
pub struct NoPromptCacheKey;
#[derive(Debug, Clone, Copy, Default)]
pub struct UsesPromptCacheKey;
