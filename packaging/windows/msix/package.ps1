<# Build an unsigned, Store-identity MSIX validation artifact. Never publishes or installs.
   Run from an x64 Visual Studio developer PowerShell with Rust and Windows SDK installed.
   Native installation testing is separate and uses an ephemeral test certificate.
#>
param([string] $OutputDirectory = '', [switch] $SkipBuild)
$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest
if (-not $IsWindows) { throw 'MSIX packaging requires native Windows and the Windows SDK.' }
$Root = (Resolve-Path (Join-Path $PSScriptRoot '../../..')).Path
$Identity = Get-Content -Raw (Join-Path $PSScriptRoot 'identity.json') | ConvertFrom-Json
if (-not $OutputDirectory) { $OutputDirectory = Join-Path $Root 'dist/msix' }
$OutputDirectory = [IO.Path]::GetFullPath($OutputDirectory)
# Never delete or reuse an old staging tree. Each invocation has its own scratch directory.
$Work = Join-Path $OutputDirectory ('work-' + [guid]::NewGuid())
$Stage = Join-Path $Work 'stage'
$Unpacked = Join-Path $Work 'unpacked'
New-Item -ItemType Directory -Force $Work | Out-Null
function Invoke-Checked([scriptblock] $Command) {
  & $Command
  if ($LASTEXITCODE -ne 0) { throw "Native command failed with exit code $LASTEXITCODE" }
}
try {
  Push-Location $Root
  $SdkRoot = Join-Path ${env:ProgramFiles(x86)} 'Windows Kits/10/bin'
  $Sdk = Get-ChildItem $SdkRoot -Directory | Where-Object {
    $_.Name -match '^10\.0\.\d+\.0$' -and
    (Test-Path (Join-Path $_.FullName 'x64/makeappx.exe')) -and
    (Test-Path (Join-Path $_.FullName 'x64/makepri.exe')) -and
    (Test-Path (Join-Path $_.FullName 'x64/signtool.exe'))
  } | Sort-Object { [version]$_.Name } -Descending | Select-Object -First 1
  if (-not $Sdk) { throw 'Install the Windows SDK including MakeAppx, MakePri and SignTool.' }
  $Tools = Join-Path $Sdk.FullName 'x64'
  $env:PATH = $Tools + ';' + $env:PATH
  $env:PAGIC_FONTS_REQUIRED = '1'
  $env:PAGIC_REQUIRE_WINRES = '1'
  # Same static CRT policy as the existing Windows packaging; no VC redist prerequisite.
  $FlagName = 'CARGO_TARGET_X86_64_PC_WINDOWS_MSVC_RUSTFLAGS'
  if ((Test-Path "Env:$FlagName") -and (Get-Item "Env:$FlagName").Value -ne '-C target-feature=+crt-static') {
    throw "Remove conflicting $FlagName before packaging."
  }
  [Environment]::SetEnvironmentVariable($FlagName, '-C target-feature=+crt-static')
  if ($env:RUSTFLAGS -or $env:CARGO_ENCODED_RUSTFLAGS) { throw 'Remove global Rust flags before packaging.' }
  Invoke-Checked { python packaging/check-icons.py }
  Invoke-Checked { python packaging/check-identity.py }
  & (Join-Path $Root 'packaging/windows/test-font-notices.ps1')
  Invoke-Checked { python packaging/windows/msix/msix.py preflight }
  if (-not $SkipBuild) {
    Invoke-Checked { cargo build --locked --release -p pagic -p pagic-cli --target $Identity.target }
  } else {
    Write-Warning 'SkipBuild requires binaries built with the same font override and static CRT flags.'
  }
  $TargetDir = if ($env:CARGO_TARGET_DIR) { $env:CARGO_TARGET_DIR } else { Join-Path $Root 'target' }
  $Binaries = Join-Path $TargetDir ($Identity.target + '/release')
  Invoke-Checked { python packaging/windows/msix/msix.py stage --destination $Stage --binaries $Binaries }
  . (Join-Path $Root 'packaging/windows/font-notices.ps1')
  Copy-PagicFontNotices $Root (Join-Path $Stage 'Legal/fonts')
  Invoke-Checked { python packaging/windows/msix/msix.py fonts --stage $Stage }
  # Record PE imports for review. Missing non-system runtime DLLs must be resolved before submission.
  foreach ($Binary in 'pagic.exe', 'pagic-cli.exe') {
    Invoke-Checked { dumpbin /dependents (Join-Path $Binaries $Binary) } |
      Set-Content -Encoding utf8 (Join-Path $Work ($Binary + '.imports.txt'))
  }
  $Config = Join-Path $Work 'priconfig.xml'
  Invoke-Checked { & (Join-Path $Tools 'makepri.exe') createconfig /cf $Config /dq en-US /o }
  # Keep scale/targetsize variants in this single package, not separate resource packs.
  [xml] $PriConfig = Get-Content -Raw $Config
  foreach ($Node in @($PriConfig.SelectNodes('//packaging'))) { $null = $Node.ParentNode.RemoveChild($Node) }
  $PriConfig.Save($Config)
  Invoke-Checked { & (Join-Path $Tools 'makepri.exe') new /pr $Stage /cf $Config /mn (Join-Path $Stage 'AppxManifest.xml') /of (Join-Path $Stage 'resources.pri') /o }
  [xml] $Manifest = Get-Content -Raw (Join-Path $Stage 'AppxManifest.xml')
  $Version = $Manifest.Package.Identity.Version
  $Package = Join-Path $OutputDirectory "Pagic-$Version-x64.msix"
  if (Test-Path $Package) { throw "Refusing to overwrite existing package: $Package" }
  # No /nv: keep Microsoft's semantic/schema validation enabled.
  Invoke-Checked { & (Join-Path $Tools 'makeappx.exe') pack /d $Stage /p $Package /o }
  Invoke-Checked { & (Join-Path $Tools 'makeappx.exe') unpack /p $Package /d $Unpacked /o }
  Invoke-Checked { python packaging/windows/msix/msix.py verify --stage $Stage --package $Package --report (Join-Path $OutputDirectory 'validation.json') }
  # Compare actual SDK-unpacked payload as well as direct archive hashes.
  foreach ($File in Get-ChildItem $Stage -Recurse -File) {
    $Relative = [IO.Path]::GetRelativePath($Stage, $File.FullName)
    $Extracted = Join-Path $Unpacked $Relative
    if ((Get-FileHash $File.FullName).Hash -ne (Get-FileHash $Extracted).Hash) { throw "Unpacked payload mismatch: $Relative" }
  }
  Copy-Item (Join-Path $Stage 'AppxManifest.xml') $OutputDirectory
  Copy-Item (Join-Path $Stage 'Legal/dependency-inventory.json') $OutputDirectory
  Copy-Item (Join-Path $Work '*.imports.txt') $OutputDirectory
  (Get-FileHash $Package -Algorithm SHA256).Hash.ToLower() + '  ' + [IO.Path]::GetFileName($Package) |
    Set-Content -Encoding ascii (Join-Path $OutputDirectory 'SHA256SUMS')
  $Source = git rev-parse HEAD
  if ($LASTEXITCODE -ne 0) { throw 'Cannot record source revision.' }
  [ordered]@{ source_commit = $Source; windows_sdk = $Sdk.Name; architecture = $Identity.architecture;
    package_version = $Version; cargo_target = $Identity.target; signing = 'unsigned Store submission candidate';
    store_ready = $false; blockers = @('Native installation/certification review', 'Dependency license review', 'OCR models absent')
  } | ConvertTo-Json | Set-Content -Encoding utf8 (Join-Path $OutputDirectory 'build-info.json')
  Write-Output "MSIX created and content-validated: $Package (submission review remains required)"
} finally {
  Pop-Location
  # Work trees contain only generated inputs; retain on failure for diagnosis. No source deletion.
}
