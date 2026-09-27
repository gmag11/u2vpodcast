use chrono::{DateTime, Duration, Utc};
use hmac::{Hmac, KeyInit, Mac};
use sha2::Sha256;

/// Domain-separation label. The share signing key is derived from the session
/// `secret_key`, so no additional secret has to be provisioned or rotated; the
/// label keeps share signatures from being confusable with any other keyed use
/// of that secret.
const SIGNING_LABEL: &[u8] = b"u2vpodcast/share/v1";
/// Payload version prefix, so the token format can evolve without ambiguity.
const PAYLOAD_VERSION: &str = "v1";
/// A HMAC-SHA256 signature is 32 bytes, i.e. 64 lowercase hex characters.
const SIGNATURE_HEX_LEN: usize = 64;

type HmacSha256 = Hmac<Sha256>;

/// The episode and expiry a token resolves to once its signature and lifetime
/// have been verified.
#[derive(Debug, Clone, PartialEq)]
pub struct VerifiedShare {
    pub yt_id: String,
    pub expires_at: DateTime<Utc>,
}

fn signing_key(secret_key: &str) -> Vec<u8> {
    let mut mac =
        HmacSha256::new_from_slice(secret_key.as_bytes()).expect("HMAC accepts any key length");
    mac.update(SIGNING_LABEL);
    mac.finalize().into_bytes().to_vec()
}

fn payload(yt_id: &str, exp: i64) -> String {
    format!("{PAYLOAD_VERSION}|{yt_id}|{exp}")
}

fn sign_hex(key: &[u8], payload: &str) -> String {
    let mut mac = HmacSha256::new_from_slice(key).expect("HMAC accepts any key length");
    mac.update(payload.as_bytes());
    mac.finalize()
        .into_bytes()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn decode_hex(value: &str) -> Option<Vec<u8>> {
    if value.len() != SIGNATURE_HEX_LEN {
        return None;
    }
    (0..value.len())
        .step_by(2)
        .map(|index| u8::from_str_radix(&value[index..index + 2], 16).ok())
        .collect()
}

/// Mints a share token for one episode and returns it with its expiry instant.
/// The token loads as `{exp}-{yt_id}-{signature}`; `yt_id` characters are
/// URL-safe and the signature is lowercase hex, so the token needs no encoding.
pub fn mint_token(secret_key: &str, yt_id: &str, ttl_days: u64) -> (String, DateTime<Utc>) {
    let expires_at = Utc::now() + Duration::days(ttl_days as i64);
    (
        mint_token_with_exp(secret_key, yt_id, expires_at.timestamp()),
        expires_at,
    )
}

/// Mints a token for an explicit expiry instant. Kept crate-visible so tests
/// can exercise expired-token paths deterministically.
pub(crate) fn mint_token_with_exp(secret_key: &str, yt_id: &str, exp: i64) -> String {
    let signature = sign_hex(&signing_key(secret_key), &payload(yt_id, exp));
    format!("{exp}-{yt_id}-{signature}")
}

/// Verifies a token's signature (in constant time) and expiry. Returns the
/// bound episode and expiry, or `None` for a malformed, forged, or expired
/// token. Rotating the signing secret makes every previously minted token fail
/// here, because the recomputed signature no longer matches.
pub fn verify_token(secret_key: &str, token: &str, now: i64) -> Option<VerifiedShare> {
    let (exp_str, rest) = token.split_once('-')?;
    let (yt_id, signature) = rest.rsplit_once('-')?;
    if yt_id.is_empty() {
        return None;
    }
    let exp = exp_str.parse::<i64>().ok()?;
    let expected = decode_hex(signature)?;
    let mut mac = HmacSha256::new_from_slice(&signing_key(secret_key)).ok()?;
    mac.update(payload(yt_id, exp).as_bytes());
    mac.verify_slice(&expected).ok()?;
    if now > exp {
        return None;
    }
    let expires_at = DateTime::<Utc>::from_timestamp(exp, 0)?;
    Some(VerifiedShare {
        yt_id: yt_id.to_string(),
        expires_at,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const SECRET: &str = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";

    #[test]
    fn minted_token_verifies_and_reports_the_episode_and_expiry() {
        let (token, expires_at) = mint_token(SECRET, "abc123", 30);
        let now = Utc::now().timestamp();
        let verified = verify_token(SECRET, &token, now).expect("valid token must verify");
        assert_eq!(verified.yt_id, "abc123");
        // The token only carries second precision, so compare timestamps.
        assert_eq!(
            verified.expires_at.timestamp(),
            expires_at.timestamp()
        );
    }

    #[test]
    fn tampered_signature_is_rejected() {
        let (token, _) = mint_token(SECRET, "abc123", 30);
        let mut tampered = token.clone();
        let last = tampered.pop().unwrap();
        tampered.push(if last == '0' { '1' } else { '0' });
        assert!(verify_token(SECRET, &tampered, Utc::now().timestamp()).is_none());
    }

    #[test]
    fn changing_the_bound_episode_is_rejected() {
        let (token, _) = mint_token(SECRET, "abc123", 30);
        // The signature only covers the payload, so swapping the episode in the
        // middle of the token must invalidate it.
        let swapped = token.replace("abc123", "zzz999");
        assert!(verify_token(SECRET, &swapped, Utc::now().timestamp()).is_none());
    }

    #[test]
    fn a_different_secret_is_rejected() {
        let (token, _) = mint_token(SECRET, "abc123", 30);
        let other = "f".repeat(64);
        assert!(verify_token(&other, &token, Utc::now().timestamp()).is_none());
    }

    #[test]
    fn expired_token_is_rejected() {
        let (token, expires_at) = mint_token(SECRET, "abc123", 1);
        let after = (expires_at + Duration::seconds(1)).timestamp();
        assert!(verify_token(SECRET, &token, after).is_none());
        // A boundary value is still valid because validity is `now <= exp`.
        assert!(verify_token(SECRET, &token, expires_at.timestamp()).is_some());
    }

    #[test]
    fn malformed_tokens_are_rejected() {
        assert!(verify_token(SECRET, "", Utc::now().timestamp()).is_none());
        assert!(verify_token(SECRET, "not-a-token", Utc::now().timestamp()).is_none());
        assert!(verify_token(SECRET, "123-only-two-parts", Utc::now().timestamp()).is_none());
        assert!(verify_token(SECRET, "-abc-00", Utc::now().timestamp()).is_none());
        // Future expiry, valid length, but not a valid signature.
        assert!(verify_token(SECRET, "9999999999-abc123-00", Utc::now().timestamp()).is_none());
    }
}
