//! Configuration files (.cheetix.json): which tweaks are on, by page, plus Arena's per-game
//! tuning. Export one on a PC, import it on another to reproduce the setup. Importing only turns
//! tweaks on; it never turns anything off.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::PathBuf;

/// The newest file layout this version reads (and the one it writes).
pub const FORMAT: u32 = 1;

#[derive(Serialize, Deserialize, Default, Clone, PartialEq)]
#[serde(default)]
pub struct Config {
    pub format: u32,
    /// what it is, e.g. "My setup" or "Recommended (Moderate Tweaks)"
    pub name: String,
    /// the Cheetix version that wrote it
    pub cheetix: String,
    /// the Windows build of the PC it came from
    pub windows: String,
    pub created: String,
    /// page category -> ids of the tweaks that are on
    pub tweaks: BTreeMap<String, Vec<String>>,
    /// Arena tuning per game id
    pub arena: BTreeMap<String, crate::arena::Opts>,
}

impl Config {
    pub fn count(&self) -> usize {
        self.tweaks.values().map(Vec::len).sum()
    }
}

/// Reads a configuration file's text, refusing anything that isn't one.
pub fn parse(text: &str) -> Result<Config, String> {
    let c: Config = serde_json::from_str(text.trim_start_matches('\u{feff}')).map_err(|e| format!("This isn't a Cheetix configuration file ({e})."))?;
    if c.format == 0 {
        return Err("This isn't a Cheetix configuration file (no format version).".into());
    }
    if c.format > FORMAT {
        return Err(format!("This file was made by a newer Cheetix ({}). Update Cheetix to import it.", c.cheetix));
    }
    if c.count() == 0 && c.arena.is_empty() {
        return Err("This configuration has no settings in it.".into());
    }
    Ok(c)
}

fn dialog(save: bool, name: &str) -> Result<Option<PathBuf>, String> {
    let script = format!(
        r#"Add-Type -AssemblyName System.Windows.Forms
$owner = New-Object Windows.Forms.Form -Property @{{ TopMost = $true; ShowInTaskbar = $false }}
$d = New-Object Windows.Forms.{kind}FileDialog -Property @{{ Title = '{title}'; Filter = 'Cheetix configuration (*.cheetix.json)|*.cheetix.json|JSON files (*.json)|*.json'; InitialDirectory = [Environment]::GetFolderPath('MyDocuments'); FileName = '{name}' }}
if ($d.ShowDialog($owner) -eq 'OK') {{ $d.FileName }}"#,
        kind = if save { "Save" } else { "Open" },
        title = if save { "Save a Cheetix configuration" } else { "Import a Cheetix configuration" },
        name = name.replace('\'', "''"),
    );
    let o = crate::sys::ps_timeout(&script, 900)?;
    let out = String::from_utf8_lossy(&o.stdout).trim().to_string();
    Ok((!out.is_empty()).then(|| PathBuf::from(out)))
}

/// Asks where to save, then writes the file. Ok(None): cancelled.
pub fn export(c: &Config, file_name: &str) -> Result<Option<PathBuf>, String> {
    let Some(path) = dialog(true, file_name)? else { return Ok(None) };
    let text = serde_json::to_string_pretty(c).map_err(|e| e.to_string())?;
    std::fs::write(&path, text).map_err(|e| format!("Couldn't save {}: {e}", path.display()))?;
    Ok(Some(path))
}

/// Asks for a file and reads it. Ok(None): cancelled.
pub fn import() -> Result<Option<(PathBuf, Config)>, String> {
    let Some(path) = dialog(false, "")? else { return Ok(None) };
    let text = std::fs::read_to_string(&path).map_err(|e| format!("Couldn't read {}: {e}", path.display()))?;
    parse(&text).map(|c| Some((path, c)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip_and_bad_files() {
        let mut c = Config { format: FORMAT, name: "My setup".into(), cheetix: "0.3.9".into(), windows: "10.0.26200".into(), ..Default::default() };
        c.tweaks.insert("privacy".into(), vec!["activity_history_policy".into()]);
        c.arena.insert("730".into(), crate::arena::Opts { cpu: "High".into(), ..Default::default() });
        let text = serde_json::to_string_pretty(&c).unwrap();
        assert!(parse(&text).ok() == Some(c.clone()));
        assert!(parse(&format!("\u{feff}{text}")).ok() == Some(c), "a BOM from Notepad must not break it");
        assert!(parse("{\"theme\": \"dark\"}").is_err(), "Cheetix's own state.json is not a configuration");
        assert!(parse("not json").is_err());
        assert!(parse(&text.replace("\"format\": 1", "\"format\": 9")).err().unwrap().contains("newer Cheetix"));
        assert!(parse("{\"format\": 1}").err().unwrap().contains("no settings"));
    }
}
