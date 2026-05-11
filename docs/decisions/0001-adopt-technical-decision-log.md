# Adopt technical decision log

- **Status**: accepted
- **Date**: 2026-05-11
- **Context**: The project needs a lightweight, scan-friendly record of stack and schema choices so future work and agents share the same rationale without rereading long plans.
- **Decision**: Use `docs/decisions/` with monotonic `NNNN-short-kebab-title.md` files and an always-apply Cursor rule (`.cursor/rules/technical-decision-log.mdc`) that instructs agents to add or update ADRs for non-trivial technical changes. Maintain an index table in this directory’s `README.md`.
- **Alternatives considered**: Relying on plan files or chat only (hard to discover); a single rolling `CHANGELOG`-style doc (poor separation of one decision per topic).
- **Consequences**: Contributors and agents should update `README.md` here when adding decisions. Superseding a decision should add a new numbered file rather than silently rewriting history.
- **Links**: `AGENTS.md` (repo root) points here.
