; Cheetix installer (Inno Setup 6). Build: tools\build_installer.ps1
; Produces installer\Output\Cheetix-Setup-<version>.exe

#define AppName "Cheetix"
#define AppVersion "0.3.2"
#define AppExe "cheetix.exe"

[Setup]
; never change AppId: upgrades and uninstall find the installed copy by it
AppId={{6F3D9C21-7B4E-4E8A-9C55-2A1B7D3E8F70}
AppName={#AppName}
AppVersion={#AppVersion}
AppVerName={#AppName} {#AppVersion}
AppPublisher={#AppName}
VersionInfoVersion={#AppVersion}
VersionInfoProductName={#AppName}
VersionInfoDescription={#AppName} Setup
DefaultDirName={autopf}\{#AppName}
; the app was called SM7 before: move an SM7 install to the new folder instead of reusing it
UsePreviousAppDir=no
DefaultGroupName={#AppName}
DisableProgramGroupPage=yes
; Cheetix changes system settings, so it installs for all users into Program Files
PrivilegesRequired=admin
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
MinVersion=10.0
WizardStyle=modern
SetupIconFile=..\assets\cheetix.ico
UninstallDisplayIcon={app}\{#AppExe}
UninstallDisplayName={#AppName}
; close a running Cheetix before files are replaced (upgrade / uninstall)
CloseApplications=yes
RestartApplications=no
Compression=lzma2/ultra64
SolidCompression=yes
OutputDir=Output
OutputBaseFilename=Cheetix-Setup-{#AppVersion}

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"; Flags: unchecked

[Files]
Source: "..\target\release\{#AppExe}"; DestDir: "{app}"; Flags: ignoreversion

[InstallDelete]
; leftovers of the SM7 install this one replaces (same AppId)
Type: filesandordirs; Name: "{autopf}\SM7"
Type: files; Name: "{autoprograms}\SM7.lnk"
Type: files; Name: "{autodesktop}\SM7.lnk"

[Icons]
Name: "{autoprograms}\{#AppName}"; Filename: "{app}\{#AppExe}"
Name: "{autodesktop}\{#AppName}"; Filename: "{app}\{#AppExe}"; Tasks: desktopicon

[Run]
Filename: "{app}\{#AppExe}"; Description: "{cm:LaunchProgram,{#AppName}}"; Flags: nowait postinstall skipifsilent
; in-app updates run the installer with /SILENT /RELAUNCH=1: start the new version when done
Filename: "{app}\{#AppExe}"; Flags: nowait; Check: Relaunch

; Settings, switch states and the saved originals that OFF restores live in %APPDATA%\Cheetix.
; They are deliberately kept on uninstall: without them, tweaks that are still ON could
; only be reset to Windows defaults instead of your own earlier settings.

[Code]
// Windows' Restart Manager can't close Cheetix's window, so ask it to close (like clicking X;
// its state is saved after every change), and force it only if it is still running.
procedure CloseCheetix();
var Code: Integer;
begin
  // sm7.exe: the same app under its earlier name, when upgrading from it
  Exec(ExpandConstant('{sys}\taskkill.exe'), '/f /im sm7.exe', '', SW_HIDE, ewWaitUntilTerminated, Code);
  Exec(ExpandConstant('{sys}\taskkill.exe'), '/im cheetix.exe', '', SW_HIDE, ewWaitUntilTerminated, Code);
  if Code = 0 then begin
    Sleep(4000);
    Exec(ExpandConstant('{sys}\taskkill.exe'), '/f /im cheetix.exe', '', SW_HIDE, ewWaitUntilTerminated, Code);
  end;
end;

function Relaunch(): Boolean;
begin
  Result := ExpandConstant('{param:RELAUNCH|0}') = '1';
end;

function PrepareToInstall(var NeedsRestart: Boolean): String;
begin
  CloseCheetix();
  Result := '';
end;

function InitializeUninstall(): Boolean;
begin
  CloseCheetix();
  Result := True;
end;
