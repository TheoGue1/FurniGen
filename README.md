# FurniGen

Parametric wardrobe design (Rust core, WASM boundary, web preview). Stack and domain notes live under [`.cursor/rules/furnigen-project-context.mdc`](.cursor/rules/furnigen-project-context.mdc).

## License

FurniGen is licensed under **GNU General Public License v3.0 or later** (`GPL-3.0-or-later`). The full license text is in [`LICENSE`](LICENSE). Project-level copyright and warranty disclaimer boilerplate is in [`COPYRIGHT`](COPYRIGHT); individual source files may carry their own SPDX or copyright lines.

## Dependency compliance

The project is meant to be distributed as GPL-covered software. When you add Rust crates or npm packages, pick licenses that are compatible with the GPLv3 combined work you plan to ship (for example, MIT, Apache-2.0, and ISC often combine cleanly with GPLv3; some licenses are not compatible—check before merging).

Automation to enforce this is not wired yet. When the Cargo workspace and web package manifest exist, plan to add **Rust**: [`cargo-deny`](https://embarkstudios.github.io/cargo-deny/) (licenses + advisories) in CI, and **JavaScript**: a license audit step (for example `license-checker`, `pnpm licenses list`, or a REUSE-style scan) so incompatible dependencies fail the pipeline instead of slipping in.
