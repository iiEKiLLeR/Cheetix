//! Everything that touches Windows: running scripts, reading metrics, processes,
//! startup entries and power plans. All calls block; the UI runs them on threads.

use std::collections::HashMap;
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};

#[cfg(windows)]
use std::os::windows::process::CommandExt;

const NO_WINDOW: u32 = 0x0800_0000;

pub fn hidden(cmd: &mut Command) -> &mut Command {
    #[cfg(windows)]
    cmd.creation_flags(NO_WINDOW);
    cmd
}

fn text(o: &Output) -> String {
    let mut s = String::from_utf8_lossy(&o.stdout).into_owned();
    let e = String::from_utf8_lossy(&o.stderr);
    if !e.trim().is_empty() {
        s.push('\n');
        s.push_str(&e);
    }
    s
}

/// Like `Command::output`, but kills the process after `secs`.
fn output_timeout(cmd: &mut Command, secs: u64) -> std::io::Result<Output> {
    use std::io::Read;
    use std::process::Stdio;
    let mut child = hidden(cmd).stdin(Stdio::null()).stdout(Stdio::piped()).stderr(Stdio::piped()).spawn()?;
    let pipe = |r: Option<Box<dyn Read + Send>>| {
        std::thread::spawn(move || {
            let mut v = vec![];
            if let Some(mut r) = r {
                let _ = r.read_to_end(&mut v);
            }
            v
        })
    };
    let out = pipe(child.stdout.take().map(|s| Box::new(s) as Box<dyn Read + Send>));
    let err = pipe(child.stderr.take().map(|s| Box::new(s) as Box<dyn Read + Send>));
    let start = std::time::Instant::now();
    let status = loop {
        if let Some(s) = child.try_wait()? {
            break s;
        }
        if start.elapsed().as_secs() >= secs {
            let _ = child.kill();
            return Err(std::io::Error::new(std::io::ErrorKind::TimedOut, "timeout"));
        }
        std::thread::sleep(std::time::Duration::from_millis(50));
    };
    Ok(Output { status, stdout: out.join().unwrap_or_default(), stderr: err.join().unwrap_or_default() })
}

pub const SCRIPT_TIMEOUT_S: u64 = 30;

static SEQ: AtomicU64 = AtomicU64::new(0);

fn temp_file(ext: &str, body: &str) -> std::io::Result<std::path::PathBuf> {
    let n = SEQ.fetch_add(1, Ordering::Relaxed);
    let p = std::env::temp_dir().join(format!("cheetix_{}_{}.{}", std::process::id(), n, ext));
    // Batch files want CRLF; PowerShell accepts either.
    std::fs::write(&p, body.replace("\r\n", "\n").replace('\n', "\r\n"))?;
    Ok(p)
}

/// Run a PowerShell script (written to a temp .ps1 so quoting can't break it).
pub fn ps(script: &str) -> Result<Output, String> {
    // batch checks of a whole page get longer
    ps_timeout(script, 120)
}

pub fn ps_timeout(script: &str, secs: u64) -> Result<Output, String> {
    let p = temp_file("ps1", script).map_err(|e| e.to_string())?;
    let out = output_timeout(Command::new("powershell").args(["-NoProfile", "-NonInteractive", "-ExecutionPolicy", "Bypass", "-File"]).arg(&p), secs)
        .map_err(|e| e.to_string());
    let _ = std::fs::remove_file(p);
    out
}

/// Run a raw command line through cmd.exe exactly as written.
pub fn cmd_raw(line: &str) -> Result<Output, String> {
    let mut c = Command::new("cmd");
    c.arg("/c");
    #[cfg(windows)]
    c.raw_arg(line);
    #[cfg(not(windows))]
    c.arg(line);
    hidden(&mut c).output().map_err(|e| e.to_string())
}

pub fn is_admin() -> bool {
    #[cfg(windows)]
    unsafe {
        windows_sys::Win32::UI::Shell::IsUserAnAdmin() != 0
    }
    #[cfg(not(windows))]
    false
}

/// Keywords that mark a script as needing an elevated session.
pub fn needs_admin(s: &str) -> bool {
    let l = s.to_lowercase();
    [
        "hklm\\", "root\\default", "sc config", "sc stop", "sc start", "sc delete", "netsh",
        "powercfg /setacvalueindex", "powercfg -setacvalueindex", "powercfg /setdcvalueindex",
        "powercfg -setdcvalueindex", "powercfg /setactive", "powercfg -setactive", "powercfg /change",
        "powercfg -change", "powercfg /h ", "powercfg -h ", "powercfg /import", "powercfg -duplicatescheme",
        "powercfg /delete", "powercfg /attributes", "powercfg -attributes", "bcdedit", "wmic", "wevtutil",
        "checkpoint-computer", "enable-computerrestore", "set-ciminstance", "new-ciminstance",
        "remove-ciminstance", "net stop wuauserv", "net start wuauserv", "c:\\windows\\temp",
        "c:\\$recycle.bin", "c:\\windows\\prefetch", "c:\\windows\\minidump", "c:\\windows\\softwaredistribution", "/cleanup-image", "pnputil", "\\logs\\",
    ]
    .iter()
    .any(|k| l.contains(k))
}

pub const ADMIN_MSG: &str = "ADMIN_REQUIRED: This change needs administrator rights. Run the app as administrator.";

/// Run a tweak's batch script. Ok = ran (non-admin errors are reported but not fatal,
/// because OFF scripts are best-effort cleanup); Err = admin needed or could not start.
pub fn run_bat(body: &str, admin: bool) -> Result<String, String> {
    if needs_admin(body) && !admin {
        return Err(ADMIN_MSG.into());
    }
    let p = temp_file("bat", body).map_err(|e| e.to_string())?;
    let out = output_timeout(Command::new("cmd").arg("/c").arg(&p), SCRIPT_TIMEOUT_S);
    let _ = std::fs::remove_file(&p);
    let out = out.map_err(|e| e.to_string())?;
    let t = text(&out);
    if !out.status.success() {
        let l = t.to_lowercase();
        if ["denied", "administrator", "requires elevation", "740", "error 5"].iter().any(|k| l.contains(k)) {
            return Err(ADMIN_MSG.into());
        }
        let code = out.status.code().unwrap_or(-1);
        return Ok(format!("{}\nNOTE: script reported a non-fatal error (exit code {code})", t.trim()));
    }
    Ok(t)
}

/// Body of `powershell -NoProfile -Command "<body>"`, if the check has that shape.
fn ps_body(check: &str) -> Option<String> {
    let c = check.trim();
    let lower = c.to_lowercase();
    let rest = ["powershell -noprofile -command ", "powershell -command "]
        .iter()
        .find(|p| lower.starts_with(*p))
        .map(|p| c[p.len()..].trim())?;
    let rest = rest.strip_prefix('"').and_then(|r| r.strip_suffix('"')).unwrap_or(rest);
    Some(rest.to_string())
}

/// One check: exit code 0 means the tweak is on.
pub fn check_one(check: &str) -> bool {
    let out = match ps_body(check) {
        Some(body) => ps(&body),
        None => cmd_raw(check),
    };
    out.map(|o| o.status.success()).unwrap_or(false)
}

/// A check can join a batch when its body only ever ends with `exit 0` / `exit 1`.
fn batchable(body: &str) -> bool {
    let l = body.to_lowercase();
    if l.contains("function") || l.contains("param(") || l.contains("trap") {
        return false;
    }
    let mut saw0 = false;
    let mut saw1 = false;
    for (i, _) in l.match_indices("exit ") {
        let tail = &l[i + 5..];
        let digits: String = tail.chars().take_while(|c| c.is_ascii_digit()).collect();
        match digits.as_str() {
            "0" => saw0 = true,
            "1" => saw1 = true,
            _ => return false,
        }
    }
    saw0 && saw1
}

/// Check many tweaks in one PowerShell process (each `exit N` becomes a write into a result table).
pub fn check_batch(items: &[(String, String)]) -> HashMap<String, bool> {
    let mut res = HashMap::new();
    let mut batch: Vec<(&String, String)> = vec![];
    let mut singles = vec![];
    for (id, chk) in items {
        match ps_body(chk) {
            Some(b) if batchable(&b) => batch.push((id, b)),
            _ => singles.push((id, chk)),
        }
    }
    if !batch.is_empty() {
        let mut s = String::from("$ErrorActionPreference = 'SilentlyContinue'\n$script:RR = @{}\n");
        for (i, (_, body)) in batch.iter().enumerate() {
            let b = body
                .replace("exit 0", &format!("$script:RR[{i}] = $true; return"))
                .replace("exit 1", &format!("$script:RR[{i}] = $false; return"));
            s += &format!("$script:RR[{i}] = $false\ntry {{ & {{ {b} }} }} catch {{}}\n");
        }
        s += "$out = @{}\n";
        for i in 0..batch.len() {
            s += &format!("$out['{i}'] = [bool]$script:RR[{i}]\n");
        }
        s += "$out | ConvertTo-Json -Compress\n";
        let parsed: Option<HashMap<String, bool>> = ps(&s).ok().and_then(|o| {
            let t = String::from_utf8_lossy(&o.stdout).into_owned();
            t.lines().rev().find(|l| l.trim_start().starts_with('{')).and_then(|l| serde_json::from_str(l.trim()).ok())
        });
        match parsed {
            Some(m) => {
                for (i, (id, _)) in batch.iter().enumerate() {
                    res.insert((*id).clone(), m.get(&i.to_string()).copied().unwrap_or(false));
                }
            }
            None => {
                for (id, _) in &batch {
                    let chk = &items.iter().find(|(i, _)| i == *id).unwrap().1;
                    res.insert((*id).clone(), check_one(chk));
                }
            }
        }
    }
    for (id, chk) in singles {
        res.insert(id.clone(), check_one(chk));
    }
    res
}

// ---------------- metrics ----------------

#[derive(Clone, Default)]
pub struct Metrics {
    pub cpu: u32,
    pub mem_used: u32,
    pub mem_total: u64,
    pub mem_free: u64,
    pub disk_used: u32,
    pub disk_total: u64,
    pub disk_free: u64,
    pub uptime_s: u64,
}

#[cfg(windows)]
fn sys_times() -> (u64, u64) {
    use windows_sys::Win32::Foundation::FILETIME;
    let z = FILETIME { dwLowDateTime: 0, dwHighDateTime: 0 };
    let (mut idle, mut kern, mut user) = (z, z, z);
    unsafe { windows_sys::Win32::System::Threading::GetSystemTimes(&mut idle, &mut kern, &mut user) };
    let f = |t: FILETIME| ((t.dwHighDateTime as u64) << 32) | t.dwLowDateTime as u64;
    (f(idle), f(kern) + f(user)) // kernel time already includes idle
}

pub fn metrics() -> Metrics {
    let mut m = Metrics::default();
    #[cfg(windows)]
    unsafe {
        use windows_sys::Win32::System::SystemInformation::*;
        static PREV: [AtomicU64; 2] = [AtomicU64::new(0), AtomicU64::new(0)];
        if PREV[1].load(Ordering::Relaxed) == 0 {
            let (i, t) = sys_times();
            PREV[0].store(i, Ordering::Relaxed);
            PREV[1].store(t, Ordering::Relaxed);
            std::thread::sleep(std::time::Duration::from_millis(250));
        }
        let (i1, t1) = sys_times();
        let (i0, t0) = (PREV[0].swap(i1, Ordering::Relaxed), PREV[1].swap(t1, Ordering::Relaxed));
        let dt = t1.saturating_sub(t0).max(1);
        m.cpu = (100.0 * (1.0 - (i1.saturating_sub(i0)) as f64 / dt as f64)).round().clamp(0.0, 100.0) as u32;

        let mut ms: MEMORYSTATUSEX = std::mem::zeroed();
        ms.dwLength = std::mem::size_of::<MEMORYSTATUSEX>() as u32;
        if GlobalMemoryStatusEx(&mut ms) != 0 {
            m.mem_total = ms.ullTotalPhys;
            m.mem_free = ms.ullAvailPhys;
            m.mem_used = (((m.mem_total - m.mem_free) as f64 / m.mem_total.max(1) as f64) * 100.0).round() as u32;
        }
        m.uptime_s = GetTickCount64() / 1000;

        let drive: Vec<u16> = std::env::var("SystemDrive").unwrap_or("C:".into()).encode_utf16().chain("\\\0".encode_utf16()).collect();
        let (mut avail, mut total, mut free) = (0u64, 0u64, 0u64);
        if windows_sys::Win32::Storage::FileSystem::GetDiskFreeSpaceExW(drive.as_ptr(), &mut avail, &mut total, &mut free) != 0 {
            m.disk_total = total;
            m.disk_free = free;
            m.disk_used = (((total - free) as f64 / total.max(1) as f64) * 100.0).round() as u32;
        }
    }
    m
}

/// (OS caption, OS version, GPU name) — one PowerShell call, run once.
pub fn static_info() -> (String, String, String) {
    let s = "$o = Get-CimInstance Win32_OperatingSystem; $g = (Get-CimInstance Win32_VideoController | Select-Object -First 1).Name; \"$($o.Caption)|$($o.Version)|$g\"";
    let t = ps(s).map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string()).unwrap_or_default();
    let mut p = t.splitn(3, '|').map(|x| x.trim().to_string());
    let os = p.next().unwrap_or_default();
    let ver = p.next().unwrap_or_default();
    let gpu = p.next().filter(|g| !g.is_empty()).unwrap_or_else(|| "N/A".into());
    (os, ver, gpu)
}

pub fn local_time() -> (String, String) {
    #[cfg(windows)]
    unsafe {
        let mut t = std::mem::zeroed();
        windows_sys::Win32::System::SystemInformation::GetLocalTime(&mut t);
        return (
            format!("{:02}:{:02}:{:02}", t.wHour, t.wMinute, t.wSecond),
            format!("{:04}-{:02}-{:02}", t.wYear, t.wMonth, t.wDay),
        );
    }
    #[allow(unreachable_code)]
    ("00:00:00".into(), "1970-01-01".into())
}

// ---------------- processes ----------------

#[derive(Clone)]
pub struct Proc {
    pub name: String,
    pub pid: u32,
    pub mem_mb: f32,
}

pub const PROTECTED: &[&str] = &[
    "system", "idle", "registry", "memory compression", "secure system", "smss", "csrss", "wininit",
    "winlogon", "services", "lsass", "fontdrvhost", "dwm", "svchost", "audiodg", "wudfhost",
];

fn parse_tasklist(t: &str) -> Vec<Proc> {
    t.lines()
        .filter_map(|l| {
            let f: Vec<&str> = l.trim().trim_matches('"').split("\",\"").collect();
            if f.len() < 5 {
                return None;
            }
            let kb: String = f[4].chars().filter(|c| c.is_ascii_digit()).collect();
            Some(Proc { name: f[0].to_string(), pid: f[1].parse().ok()?, mem_mb: (kb.parse::<f32>().ok()? / 1024.0 * 10.0).round() / 10.0 })
        })
        .collect()
}

/// Top 70 processes by memory, plus the total count.
pub fn processes() -> (Vec<Proc>, usize) {
    let out = hidden(Command::new("tasklist").args(["/fo", "csv", "/nh"])).output();
    let mut v = out.map(|o| parse_tasklist(&String::from_utf8_lossy(&o.stdout))).unwrap_or_default();
    let total = v.len();
    v.sort_by(|a, b| b.mem_mb.total_cmp(&a.mem_mb));
    v.truncate(70);
    (v, total)
}

pub fn kill(pid: u32, name: &str) -> Result<String, String> {
    let exe = name.trim().to_lowercase();
    let bare = exe.trim_end_matches(".exe");
    if pid <= 4 {
        return Err("Invalid process id".into());
    }
    if PROTECTED.contains(&bare) {
        return Err(format!("Refused: \"{bare}\" is a protected system process"));
    }
    let now = hidden(Command::new("tasklist").args(["/fi", &format!("PID eq {pid}"), "/fo", "csv", "/nh"])).output().map_err(|e| e.to_string())?;
    let found = parse_tasklist(&String::from_utf8_lossy(&now.stdout));
    match found.first() {
        None => return Ok("ALREADY_GONE".into()),
        Some(p) if p.name.to_lowercase() != exe => return Err("Process list is stale — refresh and try again".into()),
        _ => {}
    }
    let o = hidden(Command::new("taskkill").args(["/PID", &pid.to_string(), "/F"])).output().map_err(|e| e.to_string())?;
    if o.status.success() { Ok("KILLED".into()) } else { Err(text(&o).trim().to_string()) }
}

// ---------------- startup ----------------

#[derive(Clone, serde::Deserialize)]
pub struct StartupItem {
    pub id: String,
    pub name: String,
    pub cmd: String,
    pub loc: String,
    pub enabled: bool,
    /// the program's own icon (64x64 PNG), when it could be found
    #[serde(default)]
    pub icon: Option<String>,
}

const STARTUP_LOCS: &str = r#"
$locs = @(
 @{Id='run-hkcu';  Root='HKCU:'; Path='Software\Microsoft\Windows\CurrentVersion\Run';             Appr='Run';   Label='HKCU RUN'},
 @{Id='run-hklm';  Root='HKLM:'; Path='Software\Microsoft\Windows\CurrentVersion\Run';             Appr='Run';   Label='HKLM RUN'},
 @{Id='run-hkcu32';Root='HKCU:'; Path='Software\WOW6432Node\Microsoft\Windows\CurrentVersion\Run'; Appr='Run32'; Label='HKCU RUN (32-BIT)'},
 @{Id='run-hklm32';Root='HKLM:'; Path='Software\WOW6432Node\Microsoft\Windows\CurrentVersion\Run'; Appr='Run32'; Label='HKLM RUN (32-BIT)'})
function ApprPath($l) { "$($l.Root)\Software\Microsoft\Windows\CurrentVersion\Explorer\StartupApproved\$($l.Appr)" }
"#;

/// Finds the program a Run command starts and saves its icon as a PNG (once per program).
const STARTUP_ICON: &str = r#"
Add-Type -AssemblyName System.Drawing
Add-Type -TypeDefinition 'using System; using System.Runtime.InteropServices; public static class Ico { [DllImport("user32.dll", CharSet = CharSet.Unicode)] public static extern uint PrivateExtractIcons(string f, int i, int cx, int cy, IntPtr[] h, uint[] id, uint n, uint fl); }'
function IconFor($cmd) {
  try {
    $c = [Environment]::ExpandEnvironmentVariables($cmd).Trim()
    if ($c.StartsWith('"')) { $p = $c.Substring(1, $c.IndexOf('"', 1) - 1) }
    elseif ($c -match '^(.+?\.(exe|com|bat|cmd))(\s|$)') { $p = $matches[1] }
    else { $p = ($c -split ' ')[0] }
    if (-not [IO.Path]::IsPathRooted($p)) { $w = Get-Command $p -CommandType Application -ErrorAction SilentlyContinue | Select-Object -First 1; if ($w) { $p = $w.Source } }
    if (-not (Test-Path -LiteralPath $p -PathType Leaf)) { return $null }
    $md5 = [Security.Cryptography.MD5]::Create()
    $f = Join-Path $iconDir (([BitConverter]::ToString($md5.ComputeHash([Text.Encoding]::UTF8.GetBytes($p.ToLower())))).Replace('-', '') + '.png')
    if (-not (Test-Path -LiteralPath $f)) {
      $h = New-Object IntPtr[] 1; $id = New-Object uint32[] 1
      [void][Ico]::PrivateExtractIcons($p, 0, 64, 64, $h, $id, 1, 0)
      $ico = if ($h[0] -ne [IntPtr]::Zero) { [Drawing.Icon]::FromHandle($h[0]) } else { [Drawing.Icon]::ExtractAssociatedIcon($p) }
      $b = New-Object Drawing.Bitmap 64, 64
      $g = [Drawing.Graphics]::FromImage($b); $g.InterpolationMode = 'HighQualityBicubic'
      $g.DrawImage($ico.ToBitmap(), 0, 0, 64, 64); $g.Dispose()
      $b.Save($f, [Drawing.Imaging.ImageFormat]::Png); $b.Dispose()
    }
    $f
  } catch { $null }
}
"#;

pub fn startup_list() -> Result<Vec<StartupItem>, String> {
    let icons = crate::arena::dir().join("icons");
    let _ = std::fs::create_dir_all(&icons);
    let s = format!(
        "{STARTUP_LOCS}{STARTUP_ICON}
$iconDir = '{}'
$out = @()
foreach ($l in $locs) {{
  $k = Get-Item -LiteralPath \"$($l.Root)\\$($l.Path)\" -ErrorAction SilentlyContinue
  if (-not $k) {{ continue }}
  foreach ($n in $k.GetValueNames()) {{
    if (-not $n) {{ continue }}
    $b = (Get-ItemProperty -LiteralPath (ApprPath $l) -Name $n -ErrorAction SilentlyContinue).$n
    $off = ($b -is [byte[]]) -and $b.Count -gt 0 -and (($b[0] % 2) -eq 1)
    $out += [pscustomobject]@{{ id = \"$($l.Id)|$n\"; name = $n; cmd = [string]$k.GetValue($n); loc = $l.Label; enabled = -not $off; icon = (IconFor ([string]$k.GetValue($n))) }}
  }}
}}
ConvertTo-Json -InputObject @($out) -Compress",
        icons.display().to_string().replace('\'', "''")
    );
    let o = ps(&s)?;
    let t = String::from_utf8_lossy(&o.stdout);
    let line = t.lines().rev().find(|l| l.trim_start().starts_with('[')).unwrap_or("[]");
    serde_json::from_str(line.trim()).map_err(|e| format!("scan failed: {e}"))
}

/// Enable/disable a Run entry the way Task Manager does: a StartupApproved blob
/// (02 + zeros = on, 03 00 00 00 + FILETIME = off). Reads it back to verify.
pub fn startup_set(id: &str, enable: bool, admin: bool) -> Result<(), String> {
    let (loc, name) = id.split_once('|').ok_or("Invalid startup item id")?;
    if !["run-hkcu", "run-hklm", "run-hkcu32", "run-hklm32"].contains(&loc) {
        return Err("Invalid startup item id".into());
    }
    if loc.starts_with("run-hklm") && !admin {
        return Err(ADMIN_MSG.into());
    }
    let q = |s: &str| s.replace('\'', "''");
    let s = format!(
        "{STARTUP_LOCS}
$l = $locs | Where-Object {{ $_.Id -eq '{loc}' }}
$p = ApprPath $l
if (-not (Test-Path -LiteralPath $p)) {{ New-Item -Path $p -Force | Out-Null }}
$bytes = if ({en}) {{ [byte[]](@(2) + @(0) * 11) }} else {{ [byte[]](@(3,0,0,0) + [BitConverter]::GetBytes([DateTime]::Now.ToFileTime())) }}
Set-ItemProperty -LiteralPath $p -Name '{name}' -Value $bytes -Type Binary -ErrorAction Stop
$b = (Get-ItemProperty -LiteralPath $p -Name '{name}').'{name}'
if ((($b[0] % 2) -eq 0) -eq {en}) {{ 'VERIFIED' }} else {{ 'MISMATCH' }}",
        loc = q(loc),
        name = q(name),
        en = if enable { "$true" } else { "$false" }
    );
    let o = ps(&s)?;
    if String::from_utf8_lossy(&o.stdout).contains("VERIFIED") {
        return Ok(());
    }
    let msg: String = text(&o).trim().chars().take(200).collect();
    Err(if msg.is_empty() { "verification failed".into() } else { msg })
}

// ---------------- power plans ----------------

#[derive(Clone)]
pub struct Plan {
    pub guid: String,
    pub name: String,
    pub active: bool,
}

pub fn plans() -> Vec<Plan> {
    let out = hidden(Command::new("powercfg").arg("/list")).output();
    let t = out.map(|o| String::from_utf8_lossy(&o.stdout).into_owned()).unwrap_or_default();
    t.lines()
        .filter_map(|l| {
            let rest = l.split_once("GUID:")?.1.trim();
            let guid = rest.get(..36)?.to_lowercase();
            let tail = &rest[36..];
            let open = tail.find('(')?;
            let close = tail.rfind(')')?; // names may contain parentheses themselves
            Some(Plan { guid, name: tail[open + 1..close].trim().to_string(), active: tail.trim_end().ends_with('*') })
        })
        .collect()
}

pub fn plan_activate(guid: &str) -> Result<(), String> {
    let ok = guid.len() == 36 && guid.chars().all(|c| c.is_ascii_hexdigit() || c == '-');
    if !ok {
        return Err("Invalid power plan GUID".into());
    }
    let o = hidden(Command::new("powercfg").args(["/setactive", guid])).output().map_err(|e| e.to_string())?;
    if o.status.success() { Ok(()) } else { Err(text(&o).trim().to_string()) }
}

/// Open a Windows setting or tool (`cmd /c start "" <cmd>`).
/// The icon Windows shows for each command (what Explorer and Start draw), saved as 64x64 PNGs.
/// Settings pages get the Settings app's icon; Store apps (Paint, Terminal...) their Start icon.
pub fn shell_icons(cmds: &[String]) -> HashMap<String, String> {
    let dir = crate::arena::dir().join("icons");
    let _ = std::fs::create_dir_all(&dir);
    let Ok(list) = temp_file("json", &serde_json::to_string(cmds).unwrap_or_default()) else { return HashMap::new() };
    let script = format!(
        r#"Add-Type -AssemblyName System.Drawing
Add-Type -ReferencedAssemblies System.Drawing -TypeDefinition @'
using System; using System.Drawing; using System.Drawing.Imaging; using System.Runtime.InteropServices;
public static class ShellIcon {{
  [ComImport, Guid("bcc18b79-ba16-442f-80c4-8a59c30c463b"), InterfaceType(ComInterfaceType.InterfaceIsIUnknown)]
  interface IShellItemImageFactory {{ [PreserveSig] int GetImage(SZ size, int flags, out IntPtr hbm); }}
  [StructLayout(LayoutKind.Sequential)] struct SZ {{ public int cx, cy; }}
  [StructLayout(LayoutKind.Sequential)] struct BMP {{ public int type, w, h, stride; public ushort planes, bpp; public IntPtr bits; }}
  [DllImport("shell32.dll", CharSet = CharSet.Unicode)] static extern int SHCreateItemFromParsingName(string p, IntPtr bc, ref Guid iid, [MarshalAs(UnmanagedType.Interface)] out IShellItemImageFactory f);
  [DllImport("gdi32.dll")] static extern int GetObject(IntPtr h, int n, out BMP b);
  [DllImport("gdi32.dll")] static extern bool DeleteObject(IntPtr h);
  public static bool Save(string item, string png) {{
    Guid iid = new Guid("bcc18b79-ba16-442f-80c4-8a59c30c463b"); IShellItemImageFactory f;
    if (SHCreateItemFromParsingName(item, IntPtr.Zero, ref iid, out f) != 0 || f == null) return false;
    IntPtr hb; SZ s; s.cx = 64; s.cy = 64;
    if (f.GetImage(s, 0x4, out hb) != 0) return false; // icon only, no thumbnail
    try {{
      BMP b; GetObject(hb, Marshal.SizeOf(typeof(BMP)), out b);
      if (b.bpp != 32 || b.bits == IntPtr.Zero) return false;
      // a top-down 32-bit DIB with premultiplied alpha
      using (var src = new Bitmap(b.w, b.h, b.stride, PixelFormat.Format32bppPArgb, b.bits))
      using (var dst = new Bitmap(src)) dst.Save(png, ImageFormat.Png);
      return true;
    }} finally {{ DeleteObject(hb); }}
  }}
  [DllImport("user32.dll", CharSet = CharSet.Unicode)] static extern uint PrivateExtractIcons(string f, int i, int cx, int cy, IntPtr[] h, uint[] id, uint n, uint fl);
  [DllImport("user32.dll")] static extern bool DestroyIcon(IntPtr h);
  // a Control Panel applet's own picture (the first icon inside the .cpl), as Control Panel shows it
  public static bool SaveFirst(string file, string png) {{
    IntPtr[] h = new IntPtr[1]; uint[] id = new uint[1];
    if (PrivateExtractIcons(file, 0, 64, 64, h, id, 1, 0) == 0 || h[0] == IntPtr.Zero) return false;
    try {{ using (var ic = Icon.FromHandle(h[0])) using (var b = ic.ToBitmap()) b.Save(png, ImageFormat.Png); return true; }}
    finally {{ DestroyIcon(h[0]); }}
  }}
}}
'@
$dir = '{dir}'
$apps = @{{}}; foreach ($a in (Get-StartApps)) {{ $apps[$a.Name] = $a.AppID }}
$store = @{{ calc = 'Calculator'; notepad = 'Notepad'; mspaint = 'Paint'; snippingtool = 'Snipping Tool'; wt = 'Terminal' }}
$md5 = [Security.Cryptography.MD5]::Create(); $out = @{{}}
foreach ($c in (Get-Content -LiteralPath '{list}' -Raw | ConvertFrom-Json)) {{
  $item = $null
  if ($c -like 'ms-settings:*') {{ $item = 'shell:AppsFolder\windows.immersivecontrolpanel_cw5n1h2txyewy!microsoft.windows.immersivecontrolpanel' }}
  elseif ($c -like 'windowsdefender:*') {{ $item = 'shell:AppsFolder\Microsoft.SecHealthUI_8wekyb3d8bbwe!SecHealthUI' }}
  else {{
    $t = ($c -split ' ')[0]; if ($t -eq 'rundll32') {{ $t = (($c -split ' ')[1] -split ',')[0] }}
    if ($store[$t] -and $apps[$store[$t]]) {{ $item = 'shell:AppsFolder\' + $apps[$store[$t]] }}
    else {{
      $w = Get-Command $t -CommandType Application -ErrorAction SilentlyContinue | Select-Object -First 1
      if (-not $w) {{ $w = Get-Command "$t.exe" -CommandType Application -ErrorAction SilentlyContinue | Select-Object -First 1 }}
      if ($w) {{ $item = $w.Source; if ($item -like '*.cpl') {{ $item = 'cpl:' + $item }} }}
      # the firewall applet keeps its picture in another file
      if ($t -eq 'firewall.cpl') {{ $item = 'cpl:' + (Join-Path $env:SystemRoot 'System32\FirewallControlPanel.dll') }}
    }}
  }}
  if (-not $item) {{ continue }}
  $f = Join-Path $dir ('sc-' + ([BitConverter]::ToString($md5.ComputeHash([Text.Encoding]::UTF8.GetBytes($item.ToLower())))).Replace('-', '') + '.png')
  $ok = (Test-Path -LiteralPath $f) -or $(if ($item -like 'cpl:*') {{ [ShellIcon]::SaveFirst($item.Substring(4), $f) }} else {{ [ShellIcon]::Save($item, $f) }})
  if ($ok) {{ $out[$c] = $f }}
}}
$out | ConvertTo-Json -Compress"#,
        dir = dir.display().to_string().replace('\'', "''"),
        list = list.display().to_string().replace('\'', "''")
    );
    let o = ps_timeout(&script, 120);
    let _ = std::fs::remove_file(list);
    let txt = o.map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string()).unwrap_or_default();
    serde_json::from_str(&txt).unwrap_or_default()
}

pub fn open(cmd: &str) {
    let _ = cmd_raw(&format!("start \"\" {cmd}"));
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn batching_rules() {
        assert!(batchable("try { if ($v -eq 1) { exit 0 } else { exit 1 } } catch { exit 1 }"));
        assert!(!batchable("exit 0"));
        assert!(!batchable("if ($x) { exit 2 } exit 1; exit 0"));
        assert_eq!(ps_body("powershell -NoProfile -Command \"exit 1\"").as_deref(), Some("exit 1"));
        assert!(ps_body("reg query \"HKCU\\x\"").is_none());
        assert!(needs_admin("reg add \"HKLM\\Software\" /f"));
        assert!(!needs_admin("reg add \"HKCU\\Software\" /f"));
    }
    #[test]
    #[ignore] // reads this PC's registry; run with --ignored
    fn startup_scan_runs() {
        let v = startup_list().expect("scan");
        eprintln!("startup items: {}", v.len());
        for i in v.iter() {
            eprintln!("  {} | {} | {} | icon={:?}", i.id, i.enabled, i.cmd, i.icon);
        }
    }
    #[test]
    #[ignore] // reads this PC's settings (no changes); run with --ignored
    fn batch_matches_single_checks() {
        let all: Vec<serde_json::Value> = serde_json::from_str(include_str!("../data/tweaks.json")).unwrap();
        let items: Vec<(String, String)> = all
            .iter()
            .filter(|t| t["c"] == "gaming" || t["c"] == "contextmenu")
            .filter_map(|t| Some((t["id"].as_str()?.to_string(), t["check"].as_str()?.to_string())))
            .collect();
        let batch = check_batch(&items);
        for (id, chk) in &items {
            assert_eq!(batch[id], check_one(chk), "{id}");
        }
        eprintln!("checked {} items, {} on", items.len(), batch.values().filter(|v| **v).count());
    }
    #[test]
    fn tasklist_parse() {
        let p = parse_tasklist("\"chrome.exe\",\"1234\",\"Console\",\"1\",\"123,456 K\"\r\n");
        assert_eq!(p[0].pid, 1234);
        assert!((p[0].mem_mb - 120.6).abs() < 0.05);
    }
}

#[cfg(test)]
mod shell_icon_tests {
    #[test]
    #[ignore] // reads icons from this PC (writes PNGs to %APPDATA%\Cheetix\icons); run with --ignored
    fn shortcut_icons() {
        let cmds: Vec<String> = ["sysdm.cpl", "timedate.cpl", "mmsys.cpl", "main.cpl", "powercfg.cpl", "ncpa.cpl", "firewall.cpl", "intl.cpl", "joy.cpl", "devmgmt.msc", "ms-settings:display", "windowsdefender:", "regedit", "mspaint", "wt", "calc", "rundll32 sysdm.cpl,EditEnvironmentVariables", "cleanmgr"].iter().map(|s| s.to_string()).collect();
        let m = super::shell_icons(&cmds);
        for c in &cmds {
            eprintln!("{c} -> {:?} {:?}", m.get(c), m.get(c).and_then(|f| crate::arena::load_art(std::path::Path::new(f))).map(|i| i.size));
        }
        assert!(m.len() >= 8, "only {} icons", m.len());
    }
}
