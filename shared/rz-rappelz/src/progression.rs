//! Lossless archive progression tables, separate from the historical gameplay formulas.
//!
//! BasicStat81 records contain eight signed 16-bit fields. Exp91 and SummonExp81
//! contain five/three signed 64-bit fields, with a one-based implicit row level.
//! The companion WDBC tables store each 64-bit field as its low/high u32 words;
//! they never pass through floating point or the legacy generic RDB converter.
use rz_dbc::{DbcFile, DbcWriter, Value};
use std::{
    collections::BTreeMap,
    fs::File,
    io::Read,
    path::{Path, PathBuf},
};

const RDB_HEADER: usize = 132;
const MAX_BASE_ROWS: usize = i16::MAX as usize + 1;
const MAX_LEVEL_ROWS: usize = u16::MAX as usize;

#[derive(Debug, thiserror::Error)]
pub enum ProgressionError {
    #[error("{table}: {reason}")]
    Invalid {
        table: &'static str,
        reason: &'static str,
    },
    #[error("{path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("{table}: {source}")]
    Dbc {
        table: &'static str,
        #[source]
        source: rz_dbc::DbcError,
    },
}
pub type Result<T> = std::result::Result<T, ProgressionError>;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExpRow {
    pub level: u16,
    pub exp: i64,
    pub jp: [i64; 4],
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SummonExpRow {
    pub level: u16,
    pub normal: i64,
    pub growth: i64,
    pub evolve: i64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProgressionBook {
    bases: BTreeMap<i32, [i16; 7]>,
    experience: Vec<ExpRow>,
    summons: Vec<SummonExpRow>,
}

fn invalid(table: &'static str, reason: &'static str) -> ProgressionError {
    ProgressionError::Invalid { table, reason }
}

fn u32_at(bytes: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap())
}

fn rdb_rows<'a>(
    bytes: &'a [u8],
    table: &'static str,
    size: usize,
    maximum: usize,
) -> Result<std::slice::ChunksExact<'a, u8>> {
    if bytes.len() < RDB_HEADER {
        return Err(invalid(table, "truncated RDB header"));
    }
    let count = u32_at(bytes, 128) as usize;
    if count == 0 || count > maximum {
        return Err(invalid(table, "invalid or excessive row count"));
    }
    let expected = count
        .checked_mul(size)
        .and_then(|v| v.checked_add(RDB_HEADER))
        .ok_or_else(|| invalid(table, "record length overflow"))?;
    if bytes.len() != expected {
        return Err(invalid(
            table,
            "truncated, trailing or incompatible RDB records",
        ));
    }
    Ok(bytes[RDB_HEADER..].chunks_exact(size))
}

fn dbc(bytes: &[u8], table: &'static str, fields: u32, maximum: usize) -> Result<DbcFile> {
    if bytes.len() < 20 || &bytes[..4] != b"WDBC" {
        return Err(invalid(table, "missing or truncated WDBC header"));
    }
    let count = u32_at(bytes, 4) as usize;
    if count == 0 || count > maximum {
        return Err(invalid(table, "invalid or excessive row count"));
    }
    let size = fields as usize * 4;
    if u32_at(bytes, 8) != fields || u32_at(bytes, 12) as usize != size || u32_at(bytes, 16) != 1 {
        return Err(invalid(table, "incompatible companion WDBC schema"));
    }
    let expected = count
        .checked_mul(size)
        .and_then(|v| v.checked_add(21))
        .ok_or_else(|| invalid(table, "record length overflow"))?;
    if bytes.len() != expected || bytes.last() != Some(&0) {
        return Err(invalid(
            table,
            "truncated, trailing or incompatible WDBC data",
        ));
    }
    DbcFile::parse(bytes).map_err(|source| ProgressionError::Dbc { table, source })
}

fn read_companion(
    root: &Path,
    name: &'static str,
    fields: usize,
    maximum: usize,
) -> Result<Vec<u8>> {
    let path = root.join("Progression").join(name);
    let maximum_bytes = 21 + maximum * fields * 4;
    let file = File::open(&path).map_err(|source| ProgressionError::Io {
        path: path.clone(),
        source,
    })?;
    let mut bytes = Vec::new();
    file.take(maximum_bytes as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(|source| ProgressionError::Io {
            path: path.clone(),
            source,
        })?;
    if bytes.len() > maximum_bytes {
        return Err(invalid(name, "companion file exceeds size limit"));
    }
    Ok(bytes)
}

fn signed_words(record: rz_dbc::Record<'_>, field: usize) -> i64 {
    ((u64::from(record.u32(field + 1)) << 32) | u64::from(record.u32(field))) as i64
}

impl ProgressionBook {
    /// Parse the dedicated original layouts; no generic conversion or gameplay mutation.
    pub fn from_rdb(basic: &[u8], exp: &[u8], summon: &[u8]) -> Result<Self> {
        let mut bases = BTreeMap::new();
        for row in rdb_rows(basic, "BasicStat", 16, MAX_BASE_ROWS)? {
            let id = i16::from_le_bytes(row[..2].try_into().unwrap());
            let values = std::array::from_fn(|i| {
                i16::from_le_bytes(row[2 + i * 2..4 + i * 2].try_into().unwrap())
            });
            Self::insert_base(&mut bases, i32::from(id), values)?;
        }
        let mut experience = Vec::new();
        for (i, row) in rdb_rows(exp, "Exp", 40, MAX_LEVEL_ROWS)?.enumerate() {
            let values: [i64; 5] = std::array::from_fn(|i| {
                i64::from_le_bytes(row[i * 8..i * 8 + 8].try_into().unwrap())
            });
            experience.push(ExpRow {
                level: (i + 1) as u16,
                exp: values[0],
                jp: values[1..].try_into().unwrap(),
            });
        }
        let mut summons = Vec::new();
        for (i, row) in rdb_rows(summon, "SummonExp", 24, MAX_LEVEL_ROWS)?.enumerate() {
            let values: [i64; 3] = std::array::from_fn(|i| {
                i64::from_le_bytes(row[i * 8..i * 8 + 8].try_into().unwrap())
            });
            summons.push(SummonExpRow {
                level: (i + 1) as u16,
                normal: values[0],
                growth: values[1],
                evolve: values[2],
            });
        }
        Self::checked(bases, experience, summons)
    }

    /// Read all three dedicated companion tables. A missing/invalid table is an error.
    pub fn open(root: &Path) -> Result<Self> {
        Self::from_dbc(
            &read_companion(root, "BasicStat.dbc", 8, MAX_BASE_ROWS)?,
            &read_companion(root, "Exp.dbc", 11, MAX_LEVEL_ROWS)?,
            &read_companion(root, "SummonExp.dbc", 7, MAX_LEVEL_ROWS)?,
        )
    }

    /// Load complete companion bytes, including validation stricter than generic WDBC.
    pub fn from_dbc(basic: &[u8], exp: &[u8], summon: &[u8]) -> Result<Self> {
        let basic = dbc(basic, "BasicStat", 8, MAX_BASE_ROWS)?;
        let exp = dbc(exp, "Exp", 11, MAX_LEVEL_ROWS)?;
        let summon = dbc(summon, "SummonExp", 7, MAX_LEVEL_ROWS)?;
        let mut bases = BTreeMap::new();
        for row in basic.iter() {
            let id = row.i32(0);
            let mut values = [0; 7];
            for (i, value) in values.iter_mut().enumerate() {
                *value = i16::try_from(row.i32(i + 1))
                    .map_err(|_| invalid("BasicStat", "statistic outside signed 16-bit range"))?;
            }
            Self::insert_base(&mut bases, id, values)?;
        }
        let mut experience = Vec::new();
        for row in exp.iter() {
            let level = u16::try_from(row.u32(0))
                .map_err(|_| invalid("Exp", "level outside unsigned 16-bit range"))?;
            experience.push(ExpRow {
                level,
                exp: signed_words(row, 1),
                jp: std::array::from_fn(|i| signed_words(row, 3 + i * 2)),
            });
        }
        let mut summons = Vec::new();
        for row in summon.iter() {
            let level = u16::try_from(row.u32(0))
                .map_err(|_| invalid("SummonExp", "level outside unsigned 16-bit range"))?;
            summons.push(SummonExpRow {
                level,
                normal: signed_words(row, 1),
                growth: signed_words(row, 3),
                evolve: signed_words(row, 5),
            });
        }
        Self::checked(bases, experience, summons)
    }

    fn insert_base(bases: &mut BTreeMap<i32, [i16; 7]>, id: i32, values: [i16; 7]) -> Result<()> {
        if !(0..=i32::from(i16::MAX)).contains(&id) || values.iter().any(|v| *v < 0) {
            return Err(invalid(
                "BasicStat",
                "negative or out-of-range ID/statistic",
            ));
        }
        if bases.insert(id, values).is_some() {
            return Err(invalid("BasicStat", "duplicate statistic ID"));
        }
        Ok(())
    }

    fn checked(
        bases: BTreeMap<i32, [i16; 7]>,
        experience: Vec<ExpRow>,
        summons: Vec<SummonExpRow>,
    ) -> Result<Self> {
        for (i, row) in experience.iter().enumerate() {
            if usize::from(row.level) != i + 1 || row.exp < 0 || row.jp.iter().any(|v| *v < 0) {
                return Err(invalid(
                    "Exp",
                    "invalid/duplicate/noncanonical level or negative value",
                ));
            }
            if i > 0 && experience[i - 1].exp >= row.exp {
                return Err(invalid("Exp", "experience is not strictly increasing"));
            }
        }
        for (i, row) in summons.iter().enumerate() {
            if usize::from(row.level) != i + 1 || row.normal < 0 || row.growth < 0 || row.evolve < 0
            {
                return Err(invalid(
                    "SummonExp",
                    "invalid/duplicate/noncanonical level or negative value",
                ));
            }
            if i > 0 && summons[i - 1].normal >= row.normal {
                return Err(invalid(
                    "SummonExp",
                    "normal experience is not strictly increasing",
                ));
            }
        }
        Ok(Self {
            bases,
            experience,
            summons,
        })
    }

    /// Deterministic WDBC output. Filenames are local to the additive Progression directory.
    pub fn to_dbc_files(&self) -> BTreeMap<String, Vec<u8>> {
        let mut basic = DbcWriter::new("niiiiiii");
        for (id, stats) in &self.bases {
            let mut values = vec![Value::I32(*id)];
            values.extend(stats.iter().map(|v| Value::I32(i32::from(*v))));
            basic.push(&values).expect("fixed BasicStat WDBC schema");
        }
        let words = |value: i64| {
            [
                Value::U32(value as u64 as u32),
                Value::U32(((value as u64) >> 32) as u32),
            ]
        };
        let mut exp = DbcWriter::new("niiiiiiiiii");
        for row in &self.experience {
            let mut values = vec![Value::U32(u32::from(row.level))];
            for value in std::iter::once(&row.exp).chain(row.jp.iter()) {
                values.extend(words(*value));
            }
            exp.push(&values).expect("fixed Exp WDBC schema");
        }
        let mut summon = DbcWriter::new("niiiiii");
        for row in &self.summons {
            let mut values = vec![Value::U32(u32::from(row.level))];
            for value in [row.normal, row.growth, row.evolve] {
                values.extend(words(value));
            }
            summon.push(&values).expect("fixed SummonExp WDBC schema");
        }
        BTreeMap::from([
            ("BasicStat.dbc".into(), basic.to_bytes()),
            ("Exp.dbc".into(), exp.to_bytes()),
            ("SummonExp.dbc".into(), summon.to_bytes()),
        ])
    }

    pub fn base(&self, id: i32) -> Option<[i16; 7]> {
        self.bases.get(&id).copied()
    }
    pub fn experience(&self, level: u16) -> Option<&ExpRow> {
        self.experience.get(usize::from(level.checked_sub(1)?))
    }
    pub fn summon_experience(&self, level: u16) -> Option<&SummonExpRow> {
        self.summons.get(usize::from(level.checked_sub(1)?))
    }
    pub fn counts(&self) -> (usize, usize, usize) {
        (self.bases.len(), self.experience.len(), self.summons.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rdb(rows: &[Vec<u8>]) -> Vec<u8> {
        let mut bytes = vec![0; RDB_HEADER];
        bytes[..8].copy_from_slice(b"20261031");
        bytes[128..132].copy_from_slice(&(rows.len() as u32).to_le_bytes());
        for row in rows {
            bytes.extend(row);
        }
        bytes
    }
    fn fixtures() -> [Vec<u8>; 3] {
        let basic = rdb(&[
            [0i16, 10, 10, 10, 10, 10, 10, 10]
                .into_iter()
                .flat_map(i16::to_le_bytes)
                .collect(),
            [302i16, 14, 16, 13, 13, 20, 20, 10]
                .into_iter()
                .flat_map(i16::to_le_bytes)
                .collect(),
        ]);
        let exp = rdb(&[
            [6i64, 3, 20, 210, 8200]
                .into_iter()
                .flat_map(i64::to_le_bytes)
                .collect(),
            [(1i64 << 53) + 33, 0, 0, 0, i64::MAX]
                .into_iter()
                .flat_map(i64::to_le_bytes)
                .collect(),
            [i64::MAX, 0, 0, 0, 0]
                .into_iter()
                .flat_map(i64::to_le_bytes)
                .collect(),
        ]);
        let summon = rdb(&[
            [5i64, 1068, 21798]
                .into_iter()
                .flat_map(i64::to_le_bytes)
                .collect(),
            [(1i64 << 53) + 17, i64::MAX, 0]
                .into_iter()
                .flat_map(i64::to_le_bytes)
                .collect(),
            [i64::MAX, 0, 0]
                .into_iter()
                .flat_map(i64::to_le_bytes)
                .collect(),
        ]);
        [basic, exp, summon]
    }
    fn book() -> ProgressionBook {
        let [a, b, c] = fixtures();
        ProgressionBook::from_rdb(&a, &b, &c).unwrap()
    }
    fn from_files(files: &BTreeMap<String, Vec<u8>>) -> Result<ProgressionBook> {
        ProgressionBook::from_dbc(
            &files["BasicStat.dbc"],
            &files["Exp.dbc"],
            &files["SummonExp.dbc"],
        )
    }

    #[test]
    fn split_words_preserve_all_bits_above_float_precision_and_at_signed_maximum() {
        let book = book();
        assert_eq!(book.counts(), (2, 3, 3));
        assert_eq!(book.base(302), Some([14, 16, 13, 13, 20, 20, 10]));
        assert_eq!(book.experience(2).unwrap().exp, (1i64 << 53) + 33);
        assert_eq!(book.experience(2).unwrap().jp[3], i64::MAX);
        assert_eq!(book.summon_experience(2).unwrap().normal, (1i64 << 53) + 17);
        assert_eq!(book.experience(3).unwrap().exp, i64::MAX);
        let files = book.to_dbc_files();
        let exp = DbcFile::parse(&files["Exp.dbc"]).unwrap();
        assert_eq!((exp.field_count, exp.record_size), (11, 44));
        assert_eq!((exp.record(1).u32(1), exp.record(1).u32(2)), (33, 1 << 21));
        let restored = from_files(&files).unwrap();
        assert_eq!(restored, book);
        assert_eq!(
            restored.to_dbc_files(),
            files,
            "canonical companion roundtrip changed bytes"
        );
        assert_eq!(book.experience(0), None);
        assert_eq!(book.experience(4), None);
        assert_eq!(book.summon_experience(0), None);
        assert_eq!(book.base(-1), None);
    }

    #[test]
    fn rdb_lengths_and_counts_fail_closed_without_partial_tables() {
        let good = fixtures();
        for table in 0..3 {
            for cut in [0, 127, 131, good[table].len() - 1] {
                let mut f = good.clone();
                f[table].truncate(cut);
                assert!(ProgressionBook::from_rdb(&f[0], &f[1], &f[2]).is_err());
            }
            for count in [0, u32::MAX] {
                let mut f = good.clone();
                f[table][128..132].copy_from_slice(&count.to_le_bytes());
                assert!(ProgressionBook::from_rdb(&f[0], &f[1], &f[2]).is_err());
            }
            let mut f = good.clone();
            f[table].push(0);
            assert!(ProgressionBook::from_rdb(&f[0], &f[1], &f[2]).is_err());
        }
        assert!(ProgressionBook::from_rdb(&good[1], &good[0], &good[2]).is_err());
    }

    #[test]
    fn negative_values_duplicate_ids_and_nonmonotone_experience_are_rejected() {
        let good = fixtures();
        for (table, offset, bytes) in [
            (0, 132, (-1i16).to_le_bytes().to_vec()),
            (0, 134, (-1i16).to_le_bytes().to_vec()),
            (0, 148, 0i16.to_le_bytes().to_vec()),
            (1, 132, (-1i64).to_le_bytes().to_vec()),
            (1, 140, (-1i64).to_le_bytes().to_vec()),
            (1, 172, 6i64.to_le_bytes().to_vec()),
            (2, 140, (-1i64).to_le_bytes().to_vec()),
            (2, 156, 5i64.to_le_bytes().to_vec()),
        ] {
            let mut f = good.clone();
            f[table][offset..offset + bytes.len()].copy_from_slice(&bytes);
            assert!(
                ProgressionBook::from_rdb(&f[0], &f[1], &f[2]).is_err(),
                "accepted table {table}, offset {offset}"
            );
        }
    }

    #[test]
    fn companion_schemas_lengths_and_table_mixups_are_rejected() {
        let good = book().to_dbc_files();
        for name in ["BasicStat.dbc", "Exp.dbc", "SummonExp.dbc"] {
            for (offset, value) in [(4, 0u32), (4, u32::MAX), (8, 99), (12, 1), (16, 2)] {
                let mut f = good.clone();
                f.get_mut(name).unwrap()[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
                assert!(from_files(&f).is_err());
            }
            let mut f = good.clone();
            f.get_mut(name).unwrap().push(0);
            assert!(from_files(&f).is_err());
            let mut f = good.clone();
            f.get_mut(name).unwrap().pop();
            assert!(from_files(&f).is_err());
            let mut f = good.clone();
            f.get_mut(name).unwrap()[0] = b'X';
            assert!(from_files(&f).is_err());
        }
        assert!(ProgressionBook::from_dbc(
            &good["Exp.dbc"],
            &good["BasicStat.dbc"],
            &good["SummonExp.dbc"]
        )
        .is_err());
        assert!(ProgressionBook::from_dbc(
            &good["BasicStat.dbc"],
            &good["SummonExp.dbc"],
            &good["Exp.dbc"]
        )
        .is_err());
    }

    #[test]
    fn invalid_levels_negative_wide_values_and_widened_basestat_fields_are_rejected() {
        let good = book().to_dbc_files();
        for (name, offset, value) in [
            ("Exp.dbc", 20, 0u32),
            ("Exp.dbc", 64, 1),
            ("Exp.dbc", 64, 3),
            ("Exp.dbc", 20, 65536),
            ("Exp.dbc", 28, 0x80000000),
            ("Exp.dbc", 36, 0xffffffff),
            ("SummonExp.dbc", 20, 0),
            ("SummonExp.dbc", 48, 1),
            ("SummonExp.dbc", 28, 0x80000000),
            ("BasicStat.dbc", 20, 65536),
            ("BasicStat.dbc", 24, 32768),
            ("BasicStat.dbc", 52, 0),
        ] {
            let mut f = good.clone();
            f.get_mut(name).unwrap()[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
            assert!(from_files(&f).is_err(), "accepted {name} offset {offset}");
        }
    }

    #[test]
    fn zero_tail_costs_are_retained_without_interpreting_them_as_free_levels() {
        let book = book();
        assert_eq!(book.experience(3).unwrap().jp, [0; 4]);
        assert_eq!(book.summon_experience(3).unwrap().growth, 0);
        assert_eq!(book.summon_experience(3).unwrap().evolve, 0);
        assert_eq!(from_files(&book.to_dbc_files()).unwrap(), book);
    }

    #[test]
    fn open_requires_all_companions_and_preserves_exact_values() {
        let path = std::env::temp_dir().join(format!(
            "rz-progression-test-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir(&path).unwrap();
        struct Cleanup(PathBuf);
        impl Drop for Cleanup {
            fn drop(&mut self) {
                for name in ["BasicStat.dbc", "Exp.dbc", "SummonExp.dbc"] {
                    let _ = std::fs::remove_file(self.0.join("Progression").join(name));
                }
                let _ = std::fs::remove_dir(self.0.join("Progression"));
                let _ = std::fs::remove_dir(&self.0);
            }
        }
        let _cleanup = Cleanup(path.clone());
        assert!(ProgressionBook::open(&path).is_err());
        std::fs::create_dir(path.join("Progression")).unwrap();
        let book = book();
        let files = book.to_dbc_files();
        for name in ["BasicStat.dbc", "Exp.dbc"] {
            std::fs::write(path.join("Progression").join(name), &files[name]).unwrap();
            assert!(
                ProgressionBook::open(&path).is_err(),
                "partial companion set was accepted"
            );
        }
        std::fs::write(
            path.join("Progression/SummonExp.dbc"),
            &files["SummonExp.dbc"],
        )
        .unwrap();
        assert_eq!(ProgressionBook::open(&path).unwrap(), book);
    }
}
