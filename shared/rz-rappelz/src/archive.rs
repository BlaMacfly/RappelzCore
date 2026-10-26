//! Archive de ressources `data.000` (index) + `data.001`..`data.008` (contenu).
//!
//! Reconstruit depuis SFrame.exe (détails et adresses : `docs/re/data_archive.md`) :
//! - `data.000` entièrement XORé par [`XOR_KEY`] avec un index roulant sur tout le fichier ;
//!   suite d'entrées `u8 longueur | nom encodé | u32 offset | u32 taille` (FUN_005df680) ;
//! - nom encodé par FUN_005cf720 (substitution répétée + permutations + 2 caractères de contrôle) ;
//! - fichier de contenu choisi par un hachage du nom encodé (FUN_005e3620) ;
//! - contenu XORé par la même clé (index remis à 0 par fichier), sauf extensions en clair.

use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};

#[derive(Debug, thiserror::Error)]
pub enum ArchiveError {
    #[error("E/S : {0}")]
    Io(#[from] std::io::Error),
    #[error("index data.000 tronqué à l'offset {0:#x}")]
    Truncated(usize),
    #[error("fichier absent de l'archive : {0}")]
    NotFound(String),
}

pub type Result<T> = std::result::Result<T, ArchiveError>;

pub const DATA_FILE_COUNT: u32 = 8;

/// Clé XOR de 256 octets (SFrame.exe, 0x00dfb1a8).
pub const XOR_KEY: [u8; 256] = hex256(concat!(
    "77e85eecb74ec1874fe6f53c1fb315436a4930a6bf53a8355be59e0e41ec22b8",
    "d480a48cce65131d4b085a6abb6fad25b8ddcc773074ac8c5a4a9a9b36bc530a",
    "3cf8960b5daa28a9b282136ef1c193a99e5f20cfd4cc5b2e16f5c94cb21c57ee",
    "14edf97297221b4aa42eb896ef4b3f8eab605d7f2cb8ad43ad768f5f92e64ea7",
    "d447196b6934b50e626da452b9e3e064433de370f590b3a20642029829503ffd",
    "975868018c1e0fef8bb341449621a8da5e8b4a531bfdf5213ff7ba6847f965df",
    "52cee0deecefcd77a20ebc382f64128df05ce00b59d62d99cde70115e067f432",
    "35d41121c3de9865ed549d1cb9b0aaa90c8ab46660e1ff2ec80043a96737db9c",
));

/// Table de substitution des noms (0x00dfb3a0).
const NAME_ENC: [u8; 128] = hex128(concat!(
    "0000000000000000000000000000000000000000000000000000000000000000",
    "67200026772c6c4e584f00372e256500385f5d2350312d24565b0059005e0000",
    "4b7d6a3040475329417879363945467b57623d5276746832344d286b006d612b",
    "7e442743214a49644255607166704851334c6e6f5a697273753b7a6300543500",
));

/// Table inverse (0x00dfb320).
const NAME_DEC: [u8; 128] = hex128(concat!(
    "0000000000000000000000000000000000000000000000000000000000000000",
    "21640033372d23625a47005f25362c0043355770587e4b2b304c007900520000",
    "44486863614d4e456e66654071592729346f53467d693850283b743900323d31",
    "6a5e517b672e6c205675425b265d72736d6b767755785424494a7a4f00416000",
));

/// Alphabet des caractères de contrôle (0x00eaa4c0).
const CHARSET: &[u8; 84] =
    b"^&T_Nsd{xo5v`rOYV+,iIU#kCJq8$'~L0P]FeBn-Au(pXHZhwDy2}agWG7K=bQ;SRt)46l@jE%9!c1[3fmMz";

/// Extensions stockées en clair (table 0x00dfccb4, comparaison sur les 4 derniers caractères).
const PLAIN_EXT: [&str; 9] = [".dds", ".tga", ".naf", ".nx3", ".cob", ".itc", ".itm", ".itb", ".nfm"];

const fn hex_nibble(c: u8) -> u8 {
    match c {
        b'0'..=b'9' => c - b'0',
        b'a'..=b'f' => c - b'a' + 10,
        _ => panic!("hex invalide"),
    }
}

const fn hex256(s: &str) -> [u8; 256] {
    let b = s.as_bytes();
    let mut out = [0u8; 256];
    let mut i = 0;
    while i < 256 {
        out[i] = hex_nibble(b[2 * i]) << 4 | hex_nibble(b[2 * i + 1]);
        i += 1;
    }
    out
}

const fn hex128(s: &str) -> [u8; 128] {
    let b = s.as_bytes();
    let mut out = [0u8; 128];
    let mut i = 0;
    while i < 128 {
        out[i] = hex_nibble(b[2 * i]) << 4 | hex_nibble(b[2 * i + 1]);
        i += 1;
    }
    out
}

pub fn xor_in_place(buf: &mut [u8], start: usize) {
    for (i, b) in buf.iter_mut().enumerate() {
        *b ^= XOR_KEY[(start + i) & 0xFF];
    }
}

fn swap_indices(len: usize) -> (usize, usize) {
    // Le client calcule (int)((float)len * 0.66f) et (int)((float)len * 0.33f).
    ((len as f32 * 0.66f32) as usize, (len as f32 * 0.33f32) as usize)
}

fn next_seed(plain: u8, seed: u32) -> u32 {
    let n = (plain as u32 * 17 + 1 + seed) % 32;
    if n == 0 {
        32
    } else {
        n
    }
}

/// Encode un nom de ressource comme le client (FUN_005cf720).
pub fn encode_name(plain: &str) -> Vec<u8> {
    let mut s: Vec<u8> = plain.bytes().map(|b| b.to_ascii_lowercase()).collect();
    let len = s.len();
    if len == 0 {
        return Vec::new();
    }
    let acc: u32 = s.iter().map(|&c| c as u32 * 17 + 1).sum();
    let mut seed = (acc + len as u32) % 32;
    if seed == 0 {
        seed = 32;
    }
    let seed0 = seed;
    for c in s.iter_mut() {
        let orig = *c;
        let mut v = orig;
        for _ in 0..seed {
            v = NAME_ENC[(v & 0x7F) as usize];
        }
        *c = v;
        seed = next_seed(orig, seed);
    }
    let sum: u32 = s.iter().map(|&c| c as u32).sum();
    let prefix = CHARSET[sum as usize % CHARSET.len()];
    let (i0, i1) = swap_indices(len);
    s.swap(0, i0);
    s.swap(1.min(len - 1), i1);
    let mut out = Vec::with_capacity(len + 2);
    out.push(prefix);
    out.extend(s);
    out.push(CHARSET[seed0 as usize]);
    out
}

/// Décode un nom de l'index ; `None` si le caractère de contrôle est invalide.
pub fn decode_name(enc: &[u8]) -> Option<String> {
    if enc.len() < 3 {
        return None;
    }
    let len = enc.len() - 2;
    let mut s = enc[1..1 + len].to_vec();
    let mut seed = CHARSET.iter().position(|&c| c == enc[enc.len() - 1])? as u32;
    if !(1..=32).contains(&seed) {
        return None;
    }
    let (i0, i1) = swap_indices(len);
    s.swap(1.min(len - 1), i1);
    s.swap(0, i0);
    for c in s.iter_mut() {
        let mut v = *c;
        for _ in 0..seed {
            v = NAME_DEC[(v & 0x7F) as usize];
        }
        *c = v;
        seed = next_seed(v, seed);
    }
    // Les noms sont en Latin-1 / Windows-1252.
    Some(s.into_iter().map(|b| b as char).collect())
}

/// Numéro du fichier de contenu (1..=8) pour un nom encodé (FUN_005e3620).
pub fn data_file_index(enc: &[u8]) -> u32 {
    let mut h: i32 = 0;
    for &b in enc {
        let mut c = b as i8 as i32;
        if (0x41..=0x5A).contains(&c) {
            c += 0x20;
        }
        h = h.wrapping_mul(31).wrapping_add(c);
    }
    (h.unsigned_abs() % DATA_FILE_COUNT) + 1
}

pub fn is_encrypted(name: &str) -> bool {
    let lower = name.to_ascii_lowercase();
    !PLAIN_EXT.iter().any(|ext| lower.ends_with(ext))
}

#[derive(Clone, Debug)]
pub struct Entry {
    pub name: String,
    pub data_file: u32,
    pub offset: u32,
    pub size: u32,
}

/// Archive data.00x du client Rappelz ouverte en lecture.
pub struct DataArchive {
    dir: PathBuf,
    entries: Vec<Entry>,
    by_name: HashMap<String, usize>,
    files: HashMap<u32, File>,
}

impl DataArchive {
    /// `dir` est le dossier du client (contenant data.000).
    pub fn open(dir: impl AsRef<Path>) -> Result<Self> {
        let dir = dir.as_ref().to_path_buf();
        let mut index = std::fs::read(dir.join("data.000"))?;
        xor_in_place(&mut index, 0);
        let mut entries = Vec::new();
        let mut p = 0usize;
        while p < index.len() {
            let len = index[p] as usize;
            if p + 1 + len + 8 > index.len() {
                return Err(ArchiveError::Truncated(p));
            }
            let enc = &index[p + 1..p + 1 + len];
            let rd = |o: usize| u32::from_le_bytes([index[o], index[o + 1], index[o + 2], index[o + 3]]);
            let offset = rd(p + 1 + len);
            let size = rd(p + 5 + len);
            if let Some(name) = decode_name(enc) {
                entries.push(Entry {
                    name,
                    data_file: data_file_index(enc),
                    offset,
                    size,
                });
            }
            p += 1 + len + 8;
        }
        let by_name = entries
            .iter()
            .enumerate()
            .map(|(i, e)| (e.name.to_ascii_lowercase(), i))
            .collect();
        Ok(Self {
            dir,
            entries,
            by_name,
            files: HashMap::new(),
        })
    }

    pub fn entries(&self) -> &[Entry] {
        &self.entries
    }

    pub fn find(&self, name: &str) -> Option<&Entry> {
        self.by_name
            .get(&name.to_ascii_lowercase())
            .map(|&i| &self.entries[i])
    }

    pub fn read(&mut self, entry: &Entry) -> Result<Vec<u8>> {
        let file = match self.files.entry(entry.data_file) {
            std::collections::hash_map::Entry::Occupied(o) => o.into_mut(),
            std::collections::hash_map::Entry::Vacant(v) => {
                v.insert(File::open(self.dir.join(format!("data.{:03}", entry.data_file)))?)
            }
        };
        let mut buf = vec![0u8; entry.size as usize];
        file.seek(SeekFrom::Start(entry.offset as u64))?;
        file.read_exact(&mut buf)?;
        if is_encrypted(&entry.name) {
            xor_in_place(&mut buf, 0);
        }
        Ok(buf)
    }

    pub fn read_name(&mut self, name: &str) -> Result<Vec<u8>> {
        let entry = self
            .find(name)
            .cloned()
            .ok_or_else(|| ArchiveError::NotFound(name.to_string()))?;
        self.read(&entry)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_roundtrip() {
        for n in ["m002_001.bmp", "game_panel_image_worldmap_over_all.bmp", "ab", "icon_skill_0001.dds", "x y z.jpg"] {
            let enc = encode_name(n);
            assert_eq!(decode_name(&enc).as_deref(), Some(n));
            assert!((1..=8).contains(&data_file_index(&enc)));
        }
    }

    #[test]
    fn plain_extensions() {
        assert!(!is_encrypted("a.dds"));
        assert!(!is_encrypted("B.NX3"));
        assert!(is_encrypted("a.jpg"));
        assert!(is_encrypted("script.lua"));
    }

    /// Test sur le vrai client s'il est présent à côté du dépôt.
    #[test]
    fn real_client_if_present() {
        let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../Rappelz NoLimits");
        if !dir.join("data.000").is_file() {
            return;
        }
        let mut a = DataArchive::open(&dir).unwrap();
        assert_eq!(a.entries().len(), 125_282);
        let e = a.entries().iter().find(|e| e.name.ends_with(".jpg") && e.size > 4).cloned().unwrap();
        let data = a.read(&e).unwrap();
        assert_eq!(&data[..3], &[0xFF, 0xD8, 0xFF]);
        let e = a.entries().iter().find(|e| e.name.ends_with(".dds") && e.size > 4).cloned().unwrap();
        assert_eq!(&a.read(&e).unwrap()[..4], b"DDS ");
    }
}
