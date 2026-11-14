//! Arena: finds installed Steam games and runs a game session with per-game settings. The
//! session itself is `arena.ps1`, started hidden; it reports through arena-status.json in the
//! Cheetix data folder and stops when arena.stop appears there.

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::path::{Path, PathBuf};

pub const CPU: [(&str, &str); 3] = [("Normal", "Normal"), ("AboveNormal", "Above normal"), ("High", "High")];
pub const GPU: [(&str, &str); 3] = [("Normal", "Normal"), ("AboveNormal", "Above normal"), ("High", "High")];
pub const IO: [(&str, &str); 2] = [("Normal", "Normal"), ("High", "High")];
/// services "Pause background services" stops for the session (only those running)
pub const SERVICES: [&str; 6] = ["SysMain", "DiagTrack", "Spooler", "MapsBroker", "WerSvc", "PcaSvc"];

#[derive(Clone)]
pub struct Game {
    pub appid: String,
    pub name: String,
    /// steamapps\common\<installdir>: every process started from here counts as the game
    pub folder: PathBuf,
}

/// What a session does for one game (saved per game).
#[derive(Serialize, Deserialize, Clone, PartialEq)]
#[serde(default)]
pub struct Opts {
    pub boost: bool,
    pub cpu: String,
    pub gpu: String,
    pub io: String,
    pub focus: bool,
    pub services: bool,
    pub search: bool,
    pub latency: bool,
    pub pin: bool,
    pub standby: bool,
    /// power plan GUID to switch to, if any
    pub plan: Option<String>,
    pub close_app: bool,
}

impl Default for Opts {
    fn default() -> Self {
        Opts { boost: true, cpu: "AboveNormal".into(), gpu: "High".into(), io: "Normal".into(), focus: true, services: true, search: false, latency: false, pin: false, standby: false, plan: None, close_app: false }
    }
}

#[derive(Deserialize, Default, Clone)]
#[serde(default)]
pub struct Proc {
    pub pid: u32,
    pub name: String,
    pub cpu: String,
    pub gpu: String,
    pub io: String,
}

/// The session's last report.
#[derive(Deserialize, Default, Clone)]
#[serde(default)]
pub struct Status {
    /// starting / waiting / active / ended
    pub phase: String,
    pub game: String,
    pub appid: String,
    /// the session's PowerShell process
    pub pid: u32,
    pub msg: String,
    pub started: String,
    pub procs: Vec<Proc>,
    pub notes: Vec<String>,
}

impl Status {
    pub fn running(&self) -> bool {
        matches!(self.phase.as_str(), "starting" | "waiting" | "active")
    }
}

/// Whether a process is still running.
pub fn alive(pid: u32) -> bool {
    use windows_sys::Win32::Foundation::{CloseHandle, STILL_ACTIVE};
    use windows_sys::Win32::System::Threading::{GetExitCodeProcess, OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION};
    if pid == 0 {
        return false;
    }
    // SAFETY: plain Win32 calls on a handle we open and close here
    unsafe {
        let h = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid);
        if h.is_null() {
            return false;
        }
        let mut code = 0u32;
        let ok = GetExitCodeProcess(h, &mut code) != 0;
        CloseHandle(h);
        ok && code == STILL_ACTIVE as u32
    }
}

fn dir() -> PathBuf {
    std::env::var("APPDATA").map(PathBuf::from).unwrap_or_else(|_| std::env::temp_dir()).join("Cheetix")
}

/// `"key"  "value"` lines of a Valve KeyValues file (all levels, in order).
fn kv(text: &str) -> Vec<(String, String)> {
    text.lines()
        .filter_map(|l| {
            let parts: Vec<&str> = l.split('"').collect();
            // ["", key, ws, value, ""]
            (parts.len() >= 5).then(|| (parts[1].to_lowercase(), parts[3].replace("\\\\", "\\")))
        })
        .collect()
}

fn steam_root() -> Option<PathBuf> {
    let out = crate::sys::cmd_raw(r"reg query HKCU\Software\Valve\Steam /v SteamPath").ok()?;
    let s = String::from_utf8_lossy(&out.stdout).to_string();
    let p = s.lines().find(|l| l.contains("SteamPath"))?.split("REG_SZ").nth(1)?.trim().replace('/', "\\");
    Some(PathBuf::from(p)).filter(|p| p.exists())
}

/// Installed Steam games, sorted by name (tools and runtimes left out).
pub fn scan() -> Result<Vec<Game>, String> {
    let root = steam_root().or_else(|| Some(PathBuf::from(r"C:\Program Files (x86)\Steam")).filter(|p| p.exists())).ok_or("Steam isn't installed (or has never been started) on this PC.")?;
    let mut libs = vec![root.clone()];
    if let Ok(t) = std::fs::read_to_string(root.join("steamapps").join("libraryfolders.vdf")) {
        libs.extend(kv(&t).into_iter().filter(|(k, _)| k == "path").map(|(_, v)| PathBuf::from(v)));
    }
    libs.sort();
    libs.dedup_by(|a, b| a.to_string_lossy().eq_ignore_ascii_case(&b.to_string_lossy()));
    let mut games = vec![];
    for lib in libs {
        let Ok(rd) = std::fs::read_dir(lib.join("steamapps")) else { continue };
        for e in rd.flatten() {
            let f = e.file_name().to_string_lossy().to_string();
            if !(f.starts_with("appmanifest_") && f.ends_with(".acf")) {
                continue;
            }
            let Ok(t) = std::fs::read_to_string(e.path()) else { continue };
            if let Some(g) = game_from_manifest(&t, &lib) {
                games.push(g);
            }
        }
    }
    games.sort_by_key(|g| g.name.to_lowercase());
    games.dedup_by(|a, b| a.appid == b.appid);
    Ok(games)
}

fn game_from_manifest(t: &str, lib: &Path) -> Option<Game> {
    let m = kv(t);
    let get = |k: &str| m.iter().find(|(a, _)| a == k).map(|(_, v)| v.clone()).unwrap_or_default();
    let (appid, name, dir) = (get("appid"), get("name"), get("installdir"));
    let tool = ["Redistributable", "Proton", "Steam Linux Runtime", "Steamworks", "SteamVR"].iter().any(|w| name.contains(w));
    let folder = lib.join("steamapps").join("common").join(&dir);
    (!appid.is_empty() && !name.is_empty() && !dir.is_empty() && !tool && folder.exists()).then_some(Game { appid, name, folder })
}

fn spawn(mode: &str, config: &Value) -> Result<(), String> {
    let d = dir();
    std::fs::create_dir_all(&d).map_err(|e| e.to_string())?;
    let script = d.join("arena.ps1");
    std::fs::write(&script, include_str!("arena.ps1")).map_err(|e| e.to_string())?;
    let cfg = d.join(format!("arena-{mode}.json"));
    std::fs::write(&cfg, config.to_string()).map_err(|e| e.to_string())?;
    let mut cmd = std::process::Command::new("powershell");
    cmd.args(["-NoProfile", "-NonInteractive", "-ExecutionPolicy", "Bypass", "-File"]).arg(&script).arg("-Config").arg(&cfg).args(["-Mode", mode]);
    // runs on after Cheetix closes
    crate::sys::hidden(&mut cmd).spawn().map(|_| ()).map_err(|e| format!("couldn't start the session: {e}"))
}

pub fn start(g: &Game, o: &Opts) -> Result<(), String> {
    let _ = std::fs::remove_file(dir().join("arena-status.json"));
    let cfg = json!({
        "dir": dir(), "appid": g.appid, "name": g.name, "folder": g.folder,
        "boost": o.boost, "cpu": o.cpu, "gpu": o.gpu, "io": o.io, "focus": o.focus,
        "services": o.services, "service_list": SERVICES, "search": o.search,
        "latency": o.latency, "pin": o.pin, "standby": o.standby, "plan": o.plan,
    });
    spawn("run", &cfg)
}

/// Asks the running session to stop; it puts everything back itself.
pub fn stop() {
    let _ = std::fs::write(dir().join("arena.stop"), "");
}

/// After a crash or power loss mid-session, put back what that session changed.
pub fn recover_if_needed() {
    if dir().join("arena-guard.json").exists() {
        let _ = spawn("recover", &json!({ "dir": dir(), "name": "", "appid": "" }));
    }
}

pub fn status() -> Option<Status> {
    let t = std::fs::read_to_string(dir().join("arena-status.json")).ok()?;
    serde_json::from_str(t.trim_start_matches('\u{feff}')).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_steam_library_files() {
        let lib = "\"libraryfolders\"\n{\n\t\"0\"\n\t{\n\t\t\"path\"\t\t\"C:\\\\Program Files (x86)\\\\Steam\"\n\t}\n\t\"1\"\n\t{\n\t\t\"path\"\t\t\"D:\\\\SteamLibrary\"\n\t\t\"apps\"\n\t\t{\n\t\t\t\"730\"\t\t\"123\"\n\t\t}\n\t}\n}";
        let paths: Vec<String> = kv(lib).into_iter().filter(|(k, _)| k == "path").map(|(_, v)| v).collect();
        assert_eq!(paths, [r"C:\Program Files (x86)\Steam", r"D:\SteamLibrary"]);

        let tmp = std::env::temp_dir().join("cheetix_arena_test");
        let _ = std::fs::create_dir_all(tmp.join("steamapps").join("common").join("Counter-Strike Global Offensive"));
        let acf = "\"AppState\"\n{\n\t\"appid\"\t\t\"730\"\n\t\"name\"\t\t\"Counter-Strike 2\"\n\t\"installdir\"\t\t\"Counter-Strike Global Offensive\"\n}";
        let g = game_from_manifest(acf, &tmp).unwrap();
        assert_eq!((g.appid.as_str(), g.name.as_str()), ("730", "Counter-Strike 2"));
        let redist = acf.replace("Counter-Strike 2", "Steamworks Common Redistributables");
        assert!(game_from_manifest(&redist, &tmp).is_none());
        let _ = std::fs::remove_dir_all(tmp);
    }

    /// The session script must parse (a syntax error would only show up when a game starts).
    #[test]
    fn session_script_parses() {
        let p = std::env::temp_dir().join("cheetix_arena_parse.ps1");
        std::fs::write(&p, include_str!("arena.ps1")).unwrap();
        let o = crate::sys::ps(&format!("$e = $null; [void][System.Management.Automation.Language.Parser]::ParseFile('{}', [ref]$null, [ref]$e); $e.Count; $e | ForEach-Object {{ $_.Message }}", p.display())).unwrap();
        let _ = std::fs::remove_file(p);
        let out = String::from_utf8_lossy(&o.stdout);
        assert_eq!(out.lines().next().map(str::trim), Some("0"), "{out}");
    }

    /// Lists the Steam games on this PC (read-only). `cargo test scan_steam -- --ignored --nocapture`
    #[test]
    #[ignore]
    fn scan_steam() {
        for g in scan().unwrap() {
            println!("{} {} {}", g.appid, g.name, g.folder.display());
        }
    }
}

#[cfg(test)]
mod live {
    use super::*;
    use std::time::{Duration, Instant};

    fn snapshot() -> String {
        let o = crate::sys::ps(
            "$s = (Get-Service SysMain,DiagTrack,Spooler,MapsBroker,WerSvc,PcaSvc,WSearch -ErrorAction SilentlyContinue | ForEach-Object { \"$($_.Name)=$($_.Status)\" }) -join ' '; \
             $t = (Get-NetTCPSetting -SettingName Internet).AutoTuningLevelLocal; \
             $m = @((powercfg /qh scheme_current SUB_PROCESSOR PROCTHROTTLEMIN) -match ': *0x') | Select-Object -Last 2; \
             $low = @(Get-Process | Where-Object { try { $_.PriorityClass -eq 'BelowNormal' } catch { $false } }).Count; \
             \"$s | tcp=$t | min=$($m -join ',') | belownormal=$low\"",
        )
        .unwrap();
        String::from_utf8_lossy(&o.stdout).trim().to_string()
    }

    /// A whole session against a stand-in game (a copy of cmd.exe in a temp folder), every
    /// option on except the power plan: it must boost the game and put everything back.
    /// Changes this PC for ~30 s; run as administrator: `cargo test live_session -- --ignored --nocapture`
    #[test]
    #[ignore]
    fn live_session() {
        let folder = std::env::temp_dir().join("cheetix_arena_fakegame");
        let _ = std::fs::create_dir_all(&folder);
        let exe = folder.join("fakegame.exe");
        std::fs::copy(r"C:\Windows\System32\cmd.exe", &exe).unwrap();
        let before = snapshot();
        println!("before: {before}");
        let mut game = std::process::Command::new(&exe).args(["/c", "ping -n 30 127.0.0.1 >nul"]).spawn().unwrap();
        let g = Game { appid: "0".into(), name: "Arena test".into(), folder: folder.clone() };
        let o = Opts { boost: true, cpu: "High".into(), gpu: "High".into(), io: "High".into(), focus: true, services: true, search: true, latency: true, pin: true, standby: true, plan: None, close_app: false };
        start(&g, &o).unwrap();
        let t = Instant::now();
        let mut live = None;
        while t.elapsed() < Duration::from_secs(60) {
            std::thread::sleep(Duration::from_secs(1));
            if let Some(s) = status().filter(|s| s.phase == "active") {
                live = Some(s);
                break;
            }
        }
        let live = live.expect("session never went live");
        println!("during: {}", snapshot());
        for p in &live.procs {
            println!("boosted {} {}: cpu {} gpu {} io {}", p.name, p.pid, p.cpu, p.gpu, p.io);
        }
        for n in &live.notes {
            println!("note: {n}");
        }
        assert!(live.procs.iter().any(|p| p.name == "fakegame" && p.cpu == "High"));
        let _ = game.kill();
        let t = Instant::now();
        let mut end = None;
        while t.elapsed() < Duration::from_secs(60) {
            std::thread::sleep(Duration::from_secs(1));
            if let Some(s) = status().filter(|s| s.phase == "ended") {
                end = Some(s);
                break;
            }
        }
        println!("ended: {}", end.expect("session never ended").msg);
        std::thread::sleep(Duration::from_secs(5)); // services take a moment to start
        let after = snapshot();
        println!("after:  {after}");
        let _ = std::fs::remove_dir_all(folder);
        assert_eq!(before, after);
        assert!(!dir().join("arena-guard.json").exists());
    }
}
