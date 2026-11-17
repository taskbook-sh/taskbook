use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};

/// Hash a password using Argon2id.
pub fn hash_password(password: &str) -> Result<String, argon2::password_hash::Error> {
    let hash = Argon2::default().hash_password(password.as_bytes())?;
    Ok(hash.to_string())
}

/// Verify a password against a stored Argon2id hash.
pub fn verify_password(password: &str, hash: &str) -> Result<bool, argon2::password_hash::Error> {
    let parsed_hash = PasswordHash::new(hash)?;
    Ok(Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Hash of "correct horse" produced by argon2 0.5 with default parameters.
    const LEGACY_HASH: &str = "$argon2id$v=19$m=19456,t=2,p=1$MDEyMzQ1Njc4OWFiY2RlZg$rk2Mi3E4dgRMg0fHaYaptWVZRKqs//6b6k3/nntqGJk";

    #[test]
    fn hash_then_verify_roundtrip() {
        let hash = hash_password("s3cret").unwrap();

        assert!(hash.starts_with("$argon2id$v=19$m=19456,t=2,p=1$"));
        assert!(verify_password("s3cret", &hash).unwrap());
        assert!(!verify_password("wrong", &hash).unwrap());
    }

    #[test]
    fn hashes_are_salted() {
        assert_ne!(
            hash_password("s3cret").unwrap(),
            hash_password("s3cret").unwrap()
        );
    }

    #[test]
    fn verifies_hash_from_previous_argon2_version() {
        assert!(verify_password("correct horse", LEGACY_HASH).unwrap());
        assert!(!verify_password("wrong horse", LEGACY_HASH).unwrap());
    }

    #[test]
    fn malformed_hash_is_an_error() {
        assert!(verify_password("s3cret", "not-a-hash").is_err());
    }
}
