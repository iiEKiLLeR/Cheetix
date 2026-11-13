# Builds sm7.exe (release) and the SM7 installer.
# Needs: Rust GNU toolchain + WinLibs MinGW (for windres), Inno Setup 6 (winget install JRSoftware.InnoSetup).
$ErrorActionPreference = 'Continue'  # cargo writes progress to stderr; exit codes are checked instead
$app = Split-Path $PSScriptRoot
Set-Location $app
$env:PATH = "$env:LOCALAPPDATA\Microsoft\WinGet\Packages\BrechtSanders.WinLibs.POSIX.UCRT_Microsoft.Winget.Source_8wekyb3d8bbwe\mingw64\bin;$env:USERPROFILE\.cargo\bin;$env:PATH"

# the installer version must match the app's
$ver = (Select-String -Path Cargo.toml -Pattern '^version = "(.+)"').Matches[0].Groups[1].Value
$iss = Get-Content installer\sm7.iss -Raw
if ($iss -notmatch "#define AppVersion `"$([regex]::Escape($ver))`"") { throw "installer\sm7.iss AppVersion does not match Cargo.toml version $ver" }

cargo build --release
if ($LASTEXITCODE -ne 0) { throw 'cargo build failed' }

$iscc = @("$env:LOCALAPPDATA\Programs\Inno Setup 6\ISCC.exe", "${env:ProgramFiles(x86)}\Inno Setup 6\ISCC.exe", "$env:ProgramFiles\Inno Setup 6\ISCC.exe") | Where-Object { Test-Path $_ } | Select-Object -First 1
if (-not $iscc) { throw 'Inno Setup 6 not found: winget install JRSoftware.InnoSetup' }
& $iscc /Q installer\sm7.iss
if ($LASTEXITCODE -ne 0) { throw 'ISCC failed' }
Get-Item "installer\Output\SM7-Setup-$ver.exe"
