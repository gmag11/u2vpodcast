## Context

See `proposal.md` for motivation. The current SponsorBlock path requests only `sponsor`/`skip`, normalizes intervals without category metadata, and uses one hash both as the snapshot identity and the processed-file identity. The same interval list drives backend MP3 cuts, frontend seeks, and one-color progress markers. Snapshot data is JSON in `sponsorblock_cache`, which permits a compatible shape transition, but the hash roles must be separated so changes to visible playable segments do not regenerate identical audio.

Configuration is loaded once at startup and is available through application state. A default-off master switch must gate every SponsorBlock entry point. When enabled, synchronization and authenticated refresh both converge on the same reconciliation path, so category selection must enter there rather than being duplicated at each caller.

## Goals / Non-Goals

**Goals:**

- Apply one default-off master gate consistently to retrieval, reconciliation, media selection, API exposure, manual refresh, playback, and markers.
- Keep one canonical supported-category set shared by configuration validation and SponsorBlock requests.
- Preserve categorized source intervals for display while deriving a separate union of rejected intervals for cuts and seeks.
- Make legacy snapshots readable and preserve their active processed media through deployment.
- Keep processing deterministic when category order, duplicate configuration values, or irrelevant SponsorBlock metadata varies.

**Non-Goals:**

- Supporting SponsorBlock action types other than `skip`.
- Assigning a unique color to every non-sponsor category or encoding rejection status in marker color.
- Adding runtime configuration UI or reloading `config.yml` without restart.
- Deleting cached snapshots or derived files merely because SponsorBlock is disabled.
- Submitting, voting on, or authenticating to SponsorBlock.

## Decisions

### Use a default-off master switch plus one validated category list

Add `sponsorblock_enabled: bool` with a serde default of `false` and `sponsorblock_rejected_categories: Vec<String>` with a default of `['sponsor']`. The boolean is the authoritative gate; a non-empty category list never enables SponsorBlock implicitly. Normalize category duplicates while retaining deterministic supported-category ordering, and validate values after YAML deserialization so startup reports the offending identifier. When enabled, an explicit empty YAML list means SponsorBlock data is displayed but no category is rejected.

Defaulting the switch off makes external retrieval and content transformation opt-in, at the cost of requiring existing operators to add `sponsorblock_enabled: true` after upgrade. The alternative of one boolean per category would make adding supported categories noisy and would not map naturally to SponsorBlock's category identifiers. Silently ignoring unknown values was rejected because a typo could leave unwanted content unprocessed.

### Fetch all supported skip categories independently of rejection selection

Only when the master switch is enabled, build the SponsorBlock `categories` request from the complete supported set and continue requesting only the `skip` action. Filtering the request by configured rejected categories would make it impossible to display playable segments. The response filter remains defensive against unsupported categories and action types.

### Persist categorized source segments and derive rejection at the boundary

Extend each stored segment with `category`, but do not persist a mutable `rejected` boolean. API serialization derives `rejected` from the current validated configuration, ensuring a changed configuration is reflected consistently after restart without rewriting every snapshot. Legacy `{start,end}` JSON deserializes with category `sponsor`; its API rejection value therefore follows the new configuration rather than being permanently true.

Normalization keeps source categories separate and deterministically orders by start, end, then category. It does not merge descriptive segments, because merging would erase marker/category information. A second projection filters configured categories and unions overlapping or adjacent intervals across categories for media generation and player seeking.

The alternative of storing two segment arrays risks divergence. Persisting `rejected` was rejected because it duplicates deployment configuration and complicates configuration changes.

### Separate snapshot identity from processing identity

Define the snapshot hash over normalized `{start,end,category}` entries so any marker-visible change reaches clients. Define the processing hash over processing-format version, a canonical selected-category list, and merged rejected intervals. Processed filenames and regeneration decisions use only the processing hash; episode payloads use the snapshot hash and categorized segments.

Add a nullable `processing_hash` cache column and backfill it from the existing `snapshot_hash`, whose old meaning is equivalent to the legacy sponsor-only processing hash. On successful refresh, write both identities under their new meanings. This avoids regenerating audio when only a playable segment changes while preserving the old active filename until reconciliation.

Keeping one hash was considered, but hashing all visible segments would produce new derived files for unchanged cuts, while hashing only rejected intervals would prevent the current frontend's hash-based refresh detection from seeing playable marker changes.

### Pass category and rejection metadata through the episode contract

When enabled, episode payload segments become `{start, end, category, rejected}`. The database model owns categorized persisted data; the API-facing episode projection adds `rejected` using application configuration. List, playlist, and refresh responses must use the same projection. When disabled, API projections suppress snapshot data and the refresh endpoint returns a disabled response without reaching the client or reconciliation path. The frontend hides the refresh action, retains no active segment set, and therefore performs no skips or marker rendering.

This explicit boolean keeps policy in the backend and avoids duplicating configuration in the SPA. Sending only a rejected-category list to the client would make every consumer recreate policy matching.

### Render source markers independently from skip intervals

While enabled, timeline marker generation returns category-aware markers for every segment. Episode cards and the persistent player render `sponsor` with the existing `--sponsorblock` token and all other categories with a new theme-aware `--sponsorblock-other` token. Rejection status affects player seeks only, not marker visibility or color. While disabled, suppressed API data and the frontend gate ensure that no marker or automatic seek is produced.

Using rejection status as the second color was rejected because the requested visual distinction is sponsor versus other categories, and playable segments must remain visible.

### Reconcile configuration changes through existing sync and refresh paths

The switch is checked before automatic reconciliation and manual refresh. When disabled, neither path calls SponsorBlock or mutates cached state; channel and global feeds ignore any cached processed filename and select original media and duration. When enabled, both paths provide the validated selected categories to the common reconciliation operation. A changed processing hash regenerates or removes the derivative as appropriate. Until reconciliation, an existing feed derivative remains the last successfully published representation; this preserves the established failure-safe media contract. Web API projections can apply current rejection policy immediately from stored categorized data.

## Risks / Trade-offs

- [SponsorBlock adds a new category] → Keep an explicit supported set and require a code update before operators can configure or display it, preventing accidental undocumented behavior.
- [Existing deployments omit the new switch and unexpectedly stop processing] → Mark the default-off behavior as breaking and document `sponsorblock_enabled: true` as the upgrade action.
- [A configuration change temporarily disagrees with an old feed derivative] → Preserve last valid media and reconcile during the next sync or manual refresh; document that configuration is applied at startup and media updates on reconciliation.
- [Legacy snapshots lack non-sponsor data] → Interpret their intervals as `sponsor` and populate all categories on the next successful synchronization or refresh.
- [Overlapping markers obscure one another] → Preserve deterministic rendering order and test mixed-category overlaps; both remain represented even though one may visually overlay part of another.
- [Larger SponsorBlock responses and cache JSON] → Continue the existing bounded response size and request only supported `skip` entries.

## Migration Plan

1. Add the nullable processing-hash column and backfill it from the current snapshot hash in an additive migration.
2. Deploy code that can deserialize both legacy intervals and categorized intervals, defaults omitted categories to `sponsor`, and defaults the new master switch to disabled.
3. Operators who want to preserve existing SponsorBlock behavior add `sponsorblock_enabled: true` before restart. With the switch omitted or false, existing cache and processed files remain intact but APIs and feeds ignore them.
4. After enablement, normal synchronization or manual refresh rewrites snapshots with categories and the new snapshot identity, regenerating media only when the processing identity changes.
5. Rollback remains possible while newly stored segment objects tolerate legacy readers only if the application rollback precedes categorized writes; therefore database rollback must restore legacy `{start,end}` JSON or deployments should roll forward. The additive hash column itself is harmless to older binaries.