# Instructions for AI Agents

## Project references

- Read [`CONTRIBUTING.md`](CONTRIBUTING.md) for contribution rules and development commands.
- Read [`ARCHITECTURE.md`](ARCHITECTURE.md) before changing application or engine boundaries.
- Read [`src/README.md`](src/README.md) for engine crate structure and Rust development.
- [`README.md`](README.md) is the user-facing guide and source for product behavior.

## Development

- Identify the relevant crate and follow its existing boundaries; avoid unrelated changes.
- Use the pinned Rust toolchain in `src/rust-toolchain.toml` and the commands documented by the project.
- Run the checks relevant to the change. Report what was run and any validation that remains.
- For Windows UI, renderer, GPU, or packaging changes, build and run the staged Windows bundle with `scripts/ci/smoke-windows.ps1`. `--version` alone does not initialize the UI.
- Keep secrets, machine-specific paths, and temporary session notes out of tracked files.
- When asked to commit, stage only files for the requested change and preserve unrelated working-tree edits.

## Fork release versioning

- Base each fork release on the latest stable release from `upstream`, then increment its patch number so the fork version sorts above the upstream version and the app updater can offer it. For example, upstream `v0.2.7` becomes fork `v0.2.8`.
- Build the release from the intended `origin` commit and record that source commit's short SHA in the release notes as `Origin commit: <sha>`.
- Keep the workspace version in `src/Cargo.toml` and the release tag identical (`0.2.8` and `v0.2.8` in the example). Do not reuse the upstream version for a fork release.
- The release workflow creates a draft after CI and all platform builds finish. Review the assets and manifest, then publish the draft; do not create a release for every commit.
