//! Updates from the project's GitHub Releases: find the newest release, download its setup
//! file and run it. The installer keeps %APPDATA%\Cheetix (settings, saved originals).

use serde_json::Value;

const REPO: &str = "iiEKiLLeR/Cheetix";

#[derive(Clone)]
pub struct Release {
    /// e.g. "0.4.0"
    pub version: String,
    url: String,
    name: String,
    /// "sha256:<hex>" as GitHub reports it for the asset, when it does
    digest: String,
}

/// "v0.10.2" -> [0, 10, 2]; anything after a number (e.g. "-beta") is ignored.
fn parse(v: &str) -> Vec<u32> {
    v.trim().trim_start_matches(['v', 'V']).split('.').map(|p| p.chars().take_while(char::is_ascii_digit).collect::<String>().parse().unwrap_or(0)).collect()
}

/// True only when `latest` is strictly newer, so an update never downgrades.
pub fn newer(latest: &str, current: &str) -> bool {
    let (mut a, mut b) = (parse(latest), parse(current));
    let n = a.len().max(b.len());
    a.resize(n, 0);
    b.resize(n, 0);
    a > b
}

fn ps_out(script: &str, secs: u64) -> Result<String, String> {
    let o = crate::sys::ps_timeout(script, secs)?;
    let (out, err) = (String::from_utf8_lossy(&o.stdout).trim().to_string(), String::from_utf8_lossy(&o.stderr).trim().to_string());
    if o.status.success() {
        Ok(out)
    } else {
        // PowerShell puts the useful part of an error on its first line
        Err(err.lines().next().unwrap_or("PowerShell failed").to_string())
    }
}

const PRELUDE: &str = "$ErrorActionPreference = 'Stop'; $ProgressPreference = 'SilentlyContinue'; [Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12\n";

/// The latest GitHub release, if it is newer than this build.
pub fn check() -> Result<Option<Release>, String> {
    latest().map(|r| r.filter(|r| newer(&r.version, env!("CARGO_PKG_VERSION"))))
}

/// The latest GitHub release with its setup file (None: no newer-looking release to offer).
fn latest() -> Result<Option<Release>, String> {
    let script = format!(
        "{PRELUDE}$r = Invoke-RestMethod -UseBasicParsing -Headers @{{ 'User-Agent' = 'Cheetix-updater' }} 'https://api.github.com/repos/{REPO}/releases/latest'\n\
         $a = @($r.assets | Where-Object {{ $_.name -like 'Cheetix-Setup-*.exe' }})[0]\n\
         [pscustomobject]@{{ tag = $r.tag_name; url = $a.browser_download_url; name = $a.name; digest = $a.digest }} | ConvertTo-Json -Compress"
    );
    let v: Value = serde_json::from_str(&ps_out(&script, 30)?).map_err(|e| e.to_string())?;
    let s = |k: &str| v[k].as_str().unwrap_or_default().to_string();
    let r = Release { version: s("tag").trim_start_matches(['v', 'V']).to_string(), url: s("url"), name: s("name"), digest: s("digest") };
    if r.url.is_empty() {
        return Err(format!("Cheetix {} has no installer attached on GitHub yet", r.version));
    }
    Ok(Some(r))
}

/// Downloads the release's installer, checks it, and starts it (asks for administrator rights).
/// The installer closes Cheetix, keeps its data, and starts the new version when it is done.
pub fn install(r: &Release) -> Result<(), String> {
    // only this repository's release files, and nothing that could break out of the script's quotes
    let safe = |s: &str| !s.contains(['\'', '"', '`', '$', '\n', '\r']);
    let ok_url = r.url.starts_with(&format!("https://github.com/{REPO}/releases/download/"));
    let ok_name = r.name.starts_with("Cheetix-Setup-") && r.name.ends_with(".exe") && !r.name.contains(['\\', '/']);
    if !(ok_url && ok_name && safe(&r.url) && safe(&r.name) && safe(&r.digest)) {
        return Err("The release on GitHub doesn't look like a Cheetix installer, so it was not downloaded.".into());
    }
    let script = format!(
        "{PRELUDE}$f = Join-Path $env:TEMP '{name}'\n\
         try {{ Invoke-WebRequest -UseBasicParsing -Uri '{url}' -OutFile $f }} catch {{ throw \"Download failed: $($_.Exception.Message)\" }}\n\
         if ('{digest}' -like 'sha256:*') {{ if (('sha256:' + (Get-FileHash $f -Algorithm SHA256).Hash.ToLower()) -ne '{digest}') {{ Remove-Item $f; throw 'The download is damaged (checksum mismatch). Try again later.' }} }}\n\
         try {{ Start-Process -FilePath $f -ArgumentList '/SILENT', '/NORESTART', '/RELAUNCH=1' -Verb RunAs }} catch {{ throw 'The installer was not started (administrator rights were refused).' }}",
        name = r.name,
        url = r.url,
        digest = r.digest
    );
    ps_out(&script, 600).map(|_| ())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn versions_compare_by_number_and_never_downgrade() {
        assert!(newer("v0.4.0", "0.3.0"));
        assert!(newer("0.10.0", "0.9.9"));
        assert!(newer("1.0", "0.99.99"));
        assert!(!newer("v0.3.0", "0.3.0"));
        assert!(!newer("0.2.9", "0.3.0"));
        assert!(!newer("0.3", "0.3.0"));
        assert!(!newer("", "0.3.0"));
    }

    /// Reads the live latest release (network). `cargo test -- --ignored`.
    #[test]
    #[ignore]
    fn latest_release_has_an_installer() {
        let r = latest().unwrap().unwrap();
        assert!(r.name.starts_with("Cheetix-Setup-") && r.digest.starts_with("sha256:"), "{} {}", r.name, r.digest);
    }

    /// Really updates this PC to the latest release (download, check, install, restart).
    /// Run by hand only: `cargo test live_update -- --ignored --exact update::tests::live_update`.
    #[test]
    #[ignore]
    fn live_update() {
        install(&latest().unwrap().unwrap()).unwrap();
    }
}
