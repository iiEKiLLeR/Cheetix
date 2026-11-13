; SM7 installer (Inno Setup 6). Build: tools\build_installer.ps1
; Produces installer\Output\SM7-Setup-<version>.exe

#define AppName "SM7"
#define AppVersion "0.2.0"
#define AppExe "sm7.exe"

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
DefaultGroupName={#AppName}
DisableProgramGroupPage=yes
; SM7 changes system settings, so it installs for all users into Program Files
PrivilegesRequired=admin
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
MinVersion=10.0
WizardStyle=modern
SetupIconFile=..\assets\sm7.ico
UninstallDisplayIcon={app}\{#AppExe}
UninstallDisplayName={#AppName}
; close a running SM7 before files are replaced (upgrade / uninstall)
CloseApplications=yes
RestartApplications=no
Compression=lzma2/ultra64
SolidCompression=yes
OutputDir=Output
OutputBaseFilename=SM7-Setup-{#AppVersion}

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"; Flags: unchecked

[Files]
Source: "..\target\release\{#AppExe}"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{autoprograms}\{#AppName}"; Filename: "{app}\{#AppExe}"
Name: "{autodesktop}\{#AppName}"; Filename: "{app}\{#AppExe}"; Tasks: desktopicon

[Run]
Filename: "{app}\{#AppExe}"; Description: "{cm:LaunchProgram,{#AppName}}"; Flags: nowait postinstall skipifsilent

; Settings, switch states and the saved originals that OFF restores live in %APPDATA%\SM7.
; They are deliberately kept on uninstall: without them, tweaks that are still ON could
; only be reset to Windows defaults instead of your own earlier settings.

[Code]
// Windows' Restart Manager can't close SM7's window, so ask it to close (like clicking X;
// its state is saved after every change), and force it only if it is still running.
procedure CloseSM7();
var Code: Integer;
begin
  Exec(ExpandConstant('{sys}\taskkill.exe'), '/im sm7.exe', '', SW_HIDE, ewWaitUntilTerminated, Code);
  if Code = 0 then begin
    Sleep(4000);
    Exec(ExpandConstant('{sys}\taskkill.exe'), '/f /im sm7.exe', '', SW_HIDE, ewWaitUntilTerminated, Code);
  end;
end;

function PrepareToInstall(var NeedsRestart: Boolean): String;
begin
  CloseSM7();
  Result := '';
end;

function InitializeUninstall(): Boolean;
begin
  CloseSM7();
  Result := True;
end;
