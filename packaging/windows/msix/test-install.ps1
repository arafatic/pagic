<# Disposable Windows runner only: signs a COPY, installs, launches with a PDF, then removes
   only the package/certificates created by this invocation. Never installs onto a user's PC.
#>
param([Parameter(Mandatory)] [string] $Package,
      [Parameter(Mandatory)] [string] $ReportDirectory)
$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest
if (-not $IsWindows -or $env:GITHUB_ACTIONS -ne 'true' -or $env:RUNNER_ENVIRONMENT -ne 'github-hosted') {
  throw 'Installation smoke testing is restricted to disposable GitHub-hosted Windows runners.'
}
$Root = (Resolve-Path (Join-Path $PSScriptRoot '../../..')).Path
$Identity = Get-Content -Raw (Join-Path $PSScriptRoot 'identity.json') | ConvertFrom-Json
if (Get-AppxPackage -Name $Identity.name) { throw 'Refusing to replace an existing Pagic installation.' }
$Temp = Join-Path $env:RUNNER_TEMP ('pagic-msix-' + [guid]::NewGuid())
New-Item -ItemType Directory $Temp | Out-Null
$Cert = $null
$Trusted = $null
$Installed = $null
$Process = $null
try {
  $Cert = New-SelfSignedCertificate -Type Custom -KeyUsage DigitalSignature -Subject $Identity.publisher `
    -CertStoreLocation 'Cert:/CurrentUser/My' -HashAlgorithm SHA256 `
    -TextExtension @('2.5.29.37={text}1.3.6.1.5.5.7.3.3', '2.5.29.19={text}')
  # Sign via the ephemeral certificate store key; no private PFX is exported or uploaded.
  $Public = Join-Path $Temp 'test.cer'
  Export-Certificate -Cert $Cert -FilePath $Public | Out-Null
  $Trusted = Import-Certificate -FilePath $Public -CertStoreLocation 'Cert:/LocalMachine/TrustedPeople'
  $Signed = Join-Path $Temp 'Pagic-test-only.msix'
  Copy-Item -LiteralPath $Package $Signed
  $Tools = Get-ChildItem (Join-Path ${env:ProgramFiles(x86)} 'Windows Kits/10/bin') -Directory |
    Where-Object { $_.Name -match '^10\.0\.\d+\.0$' -and (Test-Path (Join-Path $_.FullName 'x64/signtool.exe')) } |
    Sort-Object { [version]$_.Name } -Descending | Select-Object -First 1
  & (Join-Path $Tools.FullName 'x64/signtool.exe') sign /fd SHA256 /sha1 $Cert.Thumbprint $Signed
  if ($LASTEXITCODE -ne 0) { throw 'Development signing failed.' }
  & (Join-Path $Tools.FullName 'x64/signtool.exe') verify /pa /v $Signed
  if ($LASTEXITCODE -ne 0) { throw 'Development signature verification failed.' }
  Add-AppxPackage -Path $Signed
  $Installed = Get-AppxPackage -Name $Identity.name
  if (-not $Installed -or $Installed.Publisher -ne $Identity.publisher -or $Installed.Architecture -ne 'X64') {
    throw 'Installed identity/publisher/architecture mismatch.'
  }
  $InstalledManifest = Get-AppxPackageManifest -Package $Installed.PackageFullName
  if ($InstalledManifest.Package.Applications.Application.Extensions.Extension.FileTypeAssociation.SupportedFileTypes.FileType -ne '.pdf') {
    throw 'Installed PDF association declaration is missing.'
  }
  # A small persistent-in-artifact PDF, generated without external libraries.
  $Sample = Join-Path $Temp 'Pagic MSIX smoke.pdf'
  python (Join-Path $PSScriptRoot 'test_msix.py') --sample-pdf $Sample
  if ($LASTEXITCODE -ne 0) { throw 'Cannot create smoke PDF.' }
  $Control = Join-Path $Temp 'control.json'
  $Exe = Join-Path $Installed.InstallLocation $Identity.executable
  $Process = Start-Process -FilePath $Exe -ArgumentList @('--new-window', '--control', ('"' + $Control + '"'), ('"' + $Sample + '"')) -PassThru
  $Deadline = (Get-Date).AddSeconds(60)
  do {
    if ($Process.HasExited) { throw "Installed application exited before opening its control endpoint: $($Process.ExitCode)" }
    if (Test-Path $Control) { break }
    Start-Sleep -Milliseconds 250
  } while ((Get-Date) -lt $Deadline)
  if (-not (Test-Path $Control)) { throw 'Installed GUI did not become ready in 60 seconds. Inspect Windows graphics/session logs.' }
  $State = & (Join-Path $Installed.InstallLocation 'pagic-cli.exe') ui --control $Control ui.state
  if ($LASTEXITCODE -ne 0) { throw 'Installed app control query failed.' }
  $Parsed = ($State -join "`n") | ConvertFrom-Json
  if (@($Parsed.documents).Count -ne 1 -or $Parsed.documents[0].pages -ne 1 -or $Parsed.documents[0].name -ne 'Pagic MSIX smoke.pdf') {
    throw 'Installed application did not open the expected one-page PDF.'
  }
  New-Item -ItemType Directory -Force $ReportDirectory | Out-Null
  $State | Set-Content -Encoding utf8 (Join-Path $ReportDirectory 'installed-ui-state.json')
  Copy-Item $Sample $ReportDirectory
  [ordered]@{ status = 'PASS'; package_full_name = $Installed.PackageFullName;
    publisher = $Installed.Publisher; install = 'PASS'; launch = 'PASS'; pdf_open = 'PASS';
    association_declaration = 'PASS'; explorer_activation = 'manual verification pending';
    data_virtualization_and_recto_coexistence = 'manual verification pending'; certification = 'not run'
  } | ConvertTo-Json | Set-Content -Encoding utf8 (Join-Path $ReportDirectory 'installation.json')
} finally {
  if ($Process -and -not $Process.HasExited) { Stop-Process -Id $Process.Id -Force }
  if ($Installed) { Remove-AppxPackage -Package $Installed.PackageFullName }
  if ($Trusted) { Remove-Item -LiteralPath ('Cert:/LocalMachine/TrustedPeople/' + $Trusted.Thumbprint) }
  if ($Cert) { Remove-Item -LiteralPath ('Cert:/CurrentUser/My/' + $Cert.Thumbprint) }
  # Delete only this invocation's temporary certificate, signed copy and control token.
  if (Test-Path $Temp) { Remove-Item -LiteralPath $Temp -Recurse -Force }
}
