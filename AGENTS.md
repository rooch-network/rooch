# AGENTS.md — Rooch AI working guide

## 1) Quick project context
- Rooch Network: Bitcoin-focused L2, Move-based “VApp container”. Main workspace under `crates/`, Move runtime in `moveos/`, frameworks in `frameworks/`, TS SDK under `sdk/`, apps under `apps/`.

## 2) Prereqs & setup
- Tooling: Rust >= 1.91.1, Node >= 18 with pnpm, optional Docker.
- Install: `cargo build` (or `make build`), `pnpm install` where needed (use pnpm only).
- Useful env: `RUST_LOG=debug`, `RUST_BACKTRACE=1`, `ROOCH_BINARY_BUILD_PROFILE=debug|optci`.

## 3) Canonical commands
- Build/test: `make build`, `make quick-check`, `make test`, `make test-rust`, `make test-move-frameworks`, `make test-move-examples`.
- Lint: `make lint` (wraps `cargo fmt -- --check` + clippy); TS: `pnpm lint`, `pnpm prettier:check`.
- Move/CLI: `cargo build --profile debug` (CLI at `target/debug/rooch`), `rooch init`, `rooch move build|test`, `rooch server start -n local`.
- Faster Rust tests: `cargo nextest run --workspace --all-features`.

## 4) Style & conventions
- Rust: rustfmt defaults, deny clippy warnings, `snake_case` modules, descriptive crate names (`rooch-da`, `rooch-db`, …).
+- Move: align module/package names with framework folders; ASCII identifiers; deterministic manifests.
- TS: Prettier + ESLint; PascalCase components, camelCase vars; sorted imports.

## 5) Testing guidance
- Rust unit near code; integration in `crates/testsuite/`. Filter with `FILTER=` for make targets.
- Move: core with `make test-move-frameworks`; examples with `make test-move-examples`.
- TS: package tests via `pnpm --filter ./sdk/typescript/<pkg> test`; keep fixtures minimal.

## 6) Commits & PRs
- Message format: `<type>(<scope>): <subject>`; types = feat/fix/refactor/ci/docs/chore/rfc.
- Open draft PR early; include Summary and `Fixes #issue` if applicable; attach repro steps/screenshots for UI.
- Before requesting review: `make lint` + relevant tests green; list known gaps/follow-ups.

## 7) Notes for AI agents
- Respect pnpm-only rule for JS/TS.
- When pruning/GC/snapshot/replay work, prefer existing commands under `rooch db state-prune ...`.
- Keep instructions concise; avoid non-ASCII unless already present.
