# Fork Branch Policy and Trim Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Keep the fork close to upstream, remove fork-specific update routing and preview controls, and establish the `main`/`custom`/`feature/*` workflow.

**Architecture:** `origin/main` mirrors `upstream/main`; `origin/custom` holds approved personal changes; short-lived `feature/*` branches start at `custom` and target it through pull requests. The application updater remains enabled but checks upstream releases.

**Tech Stack:** Git, GitHub branches and pull requests, Rust, Slint, Bash.

**Spec:** User request in the 2026-10-18 Codex conversation.

## Global Constraints

- Do not publish or install a release as part of this change.
- Preserve the Content Desk integration and reusable asset library.
- Keep `origin/main` free of personal modifications.
- Do not touch the pre-existing untracked `stage/` directory.

## Review Focus

- Update checks must resolve releases and manifests from the upstream repository.
- Preview UI must contain neither the fullscreen button nor playback-rate controls.
- The `custom` branch must preserve existing custom work while `main` matches upstream.

---

### Task 1: Branch policy and updater source

**Files:**
- Modify: `AGENTS.md`
- Modify: `src/crates/concat-host/src/updates.rs`

**Interfaces:** The app updater continues to use the existing GitHub release and manifest mechanism, with upstream as its repository.

- [x] Add a unit test asserting releases and manifests point to `jub0t/Concat`; run it and observe failure.
- [x] Restore the updater's upstream repository URL and verify the test passes.
- [x] Replace the release-only AGENTS rule with main/custom/feature sync, PR, and release guidance.

### Task 2: Remove preview controls

**Files:**
- Modify: `src/crates/concat/src/dock.rs`
- Modify: `src/crates/concat/src/lib.rs`
- Modify: `src/crates/concat/src/studio.rs`
- Modify: `src/crates/concat/ui/editor.slint`
- Modify: `src/crates/concat/ui/icons.slint`
- Modify: `src/crates/concat/ui/workspace/preview-pane.slint`
- Modify: `src/crates/concat/ui/workspace/seat.slint`
- Create: `scripts/ci/test-fork-preview-controls-removed.sh`

**Interfaces:** Preview playback returns to upstream behavior; project timeline speed editing remains untouched.

- [x] Add a source-contract test for absence of preview-rate and fullscreen-toggle wiring; verify it fails.
- [x] Revert the preview-control commit and run the contract test.
- [x] Run relevant Rust checks and `git diff --check`.

### Task 3: Create the reviewable integration

- [x] Commit the feature changes on `feature/remove-fork-update-preview-controls`.
- [x] Push the feature branch and open a pull request targeting `custom`.
- [x] Leave the PR unmerged for review; do not publish a release.
