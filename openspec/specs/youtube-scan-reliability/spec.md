# youtube-scan-reliability

## Purpose

Defines that YouTube scans are reliable: the video listing uses the same credential set as downloads, and channel metadata extraction tolerates the attribute-order and quoting variations of real og-protocol HTML.

## Requirements

### Requirement: The video listing uses the configured cookies like downloads do

When a cookies file is configured, `Ytdlp::get_latest` SHALL pass it to yt-dlp exactly as the download path does. The listing and the download SHALL see the same effective YouTube content access.

#### Scenario: Cookie-protected channel lists consistent with downloads
- **WHEN** a channel is scanned with a configured `cookies.txt`
- **THEN** the listing includes the same entries the download step is able to fetch (age-restricted/membership content is not silently omitted from the scan)

### Requirement: Channel metadata extraction matches real og-format variations

Channel title/description/image extraction SHALL succeed when the `property` and `content` attributes appear in either order and use either single or double quotes, including when other attributes sit between them. Absent or malformed metadata SHALL yield an empty string (never a panic) and never a wrong value.

#### Scenario: Single-quoted reversed-order meta is parsed
- **WHEN** the HTML contains `content='...' property="og:title"` (reversed, single-quoted)
- **THEN** the title is extracted correctly

#### Scenario: Extra attribute between property and content is tolerated
- **WHEN** the tag is `meta property="og:title" data-x="y" content="..."` (extra attribute in between)
- **THEN** the content value is still extracted

#### Scenario: Missing metadata yields empty string
- **WHEN** no matching og meta tag exists
- **THEN** the helper returns an empty string without error or panic