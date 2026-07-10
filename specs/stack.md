# ai-client stack

Status: confirmed

Last reviewed: 2026-07-10

## Application surface

- **Decision:** A single Rust library crate for native provider clients and shared transport mechanics.
- **Status:** confirmed
- **Source:** `Cargo.toml`, `AGENTS.md`, `specs/todo.md`
- **Constraint:** Do not create a workspace until the split triggers in `specs/todo.md` are met.

## Language and public runtime

- **Decision:** Rust 2021 with an MSRV close to current stable while the crate remains experimental.
- **Status:** inherited
- **Source:** `Cargo.toml`, `README.md`
- **Constraint:** Keep the public async API executor-agnostic. Runtime-specific dependencies may be used by deterministic tests without becoming part of the public contract.

## HTTP and serialization

- **Decision:** `reqwest` 0.12 provides HTTP mechanics; `serde` and `serde_json` own reviewed wire serialization.
- **Status:** confirmed
- **Source:** `Cargo.toml`, `specs/todo.md`
- **Constraint:** Provider semantics remain in provider modules. Shared core code owns authentication injection, URL resolution, request execution, metadata extraction, and mechanical decoding only.
- **Constraint:** Do not accept an externally configured reqwest client until the crate can enforce same-origin redirect handling for provider-specific credential headers.

## API construction

- **Decision:** Handwritten Rust types plus `bon` where it does not obstruct conditional typed builders.
- **Status:** confirmed
- **Source:** `Cargo.toml`, `specs/todo.md`
- **Constraint:** Typed model-marker and capability-bounded builders may replace derive-generated builders when needed. They must erase into private non-generic wire requests before transport.

## TLS and features

- **Decision:** Crate features map directly to reqwest features. The default is rustls; `--no-default-features` enables no TLS backend; `native-tls` enables native TLS without rustls.
- **Status:** confirmed
- **Source:** `AGENTS.md`, `specs/todo.md`
- **Constraint:** Enabling both TLS features is supported as reqwest's combined configuration and must compile. Streaming and later Realtime dependencies remain optional.

## Testing and quality

- **Decision:** Rust unit/integration tests, deterministic local HTTP fixtures, clippy with warnings denied, doctests, and an explicit feature matrix.
- **Status:** confirmed
- **Source:** `AGENTS.md`, `specs/todo.md`
- **Constraint:** Live-provider tests remain opt-in and credential-gated. Compile-fail coverage is required when typed provider/model capabilities land.

## Release and compatibility

- **Decision:** Breaking public API cleanup targets `0.4.0`; routine releases commit and tag directly on `main` in the exact order required by `AGENTS.md`.
- **Status:** confirmed
- **Source:** `AGENTS.md`, `specs/todo.md`
- **Constraint:** Roadmap implementation commits are not releases. Do not bump, tag, or describe unreleased test-only work as published behavior.
