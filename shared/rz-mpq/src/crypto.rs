//! Table de chiffrement et fonctions de hachage MPQ (algorithmes Blizzard publics).

use std::sync::OnceLock;

pub const HASH_TABLE_OFFSET: u32 = 0;
pub const HASH_NAME_A: u32 = 1;
pub const HASH_NAME_B: u32 = 2;
pub const HASH_FILE_KEY: u32 = 3;

fn crypt_table() -> &'static [u32; 0x500] {
    static TABLE: OnceLock<[u32; 0x500]> = OnceLock::new();
    TABLE.get_or_init(|| {
        let mut table = [0u32; 0x500];
        let mut seed: u32 = 0x0010_0001;
        for index1 in 0..0x100usize {
            let mut index2 = index1;
            for _ in 0..5 {
                seed = (seed * 125 + 3) % 0x2A_AAAB;
                let temp1 = (seed & 0xFFFF) << 0x10;
                seed = (seed * 125 + 3) % 0x2A_AAAB;
                let temp2 = seed & 0xFFFF;
                table[index2] = temp1 | temp2;
                index2 += 0x100;
            }
        }
        table
    })
}

/// Normalise un chemin comme le fait Storm : séparateur `\` et majuscules ASCII.
pub fn normalize_path(name: &str) -> String {
    name.chars()
        .map(|c| if c == '/' { '\\' } else { c.to_ascii_uppercase() })
        .collect()
}

pub fn hash_string(name: &str, hash_type: u32) -> u32 {
    let table = crypt_table();
    let mut seed1: u32 = 0x7FED_7FED;
    let mut seed2: u32 = 0xEEEE_EEEE;
    for b in name.bytes() {
        let ch = match b {
            b'/' => b'\\',
            _ => b.to_ascii_uppercase(),
        } as u32;
        seed1 = table[(hash_type * 0x100 + ch) as usize] ^ seed1.wrapping_add(seed2);
        seed2 = ch
            .wrapping_add(seed1)
            .wrapping_add(seed2)
            .wrapping_add(seed2 << 5)
            .wrapping_add(3);
    }
    seed1
}

pub fn encrypt_block(data: &mut [u32], mut key: u32) {
    let table = crypt_table();
    let mut seed: u32 = 0xEEEE_EEEE;
    for v in data.iter_mut() {
        seed = seed.wrapping_add(table[0x400 + (key & 0xFF) as usize]);
        let plain = *v;
        *v = plain ^ key.wrapping_add(seed);
        key = ((!key) << 0x15).wrapping_add(0x1111_1111) | (key >> 0x0B);
        seed = plain
            .wrapping_add(seed)
            .wrapping_add(seed << 5)
            .wrapping_add(3);
    }
}

pub fn decrypt_block(data: &mut [u32], mut key: u32) {
    let table = crypt_table();
    let mut seed: u32 = 0xEEEE_EEEE;
    for v in data.iter_mut() {
        seed = seed.wrapping_add(table[0x400 + (key & 0xFF) as usize]);
        let plain = *v ^ key.wrapping_add(seed);
        *v = plain;
        key = ((!key) << 0x15).wrapping_add(0x1111_1111) | (key >> 0x0B);
        seed = plain
            .wrapping_add(seed)
            .wrapping_add(seed << 5)
            .wrapping_add(3);
    }
}

pub fn bytes_to_u32s(bytes: &[u8]) -> Vec<u32> {
    bytes
        .chunks_exact(4)
        .map(|c| u32::from_le_bytes([c[0], c[1], c[2], c[3]]))
        .collect()
}

pub fn u32s_to_bytes(words: &[u32]) -> Vec<u8> {
    words.iter().flat_map(|w| w.to_le_bytes()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_table_keys() {
        // Valeurs de référence connues de StormLib.
        assert_eq!(hash_string("(hash table)", HASH_FILE_KEY), 0xC3AF_3770);
        assert_eq!(hash_string("(block table)", HASH_FILE_KEY), 0xEC83_B3A3);
    }

    #[test]
    fn roundtrip() {
        let orig: Vec<u32> = (0..64).map(|i| i * 0x0101_0101).collect();
        let mut v = orig.clone();
        encrypt_block(&mut v, 0xDEAD_BEEF);
        assert_ne!(v, orig);
        decrypt_block(&mut v, 0xDEAD_BEEF);
        assert_eq!(v, orig);
    }
}
