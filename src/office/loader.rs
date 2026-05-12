use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use super::config::OfficeConfig;

/// Resolves the live office.ron path.
///
/// Lookup order:
///   1. $CLANKERTOPIA_OFFICE if set
///   2. ./office.ron next to the binary's current working directory
///   3. $XDG_CONFIG_HOME/clankertopia/office.ron (or platform equivalent)
///
/// The first existing file wins for *reading*. For *writing* (new file), we
/// prefer the cwd location to keep setups portable; fall back to XDG only if
/// the cwd is not writable.
pub fn config_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();
    if let Ok(p) = std::env::var("CLANKERTOPIA_OFFICE") {
        paths.push(PathBuf::from(p));
    }
    paths.push(PathBuf::from("office.ron"));
    if let Some(cfg) = dirs::config_dir() {
        paths.push(cfg.join("clankertopia").join("office.ron"));
    }
    paths
}

pub fn resolve_existing() -> Option<PathBuf> {
    config_paths().into_iter().find(|p| p.exists())
}

/// Where to *write* a new file when none exists yet. Prefers cwd.
pub fn resolve_write_target() -> PathBuf {
    let candidates = config_paths();
    // CLANKERTOPIA_OFFICE wins if set; otherwise cwd.
    candidates
        .into_iter()
        .next()
        .unwrap_or_else(|| PathBuf::from("office.ron"))
}

pub fn load_or_init() -> Result<(OfficeConfig, PathBuf)> {
    if let Some(path) = resolve_existing() {
        let text = fs::read_to_string(&path)
            .with_context(|| format!("reading {}", path.display()))?;
        let cfg: OfficeConfig = ron::from_str(&text)
            .with_context(|| format!("parsing {}", path.display()))?;
        eprintln!("[clankertopia] loaded office from {}", path.display());
        return Ok((cfg, path));
    }

    let cfg = OfficeConfig::default_template();
    let path = resolve_write_target();
    save(&cfg, &path)?;
    eprintln!(
        "[clankertopia] no office.ron found; wrote default to {}",
        path.display()
    );
    Ok((cfg, path))
}

pub fn save(cfg: &OfficeConfig, path: &Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent).ok();
        }
    }
    let pretty = ron::ser::PrettyConfig::new()
        .depth_limit(8)
        .new_line("\n".to_string())
        .indentor("    ".to_string())
        .separate_tuple_members(false)
        .enumerate_arrays(false);
    let text = ron::ser::to_string_pretty(cfg, pretty).context("serializing office config")?;

    let tmp = path.with_extension("ron.tmp");
    {
        let mut f = fs::File::create(&tmp)
            .with_context(|| format!("creating {}", tmp.display()))?;
        f.write_all(text.as_bytes())?;
        f.sync_all().ok();
    }
    fs::rename(&tmp, path)
        .with_context(|| format!("renaming {} -> {}", tmp.display(), path.display()))?;
    Ok(())
}
