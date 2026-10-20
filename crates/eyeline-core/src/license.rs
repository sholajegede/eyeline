use base64::{engine::general_purpose::URL_SAFE_NO_PAD as B64, Engine};
use ed25519_dalek::{Signature, VerifyingKey};
use serde::{Deserialize, Serialize};

pub const CLOCK_SKEW_SECS: i64 = 300;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Claims {
    pub license_id: String,
    pub device_id: String,
    pub issued_at: i64,
    pub expires_at: i64,
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum LicenseError {
    #[error("license token is malformed")]
    Malformed,
    #[error("license public key is invalid")]
    BadPublicKey,
    #[error("license signature does not match")]
    BadSignature,
    #[error("license belongs to a different device")]
    WrongDevice,
    #[error("license token is not valid yet")]
    NotYetValid,
    #[error("license token has expired")]
    Expired,
}

pub fn verify_token(
    token: &str,
    public_key: &[u8; 32],
    device_id: &str,
    now: i64,
) -> Result<Claims, LicenseError> {
    let (payload_b64, sig_b64) = token.split_once('.').ok_or(LicenseError::Malformed)?;
    let payload = B64.decode(payload_b64).map_err(|_| LicenseError::Malformed)?;
    let sig_bytes = B64.decode(sig_b64).map_err(|_| LicenseError::Malformed)?;
    let signature = Signature::from_slice(&sig_bytes).map_err(|_| LicenseError::Malformed)?;
    let key = VerifyingKey::from_bytes(public_key).map_err(|_| LicenseError::BadPublicKey)?;

    key.verify_strict(&payload, &signature)
        .map_err(|_| LicenseError::BadSignature)?;

    let claims: Claims = serde_json::from_slice(&payload).map_err(|_| LicenseError::Malformed)?;
    if claims.device_id != device_id {
        return Err(LicenseError::WrongDevice);
    }
    if claims.issued_at > now + CLOCK_SKEW_SECS {
        return Err(LicenseError::NotYetValid);
    }
    if claims.expires_at <= now {
        return Err(LicenseError::Expired);
    }
    Ok(claims)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::{Signer, SigningKey};

    const NOW: i64 = 1_800_000_000;

    fn key() -> SigningKey {
        SigningKey::from_bytes(&[7u8; 32])
    }

    fn token(claims: &Claims, signer: &SigningKey) -> String {
        let payload = serde_json::to_vec(claims).unwrap();
        let sig = signer.sign(&payload);
        format!("{}.{}", B64.encode(&payload), B64.encode(sig.to_bytes()))
    }

    fn claims() -> Claims {
        Claims {
            license_id: "lic_1".into(),
            device_id: "dev_a".into(),
            issued_at: NOW - 10,
            expires_at: NOW + 86_400,
        }
    }

    fn public() -> [u8; 32] {
        key().verifying_key().to_bytes()
    }

    #[test]
    fn accepts_a_valid_token() {
        let t = token(&claims(), &key());
        assert_eq!(verify_token(&t, &public(), "dev_a", NOW), Ok(claims()));
    }

    #[test]
    fn rejects_a_token_signed_by_another_key() {
        let other = SigningKey::from_bytes(&[9u8; 32]);
        let t = token(&claims(), &other);
        assert_eq!(verify_token(&t, &public(), "dev_a", NOW), Err(LicenseError::BadSignature));
    }

    #[test]
    fn rejects_a_tampered_payload() {
        let t = token(&claims(), &key());
        let (_, sig) = t.split_once('.').unwrap();
        let mut forged = claims();
        forged.expires_at += 10_000_000;
        let forged_payload = B64.encode(serde_json::to_vec(&forged).unwrap());
        let t = format!("{forged_payload}.{sig}");
        assert_eq!(verify_token(&t, &public(), "dev_a", NOW), Err(LicenseError::BadSignature));
    }

    #[test]
    fn rejects_the_wrong_device() {
        let t = token(&claims(), &key());
        assert_eq!(verify_token(&t, &public(), "dev_b", NOW), Err(LicenseError::WrongDevice));
    }

    #[test]
    fn rejects_an_expired_token() {
        let t = token(&claims(), &key());
        assert_eq!(verify_token(&t, &public(), "dev_a", NOW + 86_400), Err(LicenseError::Expired));
    }

    #[test]
    fn rejects_a_token_from_the_future() {
        let mut c = claims();
        c.issued_at = NOW + 10_000;
        let t = token(&c, &key());
        assert_eq!(verify_token(&t, &public(), "dev_a", NOW), Err(LicenseError::NotYetValid));
    }

    #[test]
    fn tolerates_small_clock_skew() {
        let mut c = claims();
        c.issued_at = NOW + 60;
        let t = token(&c, &key());
        assert!(verify_token(&t, &public(), "dev_a", NOW).is_ok());
    }

    #[test]
    fn rejects_garbage() {
        assert_eq!(verify_token("nope", &public(), "dev_a", NOW), Err(LicenseError::Malformed));
        assert_eq!(verify_token("a.b", &public(), "dev_a", NOW), Err(LicenseError::Malformed));
    }
}
