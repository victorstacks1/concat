# Content Desk export integration Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Export a Reel from Concat to a selected Content Desk Reel card, then queue it for the near-term slot or the next strategy slot.

**Architecture:** Content Desk exposes a key-authenticated `GET/POST /api/integrations/concat/reels` route. It lists only the API key owner's active `Gravado` Reels and reuses the existing R2 multipart upload, publication validation, strategy, and queue behavior. Concat stores the Site URL and API key in its preferences, renders to a temporary MP4 for this destination, uploads it in chunks, and submits the selected card and mode.

**Tech Stack:** Content Desk: TypeScript, Next/Vinext route handlers, existing Supabase REST store, Cloudflare R2, and Node's test runner. Concat: Rust, Slint, existing `ureq` dependency family, and Cargo unit tests.

**Spec:** `docs/superpowers/specs/2026-10-16-content-desk-export-integration-design.md` in the Concat repository.

Paths prefixed with `work/` are relative to the Content Desk checkout at `C:\Users\victor.borrajo_g4edu\Documents\Codex\2026-10-16\content-desk-sites-project-appgprj-6aa726b8c5908191bd721c76c77f4515\work`. Absolute paths are in the Concat checkout.

## Global Constraints

- Preserve the existing `POST /api/production/reels` card-creation contract.
- Require the existing `x-api-key` for every integration request; scope all reads and writes to its user.
- GET returns only card ID and title for active Reels whose status is exactly `Gravado`, with `Cache-Control: no-store`.
- Upload videos below 300 MB using the existing 32 MiB R2 multipart chunks and bind upload tokens to user and card.
- `now` uses the existing near-term slot, currently two minutes ahead; `strategy` uses the next free configured Reels slot.
- A successful final POST means queued for the Content Desk publication runner, not confirmed published by Buffer.
- Do not add a new Rust HTTP dependency if an existing dependency can serve the integration.
- Never log or return API keys or upload tokens.

## Review Focus

- A revoked, malformed, or missing API key must not reveal cards or accept upload actions. Test each auth failure returns the correct status without touching the data store.
- A card from another user, a deleted card, or a card no longer in `Gravado` must not be attached or approved. Test these states at final submission after upload.
- A failure during multipart upload must abort the upload and leave card status and media reference unchanged. Test abort and cleanup calls.
- Empty title fields must use the same display fallback as the Reels board without returning extra card fields. Test the fallback and response shape.
- A Content Desk response loss or error must not be reported by Concat as a queued success. Test HTTP error and invalid JSON handling in the Concat client.

---

### Task 1: Content Desk integration API

**Files:**
- Create: `work/app/api/integrations/concat/reels/route.ts`
- Create: `work/lib/concat-reels-integration.mjs`
- Create: `work/tests/concat-reels-integration.test.mjs`
- Reuse: `work/lib/reels-api-key-store.ts`, `work/lib/publishing/reels-media.mjs`, `work/lib/publishing/format-validation.mjs`, `work/lib/reels-media-production.ts`, `work/lib/production-workflow.mjs`, and the existing production REST patterns in `work/app/api/workspace/route.ts`.

**Interfaces:**
- `mapConcatReelsCards(rows)` returns `{ id, title }[]` for non-deleted `reels` rows in `Gravado`; title uses the board's existing title fallback.
- `buildConcatReelsApproval({ production, productions, mode, strategy, channels, now })` validates eligibility and publishing requirements and returns the approved card fields including `data_postar` and incremented `publication_attempt`, or throws a typed validation error.
- `GET` authenticates `x-api-key`, queries by key owner, and returns `{ cards }` with `Cache-Control: no-store`.
- `POST` accepts `start`, `part`, `abort`, and final `{ action: "publish", production_id, mode, uploadToken, parts }`; every action authenticates and scopes the operation to the key owner. The final publish action completes the multipart upload and atomically attaches the video and approves the card.

- [x] **Step 1: Write failing tests** in `work/tests/concat-reels-integration.test.mjs` for: mapping only active Reels in `Gravado`; ID/title-only response mapping; title fallback; rejecting other modes and ineligible cards; `now` date selection; strategy collision selecting the next slot; rejecting a card missing caption or configured channel.
- [x] **Step 2: Run the focused tests and verify RED.** Run `node --test tests/concat-reels-integration.test.mjs` from `work/`. Expected: module import or the named helper assertions fail because the helpers do not exist.
- [x] **Step 3: Implement the two pure helpers** in `work/lib/concat-reels-integration.mjs`, delegating strategy calculation and publishing validation to the existing modules.
- [x] **Step 4: Run the focused tests and verify GREEN.** Run `node --test tests/concat-reels-integration.test.mjs`. Expected: all helper tests pass.
- [x] **Step 5: Write handler tests** for key authentication, user-scoped GET results, cache headers, invalid key rejection, dispatch of `start`/`part`/`abort`/`publish`, and cleanup on upload failure. Export `createConcatReelsHandlers({ authenticate, listCards, startUpload, uploadPart, abortUpload, publish })` from `work/lib/concat-reels-integration.mjs`; route handlers call this tested factory with Site services.
- [x] **Step 6: Run the route tests and verify RED.** Run `node --test tests/concat-reels-integration.test.mjs`. Expected: route assertions fail before the route is implemented.
- [x] **Step 7: Implement GET and POST** in `work/app/api/integrations/concat/reels/route.ts`. Reuse the existing key store, R2 token/upload helpers, server-side Supabase access, publication validation, and approval date logic. The final publish action completes the R2 upload, validates the candidate, and conditionally updates only an active `Gravado` card; on validation or update failure delete the new object and preserve the original card state.
- [x] **Step 8: Run `node --test tests/concat-reels-integration.test.mjs` and verify GREEN.** Expected: auth, listing, upload failure, validation, and approval scheduling tests pass.
- [x] **Step 9: Run `npm exec tsc -- --noEmit` from `work/`.** Expected: no new route or type errors.

### Task 2: Concat preferences and Content Desk client

**Files:**
- Modify: `C:\Users\victor.borrajo_g4edu\Documents\Codex\2026-10-16\e\work\concat\src\crates\concat\src\prefs.rs`
- Modify: `C:\Users\victor.borrajo_g4edu\Documents\Codex\2026-10-16\e\work\concat\src\crates\concat\src\panes\settings.rs`
- Modify: `C:\Users\victor.borrajo_g4edu\Documents\Codex\2026-10-16\e\work\concat\src\crates\concat\ui\dialogs\settings.slint`
- Modify: `C:\Users\victor.borrajo_g4edu\Documents\Codex\2026-10-16\e\work\concat\src\crates\concat\src\lib.rs`
- Create: `C:\Users\victor.borrajo_g4edu\Documents\Codex\2026-10-16\e\work\concat\src\crates\concat\src\content_desk.rs`
- Modify: `C:\Users\victor.borrajo_g4edu\Documents\Codex\2026-10-16\e\work\concat\src\crates\concat\src\lib.rs` to register the module.

**Interfaces:**
- `Preferences` adds optional `content_desk_url` and `content_desk_api_key` fields with serde defaults, preserving old `settings.json` compatibility.
- `content_desk::list_cards(base_url, api_key) -> Result<Vec<ContentDeskCard>, ContentDeskError>` requests the GET endpoint and parses only card IDs and titles.
- `content_desk::upload_and_queue(base_url, api_key, production_id, mode, path, progress) -> Result<QueuedPublication, ContentDeskError>` performs the authenticated multipart actions and final publish POST. The implementation uses the existing HTTP client dependency and bounded reads.
- Settings exposes URL and masked key editing callbacks and persists edits through `Preferences::save`.

- [x] **Step 1: Write failing Rust tests** for old-preferences deserialization, preserving a saved API key, URL normalization, card response parsing, rejection of unsuccessful HTTP responses, and rejection of malformed JSON in `content_desk.rs` and `prefs.rs`. Add a `SettingsPane::data()` assertion that the key is represented by the password field without being included in status or log text.
- [ ] **Step 2: Run focused tests and verify RED.** Blocked: `cargo` and `rustc` are not installed or discoverable in this Windows environment.
- [x] **Step 3: Add preference fields and settings controls** with defaults that keep the Content Desk URL usable and the API key empty. Saving must retain all existing preference fields.
- [x] **Step 4: Implement and test the Content Desk HTTP client** in `content_desk.rs`. The implementation uses the existing installed HTTP dependency and sends chunks no larger than the server's 32 MiB chunk size. Rust tests remain unverified because the toolchain is unavailable.
- [ ] **Step 5: Run focused tests and verify GREEN.** Blocked: `cargo` is unavailable, so the Rust tests cannot run.

### Task 3: Content Desk export and queue flow in Concat

**Files:**
- Modify: `C:\Users\victor.borrajo_g4edu\Documents\Codex\2026-10-16\e\work\concat\src\crates\concat\src\panes\export.rs`
- Modify: `C:\Users\victor.borrajo_g4edu\Documents\Codex\2026-10-16\e\work\concat\src\crates\concat\ui\dialogs\export.slint`
- Modify: `C:\Users\victor.borrajo_g4edu\Documents\Codex\2026-10-16\e\work\concat\src\crates\concat\src\lib.rs`
- Modify: `C:\Users\victor.borrajo_g4edu\Documents\Codex\2026-10-16\e\work\concat\src\crates\concat\src\i18n.rs`
- Modify: `C:\Users\victor.borrajo_g4edu\Documents\Codex\2026-10-16\e\work\concat\src\crates\concat\locales\en.json`
- Modify: `C:\Users\victor.borrajo_g4edu\Documents\Codex\2026-10-16\e\work\concat\src\crates\concat\locales\pt-BR.json`

**Interfaces:**
- `ExportMsg` gains messages for opening the Content Desk chooser, loading cards, selecting a card/mode, and completing its render/upload request.
- `ExportPane` retains selected card ID/title, mode, request state, progress, and the temporary output path; `data()` exposes only display-safe state to Slint.
- The existing render worker is reused with a temporary MP4 destination for Content Desk. A successful render hands the file to `content_desk::upload_and_queue`; completion or cancellation removes the temporary file.

- [x] **Step 1: Write failing tests** for `ExportPane`'s mode selection, list loading and error state, disabled actions while a render/upload runs, and completion/error messages without logging secrets.
- [ ] **Step 2: Run focused tests and verify RED.** Blocked: `cargo` is unavailable in this environment.
- [x] **Step 3: Implement export-pane state and messages** in `panes/export.rs`, invoking the Content Desk client on worker threads and routing results back to the UI thread.
- [x] **Step 4: Add the chooser and progress states** to `dialogs/export.slint`, preserving the regular Export action and matching the existing dialog layout.
- [x] **Step 5: Wire callbacks and translations** through `lib.rs`, `i18n.rs`, and the locale files. Non-English locales use the complete English fallback strings; `pt-BR` has translated strings.
- [ ] **Step 6: Run focused tests and verify GREEN.** Blocked: `cargo` is unavailable, so these tests and Slint compilation cannot run.

### Task 4: Cross-project verification and Site release

**Files:**
- Modify as needed: the Task 1 and Task 2/3 files only.
- Use the existing Content Desk Site project and preserve its current public audience.

**Interfaces:**
- The deployed Site route and the Concat client use the same action names, chunk size, response fields, mode enum, and error shape.
- Queued publication responses explicitly distinguish scheduling from completed Buffer publication.

- [x] **Step 1: Add or complete contract tests** proving the Site's request/response shapes match the Concat client's parser and that a successful POST returns `queued` with `data_postar`.
- [x] **Step 2: Run Content Desk focused and full checks.** Focused tests pass 10/10 and `npm run build` succeeds; `npm exec tsc -- --noEmit` still reports pre-existing errors in unrelated files.
- [ ] **Step 3: Run Concat focused and workspace checks.** From the Concat `src/` directory run `cargo test -p concat panes::export::tests::content_desk`, `cargo test -p concat prefs::tests`, and `cargo fmt --check`. Expected: all pass.
- [x] **Step 4: Package and publish the Content Desk Site** using the existing Site workflow and project ID `appgprj_6aa726b8c5908191bd721c76c77f4515`; the Site stayed public and deployment version 212 succeeded at `https://content-desk.stacksbr.chatgpt.site`.
- [x] **Step 5: Report the local Concat build result and deployed Site URL**, along with any platform-specific validation limit. The Concat build and Rust tests could not run because this environment has no Cargo/Rust toolchain; the authenticated GitHub commit is on branch `feat/content-desk-export`.

---

## Execution Notes

- Execute inline in this session using `superpowers:executing-plans`, with TDD for each task. The user has selected this execution method.
- Do not add a runtime Rust dependency unless the existing HTTP client cannot support the request; if necessary, pause and revise the plan before changing dependency files.
- The Concat app is not part of the Content Desk Site deployment. Publishing the Site makes the API endpoint live; Concat must be built locally to exercise the full native flow.
- Keep the Content Desk API key out of test fixtures, logs, command arguments, and screenshots.
