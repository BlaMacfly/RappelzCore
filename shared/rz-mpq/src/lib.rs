//! Archives MPQ v1, le format de conteneur des clients World of Warcraft.
//!
//! RappelzRust range ses ressources comme un client WoW 3.3.5 :
//! `Data/common.MPQ`, `Data/common-2.MPQ`, …, `Data/<locale>/locale-<locale>.MPQ`,
//! puis `Data/patch.MPQ`, `Data/patch-2.MPQ`… qui écrasent les précédentes.
//! Les archives produites sont lisibles par les outils MPQ classiques
//! (Ladik's MPQ Editor, StormLib).

pub mod crypto;

use std::collections::BTreeSet;
use std::fs::File;
use std::io::{self, BufWriter, Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};

use crypto::*;

pub const MPQ_MAGIC: &[u8; 4] = b"MPQ\x1A";
const HEADER_SIZE: u32 = 32;

pub const FLAG_IMPLODE: u32 = 0x0000_0100;
pub const FLAG_COMPRESS: u32 = 0x0000_0200;
pub const FLAG_ENCRYPTED: u32 = 0x0001_0000;
pub const FLAG_FIX_KEY: u32 = 0x0002_0000;
pub const FLAG_SINGLE_UNIT: u32 = 0x0100_0000;
pub const FLAG_SECTOR_CRC: u32 = 0x0400_0000;
pub const FLAG_EXISTS: u32 = 0x8000_0000;

const HASH_EMPTY: u32 = 0xFFFF_FFFF;
const HASH_DELETED: u32 = 0xFFFF_FFFE;

const COMP_ZLIB: u8 = 0x02;

pub const LISTFILE: &str = "(listfile)";

#[derive(Debug, thiserror::Error)]
pub enum MpqError {
    #[error("E/S : {0}")]
    Io(#[from] io::Error),
    #[error("en-tête MPQ introuvable")]
    NoHeader,
    #[error("fichier absent de l'archive : {0}")]
    NotFound(String),
    #[error("compression non supportée (masque {0:#04x})")]
    UnsupportedCompression(u8),
    #[error("archive corrompue : {0}")]
    Corrupt(&'static str),
    #[error("archive trop grande pour le format MPQ v1 (> 4 Gio)")]
    TooLarge,
}

pub type Result<T> = std::result::Result<T, MpqError>;

#[derive(Clone, Copy, Debug)]
struct HashEntry {
    name_a: u32,
    name_b: u32,
    locale: u16,
    block_index: u32,
}

#[derive(Clone, Copy, Debug)]
pub struct BlockEntry {
    pub file_pos: u32,
    pub compressed_size: u32,
    pub file_size: u32,
    pub flags: u32,
}

/// Archive MPQ ouverte en lecture.
pub struct MpqArchive {
    path: PathBuf,
    file: File,
    archive_offset: u64,
    sector_size: u32,
    hashes: Vec<HashEntry>,
    blocks: Vec<BlockEntry>,
}

impl MpqArchive {
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref().to_path_buf();
        let mut file = File::open(&path)?;
        let len = file.metadata()?.len();

        // L'en-tête peut être précédé de données utilisateur, aligné sur 512 octets.
        let mut archive_offset = None;
        let mut pos = 0u64;
        let mut magic = [0u8; 4];
        while pos + HEADER_SIZE as u64 <= len {
            file.seek(SeekFrom::Start(pos))?;
            file.read_exact(&mut magic)?;
            if &magic == MPQ_MAGIC {
                archive_offset = Some(pos);
                break;
            }
            pos += 512;
        }
        let archive_offset = archive_offset.ok_or(MpqError::NoHeader)?;

        let mut hdr = [0u8; 28];
        file.read_exact(&mut hdr)?;
        let rd = |o: usize| u32::from_le_bytes([hdr[o], hdr[o + 1], hdr[o + 2], hdr[o + 3]]);
        let sector_shift = u16::from_le_bytes([hdr[10], hdr[11]]);
        let hash_pos = rd(12);
        let block_pos = rd(16);
        let hash_count = rd(20);
        let block_count = rd(24);

        let hash_words = read_table(&mut file, archive_offset + hash_pos as u64, hash_count, "(hash table)")?;
        let block_words = read_table(&mut file, archive_offset + block_pos as u64, block_count, "(block table)")?;

        let hashes = hash_words
            .chunks_exact(4)
            .map(|w| HashEntry {
                name_a: w[0],
                name_b: w[1],
                locale: (w[2] & 0xFFFF) as u16,
                block_index: w[3],
            })
            .collect();
        let blocks = block_words
            .chunks_exact(4)
            .map(|w| BlockEntry {
                file_pos: w[0],
                compressed_size: w[1],
                file_size: w[2],
                flags: w[3],
            })
            .collect();

        Ok(Self {
            path,
            file,
            archive_offset,
            sector_size: 512u32 << sector_shift,
            hashes,
            blocks,
        })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    fn find_block(&self, name: &str) -> Option<BlockEntry> {
        if self.hashes.is_empty() {
            return None;
        }
        let mask = self.hashes.len() - 1;
        let start = hash_string(name, HASH_TABLE_OFFSET) as usize & mask;
        let a = hash_string(name, HASH_NAME_A);
        let b = hash_string(name, HASH_NAME_B);
        let mut fallback = None;
        let mut i = start;
        loop {
            let h = self.hashes[i];
            if h.block_index == HASH_EMPTY {
                break;
            }
            if h.block_index != HASH_DELETED && h.name_a == a && h.name_b == b {
                let block = *self.blocks.get(h.block_index as usize)?;
                if block.flags & FLAG_EXISTS != 0 {
                    if h.locale == 0 {
                        return Some(block);
                    }
                    fallback.get_or_insert(block);
                }
            }
            i = (i + 1) & mask;
            if i == start {
                break;
            }
        }
        fallback
    }

    pub fn contains(&self, name: &str) -> bool {
        self.find_block(name).is_some()
    }

    pub fn file_size(&self, name: &str) -> Option<u32> {
        self.find_block(name).map(|b| b.file_size)
    }

    pub fn read_file(&mut self, name: &str) -> Result<Vec<u8>> {
        let block = self
            .find_block(name)
            .ok_or_else(|| MpqError::NotFound(name.to_string()))?;
        self.read_block(name, block)
    }

    fn read_block(&mut self, name: &str, block: BlockEntry) -> Result<Vec<u8>> {
        let mut raw = vec![0u8; block.compressed_size as usize];
        self.file
            .seek(SeekFrom::Start(self.archive_offset + block.file_pos as u64))?;
        self.file.read_exact(&mut raw)?;

        let key = if block.flags & FLAG_ENCRYPTED != 0 {
            Some(file_key(name, &block))
        } else {
            None
        };

        if block.flags & FLAG_SINGLE_UNIT != 0 {
            if let Some(k) = key {
                decrypt_bytes(&mut raw, k);
            }
            if block.flags & FLAG_COMPRESS != 0 && block.compressed_size < block.file_size {
                return decompress_sector(&raw, block.file_size as usize);
            }
            return Ok(raw);
        }

        if block.flags & (FLAG_COMPRESS | FLAG_IMPLODE) == 0 {
            if let Some(k) = key {
                for (i, chunk) in raw.chunks_mut(self.sector_size as usize).enumerate() {
                    decrypt_bytes(chunk, k.wrapping_add(i as u32));
                }
            }
            return Ok(raw);
        }

        let sectors = block.file_size.div_ceil(self.sector_size) as usize;
        let mut table_len = (sectors + 1) * 4;
        if block.flags & FLAG_SECTOR_CRC != 0 {
            table_len += 4;
        }
        if raw.len() < table_len {
            return Err(MpqError::Corrupt("table des secteurs tronquée"));
        }
        let mut offsets = bytes_to_u32s(&raw[..table_len]);
        if let Some(k) = key {
            decrypt_block(&mut offsets, k.wrapping_sub(1));
        }

        let mut out = Vec::with_capacity(block.file_size as usize);
        for s in 0..sectors {
            let (start, end) = (offsets[s] as usize, offsets[s + 1] as usize);
            if start > end || end > raw.len() {
                return Err(MpqError::Corrupt("offset de secteur invalide"));
            }
            let mut sector = raw[start..end].to_vec();
            if let Some(k) = key {
                decrypt_bytes(&mut sector, k.wrapping_add(s as u32));
            }
            let expected = (block.file_size as usize - out.len()).min(self.sector_size as usize);
            if sector.len() < expected {
                if block.flags & FLAG_IMPLODE != 0 {
                    return Err(MpqError::UnsupportedCompression(0x08));
                }
                out.extend(decompress_sector(&sector, expected)?);
            } else {
                out.extend_from_slice(&sector);
            }
        }
        Ok(out)
    }

    /// Noms listés dans le `(listfile)` de l'archive.
    pub fn list(&mut self) -> Result<Vec<String>> {
        if !self.contains(LISTFILE) {
            return Ok(Vec::new());
        }
        let data = self.read_file(LISTFILE)?;
        Ok(String::from_utf8_lossy(&data)
            .split(['\r', '\n', ';'])
            .filter(|s| !s.is_empty())
            .map(str::to_string)
            .collect())
    }
}

fn read_table(file: &mut File, pos: u64, entries: u32, key_name: &str) -> Result<Vec<u32>> {
    let mut buf = vec![0u8; entries as usize * 16];
    file.seek(SeekFrom::Start(pos))?;
    file.read_exact(&mut buf)?;
    let mut words = bytes_to_u32s(&buf);
    decrypt_block(&mut words, hash_string(key_name, HASH_FILE_KEY));
    Ok(words)
}

fn file_key(name: &str, block: &BlockEntry) -> u32 {
    let base = name.rsplit(['\\', '/']).next().unwrap_or(name);
    let mut key = hash_string(base, HASH_FILE_KEY);
    if block.flags & FLAG_FIX_KEY != 0 {
        key = key.wrapping_add(block.file_pos) ^ block.file_size;
    }
    key
}

fn decrypt_bytes(data: &mut [u8], key: u32) {
    let n = data.len() / 4 * 4;
    let mut words = bytes_to_u32s(&data[..n]);
    decrypt_block(&mut words, key);
    data[..n].copy_from_slice(&u32s_to_bytes(&words));
}

fn decompress_sector(data: &[u8], expected: usize) -> Result<Vec<u8>> {
    let (&mask, payload) = data.split_first().ok_or(MpqError::Corrupt("secteur vide"))?;
    if mask != COMP_ZLIB {
        return Err(MpqError::UnsupportedCompression(mask));
    }
    let mut out = Vec::with_capacity(expected);
    flate2::read::ZlibDecoder::new(payload).read_to_end(&mut out)?;
    Ok(out)
}

/// Écriture séquentielle d'une archive MPQ v1 (les données partent sur disque au fil de l'eau).
pub struct MpqWriter {
    out: BufWriter<File>,
    pos: u64,
    sector_shift: u16,
    compression_level: u32,
    entries: Vec<(String, BlockEntry)>,
    names: BTreeSet<String>,
}

impl MpqWriter {
    /// `sector_shift` = 3 donne des secteurs de 4096 octets, comme les MPQ de WoW.
    pub fn create(path: impl AsRef<Path>, sector_shift: u16) -> Result<Self> {
        let mut out = BufWriter::with_capacity(1 << 20, File::create(path)?);
        out.write_all(&[0u8; HEADER_SIZE as usize])?;
        Ok(Self {
            out,
            pos: HEADER_SIZE as u64,
            sector_shift,
            compression_level: 6,
            entries: Vec::new(),
            names: BTreeSet::new(),
        })
    }

    /// Niveau zlib (0 à 9, 6 par défaut).
    pub fn set_compression_level(&mut self, level: u32) {
        self.compression_level = level.min(9);
    }

    pub fn size(&self) -> u64 {
        self.pos
    }

    pub fn file_count(&self) -> usize {
        self.entries.len()
    }

    pub fn add_file(&mut self, name: &str, data: &[u8], compress: bool) -> Result<()> {
        let prepared = PreparedFile::new(name, data, compress, self.sector_shift, self.compression_level)?;
        self.add_prepared(prepared)
    }

    /// Ajoute un fichier déjà préparé (compressé) par [`PreparedFile::new`], éventuellement
    /// sur un autre thread : seule l'écriture sur disque est séquentielle.
    pub fn add_prepared(&mut self, file: PreparedFile) -> Result<()> {
        if !self.names.insert(normalize_path(&file.name)) {
            // Même nom ajouté deux fois : la dernière version gagne, comme un patch.
            let key = normalize_path(&file.name);
            self.entries.retain(|(n, _)| normalize_path(n) != key);
        }
        let file_pos = self.pos;
        if file_pos + file.payload.len() as u64 > u32::MAX as u64 {
            return Err(MpqError::TooLarge);
        }
        self.out.write_all(&file.payload)?;
        self.pos += file.payload.len() as u64;
        self.entries.push((
            file.name,
            BlockEntry {
                file_pos: file_pos as u32,
                compressed_size: file.payload.len() as u32,
                file_size: file.file_size,
                flags: file.flags,
            },
        ));
        Ok(())
    }

    pub fn sector_shift(&self) -> u16 {
        self.sector_shift
    }

    pub fn compression_level(&self) -> u32 {
        self.compression_level
    }

    pub fn finish(mut self) -> Result<()> {
        let mut listfile = String::new();
        for (name, _) in &self.entries {
            listfile.push_str(name);
            listfile.push_str("\r\n");
        }
        self.add_file(LISTFILE, listfile.as_bytes(), true)?;

        let count = self.entries.len();
        let hash_size = (count * 4 / 3 + 1).next_power_of_two().max(16);
        let mut hashes = vec![[HASH_EMPTY; 4]; hash_size];
        let mut blocks = Vec::with_capacity(count * 4);
        for (i, (name, block)) in self.entries.iter().enumerate() {
            let mut slot = hash_string(name, HASH_TABLE_OFFSET) as usize & (hash_size - 1);
            while hashes[slot][3] != HASH_EMPTY {
                slot = (slot + 1) & (hash_size - 1);
            }
            hashes[slot] = [
                hash_string(name, HASH_NAME_A),
                hash_string(name, HASH_NAME_B),
                0, // locale neutre, plateforme 0
                i as u32,
            ];
            blocks.extend([block.file_pos, block.compressed_size, block.file_size, block.flags]);
        }
        let mut hash_words: Vec<u32> = hashes.into_iter().flatten().collect();
        encrypt_block(&mut hash_words, hash_string("(hash table)", HASH_FILE_KEY));
        encrypt_block(&mut blocks, hash_string("(block table)", HASH_FILE_KEY));

        let hash_pos = self.pos;
        self.out.write_all(&u32s_to_bytes(&hash_words))?;
        let block_pos = hash_pos + hash_words.len() as u64 * 4;
        self.out.write_all(&u32s_to_bytes(&blocks))?;
        let archive_size = block_pos + blocks.len() as u64 * 4;
        if archive_size > u32::MAX as u64 {
            return Err(MpqError::TooLarge);
        }

        let mut hdr = Vec::with_capacity(HEADER_SIZE as usize);
        hdr.extend_from_slice(MPQ_MAGIC);
        hdr.extend(HEADER_SIZE.to_le_bytes());
        hdr.extend((archive_size as u32).to_le_bytes());
        hdr.extend(0u16.to_le_bytes()); // format v1
        hdr.extend(self.sector_shift.to_le_bytes());
        hdr.extend((hash_pos as u32).to_le_bytes());
        hdr.extend((block_pos as u32).to_le_bytes());
        hdr.extend((hash_size as u32).to_le_bytes());
        hdr.extend((count as u32).to_le_bytes());
        let mut file = self.out.into_inner().map_err(|e| e.into_error())?;
        file.seek(SeekFrom::Start(0))?;
        file.write_all(&hdr)?;
        file.sync_all()?;
        Ok(())
    }
}

/// Fichier mis au format MPQ (table des secteurs + secteurs zlib), prêt à écrire.
pub struct PreparedFile {
    name: String,
    payload: Vec<u8>,
    file_size: u32,
    flags: u32,
}

impl PreparedFile {
    pub fn new(name: &str, data: &[u8], compress: bool, sector_shift: u16, level: u32) -> Result<Self> {
        if data.len() > u32::MAX as usize {
            return Err(MpqError::TooLarge);
        }
        let sector_size = 512usize << sector_shift;
        let (payload, flags) = if compress && !data.is_empty() {
            let sectors: Vec<&[u8]> = data.chunks(sector_size).collect();
            let mut table = Vec::with_capacity(sectors.len() + 1);
            let mut body = Vec::with_capacity(data.len() / 2);
            let mut off = ((sectors.len() + 1) * 4) as u32;
            for s in sectors {
                table.push(off);
                let mut enc = flate2::write::ZlibEncoder::new(Vec::new(), flate2::Compression::new(level));
                enc.write_all(s)?;
                let z = enc.finish()?;
                if z.len() + 1 < s.len() {
                    body.push(COMP_ZLIB);
                    body.extend_from_slice(&z);
                    off += z.len() as u32 + 1;
                } else {
                    body.extend_from_slice(s);
                    off += s.len() as u32;
                }
            }
            table.push(off);
            let mut payload = u32s_to_bytes(&table);
            payload.extend(body);
            (payload, FLAG_EXISTS | FLAG_COMPRESS)
        } else {
            (data.to_vec(), FLAG_EXISTS)
        };
        Ok(Self {
            name: name.replace('/', "\\"),
            payload,
            file_size: data.len() as u32,
            flags,
        })
    }

    pub fn stored_size(&self) -> usize {
        self.payload.len()
    }
}

/// Ensemble d'archives consultées par priorité, comme le `Data/` d'un client WoW.
pub struct MpqChain {
    archives: Vec<MpqArchive>,
}

impl MpqChain {
    pub fn new() -> Self {
        Self { archives: Vec::new() }
    }

    /// Ajoute une archive de priorité supérieure à toutes les précédentes.
    pub fn push(&mut self, archive: MpqArchive) {
        self.archives.push(archive);
    }

    /// Ouvre `Data/` dans l'ordre de WoW : common*, locale, patch*, patch de locale.
    pub fn open_client_data(data_dir: impl AsRef<Path>, locale: &str) -> Result<Self> {
        let data_dir = data_dir.as_ref();
        let mut chain = Self::new();
        let loc_dir = data_dir.join(locale);
        let groups: [(&Path, String); 4] = [
            (data_dir, "common".to_string()),
            (&loc_dir, format!("locale-{locale}")),
            (data_dir, "patch".to_string()),
            (&loc_dir, format!("patch-{locale}")),
        ];
        for (dir, prefix) in groups {
            for path in numbered_archives(dir, &prefix) {
                chain.push(MpqArchive::open(path)?);
            }
        }
        Ok(chain)
    }

    pub fn archives(&self) -> &[MpqArchive] {
        &self.archives
    }

    pub fn contains(&self, name: &str) -> bool {
        self.archives.iter().any(|a| a.contains(name))
    }

    pub fn read_file(&mut self, name: &str) -> Result<Vec<u8>> {
        for archive in self.archives.iter_mut().rev() {
            if archive.contains(name) {
                return archive.read_file(name);
            }
        }
        Err(MpqError::NotFound(name.to_string()))
    }

    pub fn list(&mut self) -> Result<Vec<String>> {
        let mut all = BTreeSet::new();
        for archive in &mut self.archives {
            all.extend(archive.list()?);
        }
        all.remove(LISTFILE);
        Ok(all.into_iter().collect())
    }
}

impl Default for MpqChain {
    fn default() -> Self {
        Self::new()
    }
}

/// `prefix.MPQ`, puis `prefix-2.MPQ`, `prefix-3.MPQ`… tant qu'ils existent.
fn numbered_archives(dir: &Path, prefix: &str) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let first = dir.join(format!("{prefix}.MPQ"));
    if first.is_file() {
        out.push(first);
    }
    for n in 2.. {
        let p = dir.join(format!("{prefix}-{n}.MPQ"));
        if !p.is_file() {
            break;
        }
        out.push(p);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn write_then_read() {
        let dir = std::env::temp_dir().join(format!("rzmpq-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("test.MPQ");
        let big: Vec<u8> = (0..20_000u32).map(|i| (i % 251) as u8).collect();
        let mut w = MpqWriter::create(&path, 3).unwrap();
        w.add_file("DBFilesClient\\Item.dbc", b"WDBC....", false).unwrap();
        w.add_file("World/Maps/test.bin", &big, true).unwrap();
        w.finish().unwrap();

        let mut a = MpqArchive::open(&path).unwrap();
        assert_eq!(a.read_file("dbfilesclient/item.dbc").unwrap(), b"WDBC....");
        assert_eq!(a.read_file("WORLD\\MAPS\\TEST.BIN").unwrap(), big);
        assert!(a.read_file("absent").is_err());
        let list = a.list().unwrap();
        assert!(list.contains(&"World\\Maps\\test.bin".to_string()));
        std::fs::remove_dir_all(&dir).ok();
    }
}
