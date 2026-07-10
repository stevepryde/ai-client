# Real-provider test suite

The live suite exists to catch valid-looking mock requests that real providers
reject, stale model identifiers, response-shape drift, and stream decoders that
only work against fixtures.

## Safety and cost gates

Every provider call is both:

1. compiled only by the default-off `live-tests` Cargo feature; and
2. marked `#[ignore]`, requiring an explicit `--ignored` selection.

Normal test commands therefore spend no tokens, including
`cargo test --all-features`. Run live tests serially to keep rate-limit failures
actionable:

```bash
cargo test --all-features --test live_openai live_openai_core \
  -- --ignored --test-threads=1
```

Tests containing `expensive`, `entitled`, or `resource` in their names are not
part of the cheap core selection. Never select an entire live test binary unless
you intend to run those cases too.

## Credentials and provisioned resources

| Variable | Used by |
| --- | --- |
| `OPENAI_API_KEY` | Native OpenAI Responses, Conversations, and legacy Chat Completions |
| `OPENAI_ORGANIZATION` | Optional OpenAI organization header |
| `OPENAI_PROJECT` | Optional OpenAI project header |
| `GEMINI_API_KEY` | Native Gemini models, token counting, generation, and streaming |
| `AI_CLIENT_COMPATIBLE_BASE_URL` | OpenAI-compatible Chat Completions endpoint, including its version prefix |
| `AI_CLIENT_COMPATIBLE_API_KEY` | Bearer credential for that endpoint |
| `AI_CLIENT_COMPATIBLE_MODEL` | Runtime model ID for that endpoint |
| `OPENAI_VECTOR_STORE_ID` | Provisioned file-search definition test |
| `OPENAI_MCP_SERVER_URL` | Reachable MCP server definition test |

An explicitly selected test treats a missing variable as a failure. This avoids
green runs that did not actually contact the provider.

## Coverage and commands

### Cheap core

These use metadata endpoints, token counting, state CRUD, or tiny prompts:

```bash
cargo test --all-features --test live_openai live_openai_core \
  -- --ignored --test-threads=1
cargo test --all-features --test live_gemini live_gemini_core \
  -- --ignored --test-threads=1
cargo test --all-features --test live_openai_compatible live_compatible_core \
  -- --ignored --test-threads=1
```

Coverage includes OpenAI's model-list envelope, Responses create/retrieve/delete,
input-token counting, input-item listing, continuation, Conversations and nested
item CRUD, plus native and compatibility streaming. Gemini covers model list/get,
token counting, ordinary generation fields, safety categories, structured JSON,
and streaming.

### Model and enum matrices

These make tiny requests across representative capability families and every
checked-in option/enum value:

```bash
cargo test --all-features --test live_openai live_openai_model_matrix \
  -- --ignored --test-threads=1
cargo test --all-features --test live_openai live_openai_option_matrix \
  -- --ignored --test-threads=1
cargo test --all-features --test live_gemini live_gemini_model_matrix \
  -- --ignored --test-threads=1
cargo test --all-features --test live_gemini live_gemini_option_matrix \
  -- --ignored --test-threads=1
```

OpenAI's metadata test checks every active Responses-compatible GPT model. The
paid generation matrix is intentionally limited to `gpt-5.1`, `gpt-5.2`,
`gpt-5.4`, and `gpt-5.5`. Its option matrices cover reasoning efforts,
prompt-cache retentions/options, include selectors, text formats, verbosity,
service tiers, and non-invoked tool definitions. Gemini covers every text model,
every adjustable safety threshold, and the full inexpensive generation config.

GPT-5.6 is currently a selected-partner preview. Its Sol, Terra, Luna, alias,
six reasoning efforts, and persisted-reasoning contexts therefore run under the
`live_openai_entitled` filter. Pro mode is deliberately under
`live_openai_expensive`.

### Entitlement and resource cases

```bash
cargo test --all-features --test live_openai live_openai_entitled \
  -- --ignored --test-threads=1
cargo test --all-features --test live_openai live_openai_resource \
  -- --ignored --test-threads=1
```

Flex/Priority service tiers may be rejected when the test project lacks that
entitlement. File search and MCP require the provisioned variables above. Such a
failure describes the tested account, not necessarily the request schema, so CI
should run these only in a project configured for them.

### Explicitly expensive cases

These invoke billed hosted tools or generate images:

```bash
cargo test --all-features --test live_openai live_openai_expensive \
  -- --ignored --test-threads=1
cargo test --all-features --test live_gemini live_gemini_expensive \
  -- --ignored --test-threads=1
```

The Gemini image option test generates every advertised aspect ratio and image
size. The OpenAI hosted-tool test invokes web search, code interpreter, and image
generation. Their names intentionally keep them outside every core filter.

### Deprecated native Chat Completions

The default-off migration surface still gets a real-provider check when its
feature is enabled:

```bash
cargo test --all-features --test live_openai_legacy \
  -- --ignored --test-threads=1
```

## CI recommendation

Run the cheap core and model matrices on a scheduled or manually dispatched job,
not on every push. Keep provider secrets in the CI secret store and set a small
project budget. Run entitlement/resource and expensive filters manually or on a
less frequent schedule. The ordinary mock/compile suite remains the fast required
check for every change.
