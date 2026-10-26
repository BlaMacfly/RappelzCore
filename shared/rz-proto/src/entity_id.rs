//! Identifiants brouillés de TS_SC_ENTER (`docs/re/network_protocol.md` §8).
//!
//! Le code d'un PNJ/monstre/objet occupe 8 octets : quatre u16 `w0..w3`, brouillés avec
//! deux aléas `r1`, `r2` (encodage FUN_0064a330, décodage côté client) :
//! `lo = w3 + 2(w0 + w2)`, `hi = w1 + 2(w0 − w2)` (mod 2¹⁶), `id = lo | hi << 16`.
//!
//! Pour un monstre, l'id attendu est la valeur brute de `db_monster.rdb` (donc
//! bit-permutée : voir `rz_rappelz::rdb::monster_id_encode`).

pub const ENCODED_LEN: usize = 8;

pub fn encode_with(id: u32, r1: u16, r2: u16) -> [u8; ENCODED_LEN] {
    let lo = id as u16;
    let hi = (id >> 16) as u16;
    let w0 = r1;
    let w1 = hi.wrapping_sub(r1.wrapping_sub(r2).wrapping_mul(2));
    let w2 = r2;
    let w3 = lo.wrapping_sub(r1.wrapping_add(r2).wrapping_mul(2));
    let mut out = [0u8; ENCODED_LEN];
    for (i, w) in [w0, w1, w2, w3].into_iter().enumerate() {
        out[i * 2..i * 2 + 2].copy_from_slice(&w.to_le_bytes());
    }
    out
}

/// Encodage avec des aléas, comme le serveur officiel.
pub fn encode(id: u32) -> [u8; ENCODED_LEN] {
    encode_with(id, rand::random(), rand::random())
}

pub fn decode(b: &[u8; ENCODED_LEN]) -> u32 {
    let w = |i: usize| u16::from_le_bytes([b[i * 2], b[i * 2 + 1]]);
    let (w0, w1, w2, w3) = (w(0), w(1), w(2), w(3));
    let lo = w3.wrapping_add(w0.wrapping_add(w2).wrapping_mul(2));
    let hi = w1.wrapping_add(w0.wrapping_sub(w2).wrapping_mul(2));
    lo as u32 | (hi as u32) << 16
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vectors() {
        assert_eq!(encode_with(100_001, 0, 0), [0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0xa1, 0x86]);
        assert_eq!(encode_with(100_001, 0x1234, 0x5678), [0x34, 0x12, 0x89, 0x88, 0x78, 0x56, 0x49, 0xb5]);
        for id in [0u32, 1, 100_001, 0xFFFF_FFFF, 0x8000_1234] {
            assert_eq!(decode(&encode(id)), id);
        }
    }
}
