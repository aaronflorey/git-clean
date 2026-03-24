use crate::error::Error;
use crate::options::{ColorMode, DeleteMode, Options};
use directories::BaseDirs;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

const CONFIG_FILE_NAME: &str = "config.toml";

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct RepoConfig {
    pub remote: Option<String>,
    pub base_branch: Option<String>,
    pub squashes: Option<bool>,
    pub delete_unpushed_branches: Option<bool>,
    pub ignored_branches: Option<Vec<String>>,
    pub delete_mode: Option<DeleteModeConfig>,
    pub color_mode: Option<ColorModeConfig>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DeleteModeConfig {
    Local,
    Remote,
    Both,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ColorModeConfig {
    Auto,
    Always,
    Never,
}

impl ColorModeConfig {
    fn from_color_mode(mode: &ColorMode) -> Self {
        match mode {
            ColorMode::Auto => ColorModeConfig::Auto,
            ColorMode::Always => ColorModeConfig::Always,
            ColorMode::Never => ColorModeConfig::Never,
        }
    }

    fn to_color_mode(self) -> ColorMode {
        match self {
            ColorModeConfig::Auto => ColorMode::Auto,
            ColorModeConfig::Always => ColorMode::Always,
            ColorModeConfig::Never => ColorMode::Never,
        }
    }
}

impl DeleteModeConfig {
    fn from_delete_mode(mode: &DeleteMode) -> Self {
        match mode {
            DeleteMode::Local => DeleteModeConfig::Local,
            DeleteMode::Remote => DeleteModeConfig::Remote,
            DeleteMode::Both => DeleteModeConfig::Both,
        }
    }

    fn to_delete_mode(self) -> DeleteMode {
        match self {
            DeleteModeConfig::Local => DeleteMode::Local,
            DeleteModeConfig::Remote => DeleteMode::Remote,
            DeleteModeConfig::Both => DeleteMode::Both,
        }
    }
}

impl RepoConfig {
    pub fn is_empty(&self) -> bool {
        self.remote.is_none()
            && self.base_branch.is_none()
            && self.squashes.is_none()
            && self.delete_unpushed_branches.is_none()
            && self.ignored_branches.is_none()
            && self.delete_mode.is_none()
            && self.color_mode.is_none()
    }

    pub fn apply_to_options(&self, options: &mut Options) {
        if let Some(remote) = self.remote.as_ref() {
            options.remote = remote.clone();
        }
        if let Some(base_branch) = self.base_branch.as_ref() {
            options.base_branch = base_branch.clone();
        }
        if let Some(squashes) = self.squashes {
            options.squashes = squashes;
        }
        if let Some(delete_unpushed_branches) = self.delete_unpushed_branches {
            options.delete_unpushed_branches = delete_unpushed_branches;
        }
        if let Some(ignored_branches) = self.ignored_branches.as_ref() {
            options.ignored_branches = ignored_branches.clone();
        }
        if let Some(delete_mode) = self.delete_mode {
            options.delete_mode = delete_mode.to_delete_mode();
        }
        if let Some(color_mode) = self.color_mode {
            options.color_mode = color_mode.to_color_mode();
        }
    }

    fn merge_patch(&mut self, patch: &RepoConfig) {
        if let Some(remote) = patch.remote.as_ref() {
            self.remote = Some(remote.clone());
        }
        if let Some(base_branch) = patch.base_branch.as_ref() {
            self.base_branch = Some(base_branch.clone());
        }
        if let Some(squashes) = patch.squashes {
            self.squashes = Some(squashes);
        }
        if let Some(delete_unpushed_branches) = patch.delete_unpushed_branches {
            self.delete_unpushed_branches = Some(delete_unpushed_branches);
        }
        if let Some(ignored_branches) = patch.ignored_branches.as_ref() {
            self.ignored_branches = Some(ignored_branches.clone());
        }
        if let Some(delete_mode) = patch.delete_mode {
            self.delete_mode = Some(delete_mode);
        }
        if let Some(color_mode) = patch.color_mode {
            self.color_mode = Some(color_mode);
        }
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(transparent)]
struct ConfigFile {
    repos: BTreeMap<String, RepoConfig>,
}

pub fn delete_mode_to_config(mode: &DeleteMode) -> DeleteModeConfig {
    DeleteModeConfig::from_delete_mode(mode)
}

pub fn color_mode_to_config(mode: &ColorMode) -> ColorModeConfig {
    ColorModeConfig::from_color_mode(mode)
}

pub fn load_repo_config(repo_path: &Path) -> Result<Option<RepoConfig>, Error> {
    let path = config_path()?;
    load_repo_config_at_path(repo_path, &path)
}

pub fn save_repo_config(repo_path: &Path, patch: &RepoConfig) -> Result<PathBuf, Error> {
    let path = config_path()?;
    save_repo_config_at_path(repo_path, patch, &path)?;
    Ok(path)
}

fn config_path() -> Result<PathBuf, Error> {
    let dirs = BaseDirs::new()
        .ok_or_else(|| Error::Config("Unable to determine the config directory".into()))?;
    Ok(dirs
        .home_dir()
        .join(".config")
        .join("git-clean")
        .join(CONFIG_FILE_NAME))
}

fn canonical_repo_key(repo_path: &Path) -> Result<String, Error> {
    Ok(repo_path.canonicalize()?.to_string_lossy().into_owned())
}

fn load_repo_config_at_path(
    repo_path: &Path,
    config_path: &Path,
) -> Result<Option<RepoConfig>, Error> {
    let config = load_config_file(config_path)?;
    let repo_key = canonical_repo_key(repo_path)?;
    Ok(config.repos.get(&repo_key).cloned())
}

fn save_repo_config_at_path(
    repo_path: &Path,
    patch: &RepoConfig,
    config_path: &Path,
) -> Result<(), Error> {
    let mut config = load_config_file(config_path)?;
    let repo_key = canonical_repo_key(repo_path)?;
    let entry = config.repos.entry(repo_key).or_default();
    entry.merge_patch(patch);

    if let Some(parent) = config_path.parent() {
        fs::create_dir_all(parent)?;
    }
    let serialized = toml::to_string_pretty(&config)
        .map_err(|e| Error::Config(format!("Invalid config: {e}")))?;
    fs::write(config_path, serialized)?;
    Ok(())
}

fn load_config_file(path: &Path) -> Result<ConfigFile, Error> {
    if !path.exists() {
        return Ok(ConfigFile::default());
    }
    let body = fs::read_to_string(path)?;
    let parsed: ConfigFile = toml::from_str(&body)
        .map_err(|e| Error::Config(format!("Unable to parse {}: {e}", path.display())))?;
    Ok(parsed)
}

#[cfg(test)]
mod test {
    use super::{load_repo_config_at_path, save_repo_config_at_path, RepoConfig};
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_save_and_load_repo_config() {
        let root = TempDir::new().unwrap();
        let repo_path = root.path().join("repo");
        let config_path = root.path().join("config").join("config.toml");
        fs::create_dir_all(&repo_path).unwrap();

        let patch = RepoConfig {
            squashes: Some(true),
            ignored_branches: Some(vec!["stale".to_owned()]),
            ..RepoConfig::default()
        };

        save_repo_config_at_path(&repo_path, &patch, &config_path).unwrap();
        let loaded = load_repo_config_at_path(&repo_path, &config_path)
            .unwrap()
            .unwrap();

        assert_eq!(loaded.squashes, Some(true));
        assert_eq!(loaded.ignored_branches, Some(vec!["stale".to_owned()]));
    }
}
