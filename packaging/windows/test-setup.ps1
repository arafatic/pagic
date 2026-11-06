# Exercise the standalone installer only on a disposable GitHub-hosted Windows runner.
param(
  [Parameter(Mandatory)][string] $Installer,
  [Parameter(Mandatory)][string] $LogPath
)
$ErrorActionPreference = 'Stop'
if (-not $IsWindows -or $env:RUNNER_ENVIRONMENT -ne 'github-hosted' -or $env:GITHUB_ACTIONS -ne 'true') {
  throw 'Run standalone install validation on a disposable GitHub-hosted Windows runner.'
}
$Installer = (Resolve-Path -LiteralPath $Installer).Path
$LogPath = [IO.Path]::GetFullPath($LogPath)
$Temp = (New-Item -ItemType Directory -Path (Join-Path $env:RUNNER_TEMP ('pagic-setup-' + [guid]::NewGuid()))).FullName
$AppDir = Join-Path $Temp 'Pagic'
$Uninstaller = Join-Path $AppDir 'unins000.exe'
$Gui = $null

function Invoke-SetupProcess([string] $Executable, [string[]] $Arguments) {
  $Process = Start-Process -FilePath $Executable -ArgumentList $Arguments -PassThru
  try {
    if (-not $Process.WaitForExit(120000)) { throw "Timed out: $Executable" }
    if ($Process.ExitCode -ne 0) { throw "$Executable failed with exit code $($Process.ExitCode)" }
  } finally {
    if (-not $Process.HasExited) { $Process.Kill($true); $null = $Process.WaitForExit(5000) }
    $Process.Dispose()
  }
}

try {
  Invoke-SetupProcess $Installer @('/VERYSILENT', '/SUPPRESSMSGBOXES', '/NORESTART', "/DIR=`"$AppDir`"", "/LOG=`"$LogPath`"")
  foreach ($Relative in 'pagic.exe', 'pagic-cli.exe', 'NOTICE', 'Legal/dependency-inventory.json') {
    if (-not (Test-Path -LiteralPath (Join-Path $AppDir $Relative) -PathType Leaf)) {
      throw "Standalone installation is missing $Relative"
    }
  }
  $Control = Join-Path $Temp 'control.json'
  $Gui = Start-Process -FilePath (Join-Path $AppDir 'pagic.exe') -ArgumentList @('--new-window', '--control', "`"$Control`"") -PassThru
  $Deadline = (Get-Date).AddSeconds(60)
  do {
    if ($Gui.HasExited) { throw "Standalone GUI exited with code $($Gui.ExitCode)" }
    $Gui.Refresh()
    if ($Gui.MainWindowHandle -ne [IntPtr]::Zero -and (Test-Path -LiteralPath $Control)) { break }
    Start-Sleep -Milliseconds 250
  } while ((Get-Date) -lt $Deadline)
  if ($Gui.MainWindowHandle -eq [IntPtr]::Zero -or -not (Test-Path -LiteralPath $Control)) {
    throw 'Standalone GUI did not become ready in 60 seconds.'
  }
  Write-Output 'Standalone installation and GUI launch passed.'
} finally {
  if ($Gui) {
    if (-not $Gui.HasExited) { $Gui.Kill($true); $null = $Gui.WaitForExit(5000) }
    $Gui.Dispose()
  }
  if (Test-Path -LiteralPath $Uninstaller) {
    Invoke-SetupProcess $Uninstaller @('/VERYSILENT', '/SUPPRESSMSGBOXES', '/NORESTART')
  }
  Remove-Item -LiteralPath $Temp -Recurse -Force
}
