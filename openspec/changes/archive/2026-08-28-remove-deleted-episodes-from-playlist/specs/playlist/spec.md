## ADDED Requirements

### Requirement: Deleting an episode removes it from the playlist

When an episode is deleted — whether individually (e.g. the channel retention-limit prune) or as part of deleting its whole channel — any playlist entry referencing that episode SHALL be deleted in the same operation, and the remaining playlist positions SHALL be reindexed contiguously. This SHALL happen regardless of whether the episode was ever added to the playlist.

#### Scenario: Retention-limit prune removes the playlist entry
- **WHEN** the retention-limit worker deletes an episode that is currently in the playlist
- **THEN** the episode's playlist entry is deleted and the remaining playlist positions are reindexed contiguously

#### Scenario: Deleting a channel removes its episodes' playlist entries
- **WHEN** a channel with two of its episodes on the playlist is deleted
- **THEN** both playlist entries are deleted along with the episodes, and the remaining playlist positions are reindexed contiguously

#### Scenario: Deleting an episode not on the playlist is a no-op for the playlist
- **WHEN** an episode that was never added to the playlist is deleted
- **THEN** the playlist is unaffected
