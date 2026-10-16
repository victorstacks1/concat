# Agent Guidance and Windows Startup Smoke Design

## Problem

The repository has detailed user and contributor documentation, but no root
`AGENTS.md` for AI coding agents. Project instructions and durable development
constraints therefore are not presented in one predictable place for future
agent sessions.

The Windows application also has a useful packaged-bundle smoke test, but the
normal CI workflow for pull requests runs engine checks and does not build and
launch the desktop editor. The app smoke runs as part of the reusable app-build
workflow, which is called by the release workflow. A broken editor startup can
therefore reach a development branch before a release is attempted.

## Goals

- Add a concise English root `AGENTS.md` that applies to AI agents working
  anywhere in this repository.
- Keep canonical user documentation, contribution requirements, and detailed
  architecture in their existing documents; link to those instead of copying
  them into agent instructions.
- Run the existing Windows packaged-bundle smoke on pull request updates so
  app startup regressions fail a visible CI check before release.
- Keep release validation for every supported platform intact.
- Keep unrelated working-tree changes out of the documentation and workflow
  commits.

## Non-goals

- Rewriting the README, CONTRIBUTING, or ARCHITECTURE documents.
- Adding a development diary or storing temporary session notes in
  `AGENTS.md`.
- Replacing the existing smoke test or changing the release artifact format.
- Claiming universal GPU-driver coverage from a GitHub-hosted Windows runner.

## Design

### Root agent instructions

Create `/AGENTS.md` in English. It is repository-wide guidance for any coding
agent, not a private user notebook. Keep it short and focused on durable
working rules:

- Read `CONTRIBUTING.md`, `ARCHITECTURE.md`, and `src/README.md` when a task
  touches their areas; treat those documents as the canonical references.
- Identify the relevant crate and follow existing boundaries before changing
  code. Avoid broad changes unrelated to the requested task.
- Use the pinned Rust toolchain and the repository's documented commands.
- Run relevant validation and state clearly what was and was not verified.
- For Windows UI, renderer, GPU, or packaging changes, require a startup smoke
  of the staged Windows bundle; `--version` alone does not initialize the UI.
- Do not commit or push unless the user asks. When asked to commit, stage only
  files for that requested change and preserve unrelated working-tree edits.
- Keep secrets, machine-specific paths, and temporary state out of tracked
  instructions.

The guide may summarize a command or invariant needed at the point of use, but
must link to the existing source of truth for longer details.

### Pull-request startup validation

Reuse `scripts/ci/smoke-windows.ps1` rather than creating another smoke
implementation. Its `--smoke` path initializes the editor through its first
published frame, while the script also checks the staged bundle's DLL
dependencies and runs with a restricted Windows-only `PATH`.

Add a Windows x86_64 app-build-and-smoke check to the normal pull-request CI.
Reuse the existing app workflow's dependency pins, staged-bundle layout, and
smoke script rather than duplicating setup where practical. The check must build
the app profile used for distribution, stage the resulting executable and its
runtime dependencies, and run the same smoke script against that exact bundle.
It must run for every pull-request update. Preserve the existing all-platform
release build and smoke behavior.

GitHub branch protection is an operational prerequisite to enforcement: mark
the new stable check as required on the fork's protected base branch. Workflow
code can report failures, but cannot prevent a merge when repository rules
allow bypassing or ignoring the check.

### GPU-specific limitation

The hosted Windows runner's adapter and driver may differ from the user's PC.
The CI smoke protects against general startup regressions and the observed
first-frame crash path. For changes to Skia, WGPU, or platform surface setup,
also launch the built app or run `--smoke` on the target Windows machine before
replacing its installed executable.

## Acceptance criteria

1. A root `AGENTS.md` exists in English, applies across the repository, links to
   canonical project documents, and includes the Windows smoke requirement.
2. A pull-request update runs a stable Windows x86_64 app startup check against
   a staged distribution-profile bundle.
3. A non-zero exit or crash from `concat.exe --smoke` fails that check, and the
   fork can require the check before merge.
4. Existing release smoke coverage for other supported platforms is unchanged.
5. Commits for this work do not include unrelated files already modified in the
   worktree.

## Validation

- Review the new `AGENTS.md` for clarity, durable scope, and links to canonical
  documentation.
- Inspect workflow triggers and job conditions to confirm the Windows check
  runs on pull-request updates without accidentally running all platform
  release packaging jobs.
- Validate the workflow syntax and run the Windows smoke against the staged
  bundle.
- Confirm release jobs still call their existing platform smoke checks.
- Verify staged commit contents before committing; preserve all unrelated
  modifications.

## Rollout

The implementation is a root agent guide plus a PR-triggered Windows smoke
check and the fork's required-status-check setting. The status-check setting is
configured in GitHub repository rules and must be verified after the workflow
has produced its first successful run.
