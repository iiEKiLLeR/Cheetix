//! Backup-and-restore engine for tweaks described as operations (data/ops.json).
//!
//! apply   – reads the current state of every target, saves it, applies the change,
//!           reads it back to verify; on any error every target is put back. The saved
//!           states are returned and kept by the app as the tweak's backup.
//! restore – writes a saved backup back and verifies each target.
//! reset   – for a tweak that was already on before the app saw it (no backup): writes
//!           the documented Windows default from the data, where one exists.
//! probe   – read-only: is each tweak on, off, partly on, or not applicable on this PC.
//!
//! All Windows access happens in one PowerShell script; Rust only builds the payload.

use serde_json::{json, Value};

/// Runs the engine. Returns the JSON object the script printed.
pub fn run(mode: &str, payload: Value, timeout_s: u64) -> Result<Value, String> {
    let mut input = payload;
    input["mode"] = json!(mode);
    let data = serde_json::to_string(&input).map_err(|e| e.to_string())?;
    // a here-string ends at a line starting with '@ - JSON never puts a newline before it
    let script = format!("$IN = @'\n{data}\n'@ | ConvertFrom-Json\n{ENGINE}");
    let out = crate::sys::ps_timeout(&script, timeout_s)?;
    let text = String::from_utf8_lossy(&out.stdout);
    let line = text.lines().rev().find(|l| l.trim_start().starts_with('{'));
    match line {
        Some(l) => serde_json::from_str(l.trim()).map_err(|e| format!("engine output unreadable: {e}")),
        None => {
            let err: String = String::from_utf8_lossy(&out.stderr).trim().chars().take(300).collect();
            Err(if err.is_empty() { "engine produced no result".into() } else { err })
        }
    }
}

/// True when any operation needs administrator rights (everything except HKCU registry values).
pub fn needs_admin(ops: &[Value]) -> bool {
    ops.iter().any(|op| {
        let hkcu = op["t"] == "reg" && op["p"].as_str().is_some_and(|p| p.to_uppercase().starts_with("HKCU\\"));
        !hkcu
    })
}

/// Plain-language lines describing what each operation changes (for the info panel).
pub fn describe(ops: &[Value]) -> Vec<String> {
    let s = |v: &Value| v.as_str().unwrap_or("").to_string();
    ops.iter()
        .map(|op| match op["t"].as_str().unwrap_or("") {
            "reg" => {
                let v = if op["v"].is_null() { "(value removed)".to_string() } else { format!("{} ({})", show(&op["v"]), s(&op["k"]).to_uppercase()) };
                format!("Registry  {}\\{} = {}", s(&op["p"]), s(&op["n"]), v)
            }
            "svc" => {
                let kind = ["Boot", "System", "Automatic", "Manual", "Disabled"].get(op["s"].as_u64().unwrap_or(9) as usize).copied().unwrap_or("?");
                let now = if op["stop"] == true { ", and stopped now" } else { "" };
                format!("Service  {} -> {kind} (Start = {}){now}", s(&op["n"]), op["s"])
            }
            "task" => format!("Scheduled task  {}{} -> Disabled", s(&op["p"]), s(&op["n"])),
            "pwr" => {
                let dc = if op["dc"].is_null() { String::new() } else { format!(", on battery {}", op["dc"]) };
                format!("Active power plan  setting {} (group {}) -> plugged in {}{}", s(&op["set"]), s(&op["sub"]), op["ac"], dc)
            }
            "keys" => format!(
                "Registry, each subkey of {}{}  {} = {}{}",
                s(&op["base"]),
                if op["has"].is_null() { String::new() } else { format!(" that has '{}'", s(&op["has"])) },
                s(&op["n"]),
                show(&op["v"]),
                if op["only"] == true { " (only where the driver already defines it)" } else { "" }
            ),
            "mm" => format!("Memory manager  {} -> Disabled (Disable-MMAgent)", s(&op["n"])),
            "feat" => format!("Optional feature  {} -> Disabled (files kept)", s(&op["n"])),
            other => format!("{other} operation"),
        })
        .collect()
}

fn show(v: &Value) -> String {
    match v {
        Value::String(s) => format!("\"{s}\""),
        Value::Number(n) => {
            let x = n.as_u64().unwrap_or(0);
            if x > 9999 { format!("{x} (0x{x:X})") } else { x.to_string() }
        }
        other => other.to_string(),
    }
}

/// One saved state, as text for the info panel ("what turning OFF puts back").
pub fn describe_backup(b: &Value) -> Vec<String> {
    let mut out = vec![];
    for e in b.as_array().into_iter().flatten() {
        let a = &e["atom"];
        let st = &e["state"];
        let s = |v: &Value| v.as_str().unwrap_or("").to_string();
        let line = match a["t"].as_str().unwrap_or("") {
            "reg" => {
                let what = if st["present"] == true {
                    format!("{} ({})", s(&st["data"]), s(&st["kind"]))
                } else if st["key"] == false {
                    "did not exist (key and value will be removed)".into()
                } else {
                    "did not exist (value will be removed)".into()
                };
                format!("{}\\{} was {}", s(&a["p"]), s(&a["n"]), what)
            }
            "task" => format!("Task {}{} was {}", s(&a["p"]), s(&a["n"]), if st["enabled"] == true { "enabled" } else { "disabled" }),
            "pwr" => format!("Power setting {} was plugged in {}, on battery {} (plan {})", s(&a["set"]), st["ac"], st["dc"], s(&st["scheme"])),
            "mm" => format!("{} was {}", s(&a["n"]), if st["enabled"] == true { "enabled" } else { "disabled" }),
            "feat" => format!("Feature {} was {}", s(&a["n"]), if st["enabled"] == true { "enabled" } else { "disabled" }),
            _ => continue,
        };
        out.push(line);
    }
    out
}

const ENGINE: &str = r#"
$ErrorActionPreference = 'Stop'
$ProgressPreference = 'SilentlyContinue'
$KINDS = @{ dword = 'DWord'; sz = 'String'; expand = 'ExpandString'; multi = 'MultiString'; qword = 'QWord'; binary = 'Binary' }

function Split-Reg([string]$p) {
  $i = $p.IndexOf('\')
  if ($i -lt 0) { $h = $p; $s = '' } else { $h = $p.Substring(0, $i); $s = $p.Substring($i + 1) }
  $hive = switch ($h.ToUpper()) { 'HKLM' { 'LocalMachine' } 'HKCU' { 'CurrentUser' } 'HKU' { 'Users' } 'HKCR' { 'ClassesRoot' } default { throw "unknown hive $h" } }
  $base = [Microsoft.Win32.RegistryKey]::OpenBaseKey($hive, [Microsoft.Win32.RegistryView]::Registry64)
  return @($base, $s)
}
function Open-Key([string]$p, [bool]$write) { $b, $s = Split-Reg $p; return $b.OpenSubKey($s, $write) }
function Key-Exists([string]$p) { $k = Open-Key $p $false; if ($null -eq $k) { return $false }; $k.Close(); return $true }

function Norm([string]$kind, $v) {
  if ($null -eq $v) { return $null }
  switch ($kind) {
    'DWord' { return [string]([uint32]([int64]$v -band 0xFFFFFFFFL)) }
    'QWord' { if ($v -is [long] -and $v -lt 0) { return [string]([BitConverter]::ToUInt64([BitConverter]::GetBytes([long]$v), 0)) }; return [string]([uint64]$v) }
    'Binary' { if ($v -is [byte[]]) { return ([BitConverter]::ToString($v) -replace '-', '') }; return ([string]$v).ToUpper() }
    'MultiString' { return (@($v) -join "`n") }
    default { return [string]$v }
  }
}
function U32([string]$s) { return [BitConverter]::ToInt32([BitConverter]::GetBytes([uint32]$s), 0) }
function To-Raw([string]$kind, $data) {
  switch ($kind) {
    'DWord' { return (U32 $data) }
    'QWord' { return [BitConverter]::ToInt64([BitConverter]::GetBytes([uint64]$data), 0) }
    'Binary' { $h = [string]$data; $b = New-Object byte[] ($h.Length / 2); for ($i = 0; $i -lt $b.Length; $i++) { $b[$i] = [Convert]::ToByte($h.Substring($i * 2, 2), 16) }; return ,$b }
    'MultiString' { return ,([string[]](([string]$data) -split "`n")) }
    default { return [string]$data }
  }
}
function Want-Reg($k, $v) {
  if ($null -eq $v) { return [ordered]@{ present = $false } }
  $kind = $KINDS[[string]$k]
  if (-not $kind) { throw "unknown value type $k" }
  if ($kind -eq 'DWord') { $n = [string]([uint32]([int64]$v -band 0xFFFFFFFFL)) } else { $n = Norm $kind $v }
  return [ordered]@{ present = $true; kind = $kind; data = $n }
}

# ---- atoms: one target each ----
function Atoms($ops) {
  foreach ($op in @($ops)) {
    switch ($op.t) {
      'reg' { [pscustomobject]@{ t = 'reg'; p = $op.p; n = $op.n; gate = $op.gate; want = (Want-Reg $op.k $op.v); def = $(if ($op.PSObject.Properties['d']) { Want-Reg $op.k $op.d } else { $null }) } }
      'svc' { $p = "HKLM\SYSTEM\CurrentControlSet\Services\$($op.n)"; [pscustomobject]@{ t = 'reg'; p = $p; n = 'Start'; gate = $p; want = (Want-Reg 'dword' $op.s); def = $(if ($null -ne $op.d) { Want-Reg 'dword' $op.d } else { $null }) } }
      'task' { [pscustomobject]@{ t = 'task'; p = $op.p; n = $op.n; want = [ordered]@{ exists = $true; enabled = $false }; def = [ordered]@{ exists = $true; enabled = $true } } }
      'pwr' { [pscustomobject]@{ t = 'pwr'; sub = $op.sub; set = $op.set; want = [ordered]@{ exists = $true; ac = [int64]$op.ac; dc = $op.dc }; def = $(if ($null -ne $op.d) { [ordered]@{ exists = $true; ac = [int64]$op.d[0]; dc = $op.d[1] } } else { $null }) } }
      'mm' { [pscustomobject]@{ t = 'mm'; n = $op.n; want = [ordered]@{ exists = $true; enabled = [bool]$op.on }; def = [ordered]@{ exists = $true; enabled = [bool]$op.d } } }
      'feat' { [pscustomobject]@{ t = 'feat'; n = $op.n; want = [ordered]@{ exists = $true; enabled = $false }; def = $null } }
      'keys' {
        $k = Open-Key $op.base $false
        if ($null -eq $k) { continue }
        $subs = @($k.GetSubKeyNames() | Where-Object { $_ -match $op.sub }); $k.Close()
        foreach ($s in $subs) {
          $p = "$($op.base)\$s"
          try { $sk = Open-Key $p $false } catch { continue }
          if ($null -eq $sk) { continue }
          try {
            $names = @($sk.GetValueNames())
            if ($op.has -and ($names -notcontains $op.has)) { continue }
            if ($op.only -and ($names -notcontains $op.n)) { continue }
          } finally { $sk.Close() }
          [pscustomobject]@{ t = 'reg'; p = $p; n = $op.n; gate = $p; want = (Want-Reg $op.k $op.v); def = $null }
        }
      }
      default { throw "unknown operation $($op.t)" }
    }
  }
}

function Active-Scheme { $o = (& powercfg /getactivescheme) -join ' '; if ($o -match '([0-9a-fA-F]{8}-[0-9a-fA-F-]{27})') { return $Matches[1].ToLower() }; throw 'no active power scheme' }

function Read-Atom($a, $scheme) {
  switch ($a.t) {
    'reg' {
      if ($a.gate -and -not (Key-Exists $a.gate)) { return [ordered]@{ na = $true } }
      $k = Open-Key $a.p $false
      if ($null -eq $k) {
        # remember the highest missing parent: restoring removes every key the tweak created
        $parts = $a.p.Split('\'); $top = $a.p
        for ($i = $parts.Count - 1; $i -ge 2; $i--) { $anc = ($parts[0..($i - 1)] -join '\'); if (Key-Exists $anc) { break }; $top = $anc }
        return [ordered]@{ key = $false; present = $false; top = $top }
      }
      try {
        if (@($k.GetValueNames()) -notcontains $a.n) { return [ordered]@{ key = $true; present = $false } }
        $kind = $k.GetValueKind($a.n).ToString()
        $raw = $k.GetValue($a.n, $null, [Microsoft.Win32.RegistryValueOptions]::DoNotExpandEnvironmentNames)
        return [ordered]@{ key = $true; present = $true; kind = $kind; data = (Norm $kind $raw) }
      } finally { $k.Close() }
    }
    'task' {
      $t = Get-ScheduledTask -TaskPath $a.p -TaskName $a.n -ErrorAction SilentlyContinue
      if (-not $t) { return [ordered]@{ na = $true } }
      return [ordered]@{ exists = $true; enabled = ([string]$t.State -ne 'Disabled') }
    }
    'pwr' {
      $sc = $scheme; if (-not $sc) { $sc = Active-Scheme }
      $out = & powercfg /qh $sc $a.sub $a.set 2>$null  # /qh: also hidden settings (EPP, USB 3 LPM...)
      if ($LASTEXITCODE -ne 0 -or -not $out) { return [ordered]@{ na = $true } }
      $hex = @([regex]::Matches(($out -join "`n"), ':\s*0x([0-9a-fA-F]{8})\s*$', 'Multiline') | ForEach-Object { [int64][Convert]::ToUInt32($_.Groups[1].Value, 16) })
      if ($hex.Count -lt 2) { return [ordered]@{ na = $true } }
      return [ordered]@{ exists = $true; scheme = $sc; ac = $hex[$hex.Count - 2]; dc = $hex[$hex.Count - 1] }
    }
    'mm' { $m = Get-MMAgent; return [ordered]@{ exists = $true; enabled = [bool]$m.($a.n) } }
    'feat' {
      $f = Get-WindowsOptionalFeature -Online -FeatureName $a.n -ErrorAction SilentlyContinue
      if (-not $f) { return [ordered]@{ na = $true } }
      return [ordered]@{ exists = $true; enabled = ([string]$f.State -match '^Enable') }
    }
  }
}

function Write-Atom($a, $st) {
  switch ($a.t) {
    'reg' {
      if (-not $st.present) {
        $k = Open-Key $a.p $true
        if ($null -eq $k) { return }
        try { $k.DeleteValue($a.n, $false) } finally { $k.Close() }
        if ($st.key -eq $false) {
          # delete the keys this tweak created, bottom-up, but only while they are empty
          $cur = $a.p; $top = $(if ($st.top) { $st.top } else { $a.p })
          while ($true) {
            $c = Open-Key $cur $false
            if ($null -eq $c) { $empty = $true } else { $empty = ($c.ValueCount -eq 0 -and $c.SubKeyCount -eq 0); $c.Close() }
            if (-not $empty) { break }
            $b, $s = Split-Reg $cur; $b.DeleteSubKey($s, $false)
            if ($cur -eq $top -or $cur.IndexOf('\') -lt 0) { break }
            $cur = $cur.Substring(0, $cur.LastIndexOf('\'))
          }
        }
        return
      }
      $b, $s = Split-Reg $a.p
      $k = $b.CreateSubKey($s)
      try { $k.SetValue($a.n, (To-Raw $st.kind $st.data), [Microsoft.Win32.RegistryValueKind]$st.kind) } finally { $k.Close() }
    }
    'task' {
      if ($st.enabled) { Enable-ScheduledTask -TaskPath $a.p -TaskName $a.n | Out-Null } else { Disable-ScheduledTask -TaskPath $a.p -TaskName $a.n | Out-Null }
    }
    'pwr' {
      $sc = $st.scheme; if (-not $sc) { $sc = Active-Scheme }
      & powercfg /setacvalueindex $sc $a.sub $a.set ([int64]$st.ac) | Out-Null
      if ($LASTEXITCODE -ne 0) { throw "powercfg could not set $($a.set)" }
      if ($null -ne $st.dc) { & powercfg /setdcvalueindex $sc $a.sub $a.set ([int64]$st.dc) | Out-Null; if ($LASTEXITCODE -ne 0) { throw "powercfg could not set $($a.set)" } }
      if ($sc -eq (Active-Scheme)) { & powercfg /setactive $sc | Out-Null }
    }
    'mm' { $p = @{}; $p[$a.n] = $true; if ($st.enabled) { Enable-MMAgent @p } else { Disable-MMAgent @p } }
    'feat' {
      if ($st.enabled) { Enable-WindowsOptionalFeature -Online -FeatureName $a.n -NoRestart | Out-Null }
      else { Disable-WindowsOptionalFeature -Online -FeatureName $a.n -NoRestart | Out-Null }
    }
  }
}

# does state $cur satisfy $want (want may only name some fields)
function Same($a, $cur, $want) {
  if ($cur.na) { return $false }
  switch ($a.t) {
    'reg' { if (-not $want.present) { return (-not $cur.present) }; return ($cur.present -and $cur.kind -eq $want.kind -and $cur.data -eq $want.data) }
    'pwr' { return ($cur.ac -eq [int64]$want.ac -and ($null -eq $want.dc -or $cur.dc -eq [int64]$want.dc)) }
    default { return ([bool]$cur.enabled -eq [bool]$want.enabled) }
  }
}
function Label($a) {
  switch ($a.t) { 'reg' { "$($a.p)\$($a.n)" } 'task' { "task $($a.p)$($a.n)" } 'pwr' { "power setting $($a.set)" } 'mm' { "memory feature $($a.n)" } 'feat' { "feature $($a.n)" } }
}
function Strip($a) { $o = [ordered]@{}; foreach ($p in $a.PSObject.Properties) { if ($p.Name -ne 'want' -and $p.Name -ne 'def') { $o[$p.Name] = $p.Value } }; return $o }
function Fix-Message([string]$m) { if ($m -match 'denied|unauthori|privilege|0x80070005') { return "ACCESS DENIED: $m" }; return $m }

$res = [ordered]@{ ok = $false }
switch ($IN.mode) {
  'apply' {
    $backup = New-Object System.Collections.ArrayList
    $skipped = New-Object System.Collections.ArrayList
    try {
      $atoms = @(Atoms $IN.ops)
      $plan = @()
      # read everything first, so targets that depend on each other are saved as they were
      foreach ($a in $atoms) {
        $cur = Read-Atom $a $null
        if ($cur.na) { [void]$skipped.Add((Label $a)); continue }
        $plan += ,@($a, $cur)
      }
      if ($plan.Count -eq 0) { throw 'NOT_SUPPORTED: none of the settings this tweak changes exist on this PC' }
      foreach ($x in $plan) {
        $a = $x[0]; $cur = $x[1]
        [void]$backup.Add([ordered]@{ atom = (Strip $a); state = $cur })
        if (-not (Same $a $cur $a.want)) { $w = $a.want; if ($a.t -eq 'pwr') { $w = [ordered]@{ scheme = $cur.scheme; ac = $a.want.ac; dc = $a.want.dc } }; Write-Atom $a $w }
      }
      foreach ($x in $plan) {
        $a = $x[0]
        $after = Read-Atom $a $x[1].scheme
        if (-not (Same $a $after $a.want)) { throw "VERIFY FAILED: $(Label $a) did not take the new value (it may be protected or overridden by policy)" }
      }
      $res.ok = $true
    } catch {
      $res.error = Fix-Message $_.Exception.Message
      for ($i = $backup.Count - 1; $i -ge 0; $i--) { try { Write-Atom $backup[$i].atom $backup[$i].state } catch { $res.rollback_error = $_.Exception.Message } }
      $res.rolled_back = ($backup.Count -gt 0)
    }
    $res.backup = $backup; $res.skipped = $skipped
  }
  'restore' {
    $failed = New-Object System.Collections.ArrayList
    foreach ($e in @($IN.backup)) {
      $a = [pscustomobject]$e.atom
      try {
        Write-Atom $a $e.state
        $after = Read-Atom $a $e.state.scheme
        $want = $e.state
        if (-not (Same $a $after $want)) { [void]$failed.Add("$(Label $a): read-back does not match the saved value") }
      } catch { [void]$failed.Add("$(Label $a): $(Fix-Message $_.Exception.Message)") }
    }
    $res.ok = ($failed.Count -eq 0); $res.failed = $failed
  }
  'reset' {
    $failed = New-Object System.Collections.ArrayList
    $done = New-Object System.Collections.ArrayList
    foreach ($a in @(Atoms $IN.ops)) {
      $cur = Read-Atom $a $null
      if ($cur.na) { continue }
      if ($null -eq $a.def) { [void]$failed.Add("$(Label $a): no saved original and no known Windows default"); continue }
      try {
        $w = $a.def; if ($a.t -eq 'pwr') { $w = [ordered]@{ scheme = $cur.scheme; ac = $a.def.ac; dc = $a.def.dc } }
        Write-Atom $a $w
        if (-not (Same $a (Read-Atom $a $cur.scheme) $a.def)) { [void]$failed.Add("$(Label $a): read-back does not match the default") } else { [void]$done.Add((Label $a)) }
      } catch { [void]$failed.Add("$(Label $a): $(Fix-Message $_.Exception.Message)") }
    }
    $res.ok = ($failed.Count -eq 0); $res.failed = $failed; $res.done = $done
  }
  'probe' {
    $states = [ordered]@{}
    $errors = [ordered]@{}
    $res.errors = $errors
    foreach ($item in @($IN.items)) {
      try {
        $atoms = @(Atoms $item.ops); $n = 0; $on = 0
        foreach ($a in $atoms) { $cur = Read-Atom $a $null; if ($cur.na) { continue }; $n++; if (Same $a $cur $a.want) { $on++ } }
        $states[$item.key] = $(if ($n -eq 0) { 'na' } elseif ($on -eq $n) { 'on' } elseif ($on -eq 0) { 'off' } else { 'partial' })
      } catch { $states[$item.key] = 'error'; $errors[$item.key] = "$($_.Exception.Message) @ line $($_.InvocationInfo.ScriptLineNumber)" }
    }
    $res.ok = $true; $res.states = $states
  }
}
ConvertTo-Json -InputObject $res -Depth 12 -Compress
"#;

#[cfg(test)]
mod tests {
    use super::*;

    /// Only touches HKCU\Software\CheetixSelfTest, a key this test creates and deletes.
    #[test]
    #[ignore] // runs PowerShell against the registry sandbox; run with --ignored
    fn registry_round_trip_restores_exactly() {
        let base = r"HKCU\Software\CheetixSelfTest";
        crate::sys::ps(&format!("Remove-Item 'Registry::HKEY_CURRENT_USER\\Software\\CheetixSelfTest' -Recurse -ErrorAction SilentlyContinue; New-Item 'Registry::HKEY_CURRENT_USER\\Software\\CheetixSelfTest\\Existing' -Force | Out-Null; Set-ItemProperty 'Registry::HKEY_CURRENT_USER\\Software\\CheetixSelfTest\\Existing' -Name Str -Value 'hello' -Type String; Set-ItemProperty 'Registry::HKEY_CURRENT_USER\\Software\\CheetixSelfTest\\Existing' -Name Bin -Value ([byte[]](1,2,255)) -Type Binary; Set-ItemProperty 'Registry::HKEY_CURRENT_USER\\Software\\CheetixSelfTest\\Existing' -Name Gone -Value 7 -Type DWord")).unwrap();
        let ops = json!([
            {"t":"reg","p":format!("{base}\\Existing"),"n":"Str","k":"dword","v":4294967295u64},   // type and value change
            {"t":"reg","p":format!("{base}\\Existing"),"n":"Bin","k":"binary","v":"00FF"},
            {"t":"reg","p":format!("{base}\\Existing"),"n":"Gone","k":"dword","v":null},          // value removed
            {"t":"reg","p":format!("{base}\\New\\Deep"),"n":"Q","k":"qword","v":18446744073709551615u64}, // key created
            {"t":"reg","p":format!("{base}\\Existing"),"n":"M","k":"multi","v":"a\nb"},
            {"t":"svc","n":"CheetixNoSuchService","s":4,"d":3},                                   // skipped: not on this PC
        ]);
        let snapshot = |ops: &Value| {
            let r = run("probe", json!({"items":[{"key":"k","ops":ops}]}), 60).unwrap();
            eprintln!("probe: {r}");
            r["states"]["k"].clone()
        };
        assert_eq!(snapshot(&ops), "off");
        let r = run("apply", json!({ "ops": ops }), 60).unwrap();
        assert_eq!(r["ok"], true, "{r}");
        assert_eq!(r["skipped"].as_array().unwrap().len(), 1);
        assert_eq!(snapshot(&ops), "on");
        let r2 = run("restore", json!({ "backup": r["backup"] }), 60).unwrap();
        assert_eq!(r2["ok"], true, "{r2}");
        let check = crate::sys::ps("$k='Registry::HKEY_CURRENT_USER\\Software\\CheetixSelfTest'; $e=Get-ItemProperty \"$k\\Existing\"; $kind=(Get-Item \"$k\\Existing\").GetValueKind('Str'); \"$($e.Str)|$kind|$(($e.Bin) -join ',')|$($e.Gone)|$([bool]$e.PSObject.Properties['M'])|$(Test-Path \"$k\\New\")\"").unwrap();
        let got = String::from_utf8_lossy(&check.stdout).trim().to_string();
        assert_eq!(got, "hello|String|1,2,255|7|False|False");

        // failure in the middle: the first change must be rolled back
        let bad = json!([
            {"t":"reg","p":format!("{base}\\Existing"),"n":"Str","k":"sz","v":"changed"},
            {"t":"reg","p":"HKLM\\SYSTEM\\CurrentControlSet\\Control\\Session Manager","n":"CheetixSelfTest","k":"nonsense","v":1},
        ]);
        let r3 = run("apply", json!({ "ops": bad }), 60).unwrap();
        assert_eq!(r3["ok"], false);
        let s = crate::sys::ps("(Get-ItemProperty 'Registry::HKEY_CURRENT_USER\\Software\\CheetixSelfTest\\Existing').Str").unwrap();
        assert_eq!(String::from_utf8_lossy(&s.stdout).trim(), "hello", "rollback failed: {r3}");

        // nothing on this PC -> not supported, nothing written
        let r4 = run("apply", json!({ "ops": [{"t":"svc","n":"CheetixNoSuchService","s":4,"d":3}] }), 60).unwrap();
        assert!(r4["error"].as_str().unwrap().starts_with("NOT_SUPPORTED"), "{r4}");
        crate::sys::ps("Remove-Item 'Registry::HKEY_CURRENT_USER\\Software\\CheetixSelfTest' -Recurse").unwrap();
    }

    /// Creates a throw-away service, scheduled task and power plan, disables/changes them through
    /// the engine, restores, checks, and deletes them again. Needs an elevated session.
    #[test]
    #[ignore]
    fn service_task_power_round_trip_on_sandbox_objects() {
        let setup = r#"
sc.exe create CheetixSelfTest binPath= "C:\Windows\System32\cmd.exe /c exit" start= demand | Out-Null
$a = New-ScheduledTaskAction -Execute 'cmd.exe' -Argument '/c exit'
$t = New-ScheduledTaskTrigger -Once -At ([datetime]'2099-01-01')
Register-ScheduledTask -TaskPath '\CheetixSelfTest\' -TaskName 'Probe' -Action $a -Trigger $t -Force | Out-Null
$g = ((powercfg /duplicatescheme SCHEME_CURRENT) -join ' ') -replace '.*?([0-9a-fA-F]{8}-[0-9a-fA-F-]{27}).*','$1'
powercfg /changename $g 'Cheetix self-test' | Out-Null
"PLAN=$g"
"#;
        let out = crate::sys::ps(setup).unwrap();
        let text = String::from_utf8_lossy(&out.stdout).to_string();
        let plan = text.lines().find_map(|l| l.strip_prefix("PLAN=")).unwrap().trim().to_string();
        let cleanup = format!("sc.exe delete CheetixSelfTest | Out-Null; Unregister-ScheduledTask -TaskPath '\\CheetixSelfTest\\' -TaskName 'Probe' -Confirm:$false; powercfg /delete {plan}");
        let result = std::panic::catch_unwind(|| {
            let ops = json!([
                {"t":"svc","n":"CheetixSelfTest","s":4,"d":3},
                {"t":"task","p":"\\CheetixSelfTest\\","n":"Probe","d":true},
            ]);
            let r = run("apply", json!({ "ops": ops }), 120).unwrap();
            assert_eq!(r["ok"], true, "{r}");
            let st = crate::sys::ps("(Get-ItemProperty 'HKLM:\\SYSTEM\\CurrentControlSet\\Services\\CheetixSelfTest').Start; (Get-ScheduledTask -TaskPath '\\CheetixSelfTest\\' -TaskName Probe).State").unwrap();
            assert_eq!(String::from_utf8_lossy(&st.stdout).split_whitespace().collect::<Vec<_>>(), ["4", "Disabled"]);
            let r2 = run("restore", json!({ "backup": r["backup"] }), 120).unwrap();
            assert_eq!(r2["ok"], true, "{r2}");
            let st = crate::sys::ps("(Get-ItemProperty 'HKLM:\\SYSTEM\\CurrentControlSet\\Services\\CheetixSelfTest').Start; (Get-ScheduledTask -TaskPath '\\CheetixSelfTest\\' -TaskName Probe).State").unwrap();
            assert_eq!(String::from_utf8_lossy(&st.stdout).split_whitespace().collect::<Vec<_>>(), ["3", "Ready"]);

            // power setting on the inactive self-test plan: change, verify, restore
            let sub = "2a737441-1930-4402-8d77-b2bebba308a3";
            let set = "d4e98f31-5ffe-4ce1-be31-1b38b384c009";
            let atom = json!({"t":"pwr","sub":sub,"set":set});
            let before = crate::sys::ps(&format!("powercfg /qh {plan} {sub} {set}")).unwrap();
            let target = if String::from_utf8_lossy(&before.stdout).contains("Current AC Power Setting Index: 0x00000000") { 2 } else { 0 };
            let backup = json!([{ "atom": atom, "state": {"exists":true,"scheme":plan,"ac":target,"dc":null} }]);
            let r3 = run("restore", json!({ "backup": backup }), 60).unwrap(); // write via the restore path
            assert_eq!(r3["ok"], true, "{r3}");
            let after = crate::sys::ps(&format!("powercfg /qh {plan} {sub} {set}")).unwrap();
            assert!(String::from_utf8_lossy(&after.stdout).contains(&format!("Current AC Power Setting Index: 0x0000000{target}")));
        });
        crate::sys::ps(&cleanup).unwrap();
        if let Err(e) = result {
            std::panic::resume_unwind(e);
        }
    }
}

#[cfg(test)]
mod probe_all {
    use super::*;
    /// Read-only: reports the state of every engine tweak on this PC (nothing is changed).
    #[test]
    #[ignore]
    fn probe_every_tweak_read_only() {
        let mut all: Vec<Value> = serde_json::from_str(include_str!("../data/ops.json")).unwrap();
        // the original tweaks that now use saved-original restore
        let legacy: Vec<Value> = serde_json::from_str(include_str!("../data/tweaks.json")).unwrap();
        all.extend(legacy.into_iter().filter(|t| t["ops"].is_array()));
        let items: Vec<Value> = all.iter().map(|t| json!({"key": format!("{}|{}", t["c"].as_str().unwrap(), t["id"].as_str().unwrap()), "ops": t["ops"]})).collect();
        let r = run("probe", json!({ "items": items }), 600).unwrap();
        let mut counts = std::collections::BTreeMap::new();
        for (k, v) in r["states"].as_object().unwrap() {
            *counts.entry(v.as_str().unwrap().to_string()).or_insert(0) += 1;
            if v != "off" {
                eprintln!("{v:8} {k}");
            }
        }
        eprintln!("errors: {}", r["errors"]);
        eprintln!("totals: {counts:?}");
        assert_eq!(r["states"].as_object().unwrap().len(), all.len());
    }
}

#[cfg(test)]
mod single {
    use super::*;
    /// A tweak with one target: its backup must still be a list and restore must work.
    #[test]
    #[ignore]
    fn single_target_round_trip() {
        crate::sys::ps(r"New-Item 'HKCU:\Software\CheetixSelfTest1' -Force | Out-Null").unwrap();
        let ops = json!([{"t":"reg","p":r"HKCU\Software\CheetixSelfTest1","n":"V","k":"dword","v":1}]);
        let r = run("apply", json!({ "ops": ops }), 60).unwrap();
        eprintln!("apply: {r}");
        assert_eq!(r["ok"], true);
        let r2 = run("restore", json!({ "backup": r["backup"] }), 60).unwrap();
        eprintln!("restore: {r2}");
        let left = crate::sys::ps(r"[bool](Get-ItemProperty 'HKCU:\Software\CheetixSelfTest1' -Name V -ErrorAction SilentlyContinue)").unwrap();
        crate::sys::ps(r"Remove-Item 'HKCU:\Software\CheetixSelfTest1' -Recurse").unwrap();
        assert_eq!(r2["ok"], true);
        assert_eq!(String::from_utf8_lossy(&left.stdout).trim(), "False");
    }
}
