# ai-client Release Strategy

This repository is private and owned directly, so releases go straight to `main`.
Do not open pull requests for routine version releases.

## Release Order

For any release or version bump, follow this order exactly:

1. Run clippy first.
   Command: `cargo clippy --all-targets --all-features -- -D warnings`
2. Run tests second.
   Command: `cargo test --all-features`
3. Decide the next crate version before pushing.
4. Update `Cargo.toml` to that version.
5. Create a git tag for that exact version.
   Tag format: `vX.Y.Z`
6. Push the commit to GitHub on `main`.
7. Push the tag to GitHub.

## Versioning Rule

This crate is currently on `0.x`, so use this release rule:

- Use a patch bump for non-breaking changes.
- Use a minor bump for breaking changes.

When deciding whether a change is breaking, check public API changes and behavior changes that consuming crates rely on.

## Tagging Rule

Consumers may reference this crate by git tag, so every published version must have a matching git tag.
Never push a release commit without also creating and pushing the corresponding tag.

## GitHub Policy

- Release directly to `main`.
- No PR workflow is required for version bumps or releases in this repository.
