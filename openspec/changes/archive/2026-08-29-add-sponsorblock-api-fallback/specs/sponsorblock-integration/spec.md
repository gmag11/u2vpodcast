## MODIFIED Requirements

### Requirement: Sponsor segments are retrieved by YouTube video id
When SponsorBlock is enabled, the system SHALL query the official SponsorBlock API by an episode's `yt_id` and SHALL request `skip` segments for every supported category, independently of which categories are configured for rejection. The system SHALL query `sponsor.ajay.app` first and SHALL retry through `api.sponsor.ajay.app` when the primary endpoint returns an error or an unusable response. A successful response with no matching segments, including SponsorBlock's no-segments HTTP 404 response, SHALL be treated as an authoritative empty snapshot rather than an error and SHALL NOT trigger the fallback. Entries with unsupported categories or action types other than `skip` SHALL be excluded. When SponsorBlock is disabled, the system SHALL NOT issue SponsorBlock requests.

#### Scenario: Sponsor segments are available
- **WHEN** SponsorBlock returns `skip` segments in supported categories for an episode's `yt_id`
- **THEN** the system accepts every supported segment for normalization regardless of its rejection configuration

#### Scenario: Primary endpoint is unavailable
- **WHEN** retrieval from `sponsor.ajay.app` fails and `api.sponsor.ajay.app` returns a usable response
- **THEN** the system uses the fallback response as the SponsorBlock snapshot

#### Scenario: Both endpoints fail
- **WHEN** neither `sponsor.ajay.app` nor `api.sponsor.ajay.app` returns a usable response
- **THEN** SponsorBlock retrieval fails and the existing snapshot-preservation and retry behavior applies

#### Scenario: Video has no sponsor segments
- **WHEN** either queried SponsorBlock endpoint reports that no matching supported `skip` segments exist for an episode's `yt_id`
- **THEN** the system stores a successful empty snapshot, selects the original MP3 for serving, and does not query any later fallback endpoint

#### Scenario: Other categories or action types are returned
- **WHEN** a SponsorBlock response contains an unsupported category or an action type other than `skip`
- **THEN** the system excludes that entry from the persisted active segment set

#### Scenario: Retrieval is disabled
- **WHEN** channel synchronization or an episode operation runs while SponsorBlock is disabled
- **THEN** the system makes no request to the SponsorBlock service
