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
    /// cover art (240x360 PNG) made from Steam's library cache, if Steam has any
    pub art: Option<PathBuf>,
    /// a program the user added (started directly instead of through Steam)
    pub exe: Option<PathBuf>,
}

/// A program added with "Add a program" (saved in Cheetix's state).
#[derive(Serialize, Deserialize, Clone, PartialEq)]
pub struct App {
    pub id: String,
    pub name: String,
    pub exe: PathBuf,
}

impl App {
    pub fn game(&self) -> Game {
        let art = dir().join("art").join(format!("{}.png", self.id));
        Game { appid: self.id.clone(), name: self.name.clone(), folder: self.exe.parent().map(Path::to_path_buf).unwrap_or_default(), art: art.exists().then_some(art), exe: Some(self.exe.clone()) }
    }
}

/// Stable id for a program path (FNV-1a of the lower-cased path).
fn app_id(exe: &Path) -> String {
    let h = exe.to_string_lossy().to_lowercase().bytes().fold(0xcbf2_9ce4_8422_2325_u64, |h, b| (h ^ b as u64).wrapping_mul(0x100_0000_01b3));
    format!("app-{h:016x}")
}

/// Asks for a program (.exe) and makes its cover from the program's own icon. None: cancelled.
pub fn pick_app() -> Result<Option<App>, String> {
    let out = dir().join("art");
    std::fs::create_dir_all(&out).map_err(|e| e.to_string())?;
    let script = r#"Add-Type -AssemblyName System.Windows.Forms, System.Drawing
$owner = New-Object Windows.Forms.Form -Property @{ TopMost = $true; ShowInTaskbar = $false }
$d = New-Object Windows.Forms.OpenFileDialog -Property @{ Title = 'Pick a program for Arena'; Filter = 'Programs (*.exe)|*.exe'; InitialDirectory = ${env:ProgramFiles} }
if ($d.ShowDialog($owner) -ne 'OK') { exit 0 }
$f = $d.FileName
$v = (Get-Item -LiteralPath $f).VersionInfo
$name = @($v.FileDescription, $v.ProductName, [IO.Path]::GetFileNameWithoutExtension($f)) | Where-Object { "$_".Trim() } | Select-Object -First 1
[pscustomobject]@{ exe = $f; name = "$name".Trim() } | ConvertTo-Json -Compress"#;
    let o = crate::sys::ps_timeout(script, 900)?;
    let txt = String::from_utf8_lossy(&o.stdout).trim().to_string();
    if txt.is_empty() {
        return Ok(None);
    }
    let v: Value = serde_json::from_str(&txt).map_err(|e| e.to_string())?;
    let exe = PathBuf::from(v["exe"].as_str().unwrap_or_default());
    if exe.parent().is_none_or(|d| d.parent().is_none()) {
        return Err("Pick a program inside its own folder (not at the root of a drive).".into());
    }
    let app = App { id: app_id(&exe), name: v["name"].as_str().unwrap_or("Program").to_string(), exe };
    app_cover(&app);
    Ok(Some(app))
}

/// A 240x360 cover: the program's large icon on a dark tile.
fn app_cover(a: &App) {
    let dst = dir().join("art").join(format!("{}.png", a.id));
    let script = format!(
        r#"Add-Type -AssemblyName System.Drawing
Add-Type -TypeDefinition 'using System; using System.Runtime.InteropServices; public static class Ico {{ [DllImport("user32.dll", CharSet = CharSet.Unicode)] public static extern uint PrivateExtractIcons(string f, int i, int cx, int cy, IntPtr[] h, uint[] id, uint n, uint fl); }}'
$h = New-Object IntPtr[] 1; $id = New-Object uint32[] 1
[void][Ico]::PrivateExtractIcons('{exe}', 0, 256, 256, $h, $id, 1, 0)
$b = New-Object Drawing.Bitmap 240, 360; $g = [Drawing.Graphics]::FromImage($b)
$g.InterpolationMode = 'HighQualityBicubic'; $g.SmoothingMode = 'HighQuality'
$bg = New-Object Drawing.Drawing2D.LinearGradientBrush (New-Object Drawing.Rectangle 0, 0, 240, 360), ([Drawing.Color]::FromArgb(255, 34, 35, 42)), ([Drawing.Color]::FromArgb(255, 12, 12, 15)), 90
$g.FillRectangle($bg, 0, 0, 240, 360)
if ($h[0] -ne [IntPtr]::Zero) {{ $ic = [Drawing.Icon]::FromHandle($h[0]).ToBitmap(); $g.DrawImage($ic, 52, 112, 136, 136) }}
$b.Save('{dst}', [Drawing.Imaging.ImageFormat]::Png)"#,
        exe = a.exe.display().to_string().replace('\'', "''"),
        dst = dst.display().to_string().replace('\'', "''")
    );
    let _ = crate::sys::ps_timeout(&script, 60);
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
    /// close Steam's web helper while the game runs
    pub steam: bool,
    pub latency: bool,
    pub pin: bool,
    pub standby: bool,
    /// power plan GUID to switch to, if any
    pub plan: Option<String>,
    pub close_app: bool,
}

impl Default for Opts {
    fn default() -> Self {
        Opts { boost: true, cpu: "AboveNormal".into(), gpu: "High".into(), io: "Normal".into(), focus: true, services: true, search: false, steam: false, latency: false, pin: false, standby: false, plan: None, close_app: false }
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

pub fn dir() -> PathBuf {
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
    make_art(&root, &mut games);
    Ok(games)
}

fn game_from_manifest(t: &str, lib: &Path) -> Option<Game> {
    let m = kv(t);
    let get = |k: &str| m.iter().find(|(a, _)| a == k).map(|(_, v)| v.clone()).unwrap_or_default();
    let (appid, name, dir) = (get("appid"), get("name"), get("installdir"));
    let tool = ["Redistributable", "Proton", "Steam Linux Runtime", "Steamworks", "SteamVR"].iter().any(|w| name.contains(w));
    let folder = lib.join("steamapps").join("common").join(&dir);
    (!appid.is_empty() && !name.is_empty() && !dir.is_empty() && !tool && folder.exists()).then_some(Game { appid, name, folder, art: None, exe: None })
}

/// The best picture Steam has for a game: the portrait cover, else the wide header.
fn art_source(cache: &Path, appid: &str) -> Option<PathBuf> {
    let mut files = vec![];
    let base = cache.join(appid);
    for e in std::fs::read_dir(&base).ok()?.flatten() {
        let p = e.path();
        if p.is_dir() {
            files.extend(std::fs::read_dir(&p).into_iter().flatten().flatten().map(|e| e.path()));
        } else {
            files.push(p);
        }
    }
    // older Steam versions keep "<appid>_library_600x900.jpg" next to the folders
    files.extend(["library_600x900.jpg", "header.jpg"].iter().map(|n| cache.join(format!("{appid}_{n}"))));
    ["library_600x900.jpg", "library_capsule.jpg", "library_header.jpg", "header.jpg"]
        .iter()
        .find_map(|want| files.iter().find(|f| f.file_name().is_some_and(|n| n.to_string_lossy().ends_with(want)) && f.exists()).cloned())
}

/// Makes a 240x360 PNG cover per game (once; again when Steam's picture changes). A wide header
/// is fitted onto a darkened, enlarged copy of itself so every cover has the same shape.
fn make_art(root: &Path, games: &mut [Game]) {
    let cache = root.join("appcache").join("librarycache");
    let out = dir().join("art");
    let _ = std::fs::create_dir_all(&out);
    let mut jobs = vec![];
    for g in games.iter() {
        let Some(src) = art_source(&cache, &g.appid) else { continue };
        let dst = out.join(format!("{}.png", g.appid));
        let stale = |d: &Path| match (std::fs::metadata(d).and_then(|m| m.modified()), std::fs::metadata(&src).and_then(|m| m.modified())) {
            (Ok(d), Ok(s)) => d < s,
            _ => true,
        };
        if stale(&dst) {
            jobs.push(json!({ "src": src, "dst": dst }));
        }
    }
    if !jobs.is_empty() {
        let list = out.join("jobs.json");
        if std::fs::write(&list, Value::from(jobs).to_string()).is_ok() {
            let script = format!(
                r#"Add-Type -AssemblyName System.Drawing
foreach ($j in (Get-Content -LiteralPath '{}' -Raw | ConvertFrom-Json)) {{
  try {{
    $img = [Drawing.Image]::FromFile($j.src); $W = 240; $H = 360
    $b = New-Object Drawing.Bitmap $W, $H; $g = [Drawing.Graphics]::FromImage($b)
    $g.InterpolationMode = 'HighQualityBicubic'; $g.SmoothingMode = 'HighQuality'; $g.PixelOffsetMode = 'HighQuality'
    $cover = [Math]::Max($W / $img.Width, $H / $img.Height)
    $cw = $img.Width * $cover; $ch = $img.Height * $cover
    $g.DrawImage($img, [single](($W - $cw) / 2), [single](($H - $ch) / 2), [single]$cw, [single]$ch)
    if ($img.Width / $img.Height -gt 1.0) {{
      $g.FillRectangle((New-Object Drawing.SolidBrush ([Drawing.Color]::FromArgb(150, 8, 8, 10))), 0, 0, $W, $H)
      $fh = $img.Height * $W / $img.Width
      $g.DrawImage($img, 0, [single](($H - $fh) / 2), [single]$W, [single]$fh)
    }}
    $b.Save($j.dst, [Drawing.Imaging.ImageFormat]::Png); $g.Dispose(); $b.Dispose(); $img.Dispose()
  }} catch {{}}
}}"#,
                list.display()
            );
            let _ = crate::sys::ps_timeout(&script, 120);
            let _ = std::fs::remove_file(list);
        }
    }
    for g in games.iter_mut() {
        g.art = Some(out.join(format!("{}.png", g.appid))).filter(|p| p.exists());
    }
}

/// A cover made by `make_art` (RGBA PNG), ready for a texture.
pub fn load_art(path: &Path) -> Option<eframe::egui::ColorImage> {
    let mut r = png::Decoder::new(std::fs::File::open(path).ok()?).read_info().ok()?;
    let mut buf = vec![0; r.output_buffer_size()];
    let info = r.next_frame(&mut buf).ok()?;
    (info.color_type == png::ColorType::Rgba && info.bit_depth == png::BitDepth::Eight)
        .then(|| eframe::egui::ColorImage::from_rgba_unmultiplied([info.width as usize, info.height as usize], &buf[..info.buffer_size()]))
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
        "dir": dir(), "appid": g.appid, "name": g.name, "folder": g.folder, "exe": g.exe,
        "boost": o.boost, "cpu": o.cpu, "gpu": o.gpu, "io": o.io, "focus": o.focus,
        "services": o.services, "service_list": SERVICES, "search": o.search, "steam": o.steam,
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
            println!("{} {} {} art={:?}", g.appid, g.name, g.folder.display(), g.art.as_ref().map(|a| load_art(a).map(|i| i.size)));
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
        let g = Game { appid: "0".into(), name: "Arena test".into(), folder: folder.clone(), art: None, exe: None };
        let o = Opts { boost: true, cpu: "High".into(), gpu: "High".into(), io: "High".into(), focus: true, services: true, search: true, steam: false, latency: true, pin: true, standby: true, plan: None, close_app: false };
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

#[cfg(test)]
mod live_steam {
    use super::*;
    use std::time::{Duration, Instant};

    fn helpers() -> usize {
        let o = crate::sys::ps("@(Get-Process steamwebhelper -ErrorAction SilentlyContinue).Count").unwrap();
        String::from_utf8_lossy(&o.stdout).trim().parse().unwrap_or(0)
    }

    fn wait_phase(phase: &str) -> Status {
        let t = Instant::now();
        while t.elapsed() < Duration::from_secs(60) {
            std::thread::sleep(Duration::from_secs(1));
            if let Some(s) = status().filter(|s| s.phase == phase) {
                return s;
            }
        }
        panic!("session never reached {phase}");
    }

    fn ps_out(script: &str) -> String {
        String::from_utf8_lossy(&crate::sys::ps(script).unwrap().stdout).trim().to_string()
    }

    /// helper pids, sorted
    fn helper_ids() -> String {
        ps_out("(@(Get-Process steamwebhelper -ErrorAction SilentlyContinue).Id | Sort-Object) -join ','")
    }

    /// "Trim Steam while playing" against the running Steam. With Millennium the same helpers must
    /// stay alive, frozen and paged out while the stand-in game runs, then run on (no new helper,
    /// so Millennium keeps its pipe). Without Millennium they close and Steam starts new ones after.
    /// `cargo test live_steam -- --ignored --nocapture`
    #[test]
    #[ignore]
    fn live_steam_reduce() {
        let before = helpers();
        assert!(before > 0, "start Steam first");
        let mil = ps_out("[bool](@(Get-Process steam)[0].Modules | Where-Object ModuleName -eq 'millennium.dll')") == "True";
        let ids_before = helper_ids();
        let ws_before = ps_out("[int]((Get-Process steamwebhelper | Measure-Object WorkingSet64 -Sum).Sum / 1MB)");
        let folder = std::env::temp_dir().join("cheetix_arena_fakegame2");
        let _ = std::fs::create_dir_all(&folder);
        let exe = folder.join("fakegame.exe");
        std::fs::copy(r"C:\Windows\System32\cmd.exe", &exe).unwrap();
        let mut game = std::process::Command::new(&exe).args(["/c", "ping -n 40 127.0.0.1 >nul"]).spawn().unwrap();
        let o = Opts { boost: false, focus: false, services: false, steam: true, ..Default::default() };
        start(&Game { appid: "0".into(), name: "Steam test".into(), folder: folder.clone(), art: None, exe: None }, &o).unwrap();
        let live = wait_phase("active");
        std::thread::sleep(Duration::from_secs(8)); // Steam would respawn the helper within seconds if it could
        let during = helpers();
        let ids_during = helper_ids();
        // frozen = the helpers use no CPU at all over 5 seconds (a thread parked in a kernel wait
        // shows as "Executive" rather than "Suspended" until it wakes, but it cannot run)
        let cpu = || ps_out("[int](@(Get-Process steamwebhelper -ErrorAction SilentlyContinue | ForEach-Object { $_.TotalProcessorTime.TotalMilliseconds }) | Measure-Object -Sum).Sum");
        let c0 = cpu();
        std::thread::sleep(Duration::from_secs(5));
        let cpu_ms = cpu().parse::<i64>().unwrap_or(-1) - c0.parse::<i64>().unwrap_or(0);
        let ws_during = ps_out("[int]((Get-Process steamwebhelper -ErrorAction SilentlyContinue | Measure-Object WorkingSet64 -Sum).Sum / 1MB)");
        println!("millennium {mil}; helpers before {before} ({ws_before} MB), during {during} ({ws_during} MB, {cpu_ms} ms CPU in 5 s); notes {:?}", live.notes);
        let _ = game.kill();
        println!("ended: {}", wait_phase("ended").msg);
        std::thread::sleep(Duration::from_secs(5));
        let ids_after = helper_ids();
        let running = ps_out("@(Get-Process steamwebhelper -ErrorAction SilentlyContinue | Where-Object { @($_.Threads | Where-Object { $_.WaitReason -ne 'Suspended' }).Count -gt 0 }).Count");
        println!("helper pids before [{ids_before}] during [{ids_during}] after [{ids_after}], running after: {running}");
        let _ = std::fs::remove_dir_all(folder);
        if mil {
            assert_eq!(ids_during, ids_before, "helpers must not be closed with Millennium");
            assert_eq!(cpu_ms, 0, "helpers were not frozen");
            assert_eq!(ids_after, ids_before, "Steam started a new helper: Millennium would lose Steam's UI");
            assert_eq!(running, during.to_string(), "helpers still frozen after the game");
        } else {
            assert_eq!(during, 0);
            let t = Instant::now();
            while t.elapsed() < Duration::from_secs(45) && helpers() == 0 {
                std::thread::sleep(Duration::from_secs(1));
            }
            assert!(helpers() > 0, "Steam's web helper did not come back");
        }
    }
}

#[cfg(test)]
mod live_app {
    use super::*;
    use std::time::{Duration, Instant};

    /// An added program: cover from its icon, launched directly (not through Steam), boosted,
    /// then everything put back when it closes. `cargo test live_app -- --ignored --nocapture`
    #[test]
    #[ignore]
    fn live_app_session() {
        let folder = std::env::temp_dir().join("cheetix_arena_app");
        let _ = std::fs::create_dir_all(&folder);
        let exe = folder.join("myprogram.exe");
        std::fs::copy(r"C:\Windows\System32\cmd.exe", &exe).unwrap();
        let app = App { id: app_id(&exe), name: "My program".into(), exe: exe.clone() };
        app_cover(&app);
        let g = app.game();
        let cover = g.art.as_ref().and_then(|a| load_art(a)).map(|i| i.size);
        println!("cover: {cover:?}");
        assert_eq!(cover, Some([240, 360]));
        start(&g, &Opts { focus: false, services: false, ..Default::default() }).unwrap();
        let t = Instant::now();
        let live = loop {
            assert!(t.elapsed() < Duration::from_secs(60), "never went live");
            std::thread::sleep(Duration::from_secs(1));
            if let Some(s) = status().filter(|s| s.phase == "active") {
                break s;
            }
        };
        for p in &live.procs {
            println!("boosted {} {}: cpu {} gpu {} io {}", p.name, p.pid, p.cpu, p.gpu, p.io);
        }
        assert!(live.procs.iter().any(|p| p.name == "myprogram"));
        for p in &live.procs {
            let _ = std::process::Command::new("taskkill").args(["/f", "/pid", &p.pid.to_string()]).output();
        }
        let t = Instant::now();
        let end = loop {
            assert!(t.elapsed() < Duration::from_secs(60), "never ended");
            std::thread::sleep(Duration::from_secs(1));
            if let Some(s) = status().filter(|s| s.phase == "ended") {
                break s;
            }
        };
        println!("ended: {}", end.msg);
        let _ = std::fs::remove_dir_all(folder);
        let _ = std::fs::remove_file(dir().join("art").join(format!("{}.png", app.id)));
    }
}
