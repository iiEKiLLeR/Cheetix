# Cheetix Arena: one game session. Started hidden by Cheetix and runs on its own, so the session
# keeps going if Cheetix is closed.
#   -Mode run      apply the chosen settings, start the game through Steam, boost its processes,
#                  and put everything back when the game exits (or arena.stop appears)
#   -Mode recover  put back what an interrupted session changed (from arena-guard.json)
# Every original value is written to arena-guard.json BEFORE it is changed.
param([string]$Config, [string]$Mode = 'run')
$ErrorActionPreference = 'Continue'
$c = Get-Content -LiteralPath $Config -Raw | ConvertFrom-Json
$guardF = Join-Path $c.dir 'arena-guard.json'
$statusF = Join-Path $c.dir 'arena-status.json'
$stopF = Join-Path $c.dir 'arena.stop'

Add-Type -TypeDefinition @'
using System; using System.Runtime.InteropServices;
public static class ArenaNative {
    [DllImport("ntdll.dll")] public static extern int RtlAdjustPrivilege(int p, bool enable, bool thread, out bool was);
    [DllImport("ntdll.dll")] public static extern int NtSetInformationProcess(IntPtr h, int cls, ref int v, int len);
    [DllImport("ntdll.dll")] public static extern int NtSetSystemInformation(int cls, ref int v, int len);
    [DllImport("gdi32.dll")] public static extern int D3DKMTSetProcessSchedulingPriorityClass(IntPtr h, int cls);
    public static void Priv(int p) { bool was; RtlAdjustPrivilege(p, true, false, out was); }
    // Steam's UI thread: the thread that owns its vguiPopupWindow (it respawns the web helper)
    delegate bool EnumProc(IntPtr h, IntPtr l);
    [DllImport("user32.dll")] static extern bool EnumWindows(EnumProc f, IntPtr l);
    [DllImport("user32.dll")] static extern uint GetWindowThreadProcessId(IntPtr h, out uint pid);
    [DllImport("user32.dll", CharSet = CharSet.Unicode)] static extern int GetClassName(IntPtr h, System.Text.StringBuilder s, int n);
    [DllImport("kernel32.dll")] static extern IntPtr OpenThread(int access, bool inherit, uint tid);
    [DllImport("kernel32.dll")] static extern int SuspendThread(IntPtr h);
    [DllImport("kernel32.dll")] static extern int ResumeThread(IntPtr h);
    [DllImport("kernel32.dll")] static extern bool CloseHandle(IntPtr h);
    public static uint UiThread(uint steamPid) {
        uint found = 0;
        EnumWindows((h, l) => {
            uint pid; uint tid = GetWindowThreadProcessId(h, out pid);
            if (pid != steamPid) return true;
            var sb = new System.Text.StringBuilder(64); GetClassName(h, sb, 64);
            if (sb.ToString() == "vguiPopupWindow") { found = tid; return false; }
            return true;
        }, IntPtr.Zero);
        return found;
    }
    // 2 = THREAD_SUSPEND_RESUME; returns the previous suspend count, or -1
    public static int Thread(uint tid, bool suspend) {
        IntPtr h = OpenThread(2, false, tid); if (h == IntPtr.Zero) return -1;
        int r = suspend ? SuspendThread(h) : ResumeThread(h); CloseHandle(h); return r;
    }
}
'@
[ArenaNative]::Priv(13)  # SeProfileSingleProcessPrivilege: purge the standby list
[ArenaNative]::Priv(14)  # SeIncreaseBasePriorityPrivilege: high I/O priority

$script:t0 = (Get-Date).ToString('o')
$script:boosted = @{}
$script:notes = New-Object System.Collections.ArrayList

function Write-Status($phase, $msg) {
    $o = [ordered]@{ phase = $phase; game = $c.name; appid = "$($c.appid)"; msg = $msg; pid = $PID; started = $script:t0; procs = @($script:boosted.Values); notes = @($script:notes) }
    [IO.File]::WriteAllText($statusF, ($o | ConvertTo-Json -Compress -Depth 4))
}
function Note($s) { [void]$script:notes.Add($s) }
function Save-Guard { [IO.File]::WriteAllText($guardF, ($script:base | ConvertTo-Json -Compress -Depth 5)) }
function Active-Scheme { if ((powercfg /getactivescheme) -match '([0-9a-fA-F-]{36})') { $Matches[1] } }

function Undo($b) {
    if ($b.steam) {
        # end helpers that are left, then let Steam's UI run again: it starts a fresh helper
        $sp = Get-Process -Id $b.steam.pid -ErrorAction SilentlyContinue
        if ($sp -and $sp.ProcessName -eq 'steam') {
            Get-Process steamwebhelper -ErrorAction SilentlyContinue | Stop-Process -Force -ErrorAction SilentlyContinue
            [void][ArenaNative]::Thread([uint32]$b.steam.tid, $false)
        }
    }
    foreach ($e in @($b.lowered)) {
        try { $p = Get-Process -Id $e.pid -ErrorAction Stop; if ($p.StartTime.Ticks -eq [long]$e.ticks) { $p.PriorityClass = $e.cls } } catch {}
    }
    foreach ($s in @($b.services)) { if ($s) { Start-Service -Name $s -ErrorAction SilentlyContinue } }
    if ($b.tcp) { netsh int tcp set global autotuninglevel=$($b.tcp) | Out-Null }
    if ($b.pin) {
        powercfg /setacvalueindex $b.pin.scheme SUB_PROCESSOR PROCTHROTTLEMIN $b.pin.value | Out-Null
        powercfg /setactive $b.pin.scheme | Out-Null
    }
    if ($b.plan) { powercfg /setactive $b.plan | Out-Null }
}

if ($Mode -eq 'recover') {
    if (-not (Test-Path $guardF)) { exit 0 }
    $b = Get-Content -LiteralPath $guardF -Raw | ConvertFrom-Json
    # the session that wrote it is still running: leave it alone
    $owner = Get-Process -Id $b.owner -ErrorAction SilentlyContinue
    if ($owner -and $owner.ProcessName -match 'powershell') { exit 0 }
    Undo $b
    Remove-Item $guardF -ErrorAction SilentlyContinue
    Note 'An interrupted session was found and everything it changed was put back.'
    Write-Status 'ended' "Recovered: settings from the last $($b.game) session were put back."
    exit 0
}

Remove-Item $stopF -ErrorAction SilentlyContinue
$script:base = [ordered]@{ owner = $PID; game = $c.name; services = @(); tcp = $null; pin = $null; plan = $null; lowered = @(); steam = $null }
Save-Guard
Write-Status 'starting' 'Applying settings...'

if ($c.plan) {
    $cur = Active-Scheme
    if ($cur -and $cur -ne $c.plan) {
        $script:base.plan = $cur; Save-Guard
        powercfg /setactive $c.plan | Out-Null
        if ($LASTEXITCODE -eq 0) { Note 'Switched to the gaming power plan.' } else { Note 'Could not switch the power plan.' }
    }
}
if ($c.pin) {
    $sch = Active-Scheme
    $hex = @((powercfg /qh $sch SUB_PROCESSOR PROCTHROTTLEMIN) -match ':\s*0x[0-9a-fA-F]+') | Select-Object -Last 2 | Select-Object -First 1
    if ($sch -and $hex -match '0x([0-9a-fA-F]+)') {
        $script:base.pin = @{ scheme = $sch; value = [Convert]::ToInt32($Matches[1], 16) }; Save-Guard
        powercfg /setacvalueindex $sch SUB_PROCESSOR PROCTHROTTLEMIN 100 | Out-Null
        powercfg /setactive $sch | Out-Null
        Note "Minimum CPU state held at 100% (was $($script:base.pin.value)%)."
    } else { Note 'Could not read the minimum CPU state, so it was left alone.' }
}
if ($c.latency) {
    $lvl = "$((Get-NetTCPSetting -SettingName Internet -ErrorAction SilentlyContinue).AutoTuningLevelLocal)".ToLower()
    if ($lvl -and $lvl -ne 'disabled') {
        $script:base.tcp = $lvl; Save-Guard
        netsh int tcp set global autotuninglevel=disabled | Out-Null
        Note "TCP receive auto-tuning turned off (was $lvl)."
    }
}
$svc = @()
if ($c.services) { $svc += $c.service_list }
if ($c.search) { $svc += 'WSearch' }
foreach ($n in $svc) {
    $s = Get-Service -Name $n -ErrorAction SilentlyContinue
    if ($s -and $s.Status -eq 'Running' -and $s.StartType -ne 'Disabled') {
        $script:base.services += $n; Save-Guard
        Stop-Service -Name $n -Force -NoWait -ErrorAction SilentlyContinue
    }
}
if (@($script:base.services).Count) { Note "Paused services: $(@($script:base.services) -join ', ')." }
if ($c.standby) {
    $cmd = 4  # MemoryPurgeStandbyList
    $r = [ArenaNative]::NtSetSystemInformation(80, [ref]$cmd, 4)
    if ($r -eq 0) { Note 'Standby memory cleared.' } else { Note ('Could not clear standby memory (0x{0:X8}).' -f $r) }
}

$root = ($c.folder.TrimEnd('\') + '\')
function Game-Procs { @(Get-Process -ErrorAction SilentlyContinue | Where-Object { $_.Path -and $_.Path.StartsWith($root, [StringComparison]::OrdinalIgnoreCase) }) }

function Boost($p) {
    $d = [ordered]@{ pid = $p.Id; name = $p.ProcessName; cpu = 'unchanged'; gpu = 'unchanged'; io = 'unchanged' }
    if ($c.boost) {
        try { if ($c.cpu -ne 'Normal') { $p.PriorityClass = $c.cpu }; $d.cpu = $c.cpu } catch { $d.cpu = 'refused' }
        try {
            $h = $p.Handle
            $io = @{ Normal = 2; High = 3 }[$c.io]
            $d.io = if ([ArenaNative]::NtSetInformationProcess($h, 33, [ref]$io, 4) -eq 0) { $c.io } else { 'refused' }
            $g = @{ Normal = 2; AboveNormal = 3; High = 4 }[$c.gpu]
            $d.gpu = if ([ArenaNative]::D3DKMTSetProcessSchedulingPriorityClass($h, $g) -eq 0) { $c.gpu } else { 'refused' }
        } catch { $d.io = 'refused'; $d.gpu = 'refused' }
    }
    $script:boosted[$p.Id] = $d
}

# processes Game Focus never lowers: the shell, audio, input, voice chat, capture and Steam
$keep = 'System|Idle|csrss|wininit|winlogon|services|lsass|smss|dwm|audiodg|fontdrvhost|sihost|ctfmon|explorer|ShellExperienceHost|StartMenuExperienceHost|SearchHost|TextInputHost|LogonUI|conhost|powershell|cheetix|steam|steamwebhelper|GameOverlayUI|discord|ts3client_win64|TeamSpeak|obs64|obs32|nvcontainer|NVIDIA Share|RadeonSoftware|MsMpEng'
function Focus($game) {
    $me = (Get-Process -Id $PID).SessionId
    $ids = @($game | ForEach-Object { $_.Id })
    foreach ($p in Get-Process -ErrorAction SilentlyContinue) {
        if ($p.SessionId -ne $me -or $ids -contains $p.Id -or $p.ProcessName -match "^($keep)$") { continue }
        try {
            if ($p.PriorityClass -eq 'Normal') {
                $script:base.lowered += @{ pid = $p.Id; ticks = $p.StartTime.Ticks; cls = 'Normal' }
                $p.PriorityClass = 'BelowNormal'
            }
        } catch {}
    }
    Save-Guard
}

$procs = Game-Procs
if (-not $procs.Count) {
    if ($c.exe) {
        Write-Status 'waiting' "Starting $($c.name)..."
        Start-Process -FilePath $c.exe -WorkingDirectory (Split-Path -LiteralPath $c.exe)
    } else {
        Write-Status 'waiting' 'Starting the game through Steam...'
        Start-Process "steam://rungameid/$($c.appid)"
    }
    $deadline = (Get-Date).AddMinutes(5)
    while (-not $procs.Count -and (Get-Date) -lt $deadline -and -not (Test-Path $stopF)) {
        Start-Sleep -Seconds 2
        $procs = Game-Procs
    }
}

if ($procs.Count) {
    $procs | ForEach-Object { Boost $_ }
    if ($c.focus) { Focus $procs; Note "Game Focus lowered $(@($script:base.lowered).Count) background programs to below normal." }
    if ($c.steam) {
        $sp = @(Get-Process steam -ErrorAction SilentlyContinue)[0]
        if (-not $sp) { Note 'Steam is not running, so its web helper was left alone.' }
        else {
            $tid = [ArenaNative]::UiThread([uint32]$sp.Id)
            if (-not $tid) { Note "Steam's window could not be found, so its web helper was left alone." }
            else {
                $script:base.steam = @{ pid = $sp.Id; tid = $tid }; Save-Guard
                if ([ArenaNative]::Thread($tid, $true) -lt 0) { $script:base.steam = $null; Save-Guard; Note "Windows refused access to Steam's UI, so its web helper was left alone." }
                else {
                    $n = @(Get-Process steamwebhelper -ErrorAction SilentlyContinue).Count
                    Get-Process steamwebhelper -ErrorAction SilentlyContinue | Stop-Process -Force -ErrorAction SilentlyContinue
                    Note "Steam's web helper closed ($n processes). Steam's window and overlay come back when the game closes."
                }
            }
        }
    }
    Write-Status 'active' 'Game running.'
    $gone = 0; $tick = 0
    while ($gone -lt 3 -and -not (Test-Path $stopF)) {
        Start-Sleep -Seconds 2
        $procs = Game-Procs
        if ($procs.Count) {
            $gone = 0
            $procs | Where-Object { -not $script:boosted.ContainsKey($_.Id) } | ForEach-Object { Boost $_ }
            Write-Status 'active' 'Game running.'
        } else { $gone++ }
    }
    $why = if (Test-Path $stopF) { 'Stopped. Everything was put back.' } else { "$($c.name) closed. Everything was put back." }
} else {
    $why = if (Test-Path $stopF) { 'Stopped before the game started. Everything was put back.' } else { "The game did not start within 5 minutes, so Arena stopped and put everything back. If Steam is updating it, try again when it's done." }
}

Undo $script:base
Remove-Item $guardF, $stopF -ErrorAction SilentlyContinue
Write-Status 'ended' $why
