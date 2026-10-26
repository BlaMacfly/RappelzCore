//! Vérificateur SRP6 identique à AzerothCore (`src/common/Cryptography/Authentication/SRP6.cpp`).
//!
//! La colonne `account.verifier` vaut g^x mod N, avec
//! x = SHA1(salt ‖ SHA1(MAJ(utilisateur) ":" MAJ(mot_de_passe))), entiers en petit-boutiste.
//! Les comptes créés par AzerothCore et par RappelzCore sont donc interchangeables.
//! Le client Rappelz transmet le mot de passe (et non une preuve SRP) : le serveur recalcule
//! le vérificateur avec le sel stocké et compare.

use num_bigint::BigUint;
use rand::RngCore;
use sha1::{Digest, Sha1};

const N_HEX: &str = "894B645E89E1535BBDAD5B8B290650530801B18EBFBF5E8FAB3C82872A3E9BB7";
const G: u32 = 7;

pub const SALT_LEN: usize = 32;
pub const VERIFIER_LEN: usize = 32;

/// Mise en majuscules comme `Utf8ToUpperOnlyLatin` d'AzerothCore (latin uniquement).
pub fn normalize(s: &str) -> String {
    s.chars()
        .map(|c| if c.is_alphabetic() && (c as u32) < 0x250 { c.to_uppercase().next().unwrap_or(c) } else { c })
        .collect()
}

pub fn calculate_verifier(username: &str, password: &str, salt: &[u8; SALT_LEN]) -> [u8; VERIFIER_LEN] {
    let inner = Sha1::new()
        .chain_update(normalize(username).as_bytes())
        .chain_update(b":")
        .chain_update(normalize(password).as_bytes())
        .finalize();
    let x_digest = Sha1::new().chain_update(salt).chain_update(inner).finalize();
    let x = BigUint::from_bytes_le(&x_digest);
    let n = BigUint::parse_bytes(N_HEX.as_bytes(), 16).expect("N SRP6");
    let v = BigUint::from(G).modpow(&x, &n);
    let mut out = [0u8; VERIFIER_LEN];
    let bytes = v.to_bytes_le();
    out[..bytes.len()].copy_from_slice(&bytes);
    out
}

/// Nouveau couple (sel, vérificateur) pour la création d'un compte.
pub fn make_registration(username: &str, password: &str) -> ([u8; SALT_LEN], [u8; VERIFIER_LEN]) {
    let mut salt = [0u8; SALT_LEN];
    rand::thread_rng().fill_bytes(&mut salt);
    let verifier = calculate_verifier(username, password, &salt);
    (salt, verifier)
}

pub fn check_password(username: &str, password: &str, salt: &[u8], verifier: &[u8]) -> bool {
    let Ok(salt) = <[u8; SALT_LEN]>::try_from(salt) else {
        return false;
    };
    let computed = calculate_verifier(username, password, &salt);
    // Comparaison en temps constant.
    computed.len() == verifier.len()
        && computed.iter().zip(verifier).fold(0u8, |acc, (a, b)| acc | (a ^ b)) == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn verifier_roundtrip() {
        let (salt, v) = make_registration("Test", "secret");
        assert!(check_password("TEST", "SECRET", &salt, &v));
        assert!(check_password("test", "secret", &salt, &v));
        assert!(!check_password("test", "autre", &salt, &v));
    }

    #[test]
    fn deterministic() {
        let salt = [0x11u8; SALT_LEN];
        assert_eq!(calculate_verifier("a", "b", &salt), calculate_verifier("A", "B", &salt));
    }
}
