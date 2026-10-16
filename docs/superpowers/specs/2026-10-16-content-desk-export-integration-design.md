# Content Desk integration in Concat

## Status

Design and written specification approved in chat before implementation planning.

## Goal

Let a person export a finished Reel from Concat directly into a Reel card in Content Desk, then publish it now or schedule it using the account's existing posting strategy.

The integration spans two existing applications:

- Concat, a native Rust and Slint video editor at `C:\Users\victor.borrajo_g4edu\Documents\Codex\2026-10-16\e\work\concat`.
- Content Desk, the existing Sites application in this workspace, with its source checked out at `work/` and its production URL at `https://content-desk.stacksbr.chatgpt.site`.

## Current behavior and constraints

- Concat has a native export dialog and stores user preferences in its application `settings.json`.
- Content Desk stores Reels in the `producao` table. Reels in the `Gravado` status are part of its approval queue.
- Content Desk already manages per-user Reels API keys. Requests authenticate with `x-api-key`; each key is tied to one Content Desk user. The existing key is currently used to create cards only.
- Reels videos are stored in Cloudflare R2 through a multipart upload flow. The uploaded video reference is saved on the production card.
- Approval validates the card's publication fields and schedules it with `getApprovalPostDateTime`. “Postar agora” uses the next near-term time; “Postar estratégia” uses the next available slot in that user's strategy.
- The existing publication runner sends due posts to Buffer and records publication state.

## User flow

1. In Concat's export dialog, choose **Content Desk** as the destination alongside the normal file export action.
2. Concat loads the current user's Reels cards whose status is `Gravado` and displays each card's title. It retains the card ID for the later request.
3. Select a card and choose **Postar agora** or **Postar estratégia**.
4. Concat renders the video to a temporary local file, uploads it to Content Desk, and submits the selected card and mode.
5. Content Desk validates and attaches the video, approves the card, and records either the existing near-term “now” time or the next strategy slot. The existing publication runner sends due posts to Buffer.
6. Concat reports that the card is queued or shows a useful request-time error. A successful response includes the card title, chosen mode, and resulting scheduled time. Buffer's eventual publication result remains available in Content Desk's publication audit.

Normal file export remains available as-is. The Content Desk destination is available only when a Site URL and API key have been configured in Concat settings.

## API contract

Add a dedicated integration route at `GET/POST /api/integrations/concat/reels` so the existing `POST /api/production/reels` card-creation contract remains unchanged.

### Authentication

- Require the existing `x-api-key` header for both methods.
- Resolve the key to its associated user and scope all card reads, writes, uploads, scheduling, and publication to that user.
- Reject missing, invalid, revoked, and unconfigured keys with clear `401` or `503` responses. Do not return secrets in response bodies.
- Mark successful key use through the existing API key store.

### GET

Return only active (`deleted_at` is null) Reels cards owned by the key's user whose status is exactly `Gravado`. The public response is deliberately small:

```json
{
  "cards": [
    { "id": "opaque-production-id", "title": "Example title" }
  ]
}
```

Use the card's `titulo`, falling back to the same title display fallback used in the Reels board when it is empty. Send `Cache-Control: no-store`. Never expose card content, user IDs, credentials, or media references from this listing.

### POST

Use the same route for the multipart upload lifecycle and the final publish request, with an explicit action in the request body or header. Reuse the existing R2 multipart upload helpers and limits rather than buffering a large video in a single request. Every upload action must authenticate the API key and bind its upload token to the same user and selected production ID.

The final request identifies the selected card and mode:

```json
{
  "action": "publish",
  "production_id": "opaque-production-id",
  "mode": "now"
}
```

`mode` accepts only `now` or `strategy`. The operation must reject a card that is not an active Reel in `Gravado`, lacks required publication fields, lacks a usable video, or is already approved or posted. It must also reject a card owned by another user.

On success, attach the completed R2 video reference, apply the existing publishing validation and approval fields, and compute `data_postar` with `getApprovalPostDateTime`. `now` uses the existing near-term slot (currently two minutes ahead); `strategy` uses the next free slot. In both cases, the existing due-publication runner sends the post to Buffer and records its state. Return the resulting card ID, title, mode, `data_postar`, and queued publication state. A failure must not leave an orphaned R2 object or a card partially approved.

The implementation should reuse existing storage, validation, strategy, and publication modules wherever their server-side contracts permit. The written plan will identify any small shared helper needed to avoid duplicating the UI's approval rules.

## Concat changes

- Add Content Desk URL/key fields to Settings, with the service URL defaulting to the current Content Desk origin and the key masked in the UI.
- Persist the URL/key using Concat's existing application preference mechanism.
- Add a Content Desk action to the export dialog. Fetch and display the available card titles, allow selecting one and one of the two modes, show upload/render progress, and expose errors in the dialog.
- Render to a temporary MP4 for the integration path, then upload in bounded chunks and call the final POST. Remove the temporary file after completion or cancellation.
- Keep the current regular export path and output destination behavior intact.
- Add the new visible strings to the locale inventory and Brazilian Portuguese locale; provide English fallback strings consistent with the existing localization system.

## Error handling and integrity

- All integration requests use HTTPS and `Cache-Control: no-store` for key-bearing or card-list responses.
- Never log the API key or upload token.
- A missing Content Desk configuration opens or points to the corresponding settings page instead of silently failing.
- If the card changes status between GET and POST, reject publication and ask the user to refresh the list.
- If upload fails, abort the multipart upload and preserve the card's original status. If publication fails after upload, remove the newly uploaded asset and leave the card unapproved.
- Treat a successful POST as approval and queueing, not confirmation that Buffer has published the Reel. Report request-time failures in Concat; report later Buffer outcomes through Content Desk's existing publication audit.

## Out of scope

- Creating or editing Content Desk cards in this flow.
- Publishing formats other than Reels.
- Changing Buffer account or channel configuration.
- Uploading the video to any service other than Content Desk's existing R2 storage.
- Changing ordinary Concat export behavior.

## Acceptance criteria

1. A configured Concat instance lists only its user's active Reels cards in `Gravado`, showing titles and retaining IDs internally.
2. A card that changes owner, format, status, or deletion state after the list is loaded cannot be published.
3. A completed MP4 can be attached to the selected card through authenticated multipart upload without loading the entire file into one request.
4. “Postar agora” follows Content Desk's existing validation and near-term scheduling behavior; the normal due-publication runner handles Buffer publication.
5. “Postar estratégia” schedules against the user's configured Reels slots and leaves publication to the normal due runner.
6. Invalid credentials, incomplete card data, upload errors, and request-time publishing validation errors are shown without exposing API secrets or leaving a partially approved card. Later Buffer outcomes remain visible in Content Desk's publication audit.
7. Ordinary Concat export still works and the feature's visible strings are localized.

