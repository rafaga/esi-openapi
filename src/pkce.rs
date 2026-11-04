use crate::errors::EsiResult;
use base64::engine::{general_purpose::URL_SAFE_NO_PAD as base64, Engine};
use rand::random;
use sha2::{Digest, Sha256};

/// PKCEVerifier is a base64urlencoded String to send in the body of the token request alongside the
/// code returned from the ESI and the Client ID
pub type PkceVerifier = String;
pub type PkceChallenge = String;

pub struct Pkce {
    pub challenge: PkceChallenge,
    pub verifier: PkceVerifier,
}

impl Pkce {
    /// The `S256` challenge of a verifier (RFC 7636, section 4.2):
    /// `BASE64URL(SHA256(ASCII(verifier)))`.
    fn challenge_of(verifier: &str) -> PkceChallenge {
        let hash = Sha256::digest(verifier.as_bytes());
        base64url(&hash.to_vec())
    }
}

pub fn generate() -> EsiResult<Pkce> {
    let verifier: Vec<u8> = (0..32).map(|_| random()).collect();

    let verifier_base64 = base64url(&verifier);
    let challenge = Pkce::challenge_of(&verifier_base64);
    Ok(Pkce {
        challenge,
        verifier: verifier_base64,
    })
}

pub fn base64url(verifier: &Vec<u8>) -> String {
    let mut verifier_out = vec![0; verifier.len() * 4 / 3 + 4];
    let written_size = base64.encode_slice(verifier, &mut verifier_out).unwrap();
    verifier_out.truncate(written_size);
    String::from_utf8(verifier_out).unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_challenge_of_the_rfc_7636_example_verifier_matches_the_rfc() {
        // RFC 7636, appendix B.
        assert_eq!(
            Pkce::challenge_of("dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk"),
            "E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM"
        );
    }

    #[test]
    fn a_generated_pair_is_a_verifier_and_its_challenge() {
        let pkce = generate().unwrap();

        assert_eq!(pkce.challenge, Pkce::challenge_of(&pkce.verifier));
        assert_ne!(pkce.challenge, pkce.verifier);
    }

    #[test]
    fn the_verifier_has_the_length_and_alphabet_of_rfc_7636() {
        // 32 random bytes encode to 43 characters without padding; the RFC
        // asks for 43 to 128 characters of [A-Za-z0-9-._~].
        for _ in 0..50 {
            let pkce = generate().unwrap();
            assert_eq!(pkce.verifier.len(), 43);
            assert_eq!(pkce.challenge.len(), 43);
            for text in [&pkce.verifier, &pkce.challenge] {
                assert!(
                    text.chars()
                        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_'),
                    "{text}"
                );
            }
        }
    }

    #[test]
    fn every_pair_is_different() {
        let first = generate().unwrap();
        let second = generate().unwrap();

        assert_ne!(first.verifier, second.verifier);
        assert_ne!(first.challenge, second.challenge);
    }

    #[test]
    fn base64url_uses_the_url_alphabet_without_padding() {
        // 0xfb 0xff encodes to "+/8=" in the standard alphabet.
        assert_eq!(base64url(&vec![0xfb, 0xff]), "-_8");
        assert_eq!(base64url(&Vec::new()), "");
        assert_eq!(base64url(&b"f".to_vec()), "Zg");
        assert_eq!(base64url(&b"fo".to_vec()), "Zm8");
        assert_eq!(base64url(&b"foo".to_vec()), "Zm9v");
    }

    #[test]
    fn base64url_round_trips_every_length_up_to_100() {
        use base64::engine::general_purpose::URL_SAFE_NO_PAD;
        for length in 0..=100u8 {
            let bytes: Vec<u8> = (0..length).map(|i| i.wrapping_mul(37)).collect();
            let text = base64url(&bytes);
            assert_eq!(URL_SAFE_NO_PAD.decode(&text).unwrap(), bytes, "{length}");
        }
    }
}
