# CI and TDD baseline (Rust + web)

- **Status**: accepted
- **Date**: 2026-05-11
- **Context**: The FurniGen plan calls for Vitest, Playwright, and `cargo test` in scripts and CI so every meaningful change can follow red → green → refactor with automated gates.
- **Decision**: Use **GitHub Actions** with two parallel jobs: **Rust** (`cargo fmt --check`, `cargo clippy -D warnings`, `cargo test --workspace`) and **web** (`npm ci` in `web/`, Vitest, `tsc --noEmit`, Playwright Chromium against `vite preview` on port 4173). Root `package.json` exposes `test`, `test:rust`, `test:web`, `test:e2e`, and `ci` as local entrypoints. E2E tests live under `web/e2e/` with feature-oriented folders (e.g. `e2e/preview/`). `web/package.json` uses an **npm `overrides`** entry so Vitest and the app resolve a **single Vite** version (avoids duplicate `vite` types during `tsc`).
- **Alternatives considered**: A single monolithic CI job (slower feedback); **pnpm** workspaces (fine later; **npm** + `package-lock.json` keeps the first slice simple on all platforms); relying on Vitest defaults without a dedicated `typecheck` step (rejects—we want `tsc` parity with `vite build`).
- **Consequences**: Contributors need **Rust stable** (fmt + clippy components) and **Node 22** locally to mirror CI; Playwright browsers must be installed once via `npx playwright install chromium` under `web/`. WASM packaging (`wasm-pack`) is intentionally **not** in this baseline yet.
- **Links**: Plan bootstrap checklist (tdd-workflow).
