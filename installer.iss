[Setup]
AppName=R-Play
AppVersion=0.1.0
AppPublisher=R-Play
DefaultDirName={autopf}\R-Play
DefaultGroupName=R-Play
OutputDir=target\release
OutputBaseFilename=rplay-setup-windows-x64
Compression=lzma
SolidCompression=yes
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
ChangesEnvironment=yes

[Files]
Source: "target\release\rplay.exe"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{group}\R-Play"; Filename: "{app}\rplay.exe"
Name: "{autodesktop}\R-Play"; Filename: "{app}\rplay.exe"

[Registry]
Root: HKCU; Subkey: "Environment"; ValueType: expandsz; ValueName: "Path"; ValueData: "{olddata};{app}"; Check: NeedsAddPath('{app}')

[Code]
function NeedsAddPath(Param: string): boolean;
var
  OrigPath: string;
begin
  if not RegQueryStringValue(HKEY_CURRENT_USER, 'Environment', 'Path', OrigPath) then
  begin
    Result := True;
    exit;
  end;
  Result := Pos(';' + ExpandConstant(Param) + ';', ';' + OrigPath + ';') = 0;
end;
