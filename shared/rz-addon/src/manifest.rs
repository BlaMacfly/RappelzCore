use std::collections::{BTreeMap, BTreeSet};
use std::path::{Component, Path, PathBuf};

use anyhow::{bail, ensure, Context, Result};

pub const INTERFACE_VERSION: u32 = 30300;
pub const MAX_SOURCE_BYTES: u64 = 1024 * 1024;

#[derive(Clone, Debug)]
pub struct Manifest {
    pub name: String,
    pub directory: PathBuf,
    pub metadata: BTreeMap<String, String>,
    pub files: Vec<String>,
    pub dependencies: Vec<String>,
    pub optional_dependencies: Vec<String>,
    pub saved_variables: Vec<String>,
    pub character_variables: Vec<String>,
    pub interface: u32,
    pub load_on_demand: bool,
}

fn list(value: Option<&String>) -> Vec<String> {
    value
        .into_iter()
        .flat_map(|s| s.split([',', ' ', '\t']))
        .filter(|s| !s.is_empty())
        .map(str::to_owned)
        .collect()
}

pub fn identifier(s: &str) -> bool {
    let mut chars = s.chars();
    chars
        .next()
        .is_some_and(|c| c == '_' || c.is_ascii_alphabetic())
        && chars.all(|c| c == '_' || c.is_ascii_alphanumeric())
}

/// Validate before joining; canonical containment also rejects escaping junctions/symlinks.
pub fn contained_file(root: &Path, name: &str) -> Result<PathBuf> {
    let normalized = name.replace('\\', "/");
    let relative = Path::new(&normalized);
    ensure!(
        !normalized.contains(':') && !normalized.contains('\0'),
        "invalid addon path: {name}"
    );
    ensure!(
        !normalized.is_empty()
            && relative
                .components()
                .all(|c| matches!(c, Component::Normal(_))),
        "addon path must be relative without traversal: {name}"
    );
    let root = root.canonicalize()?;
    let file = root
        .join(relative)
        .canonicalize()
        .with_context(|| format!("missing addon file {name}"))?;
    ensure!(
        file.starts_with(&root),
        "addon file escapes its directory: {name}"
    );
    ensure!(file.is_file(), "addon source is not a file: {name}");
    ensure!(
        file.metadata()?.len() <= MAX_SOURCE_BYTES,
        "addon file exceeds 1 MiB: {name}"
    );
    Ok(file)
}

pub fn read_source(root: &Path, name: &str) -> Result<String> {
    use std::io::Read;
    let path = contained_file(root, name)?;
    let mut bytes = Vec::new();
    std::fs::File::open(path)?
        .take(MAX_SOURCE_BYTES + 1)
        .read_to_end(&mut bytes)?;
    ensure!(
        bytes.len() as u64 <= MAX_SOURCE_BYTES,
        "addon file exceeds 1 MiB: {name}"
    );
    Ok(String::from_utf8(bytes)?
        .trim_start_matches('\u{feff}')
        .to_string())
}

impl Manifest {
    pub fn parse(name: &str, directory: PathBuf, source: &str) -> Result<Self> {
        ensure!(
            identifier(name),
            "addon directory must be an ASCII identifier: {name}"
        );
        let mut metadata = BTreeMap::new();
        let mut files = Vec::new();
        for line in source.trim_start_matches('\u{feff}').lines().map(str::trim) {
            if let Some(entry) = line.strip_prefix("##") {
                if let Some((key, value)) = entry.split_once(':') {
                    metadata.insert(key.trim().to_ascii_lowercase(), value.trim().to_string());
                }
            } else if !line.is_empty() && !line.starts_with('#') {
                ensure!(files.len() < 128, "too many addon files");
                files.push(line.to_string());
            }
        }
        let interface = metadata
            .get("interface")
            .context("missing ## Interface")?
            .parse()?;
        let dependencies = list(
            metadata
                .get("dependencies")
                .or_else(|| metadata.get("requireddeps")),
        );
        let optional_dependencies = list(metadata.get("optionaldeps"));
        ensure!(
            dependencies
                .iter()
                .chain(&optional_dependencies)
                .all(|d| identifier(d)),
            "invalid dependency name"
        );
        let saved_variables = list(metadata.get("savedvariables"));
        let character_variables = list(metadata.get("savedvariablespercharacter"));
        ensure!(
            saved_variables
                .iter()
                .chain(&character_variables)
                .all(|v| identifier(v)),
            "invalid SavedVariables name"
        );
        let vars: BTreeSet<_> = saved_variables.iter().chain(&character_variables).collect();
        ensure!(
            vars.len() == saved_variables.len() + character_variables.len(),
            "duplicate SavedVariables name"
        );
        ensure!(vars.len() <= 64, "too many SavedVariables");
        let load_on_demand = metadata.get("loadondemand").is_some_and(|v| v == "1");
        Ok(Self {
            name: name.into(),
            directory,
            metadata,
            files,
            dependencies,
            optional_dependencies,
            saved_variables,
            character_variables,
            interface,
            load_on_demand,
        })
    }

    pub fn title(&self) -> &str {
        self.metadata
            .get("title")
            .map(String::as_str)
            .unwrap_or(&self.name)
    }
}

pub fn load_order(
    manifests: &[Manifest],
    enabled: &BTreeMap<String, bool>,
    requested: &BTreeSet<String>,
) -> Result<Vec<usize>> {
    fn visit(
        i: usize,
        ms: &[Manifest],
        enabled: &BTreeMap<String, bool>,
        marks: &mut [u8],
        out: &mut Vec<usize>,
    ) -> Result<()> {
        let m = &ms[i];
        ensure!(
            enabled.get(&m.name).copied().unwrap_or(true),
            "dependency {} is disabled",
            m.name
        );
        ensure!(
            m.interface == INTERFACE_VERSION,
            "{} requires unsupported Interface {}",
            m.name,
            m.interface
        );
        if marks[i] == 2 {
            return Ok(());
        }
        ensure!(marks[i] == 0, "addon dependency cycle at {}", m.name);
        marks[i] = 1;
        for dep in &m.dependencies {
            let j = ms
                .iter()
                .position(|m| &m.name == dep)
                .with_context(|| format!("{} requires missing addon {dep}", m.name))?;
            visit(j, ms, enabled, marks, out)?;
        }
        for dep in &m.optional_dependencies {
            if let Some(j) = ms.iter().position(|m| &m.name == dep) {
                if enabled.get(dep).copied().unwrap_or(true) {
                    visit(j, ms, enabled, marks, out)?;
                }
            }
        }
        marks[i] = 2;
        out.push(i);
        Ok(())
    }
    let mut out = Vec::new();
    let mut marks = vec![0; manifests.len()];
    for (i, m) in manifests.iter().enumerate() {
        if enabled.get(&m.name).copied().unwrap_or(true)
            && (!m.load_on_demand || requested.contains(&m.name))
        {
            visit(i, manifests, enabled, &mut marks, &mut out)?;
        }
    }
    if out.len() > 32 {
        bail!("at most 32 loaded addons are supported")
    }
    Ok(out)
}
