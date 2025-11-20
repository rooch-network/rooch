# Repository Guidelines

## Project Structure & Module Organization
- `crates/` holds Rust crates (core runtime, DA, storage, testsuite).
- `moveos/` contains Move runtime plumbing; `frameworks/` hosts Move packages (`moveos-stdlib`, `rooch-framework`, `bitcoin-move`, `rooch-nursery`).
- `examples/` offers sample Move packages; `apps/` and `sdk/typescript/` house dashboards and TS SDKs; `docs/`, `scripts/`, `docker/`, `infra/`, and `kube/` support documentation and ops.

## Build, Test, and Development Commands
- `make build` builds Rust (release `optci`) and core Move frameworks; `make quick-check` runs a fast Rust debug build plus `move-framework`.
- `make test` runs Rust (nextest + integration) and Move suites. Narrow scope with `FILTER=name` (e.g., `make test-integration FILTER=payment_channel` or `make test-move-frameworks FILTER=did`).
- Rust checks: `cargo fmt -- --check`, `cargo clippy --workspace --all-targets --all-features --tests --benches -- -D warnings`; `make lint` wraps them.
- JS/TS uses pnpm only (`npx only-allow pnpm`): `pnpm lint`, `pnpm prettier:check`, `pnpm test-suite`, or package-specific commands with `pnpm --filter ./sdk/typescript/<pkg>`.
- Build the Rooch CLI for Move workflows with `cargo build --profile debug` (binary at `target/debug/rooch`), then verify via `make verify`.

## Coding Style & Naming Conventions
- Rust: rustfmt defaults (4-space indent), deny clippy warnings, `snake_case` files/modules, descriptive crate scopes (`rooch-da`, `rooch-db`, etc.).
- Move: keep module/package names aligned with framework folders; prefer ASCII identifiers and deterministic ordering in manifests.
- TypeScript: Prettier + ESLint (`prettier.config.js`); `PascalCase` components, `camelCase` variables; keep imports sorted.

## Testing Guidelines
- Rust: unit tests sit next to code; integration lives in `crates/testsuite/`. For fast coverage use `cargo nextest run --workspace --all-features`; for CI parity use `make test-rust`.
- Move: `make test-move-frameworks` for the core packages; `make test-move-examples` for samples. Add targeted tests with `FILTER=` when isolating failures.
- TypeScript: run package tests via `pnpm --filter ./sdk/typescript/<pkg> test` and keep fixtures minimal.

## Commit & Pull Request Guidelines
- Use `<type>(<scope>): <subject>` (types: feat, fix, refactor, ci, docs, chore, rfc), mirroring history such as `chore(deps): bump inferno...`. Keep subjects in present tense with a concise scope.
- Open draft PRs early; fill the template with a clear Summary and `Fixes #issue` when relevant. Include repro steps, configs, and screenshots for UI-facing work.
- Before review, ensure `make lint` plus relevant tests are green; link any blocked items or follow-ups in the PR description.***
