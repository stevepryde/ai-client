# ai-client architecture roadmap

Status: in progress

Last reviewed: 2026-07-10

Target release for the first breaking slice: `0.4.0`

Stack contract: [`stack.md`](stack.md)

## Current execution plan

### Task contract

- Advance this roadmap through the largest coherent unblocked units, beginning with the highest-risk foundation.
- Keep provider-native APIs full fidelity; no universal lowest-common-denominator request.
- Make typed model markers and capability-bounded builders the default known-model path, erasing into private non-generic wire requests.
- Require typed native OpenAI Responses model markers; runtime selection uses a closed application enum and typed match arms.
- Keep OpenAI Responses primary, deprecate native OpenAI Chat Completions, and preserve Chat-Completions-shaped interoperability in a separate typed `openai_compatible` dialect family.
- Do not release or tag an implementation unit unless it intentionally completes the release contract in `AGENTS.md`.

### Roadmap units

| Unit | Status | Scope | Evidence required |
| --- | --- | --- | --- |
| F1 — safe shared JSON transport | accepted | Private client state, same-origin relative paths, shared non-streaming transport, typed errors/metadata, TLS wiring, encoded query/path data | **test-only**: 34 unit/integration tests, one doctest, full feature matrix, TLS/stream dependency-tree checks, and independent review; no live-provider smoke |
| F2 — streaming transport | accepted | Crate-owned SSE and internal byte streams, handshake errors/metadata, wire stream invariants, adversarial framing and JSON-array tests | **test-only**: 42 unit/integration tests, one doctest, full stream feature matrix/dependency-tree checks, and independent review; no live-provider smoke |
| F3 — typed model requests | accepted, corrected | OpenAI Responses known-model markers, resource-scoped capabilities, mode-typestate builders, and non-generic wire erasure; the later live correction removed dynamic native requests | Compile-pass/fail fixtures plus live GPT-5.1/GPT-5.4 mode matrices |
| F4 — compatibility separation | accepted | Native Chat Completions deprecation and typed `openai_compatible` dialect framework | **test-only**: 58 unit/integration tests, four compatibility compile-pass and twelve compile-fail fixtures, doctests, full feature matrix, and independent review; no live-provider smoke |
| F5 — Responses refactor and parity | accepted | Focused resource modules, operation parity, lossless unknown variants, typed closed schemas, and Conversations | **test-only**: pinned-source verification, 7/7 Responses and 8/8 Conversations operations, 116 runtime tests, five Responses compile-pass and nine compile-fail fixtures, compatibility fixtures, doctests, strict clippy, full feature matrix, and independent review; no live-provider smoke |
| F6 — standalone Images API | pending | Generation, edits, variations, multipart inputs, binary/base64 outputs, and supported partial-image streaming | Pinned Images coverage matrix, exact mock multipart/binary/stream fixtures, and opt-in live smoke |

### F1 architecture contract

- Existing non-streaming OpenAI and Gemini calls share one private JSON transport; provider modules own auth and error-envelope semantics.
- Public non-streaming methods return `AiResponse<T>` so metadata is concurrency-safe and never hidden as mutable client state.
- `ResponseMetadata` is crate-owned and exposes request ID, retry information, and rate-limit headers without exposing the raw response or header map.
- API errors contain provider, stable operation name, status, normalized provider fields, request ID, retry information, and a bounded raw fallback. Decode errors never log or display response bodies.
- Configured base URLs preserve an optional path prefix and reject credentials, query strings, and fragments. Authenticated operations accept only validated relative paths and separately encoded dynamic path segments/query values.
- F1 does not accept an externally supplied reqwest client: its redirect policy cannot be inspected or overridden, so a client configured to follow cross-origin redirects could forward provider-specific credentials such as Gemini's `x-goog-api-key`. Revisit injection behind a transport seam that can preserve the same-origin invariant.
- `--no-default-features` enables no TLS backend; native-only and rustls-only builds remain distinct. Enabling both is supported if reqwest supports the combination.
- Retries, SSE/byte streams, multipart, resource handles, model markers, compatibility dialects, and portable provider traits are deferred to their own units.

### F2 architecture contract

- `AiStream<T>` is the crate-owned public stream container. Streaming methods return `AiResponse<AiStream<T>>`, so the existing response wrapper owns handshake metadata while the stream implements `futures::Stream` and yields crate-owned `AiStreamError` values instead of reqwest or reqwest-streams errors.
- `SseJsonEvent<T>` preserves SSE `event`, `id`, and `retry` fields plus the complete raw JSON value alongside typed provider data. Numeric SSE `retry` values are milliseconds and map to `Duration::from_millis` without implying reconnect behavior. An unknown tagged provider variant therefore remains fully inspectable through the raw value.
- OpenAI Responses and legacy Chat Completions streaming return `AiResponse<AiStream<SseJsonEvent<T>>>`; Gemini's JSON-array stream returns `AiResponse<AiStream<T>>` through the same public error boundary.
- Stream handshakes use the same provider error decoder and response metadata model as non-streaming calls. A non-success HTTP response must return `AiError::Api` before any stream is exposed.
- OpenAI streaming methods overwrite the wire `stream` switch with `true`; non-streaming methods overwrite it with `None`/`false`. Callers cannot select the wrong wire mode through request data.
- The shared SSE decoder owns framing only. It must handle LF and CRLF, comments, multiple `data` lines, `event`, `id`, `retry`, blank events, `[DONE]`, final buffered data, arbitrary byte chunk boundaries, and split UTF-8.
- A private raw-byte adapter underpins SSE and JSON-array streams. Public binary/collected media responses, multipart, retries, reconnects, WebSockets, and Realtime remain deferred until an image, file, or audio endpoint proves those contracts.
- F2 does not change model types, introduce capability builders, deprecate Chat Completions, add compatibility dialects, or expand Responses item/tool coverage.

### F3 architecture contract

- F3 owns native OpenAI Responses request construction end to end. `OpenAIModel` remains for model retrieval and legacy Chat Completions, but Responses moves to resource-scoped marker types implementing the unsealed `OpenAIResponsesModel` trait.
- The initial checked-in marker manifest covers every model ID already supported by this crate. Capability implementations cover the temperature/sampling, reasoning-effort, prompt-caching, structured-output, and image-tool settings only where current official evidence supports them; legacy sanitizer matches are not treated as evidence.
- Reasoning uses associated effort types so standard, extended/xhigh, and pro model families expose only their accepted values. Sampling and prompt-cache methods exist only behind their exact capability bounds.
- Conditional combinations use focused mode typestate where the provider contract depends on another field. GPT-5.1/GPT-5.4 sampling versus reasoning is encoded in the builder state and verified live.
- `ResponseRequestBuilder<M, MissingInput>` uses typestate only for required input. `build()` exists only for `HasInput` and erases immediately into `PreparedResponseRequest`, which owns one private, non-generic wire request. Both streamed and non-streamed client paths accept the same prepared type and continue to own the wire stream switch.
- `PreparedResponseRequest` keeps prompts and wire fields private and redacts `Debug` without exposing the wire request or stream switch.
- Native OpenAI Responses has no string-model request builder or runtime capability catalog. Callers define local model markers for fine-tuned models and aliases. Compatibility gateways remain owned by F4's separate dialect family.
- Direct public construction of the old Responses wire request and its invariant-bypassing generated builder is removed in this planned `0.4.0` breaking slice rather than retained as a duplicate API.
- F3 compile-checks only the stable settings it owns. Input modality safety and complete structured-output/tool capability modeling remain deferred to the F5 input/tool union refactor; F4 compatibility dialects reuse the pattern but do not inherit native OpenAI guarantees.

### F4 architecture contract

- F4a separates Chat-Completions-shaped interoperability from native OpenAI. It introduces `openai_compatible` with public dialect/resource traits, a dialect-typed client, typed model markers and capability-bounded builders, and one explicit `CustomDialect` configuration path. No named non-OpenAI provider is claimed until official conformance evidence is pinned.
- Native OpenAI Chat Completions remains default-off behind `chat-completions`, but its module, request/response/event types, and client entrypoints are deprecated for the 0.4 migration window. OpenAI callers are directed to native Responses; compatibility callers are directed to `openai_compatible`.
- `OpenAICompatibleClient<D>` owns the shared safe transport. `ChatCompletionsDialect` gates `chat()`, and prepared requests remain dialect-bound so a request cannot cross clients. Typed builders erase model generics into one private non-generic wire request before transport.
- `CustomDialect` requires an explicit validated base URL and safe crate-owned auth strategy. Its dynamic model/request path validates structural safety only: model capabilities are explicitly unvalidated, and advanced fields are sent as configured without coercion.
- Chat builders require messages through typestate. Each dialect owns an associated serializable message type that erases to private JSON objects; `CustomDialect` provides both a text convenience constructor and an explicit raw-object message escape hatch. Capability-specific methods exist only for the exact dialect/model bounds. Dialect-specific options are associated types, not fields for every future provider.
- `extra_body` is collision-checked against typed wire fields; collisions are build errors and no typed value is silently replaced or discarded.
- Non-streaming and streaming compatibility calls use `AiResponse` handshake metadata, structured provider errors, crate-owned stream errors, automatic wire stream mode, and raw-preserving response/SSE wrappers.
- F4a includes mock conformance and compile-pass/fail fixtures for `CustomDialect`. Named dialect markers, Responses-shaped compatibility, Images/Audio compatibility, and portable cross-provider requests remain deferred.

### F5 architecture contract

- F5 is checked against the exact OpenAI OpenAPI document and digest recorded in `openai/source.json`; the pinned document is a coverage and drift oracle, not generated public Rust API.
- Responses is a borrowed resource handle with seven operations: create, retrieve, delete, cancel, list input items, count input tokens, and compact. Streaming create/retrieve are distinct Rust methods because their return types differ.
- Conversations is a borrowed resource handle with create/retrieve/update/delete plus nested item create/retrieve/delete/list operations. It reuses Responses item types and does not introduce deprecated Assistants or Threads types.
- The current official create schema permits omitted input. F5 removes F3's hard `MissingInput` gate so conversation-linked, continuation, and reusable-prompt requests remain representable; typed model markers and all model capability bounds remain intact.
- Responses protocol types are split into focused request, input, output, tool, event, and operation modules. Typed builders still erase into a private non-generic wire request before transport owns streaming mode.
- Every known tagged input/output/content/annotation/event discriminator is decoded as its documented shape. A known discriminator with malformed fields is an error; an unknown discriminator preserves the entire semantic JSON object in a redacted raw wrapper and serializes losslessly.
- Create covers all 31 pinned top-level fields, 16 tool definitions, nine tool-choice forms, three text formats, and the pinned input/output unions. All 53 pinned Responses stream event tags are represented, including Responses audio events even though standalone Audio remains deferred.
- The checked-in multimodal model markers opt into heterogeneous item input from current model evidence; downstream-defined markers must opt in explicitly. Open JSON Schema/provider-extension objects remain typed as map-backed boundaries, while closed nested action/result/configuration schemas are concrete Rust types.
- Opaque validated IDs and encoded path segments prevent identifier injection. Query/pagination types encode cursors, order, include selectors, stream offsets, and obfuscation explicitly.
- Existing `OpenAIClient::generate_response*` methods remain migration forwarding methods while `client.responses()` and `client.conversations()` become canonical.
- F5 acceptance requires the checked-in 7/7 and 8/8 operation matrix, pinned-source verification, exact mock verb/path/query/body tests, official-example fixtures except where a recorded source defect contradicts its referenced schema, unknown round-trip and malformed-known rejection, compile fixtures, strict clippy, and the full feature matrix.

## Goal

Grow `ai-client` into a provider-extensible Rust client that:

- exposes each provider's native API without forcing it through a lowest-common-denominator abstraction;
- makes OpenAI Responses the primary text and multimodal path;
- covers native OpenAI Responses and standalone Images deeply, while keeping other API families addable later without claiming or planning full-platform coverage;
- provides a layered unified API that can switch providers without making provider-specific features inaccessible;
- remains safe, forward-compatible, testable, and economical to compile.

This roadmap deliberately prioritizes the transport and type-system foundations before adding many endpoints. The current crate is small enough to correct those foundations now; adding endpoint modules first would duplicate the current problems across every provider.

## Executive decisions

1. **Keep provider-native APIs first-class.** OpenAI, Gemini, and future providers keep their own request, response, event, and resource types.
2. **Share transport mechanics, not provider semantics.** Authentication, JSON, multipart, binary bodies, SSE, pagination, retries, and error decoding belong in a common internal core.
3. **Organize OpenAI by resource.** The current product boundary is `client.responses()` and `client.images()`; already-completed Conversations support remains as the durable state companion to Responses.
4. **Deprecate OpenAI's native Chat Completions surface.** Keep it default-off for migration, direct new OpenAI work to Responses, and move reusable compatibility support into a separate typed `openai_compatible` provider family. Do not implement the deprecated Assistants/Threads API unless a concrete downstream migration requirement appears.
5. **Keep native Responses model selection typed.** Runtime selection uses closed application enums and typed match arms; custom aliases use explicit marker types. Unknown response variants still retain their raw payload.
6. **Do not create a workspace yet.** Keep one crate while the providers share the same light dependency set. Reassess a split only when a third substantial provider or another deliberately approved heavy subsystem lands.
7. **Treat the official OpenAPI document as a coverage oracle, not as the public Rust API.** Pin it for drift checks and selective code generation, while keeping reviewed, idiomatic public types.
8. **Unification must be additive, never restrictive.** Full-fidelity native requests remain canonical. Static generic code uses associated provider types; runtime switching uses an explicitly portable request plus typed per-backend defaults or a closed enum of native requests.

## Current-state assessment

### What is already good

- Provider modules are separated at the top level.
- OpenAI Responses is the default direction and Chat Completions is default-off.
- JSON request/response types are explicit instead of being untyped maps everywhere.
- Image input and Responses image-generation output have initial typed support.
- Builders redact API keys in their `Debug` output.
- The code is warning-free under clippy and all current tests pass.

### Architectural blockers

| Priority | Finding | Evidence | Consequence |
| --- | --- | --- | --- |
| P0 | Secrets and transport internals are public | `OpenAIClient::api_key`, `OpenAIClient::client`, and the Gemini equivalents are public | Callers can leak credentials, bypass invariants, and become coupled to reqwest |
| P0 | Authenticated raw methods accept arbitrary URLs | Public `get` and `post` take `&str` URLs | A caller can accidentally send the provider authorization header to another origin |
| P0 | Streaming is not a safe transport primitive | Stream methods do not force `stream: true`, do not decode non-2xx responses before returning a stream, and expose `reqwest_streams::StreamBodyError` | Easy silent misuse, poor errors, and leaked implementation details |
| P0 | The SSE decoder is incomplete | It only recognizes `\n\n` and `data: `, drops event metadata, and has no chunk-boundary fixture tests | CRLF, multi-line data, end-of-stream, and new event shapes can fail or disappear |
| P0 | Model support is closed and mutates requests silently | `OpenAIModel` is exhaustive in practice; `sanitise_request_params` removes or coerces caller values | New aliases, snapshots, fine-tuned models, and compatible gateways are blocked; caller intent is hidden |
| P0 | Error handling loses operational context | API errors are status plus raw string; decode failure logs the entire response body | No typed provider code, request ID, retry hints, or safe body handling; prompts/output may leak into logs |
| P1 | OpenAI types are concentrated in one endpoint file | `create_response.rs` contains requests, tools, images, outputs, stream events, and tests | It will become a giant source file as tools and modalities grow |
| P1 | Unknown tagged variants discard their payload | `#[serde(other)] Unknown` is used for output items, content parts, and stream events | Forward compatibility avoids a crash but loses the exact new data callers need |
| P1 | Provider transport logic is duplicated | Builders, `get`, `post`, and `parse_response` are repeated for OpenAI and Gemini | Every new provider will repeat timeout, error, retry, and observability bugs |
| P1 | Resource methods and raw transport share one client namespace | `OpenAIClient` directly owns models, Chat Completions, and Responses methods | Full OpenAI coverage would turn this into a giant impl block |
| P1 | TLS features do not match their names | reqwest directly enables `rustls-tls`; the crate features add TLS again | `--no-default-features` still compiles rustls and `native-tls` compiles both stacks |
| P1 | URL query construction is not encoded | `utils::Url` concatenates strings | Cursors and future filters can produce invalid or ambiguous URLs |
| P2 | Request/response derives are broader than needed | Most request types derive `Deserialize`; most response types derive `Serialize` | Public contracts are larger and accidental round-trip assumptions become entrenched |
| P2 | Model capabilities are enforced by silent mutation | Capabilities are hard-coded in enum matches and `sanitise_request_params` rewrites values | Invalid combinations do not fail visibly, while arbitrary/new models are blocked by the closed enum |

## Target architecture

Use this module shape while the project remains a single crate:

```text
src/
  core/
    auth.rs
    error.rs
    http.rs
    multipart.rs
    pagination.rs
    request_options.rs
    sse.rs
  openai/
    client.rs
    models.rs
    responses/
      mod.rs
      request.rs
      input.rs
      output.rs
      tools.rs
      events.rs
    conversations.rs
    images.rs
    files.rs
    uploads.rs
    audio/
      mod.rs
      speech.rs
      transcription.rs
    realtime/
      mod.rs
      client.rs
      events.rs
      session.rs
    embeddings.rs
    moderations.rs
    vector_stores.rs
    batches.rs
  openai_compatible/
    client.rs
    dialect.rs
    chat/
      request.rs
      response.rs
      events.rs
      capability.rs
    responses/              # only for dialects that implement it
    images/                 # only for dialects that implement it
    audio/                  # only for dialects that implement it
    providers/
      anthropic.rs
      gemini.rs
      groq.rs
      mistral.rs
      openrouter.rs
      custom.rs
  gemini/
    client.rs
    models.rs
    content/
      input.rs
      output.rs
      config.rs
      safety.rs
  unified/                 # optional; add only after the native APIs are stable
    text.rs
    embeddings.rs
    images.rs
```

The public OpenAI shape should read like the API:

```rust,ignore
let client = OpenAIClient::builder().api_key_from_env()?.build()?;

let response = client.responses().create(request).await?;
let mut events = client.responses().create_stream(request).await?;
let image = client.images().generate(request).await?;
let transcript = client.audio().transcriptions().create(request).await?;
```

Resource handles should be cheap borrows or clones over one shared private transport. They must not each create their own HTTP client.

### Shared core boundary

The shared core should own only mechanics:

- base URL and same-origin relative-path resolution;
- provider auth/header injection;
- JSON, form, multipart, bytes, and empty-body requests;
- typed response metadata such as request ID and rate-limit headers;
- SSE framing and byte streaming;
- cursor page types;
- retry policy and idempotency rules;
- redacted tracing;
- test transport injection or a local mock-server seam.

It should not know what a model, prompt, safety setting, tool call, or conversation means.

### Provider extensibility

Do not introduce a universal wire request or a `Provider` trait that attempts to represent every API. It would either erase important OpenAI features or become a bag of provider-specific options.

Instead:

- give each provider a native client built on the shared core;
- introduce narrow operation traits only where at least two providers and a downstream caller need the same operation;
- keep provider-native request/response types as the full-fidelity source of truth;
- use associated request, response, options, and event types for compile-time generic code;
- use a separate portable request only where runtime switching is valuable;
- attach typed provider defaults to routed backends, and allow callers to opt into a native request when one call needs advanced features.

Likely future portable capabilities are `TextGeneration`, `Embeddings`, `ImageGeneration`, `SpeechToText`, and `TextToSpeech`. Realtime sessions, tool ecosystems, files, fine-tuning, and administration should remain provider-native.

### OpenAI-compatible provider family

Treat OpenAI compatibility as a family of related dialects, not a promise that every provider implements the same OpenAI API. Providers differ by endpoint, accepted fields, model capabilities, error envelopes, streaming quirks, and provider-specific extensions. Some silently ignore unsupported fields, which the crate should not emulate.

Use a provider/dialect marker as the first generic parameter:

```rust,ignore
pub trait OpenAICompatibleDialect: Send + Sync + 'static {
    type ErrorDecoder: CompatibleErrorDecoder;

    fn default_base_url() -> Option<&'static str>;
    fn apply_auth(headers: &mut HeaderMap, credential: &Credential) -> AiResult<()>;
}

pub trait ChatCompletionsDialect: OpenAICompatibleDialect {}
pub trait ResponsesDialect: OpenAICompatibleDialect {}
pub trait ImagesDialect: OpenAICompatibleDialect {}
pub trait AudioDialect: OpenAICompatibleDialect {}

pub struct OpenAICompatibleClient<D> {
    transport: HttpTransport,
    _dialect: PhantomData<fn() -> D>,
}
```

Resource access is capability-bounded:

```rust,ignore
impl<D: ChatCompletionsDialect> OpenAICompatibleClient<D> {
    pub fn chat(&self) -> CompatibleChatResource<'_, D> { /* ... */ }
}

impl<D: ResponsesDialect> OpenAICompatibleClient<D> {
    pub fn responses(&self) -> CompatibleResponsesResource<'_, D> { /* ... */ }
}
```

A Gemini compatibility client therefore cannot call `responses()` unless its marker explicitly implements `ResponsesDialect`; a Groq marker may implement both Chat Completions and Responses; a provider exposing only chat implements only `ChatCompletionsDialect`.

Requests remain generic over both dialect and typed model marker:

```rust,ignore
let client = OpenAICompatibleClient::<Groq>::builder()
    .api_key_from_env()?
    .build()?;

let request = client
    .chat()
    .request::<Llama3_3_70B>()
    .messages(messages)
    .tools(tools)
    .build();
```

Provider-specific methods can be inherent methods on `CompatibleChatRequestBuilder<Groq, M, S>` or extension traits scoped to that dialect. Generic OpenAI-shaped methods require capability bounds implemented for the exact dialect/model combination. As with native requests, the typed builder erases into one private non-generic wire request before transport.

Design rules:

- Native provider APIs remain preferred for full fidelity. `GeminiClient` uses Interactions/GenerateContent; `OpenAICompatibleClient<Gemini>` is an interoperability option.
- Do not publicly alias `openai::chat_completions` request types into provider modules. Share private wire components and decoders underneath.
- A compatibility provider may expose OpenAI-shaped Images, Audio, Videos, Batches, or Responses only when its dialect implements the matching resource trait.
- Known provider markers supply typed auth, default base URL, error decoding, supported resources, model markers, capability traits, and provider-specific options.
- `CustomDialect` supports a configured base URL/auth strategy and dynamic models, with explicit runtime validation and raw extension options. It must not claim known capability guarantees.
- Keep a deliberate `extra_body`/headers/query escape hatch, but make typed fields win and reject key collisions.
- Never silently discard parameters because a compatibility endpoint ignores them; omit unsupported builder methods or return a validation error on the dynamic path.
- Maintain conformance fixtures per dialect. “OpenAI-compatible” is a test claim, not merely a configurable base URL.

#### Chat Completions deprecation policy

- Mark `openai::chat_completions`, `OpenAIClient::generate_content`, and `generate_content_streamed` deprecated in `0.4.0`, while retaining the existing default-off feature for a documented migration window.
- Point OpenAI callers to `openai::responses`.
- Point compatibility callers to `openai_compatible::<Dialect>::chat`.
- Keep the compatibility Chat Completions resource supported and non-deprecated for dialects where it is the provider's current API surface.
- Do not describe OpenAI's endpoint as formally shut down: OpenAI currently recommends Responses for new projects but still documents and supports Chat Completions.
- Decide removal of the deprecated native OpenAI surface separately from the compatibility protocol implementation; removing one must not remove the other.

### Layered unified interface

There are three distinct use cases. They should not be forced through one request type.

#### 1. Provider-native, full fidelity

This is the canonical API and always exposes every supported provider feature:

```rust,ignore
openai.responses().create(OpenAIResponseRequest { /* all OpenAI fields */ }).await?;
gemini.models().generate_content(GeminiGenerateContentRequest { /* all Gemini fields */ }).await?;
```

No unified layer may prevent access to these entry points.

#### 2. Statically generic, full fidelity

Generic libraries can abstract over an operation without standardizing its wire shape. The trait uses associated types, so each provider keeps its native request, response, and stream events:

```rust,ignore
pub trait TextGeneration: Send + Sync {
    type Request;
    type Response;
    type StreamEvent;

    fn generate(
        &self,
        request: Self::Request,
    ) -> impl Future<Output = AiResult<Self::Response>> + Send;
}
```

This is useful for generic orchestration, middleware, testing, and dependency injection. It does **not** by itself make one request reusable across providers, and that is acceptable: it preserves full capability.

If a shared semantic request proves useful, parameterize it by provider extensions instead of erasing them:

```rust,ignore
pub trait TextProvider {
    type Model;
    type Options: Default;
    type ExtraInput;
    type Response;
    type StreamEvent;
}

pub struct TextRequest<P: TextProvider> {
    pub model: P::Model,
    pub input: Vec<TextInput<P::ExtraInput>>,
    pub common: CommonTextOptions,
    pub provider: P::Options,
}
```

`OpenAITextOptions` can then expose tools, reasoning, prompt caching, response storage, and Responses-specific controls, while `GeminiTextOptions` exposes safety settings and Gemini generation controls. Common options must be genuinely semantic and their conversion must return an unsupported-option error or warning; they must never be silently dropped.

This generic shape should remain a convenience over native request types, not a second independently maintained wire model. Each provider owns an explicit conversion into its native request.

#### 3. Runtime switching, explicitly portable

Runtime provider selection necessarily needs a contract that all selected backends can honor. Keep that contract honest and small:

```rust,ignore
pub struct PortableTextRequest {
    pub input: Vec<PortableMessage>,
    pub max_output_tokens: Option<u64>,
}

pub enum TextBackend {
    OpenAIGpt5(OpenAITextBackend<Gpt5>),
    Gemini25Flash(GeminiTextBackend<Gemini25Flash>),
}
```

The user-owned backend enum closes over the exact provider/model configurations supported by that application. Each backend converts the portable request, merges its typed defaults, and calls the native API. Adding or changing a provider/model then produces compile errors until the corresponding typed configuration and match arms exist. This supports runtime failover while still allowing an OpenAI backend to use reasoning, tools, caching, or structured output by default.

If model selection comes from configuration, parse it into a closed application enum and match each variant to a typed model builder. Unknown values are configuration errors; they do not fall through to a string-model request.

For provider-specific settings that vary per call, use one of these explicit paths:

- call the provider-native API;
- use a closed `RoutedTextRequest::{Portable, OpenAI, Gemini}` enum with equally typed response variants;
- accept a typed provider customization callback at router construction time, where the concrete provider type is known.

Do not use `serde_json::Value`, `Any`, string-keyed extension maps, or a struct containing `openai: Option<_>`, `gemini: Option<_>`, and one field per future provider. Those designs defer errors to runtime, allow invalid combinations, and become less extensible with every provider.

### Model-specific capabilities

Provider-specific and model-specific compile-time checks are mandatory for native OpenAI Responses requests.

Use resource-scoped model marker traits:

```rust,ignore
pub trait OpenAIResponsesModel {
    const ID: &'static str;
}

pub trait SupportsReasoning: OpenAIResponsesModel {
    type Effort;
}

pub trait SupportsTemperature: OpenAIResponsesModel {}

pub struct Gpt5;

impl OpenAIResponsesModel for Gpt5 {
    const ID: &'static str = "gpt-5";
}

impl SupportsReasoning for Gpt5 {
    type Effort = Gpt5ReasoningEffort;
}
```

Requests and builders are generic over the model marker. Capability-specific methods exist only when their bounds are satisfied:

```rust,ignore
pub struct ResponseRequest<M: OpenAIResponsesModel> {
    model: PhantomData<M>,
    input: OpenAIResponsesInput,
    // fields common to every Responses model
}

impl<M: SupportsReasoning> ResponseRequestBuilder<M> {
    pub fn reasoning(mut self, effort: M::Effort) -> Self {
        // ...
    }
}

impl<M: SupportsTemperature> ResponseRequestBuilder<M> {
    pub fn temperature(mut self, temperature: Temperature) -> Self {
        // ...
    }
}
```

This provides two useful levels of compile-time checking:

- changing `OpenAI` to `Gemini` changes the provider request/config types and forces the caller to supply a valid Gemini configuration;
- changing `Gpt4o` to `Gpt5` changes which builder methods and setting value types are available.

Use associated setting types when support is not merely yes/no. For example, `SupportsReasoning::Effort` lets GPT-5 and a Pro model expose different valid effort enums. Independent capability traits avoid a combinatorial explosion of fixed capability-profile types.

Capability traits must be scoped to the resource or protocol (`openai::responses::capability`, `openai::realtime::capability`) because support can differ across endpoints even for the same model name.

Only encode relatively stable request-shape rules in the type system: whether a setting exists, which setting type it accepts, and which modality/tool family is legal. Keep account entitlements, staged rollouts, regions, quotas, rate limits, and other deployment state as runtime errors. This prevents typestate from becoming a brittle mirror of server operations.

#### Custom models

- Parse runtime configuration into a closed application enum whose match arms use typed model markers.
- Permit advanced users to define a local model marker and opt into capability traits for fine-tuned models and aliases. This makes the assertion compile-time visible and puts responsibility on the caller without requiring a crate release.
- Do not seal model/capability traits unless an invariant truly cannot be upheld by downstream implementations.
- Document that compile-time support reflects the crate's pinned provider specification; live APIs can still change, so compile-time checks complement rather than replace wire fixtures and opt-in live tests.

#### Implementation shape: generic facade, non-generic wire core

The typed API must not duplicate request serialization or transport logic for every model. Keep the generics in the caller-facing builder and erase them when construction is complete:

```rust,ignore
pub struct ResponseRequestBuilder<M, InputState> {
    wire: OpenAIResponsesWireRequest,
    _model: PhantomData<fn() -> M>,
    _input: PhantomData<InputState>,
}

impl<M: SupportsReasoning, S> ResponseRequestBuilder<M, S> {
    pub fn reasoning(mut self, effort: M::Effort) -> Self {
        self.wire.reasoning = Some(effort.into_wire());
        self
    }
}

impl<M: OpenAIResponsesModel> ResponseRequestBuilder<M, HasInput> {
    pub fn build(mut self) -> PreparedResponseRequest {
        self.wire.model = M::ID.into();
        PreparedResponseRequest(self.wire)
    }
}
```

`PhantomData` model/state markers are zero-sized. Only the small builder methods are monomorphized. `PreparedResponseRequest`, `OpenAIResponsesWireRequest`, response decoding, streaming, retries, and HTTP transport remain non-generic and have exactly one implementation.

Implementation rules:

- Keep the wire request private so callers cannot bypass builder invariants after model-type erasure.
- Use typestate only for genuinely required construction state such as input; do not encode every optional field into the builder type.
- Put model capability bounds on builder methods, not on the shared serializer or transport.
- Convert model-specific associated setting types into private wire enums at the method boundary.
- Do not parameterize response types by model unless the response shape actually differs in a way callers must know statically.
- Let the dynamic builder reuse the same private wire core, adding explicit runtime validation before it produces `PreparedResponseRequest`.
- Prefer a small handwritten builder if the builder-derive library makes conditional capability methods or diagnostics awkward.
- Generate repetitive marker and capability implementations from a reviewed, checked-in model manifest if the table becomes large; keep the resulting public types stable and source-controlled.

This confines type-level complexity to the API edge while keeping the implementation underneath ordinary, debuggable, and shared.

### Crate split trigger

Stay in one crate for now. Split into `ai-client-core`, `ai-client-openai`, `ai-client-gemini`, and a re-exporting `ai-client` facade only when one of these becomes true:

- Realtime/WebRTC adds a large dependency graph that non-OpenAI users should not compile;
- at least three substantial providers have independent release cadence;
- provider feature combinations make CI or semver management materially difficult;
- downstream crates need to depend on the shared traits without any provider implementation.

The module layout above is intentionally compatible with that later split.

## API and type rules

### Identifiers and models

- [x] Replace closed Responses model selection with typed known-model markers. `OpenAIModel` remains intentionally scoped to model retrieval and legacy Chat Completions.
- [x] Make native Responses request construction generic over its model marker, then erase it into one private non-generic wire request before transport.
- [x] Add resource-scoped capability traits and capability-bounded builder methods for known models.
- [x] Use associated setting types for capabilities whose valid values differ by model.
- [x] Accept aliases, snapshots, and fine-tuned model IDs through downstream-defined native model markers. OpenAI-compatible gateway names remain owned by F4's separate dialect family.
- [ ] Add resource ID newtypes where mixing IDs would be dangerous (`ResponseId`, `ConversationId`, `FileId`, `VectorStoreId`).
- [x] Do not rewrite a request based on a capability table. Invalid native Responses combinations fail to compile.

### Request and response compatibility

- [ ] Requests derive `Serialize`; responses derive `Deserialize`; add the reverse direction only for a demonstrated use case.
- [ ] Use typed enums for stable finite inputs such as roles, image detail, and status, but include an owned unknown/string form where the server may add values.
- [ ] Preserve the complete raw object for unknown tagged response items and stream events.
- [ ] Use `#[non_exhaustive]` on public enums and structs that are expected to grow.
- [ ] Prefer builders for large requests and typed constructors for required invariants.
- [x] Keep wire-only switches private. In particular, remove public responsibility for `stream`; streamed and non-streamed client operations set it correctly.
- [ ] Offer `extra_body`, `extra_query`, and `extra_headers` request options as a deliberate forward-compatibility escape hatch. Reject collisions with typed fields.
- [ ] Provide response helpers such as `output_text()`, `function_calls()`, `images()`, and `request_id()` without hiding the complete typed response.

### Client safety and configuration

- [x] Make API keys and reqwest clients private.
- [x] Remove public arbitrary-URL `get` and `post` methods. Internal resource calls must use relative paths resolved against the configured provider origin.
- [x] If a raw public request API is needed, require a validated relative path and keep auth same-origin. Make cross-origin requests an explicitly unauthenticated API. No raw API is exposed by F1.
- [x] Add configurable base URL, organization/project headers, user agent, default headers, connect timeout, and request timeout.
- [ ] Add an externally supplied HTTP-client seam only when the crate can still enforce same-origin redirect handling for provider-specific credential headers.
- [ ] Support environment-key loading as an opt-in convenience, not hidden global state.
- [x] Ensure `Debug`, errors, and traces never contain credentials, authorization headers, uploaded file contents, prompts, or model output by default.
- [x] Include crate name and version in the default user agent.

### Errors, retries, and response metadata

- [x] Replace the current status/string API error with a structured error containing provider, endpoint, status, provider error type/code/param/message, request ID, retry-after, and a safely bounded raw payload.
- [x] Distinguish configuration, transport, timeout, HTTP API, and JSON decode errors.
- [ ] Add crate-owned stream framing and protocol errors in F2.
- [x] Decode documented provider error envelopes before falling back to raw text.
- [x] Do not log full successful or failed response bodies automatically on decode failure.
- [x] Return or expose response metadata, especially request IDs and rate-limit headers.
- [ ] Retry only retryable statuses and transport failures, honor `Retry-After`, use bounded exponential backoff with jitter, and never automatically retry a non-idempotent operation unless an idempotency key or endpoint guarantee makes it safe.
- [ ] Make retry policy configurable and observable.

### Streaming and binary transports

- [x] Replace the custom OpenAI parser with a standards-correct, provider-neutral SSE decoder behind the crate's own error/event API.
- [x] Handle `\n` and `\r\n`, comments, multiple `data:` lines, `event`, `id`, `retry`, blank events, `[DONE]`, final buffered data, arbitrary byte chunk boundaries, and split UTF-8.
- [x] Check HTTP status and decode the normal API error envelope before returning any stream.
- [x] Preserve unknown event names and JSON payloads.
- [x] Return crate-owned stream and stream-error types so implementation dependencies can change without a breaking release.
- [ ] Add generic byte-stream and collected-bytes responses for speech, images, files, and video.
- [ ] Add multipart request support before standalone images, uploads, transcription, or voice resources.
- [ ] Keep WebSocket/WebRTC concerns out of the SSE abstraction.

## OpenAI coverage plan

The official documented OpenAPI file currently contains roughly 178 path entries, including duplicated beta paths and large organization/admin surfaces. Raw endpoint count is therefore not a useful first milestone. Coverage should be tracked by resource and operation, with the pinned official spec used to detect drift.

### P0 — foundation and current API correctness

- [x] Extract the shared transport and typed error model for existing JSON operations.
- [x] Lock credentials and raw transport behind private fields and same-origin paths.
- [x] Fix TLS feature wiring so `rustls-tls` and `native-tls` map to reqwest correctly and `--no-default-features` enables neither.
- [x] Replace query string concatenation with encoded query serialization.
- [x] Replace the SSE parser and add adversarial chunk-boundary tests.
- [x] Make streaming and non-streaming entry points own their wire mode automatically.
- [x] Introduce typed known-model markers and capability-bounded builders; later remove the dynamic native path after live testing proved it bypassed invariants.
- [ ] Split Responses types into focused modules before expanding their unions.
- [x] Add mock HTTP tests for success, provider errors, malformed bodies, non-JSON bodies, timeouts, rate limits, and stream handshake errors for the existing provider operations.

Exit criteria:

- no public secret or unrestricted authenticated URL API;
- no provider transport implementation is duplicated;
- default, no-default, rustls, native-tls, stream, and all-feature builds are tested;
- streamed and non-streamed calls have consistent errors and response metadata.

### P1 — Responses and conversations parity

Complete the OpenAI Responses resource before adding another text-generation abstraction.

- [ ] Responses create, retrieve, delete, cancel, input-items listing, input-token counting, and compaction.
- [ ] Background responses, polling helpers, storage controls, and conversation linkage.
- [ ] Complete create parameters from the pinned spec, including metadata, include selectors, prompt templates/variables, conversation, background mode, truncation, tool choice, parallel/max tool calls, service tier, safety identifier, and applicable cache controls.
- [ ] Complete input unions: text, image, file, audio, item references, prior assistant output, function/tool outputs, approval responses, and compaction/context items exposed by the current spec.
- [ ] Complete output unions: messages, reasoning, refusals, citations/annotations, function/custom tool calls, hosted tool calls, MCP items, computer calls, shell/code results, images, and compaction items.
- [ ] Complete tool definitions and tool-choice types for function/custom tools, web search, file search, code interpreter, image generation, computer use, remote MCP, shell, and any newer tools in the pinned spec.
- [ ] Parse every documented Responses stream event; preserve unknown events raw.
- [ ] Add accumulation helpers that reconstruct a final response from a stream without requiring callers to write an event state machine.
- [ ] Add Conversations CRUD and item CRUD/listing as the durable state companion to Responses.
- [x] Deprecate the native OpenAI Chat Completions module and methods, keep them default-off for migration, and ensure their internals are not the public compatibility abstraction.

Exit criteria:

- every Responses and Conversations operation in the pinned spec is marked supported, intentionally deferred, or not applicable in a checked-in coverage matrix;
- official request/response/event examples deserialize in fixture tests unless a recorded source defect contradicts the referenced schema, in which case the strict binding policy has an explicit regression test;
- new unknown tool/event variants survive round-trip inspection as raw payloads.

### P1b — OpenAI-compatible dialect framework

- [x] Add `OpenAICompatibleClient<D>` over the shared transport with private same-origin auth handling.
- [x] Add the resource-scoped `ChatCompletionsDialect`; defer `ResponsesDialect` and Images/Audio/Video/Batches traits until real adapters exist.
- [x] Reuse the typed model-marker and capability-bounded builder design for `ChatRequestBuilder<D, M, State>`.
- [x] Erase typed builders into private non-generic compatibility wire requests before transport.
- [x] Add `CustomDialect` plus dynamic models as the explicit minimally-assumed escape hatch.
- [ ] Implement initial known dialect markers from official provider documentation, prioritizing Gemini compatibility (already represented natively in the crate), Groq, OpenRouter, Mistral, and Anthropic compatibility.
- [x] Keep provider extensions typed and namespaced; do not place all providers' optional fields into the common request.
- [x] Build conformance coverage for the only claimed dialect, `CustomDialect`, including endpoint/auth/wire/response/stream/error behavior.
- [x] Add compile-fail tests showing that unsupported resources and model/provider settings are unavailable.
- [x] Add mock-server fixtures for the only claimed dialect; live smoke tests remain deferred until a named adapter is claimed.
- [x] Deprecate the old native OpenAI Chat Completions entry points and publish migration examples for both Responses and compatible-provider callers.

Exit criteria:

- adding a new compatibility dialect does not require copying the HTTP client or editing a universal options struct;
- a dialect exposes only the resources and typed settings its conformance suite proves;
- OpenAI Chat Completions can later be removed without removing Chat-Completions-shaped support for other providers.

### P2 — standalone Images API

- [ ] Standalone Images generation, edits, and variations.
- [ ] Multipart image inputs, masks, multiple input images where supported, output bytes/base64, quality, format, compression, background, size, and partial-image streaming.
- [ ] Keep the Responses image-generation tool and standalone Images API as separate typed paths; they have different capabilities and response shapes.
- [ ] Size limits and MIME-type validation should be explicit local validation, never silent coercion.

Exit criteria:

- image generation and editing work with both in-memory bytes and file-backed multipart input without loading large outputs twice;
- generated media can be streamed to a caller-provided sink.

### Deferred beyond the current product scope

The following sections are retained only as architectural notes for possible future requests. They are not active roadmap commitments. Do not implement them unless scope is explicitly expanded after Responses and Images are accepted.

### Deferred — audio and voice over HTTP

Build the HTTP audio APIs before Realtime because they exercise binary and multipart foundations with much less protocol complexity.

- [ ] Speech generation with collected bytes and streaming output.
- [ ] Transcriptions with multipart input, JSON/text/SRT/VTT/verbose response formats, timestamps, prompt, language, and streaming events where supported.
- [ ] Translations with the supported response formats.
- [ ] Typed audio formats, voices, usage, and timestamp granularities with extensible string fallbacks.
- [ ] Custom voice and voice-consent resources behind an explicit feature if account availability remains limited.
- [ ] No implicit audio decoding dependency in core; return encoded audio bytes/streams plus declared media type.

Exit criteria:

- speech can be streamed without buffering the full file;
- transcription can upload large supported files without copying them into a base64 JSON body;
- audio-specific API errors use the same common error and request metadata model.

### Deferred — Realtime voice and multimodal sessions

Realtime is a separate protocol client, not another REST resource method.

- [ ] Start with server-side WebSocket sessions and the complete typed client/server event protocol.
- [ ] Model session configuration, conversation items, audio buffers, VAD, transcription, tool calls, interruptions, truncation, and response lifecycle explicitly.
- [ ] Preserve unknown events raw and support protocol-version drift.
- [ ] Add reconnect/cancellation semantics only where the protocol makes them safe; do not pretend a dropped realtime session is a retryable REST request.
- [ ] Add ephemeral/client-secret creation through the REST client.
- [ ] Add WebRTC and SIP call-control resources only behind dedicated optional features and only if this crate is intended to own media/session negotiation. Otherwise provide the REST/session types and document integration boundaries.
- [ ] Keep audio codecs and playback/capture out of the base crate; expose bytes/frames and format metadata.

Exit criteria:

- a text-and-audio WebSocket session can be driven with typed events end-to-end;
- backpressure, cancellation, interruption, and connection closure behavior are documented and tested;
- non-Realtime users do not compile WebSocket/WebRTC dependencies.

### Deferred — broad data-plane coverage

- [ ] Embeddings.
- [ ] Text/image Moderations.
- [ ] Vector Stores, files, batches, and search.
- [ ] Batches, including typed request JSONL helpers and result correlation by custom ID.
- [ ] Containers and container files needed by hosted tools.
- [ ] Videos create/retrieve/list/delete/content/remix/edit/extend as supported by the pinned spec.
- [ ] Models list/retrieve/delete with arbitrary model IDs and pagination where applicable.

These should reuse common pagination, binary, multipart, polling, and long-running-operation helpers rather than inventing resource-specific variants.

### Deferred — control plane and specialist APIs

- [ ] Fine-tuning jobs, events, checkpoints, permissions, pause/resume/cancel.
- [ ] Evals, runs, and output items.
- [ ] Usage and costs.
- [ ] Organization/project administration only if this crate is intentionally expanding from an inference SDK into a full OpenAI platform SDK.
- [ ] Gate admin-key resources separately so normal API-key users do not accidentally discover them as ordinary inference operations.
- [ ] Do not implement deprecated Assistants/Threads for new work. If migration support is unavoidable, isolate it behind a default-off `assistants` feature with its announced 2026-08-26 shutdown date in the docs.

## Gemini and future-provider cleanup

The OpenAI work should improve the provider framework rather than leaving Gemini on a parallel architecture.

- [ ] Move Gemini onto the shared private transport and error model.
- [ ] Split `generate_content.rs` into input, output, configuration, safety, and image modules.
- [ ] Replace the Gemini closed model enum in request paths with an extensible model ID plus known constants.
- [ ] Keep Gemini's native `generateContent` semantics; do not rename it to mimic OpenAI Responses.
- [ ] Add contract tests that demonstrate a new provider can supply auth, base URL, error decoding, and resources without copying the entire HTTP client.
- [ ] Prototype the layered unified text interface against OpenAI and Gemini, but do not mark it stable until a third provider or real runtime-routing consumer proves the abstractions.

## Feature policy

Prefer a few meaningful feature gates over one feature per endpoint:

```toml
[features]
default = ["openai", "gemini", "rustls-tls"]
openai = []
openai-compatible = []
gemini = []
stream = ["dep:futures"]
realtime = ["openai", "stream", "dep:..."]
webrtc = ["realtime", "dep:..."]
chat-completions = ["openai"] # deprecated native OpenAI migration surface
rustls-tls = ["reqwest/rustls-tls"]
native-tls = ["reqwest/native-tls"]
```

Exact defaults can remain backward-compatible for `0.4.0`, but the invariants are:

- TLS is not hard-coded on the dependency line;
- heavy Realtime/WebRTC dependencies are default-off;
- normal OpenAI resources do not each get a feature flag;
- known compatibility dialect markers remain lightweight; add provider-specific feature flags only when they introduce dependencies or large generated surfaces;
- the CI matrix proves every advertised feature combination compiles;
- mutually enabled TLS backends are documented or rejected if the chosen stack cannot handle them predictably.

## Testing and conformance

- [ ] Pin the official OpenAI OpenAPI document by digest and record its retrieval date.
- [ ] Generate `specs/openai-coverage.md` from a small checked-in tool that compares documented operations with a reviewed support manifest.
- [ ] Use the spec to generate drift reports and selected internal wire types; do not publish unreviewed generated names as the ergonomic API.
- [ ] Add provider fixture directories for official examples and captured, redacted edge cases.
- [ ] Add mock-server integration tests that assert method, path, query, headers, JSON/multipart bodies, status handling, and response metadata.
- [ ] Add SSE fuzz/property tests over arbitrary byte chunk boundaries and line endings.
- [x] Add compile-fail or builder tests for illegal request combinations where practical.
- [x] Use `trybuild` compile-fail fixtures to prove that provider-specific options cannot cross providers and unsupported known-model settings are unavailable.
- [x] Add compile-pass fixtures for downstream-defined custom model markers and capability implementations.
- [ ] Run `cargo clippy --all-targets --all-features -- -D warnings`, `cargo test --all-features`, doctests, and a feature matrix in CI.
- [ ] Add `cargo-semver-checks` or `cargo-public-api` reporting before releases.
- [ ] Keep live provider tests opt-in, credential-gated, low-cost, and separate from deterministic CI.
- [ ] Add one downstream compatibility test for the primary real consumer before a breaking release.

## Documentation and release contract

- [ ] Rewrite the README around provider-native resource examples, with Responses first.
- [ ] Publish a support matrix by provider, resource, operation, streaming, and maturity (`stable`, `experimental`, `legacy`).
- [ ] Document storage/data-retention implications for Responses background mode, Conversations, and Images without implying all endpoints have the same behavior.
- [ ] Document retry and idempotency behavior.
- [ ] Add a `0.3 -> 0.4` migration guide covering model IDs, client privacy, resource accessors, errors, and streaming.
- [ ] Make the foundational API cleanup one coherent `0.4.0` breaking release; do not preserve duplicate legacy aliases indefinitely in a pre-1.0 private crate.
- [ ] Continue releasing Responses additions as patch versions when non-breaking; use a `0.x` minor bump for subsequent breaking public API changes.

## Recommended implementation order

1. **Foundation:** private client state, shared transport, same-origin paths, typed errors/metadata, TLS wiring, encoded queries.
2. **Streaming:** crate-owned SSE and byte streams, status handling, stream-mode invariants, adversarial tests.
3. **Extensible types:** typed model markers, capability traits, typed resource IDs, unknown raw payloads, and request options.
4. **Compatibility separation:** deprecate native OpenAI Chat Completions, extract private reusable wire pieces, and add resource-scoped `openai_compatible` dialect traits plus initial adapters.
5. **Responses refactor and parity:** split modules, full request/item/tool/event coverage, lifecycle operations, Conversations.
6. **Standalone Images:** generation, edits, variations, multipart inputs, binary/base64 outputs, and supported image streaming.
7. **Portable capability layer:** only after real downstream usage proves the common semantics.

All other OpenAI resource families are deferred unless the product scope is explicitly expanded.

## Explicit non-goals

- A single universal request type containing every provider's options.
- Treating “OpenAI-compatible” as one exact API or assuming a base URL change proves compatibility.
- Hiding provider-native APIs behind dynamic dispatch.
- Silently rewriting caller requests to match a hard-coded model table.
- Logging raw prompts, outputs, uploaded content, or credentials for convenience.
- Blindly publishing an enormous generated OpenAPI client as the primary Rust API.
- Investing in deprecated OpenAI Assistants/Threads without a concrete migration need.
- Pulling WebRTC, audio codecs, or media-device dependencies into default builds.

## Definition of architectural success

The architecture is ready to scale when a new provider can be added by supplying authentication, error decoding, resource modules, and provider-native types without copying HTTP/SSE/pagination code; and when a newly documented OpenAI operation can be added to the appropriate resource module without changing the root client, weakening type safety, or breaking unrelated providers.

## Sources checked for this pass

- Current crate source at `v0.3.1` (`0d47c0d`).
- [Official OpenAI API reference](https://platform.openai.com/docs/api-reference).
- [Official OpenAI OpenAPI specification pointer](https://github.com/openai/openai-openapi).
- [OpenAI Responses quickstart and tools overview](https://platform.openai.com/docs/quickstart/make-your-first-api-request).
- [OpenAI Realtime API reference](https://platform.openai.com/docs/api-reference/realtime).
- [OpenAI Audio API reference](https://platform.openai.com/docs/api-reference/audio).
- [OpenAI Images API reference](https://platform.openai.com/docs/api-reference/images).
- [OpenAI Assistants deprecation notice](https://platform.openai.com/docs/assistants/deep-dive).
- [Gemini OpenAI compatibility](https://ai.google.dev/gemini-api/docs/openai).
- [Gemini native API reference](https://ai.google.dev/api).
- [Mistral Chat Completions API](https://docs.mistral.ai/api).
- [Groq OpenAI compatibility and Responses support](https://console.groq.com/docs/openai).
- [OpenRouter API reference](https://openrouter.ai/docs/api/reference/overview).
- [Claude OpenAI compatibility release note](https://platform.claude.com/docs/en/release-notes/overview).
