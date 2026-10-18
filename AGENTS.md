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

## Fork branches and upstream synchronization

- `upstream` is `jub0t/Concat`, the original project. `origin` is `victorstacks1/concat`, this fork.
- Keep `origin/main` as a clean mirror of `upstream/main`. Do not develop personal features or change release configuration on `main`.
- Keep approved personal changes on `origin/custom`.
- Start temporary `feature/*` branches from `custom`. For upstream integration, merge the updated `main` into a feature branch, resolve conflicts, run the relevant checks, then open a pull request targeting `custom`. Update `custom` only through an approved pull request.
- Do not update `origin/main` if it has fork-only commits; inspect and preserve them before syncing. Prefer a fast-forward from `upstream/main`.
- Keep the Concat in-app updater pointed at upstream releases (`jub0t/Concat`). Do not redirect it to fork releases unless explicitly requested.

## Fork releases

- Create a fork release from `custom` only when explicitly requested, after its pull request is approved and CI/builds pass. Do not release every commit.
- Base the fork version on the latest stable upstream release and increment its patch version so it remains a valid three-part version (for example, upstream `v0.2.7` yields fork `v0.2.8`). Keep `src/Cargo.toml` and the release tag identical.
- Record `Upstream base: <version>` and `Origin commit: <short-sha>` in the release notes.
- The app updater follows upstream releases; a fork release is not offered by that updater and must be installed from the fork release page.
- The release workflow creates a draft after CI and platform builds finish. Review its assets and manifest before publishing.
