; Inno Setup Script for Pagic
; Defines Windows setup installer configuration for Pagic

#ifndef MyAppVersion
#define MyAppVersion "0.14.0"
#endif

#define MyAppName "Pagic"
#define MyAppPublisher "arafatic"
#define MyAppURL "https://github.com/arafatic/pagic"
#define MyAppExeName "pagic.exe"

[Setup]
AppId={{CEEEE167-9578-5601-A625-6EEF718D8D48}
AppName={#MyAppName}
AppVersion={#MyAppVersion}
AppPublisher={#MyAppPublisher}
AppPublisherURL={#MyAppURL}
AppSupportURL={#MyAppURL}
AppUpdatesURL={#MyAppURL}/releases
DefaultDirName={autopf}\{#MyAppName}
DefaultGroupName={#MyAppName}
DisableProgramGroupPage=yes
LicenseFile=..\..\LICENSE-MIT
OutputDir=..\..\
OutputBaseFilename=Pagic-windows-x64-setup
SetupIconFile=..\..\assets\app-icon\pagic.ico
UninstallDisplayIcon={app}\{#MyAppExeName}
Compression=lzma2/ultra64
SolidCompression=yes
WizardStyle=modern
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
ChangesAssociations=yes

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"

[Files]
Source: "..\..\target\release\{#MyAppExeName}"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\..\assets\app-icon\pagic.ico"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\..\LICENSE-MIT"; DestDir: "{app}"; Flags: ignoreversion skipifsourcedoesntexist
Source: "..\..\LICENSE-APACHE"; DestDir: "{app}"; Flags: ignoreversion skipifsourcedoesntexist
Source: "..\..\README.md"; DestDir: "{app}"; Flags: ignoreversion skipifsourcedoesntexist

Source: "..\..\target\font-licences\OFL-*.txt"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{group}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"; IconFilename: "{app}\pagic.ico"
Name: "{group}\{cm:UninstallProgram,{#MyAppName}}"; Filename: "{uninstallexe}"
Name: "{autodesktop}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"; IconFilename: "{app}\pagic.ico"; Tasks: desktopicon

[Run]
Filename: "{app}\{#MyAppExeName}"; Description: "{cm:LaunchProgram,{#StringChange(MyAppName, '&', '&&')}}"; Flags: nowait postinstall skipifsilent
