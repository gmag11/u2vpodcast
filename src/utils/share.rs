use hmac::{Hmac, KeyInit, Mac};
use sha2::Sha256;

/// Domain-separation label mixed into every share hash, so the same secret can
/// never be confused across purposes.
const LABEL: &str = "u2vpodcast/share/v1";
/// A HMAC-SHA256 hash is 32 bytes, i.e. 64 lowercase hex characters.
const HASH_HEX_LEN: usize = 64;

type HmacSha256 = Hmac<Sha256>;

/// A parsed share token: the episode's public `yt_id` and the hash of its URI.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShareToken<'a> {
    pub yt_id: &'a str,
    pub hash: &'a str,
}

fn mac(secret: &str, webpage_url: &str) -> HmacSha256 {
    let mut mac =
        HmacSha256::new_from_slice(secret.as_bytes()).expect("HMAC accepts any key length");
    mac.update(LABEL.as_bytes());
    mac.update(b"|");
    mac.update(webpage_url.as_bytes());
    mac
}

fn encode_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn decode_hex(value: &str) -> Option<Vec<u8>> {
    if value.len() != HASH_HEX_LEN {
        return None;
    }
    (0..value.len())
        .step_by(2)
        .map(|index| u8::from_str_radix(&value[index..index + 2], 16).ok())
        .collect()
}

/// The permanent, deterministic share token for one episode: `{yt_id}-{hash}`,
/// where `{hash}` is the hex HMAC-SHA256 of the episode's canonical URI under
/// the effective share secret. The same inputs always produce the same token.
pub fn token_for(secret: &str, yt_id: &str, webpage_url: &str) -> String {
    let hash = mac(secret, webpage_url).finalize().into_bytes();
    format!("{yt_id}-{}", encode_hex(&hash))
}

/// Splits a token into its `yt_id` and hash, rejecting a missing/empty `yt_id`
/// and a hash that is not 64 hex characters. `yt_id` values may contain `-`:
/// the hash never does, so the last `-` is the separator.
pub fn parse_token(token: &str) -> Option<ShareToken<'_>> {
    let (yt_id, hash) = token.rsplit_once('-')?;
    if yt_id.is_empty() {
        return None;
    }
    decode_hex(hash)?;
    Some(ShareToken { yt_id, hash })
}

/// Verifies, in constant time, that `provided_hash` is the share hash of
/// `webpage_url` under `secret`.
pub fn verify_token(secret: &str, provided_hash: &str, webpage_url: &str) -> bool {
    let Some(provided) = decode_hex(provided_hash) else {
        return false;
    };
    mac(secret, webpage_url).verify_slice(&provided).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    const SECRET: &str = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
    const URI: &str = "https://www.youtube.com/watch?v=abc123";

    #[test]
    fn tokens_are_deterministic_and_url_safe() {
        let first = token_for(SECRET, "abc123", URI);
        let second = token_for(SECRET, "abc123", URI);
        assert_eq!(first, second);
        assert!(first.starts_with("abc123-"));
        assert_eq!(parse_token(&first).unwrap().hash.len(), 64);
    }

    #[test]
    fn a_valid_token_parses_and_verifies() {
        let token = token_for(SECRET, "abc123", URI);
        let parsed = parse_token(&token).expect("valid token must parse");
        assert_eq!(parsed.yt_id, "abc123");
        assert!(verify_token(SECRET, parsed.hash, URI));
    }

    #[test]
    fn a_tampered_hash_is_rejected() {
        let token = token_for(SECRET, "abc123", URI);
        let mut tampered = token.clone();
        let last = tampered.pop().unwrap();
        tampered.push(if last == '0' { '1' } else { '0' });
        let parsed = parse_token(&tampered).expect("still parseable");
        assert!(!verify_token(SECRET, parsed.hash, URI));
    }

    #[test]
    fn a_changed_uri_or_secret_is_rejected() {
        let token = token_for(SECRET, "abc123", URI);
        let parsed = parse_token(&token).unwrap();
        assert!(!verify_token(
            SECRET,
            parsed.hash,
            "https://www.youtube.com/watch?v=other"
        ));
        let other_secret = "f".repeat(64);
        assert!(!verify_token(&other_secret, parsed.hash, URI));
    }

    #[test]
    fn yt_ids_containing_a_dash_round_trip() {
        let token = token_for(SECRET, "a-b_c", URI);
        let parsed = parse_token(&token).unwrap();
        assert_eq!(parsed.yt_id, "a-b_c");
        assert!(verify_token(SECRET, parsed.hash, URI));
    }

    #[test]
    fn malformed_tokens_are_rejected() {
        assert!(parse_token("").is_none());
        assert!(parse_token("abc").is_none());
        assert!(parse_token("-0123456789").is_none());
        assert!(parse_token("abc-").is_none());
        assert!(parse_token(&format!("abc-{}", "0".repeat(63))).is_none());
        assert!(parse_token(&format!("abc-{}", "z".repeat(64))).is_none());
        assert!(!verify_token(SECRET, "not-hex", URI));
        assert!(!verify_token(SECRET, &"0".repeat(63), URI));
    }
}
