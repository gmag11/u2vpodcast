## 1. Model layer: playlist cleanup helpers

- [x] 1.1 In `src/models/playlist.rs`, add a `PlaylistItem::purge_episode(conn, episode_id)` helper that deletes the `playlist_items` row for a single episode id (no-op if absent) and reindexes remaining positions, reusing the existing `reindex` helper; verify with a new unit test that removes a mid-list episode and asserts the remaining rows are contiguous
- [x] 1.2 In `src/models/playlist.rs`, add a `PlaylistItem::purge_for_channel(conn, channel_id)` helper that deletes all `playlist_items` rows whose `episode_id` belongs to the given channel (via a subquery on `episodes`) and reindexes remaining positions; verify with a new unit test seeding a channel with two playlisted episodes and asserting both playlist rows are gone and remaining positions are contiguous

## 2. Wire cleanup into episode and channel deletion

- [x] 2.1 Update `Episode::remove` in `src/models/episode.rs` to run inside a transaction: delete the episode row, then call `PlaylistItem::purge_episode`, then commit; verify with a unit test that deletes a playlisted episode and asserts its playlist entry is gone
- [x] 2.2 Update `Channel::delete` in `src/models/channel.rs` to call `PlaylistItem::purge_for_channel` inside its existing transaction before deleting the channel's episodes; verify with a unit test that deletes a channel with playlisted episodes and asserts their playlist entries are gone
- [x] 2.3 Verify deleting an episode that was never on the playlist still succeeds and leaves the playlist unaffected (unit test on both `Episode::remove` and `Channel::delete` paths)

## 3. Full verification

- [ ] 3.1 Run `cargo test` and confirm all existing and new tests pass
- [x] 3.2 Run `cargo clippy` and confirm no new warnings on the touched files
