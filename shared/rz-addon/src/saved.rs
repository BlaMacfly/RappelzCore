use std::collections::{BTreeMap, HashSet};
use std::io::Write;
use std::path::{Path, PathBuf};

use anyhow::{bail, ensure, Result};
use mlua::{Lua, Table, Value};
use serde::{Deserialize, Serialize};

const MAX_BYTES: u64 = 1024 * 1024;
const MAX_NODES: usize = 20_000;

/// Tagged values preserve Lua numeric/boolean table keys without executing saved code.
#[derive(Debug, Serialize, Deserialize)]
pub enum SavedValue {
    Nil,
    Bool(bool),
    Number(f64),
    String(String),
    Table(Vec<(SavedValue, SavedValue)>),
}

impl SavedValue {
    fn from_lua(
        value: Value,
        seen: &mut HashSet<usize>,
        budget: &mut usize,
        depth: usize,
    ) -> Result<Self> {
        ensure!(
            depth <= 32 && *budget > 0,
            "SavedVariables exceeds depth/node limit"
        );
        *budget -= 1;
        Ok(match value {
            Value::Nil => Self::Nil,
            Value::Boolean(v) => Self::Bool(v),
            Value::Integer(v) => Self::Number(v as f64),
            Value::Number(v) if v.is_finite() => Self::Number(v),
            Value::String(v) => Self::String(v.to_str()?.to_string()),
            Value::Table(t) => {
                let ptr = t.to_pointer() as usize;
                ensure!(seen.insert(ptr), "cyclic SavedVariables table");
                let mut values = Vec::new();
                for pair in t.pairs::<Value, Value>() {
                    let (k, v) = pair?;
                    ensure!(matches!(k, Value::Boolean(_) | Value::Integer(_) | Value::Number(_) | Value::String(_)), "unsupported SavedVariables key");
                    values.push((Self::from_lua(k, seen, budget, depth + 1)?, Self::from_lua(v, seen, budget, depth + 1)?));
                }
                seen.remove(&ptr);
                Self::Table(values)
            }
            _ => bail!("unsupported SavedVariables value (only finite numbers, UTF-8 strings, booleans and tables)"),
        })
    }

    fn to_lua(&self, lua: &Lua, budget: &mut usize, depth: usize) -> Result<Value> {
        ensure!(
            depth <= 32 && *budget > 0,
            "SavedVariables exceeds depth/node limit"
        );
        *budget -= 1;
        Ok(match self {
            Self::Nil => Value::Nil,
            Self::Bool(v) => Value::Boolean(*v),
            Self::Number(v) => {
                ensure!(v.is_finite(), "non-finite saved number");
                Value::Number(*v)
            }
            Self::String(v) => Value::String(lua.create_string(v)?),
            Self::Table(entries) => {
                let table = lua.create_table()?;
                for (k, v) in entries {
                    ensure!(
                        matches!(k, Self::Bool(_) | Self::Number(_) | Self::String(_)),
                        "invalid saved key"
                    );
                    table.raw_set(
                        k.to_lua(lua, budget, depth + 1)?,
                        v.to_lua(lua, budget, depth + 1)?,
                    )?;
                }
                Value::Table(table)
            }
        })
    }
}

/// Hex-encoded identities cannot create path components, even for non-ASCII account names.
fn identity(value: &str) -> String {
    value
        .as_bytes()
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

pub fn account_dir(root: &Path, account: &str) -> PathBuf {
    root.join("WTF")
        .join("Account")
        .join(identity(account))
        .join("SavedVariables")
}

pub fn character_dir(root: &Path, account: &str, realm: &str, character: &str) -> PathBuf {
    account_dir(root, account)
        .join("Characters")
        .join(identity(realm))
        .join(identity(character))
}

pub fn atomic_json(path: &Path, data: &impl Serialize) -> Result<()> {
    let bytes = serde_json::to_vec_pretty(data)?;
    ensure!(bytes.len() as u64 <= MAX_BYTES, "saved data exceeds 1 MiB");
    let parent = path.parent().expect("saved file has parent");
    std::fs::create_dir_all(parent)?;
    let mut temporary = tempfile::NamedTempFile::new_in(parent)?;
    temporary.write_all(&bytes)?;
    temporary.as_file().sync_all()?;
    temporary.persist(path)?;
    Ok(())
}

pub fn read_json<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T> {
    use std::io::Read;
    let mut bytes = Vec::new();
    std::fs::File::open(path)?
        .take(MAX_BYTES + 1)
        .read_to_end(&mut bytes)?;
    ensure!(bytes.len() as u64 <= MAX_BYTES, "saved data exceeds 1 MiB");
    Ok(serde_json::from_slice(&bytes)?)
}

pub fn restore(lua: &Lua, path: &Path, names: &[String]) -> Result<()> {
    if !path.exists() || names.is_empty() {
        return Ok(());
    }
    let values: BTreeMap<String, SavedValue> = read_json(path)?;
    let globals = lua.globals();
    let mut budget = MAX_NODES;
    for name in names {
        if let Some(value) = values.get(name) {
            globals.raw_set(name.as_str(), value.to_lua(lua, &mut budget, 0)?)?;
        }
    }
    Ok(())
}

pub fn save(lua: &Lua, path: &Path, names: &[String]) -> Result<()> {
    if names.is_empty() {
        return Ok(());
    }
    let globals: Table = lua.globals();
    let mut values = BTreeMap::new();
    let mut budget = MAX_NODES;
    for name in names {
        values.insert(
            name,
            SavedValue::from_lua(
                globals.raw_get(name.as_str())?,
                &mut HashSet::new(),
                &mut budget,
                0,
            )?,
        );
    }
    atomic_json(path, &values)
}
