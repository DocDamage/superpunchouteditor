[CmdletBinding()]
param(
    [Parameter(Mandatory = $true)]
    [string]$RomPath,

    [Parameter(Mandatory = $true)]
    [string]$InstallerPath,

    [string]$EmulatorPath,

    [string]$GitCommit,

    [string]$OutputPath = (Join-Path (Get-Location) 'windows-acceptance-evidence.json')
)

$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest

function Resolve-RequiredFile {
    param(
        [Parameter(Mandatory = $true)]
        [string]$Path,

        [Parameter(Mandatory = $true)]
        [string]$Label
    )

    if (-not (Test-Path -LiteralPath $Path -PathType Leaf)) {
        throw "$Label file does not exist: $Path"
    }

    return (Resolve-Path -LiteralPath $Path).Path
}

function Get-FileEvidence {
    param(
        [Parameter(Mandatory = $true)]
        [string]$Path
    )

    $item = Get-Item -LiteralPath $Path
    $hash = Get-FileHash -LiteralPath $Path -Algorithm SHA256

    return [ordered]@{
        fileName = $item.Name
        sizeBytes = $item.Length
        sha256 = $hash.Hash.ToLowerInvariant()
    }
}

$resolvedRom = Resolve-RequiredFile -Path $RomPath -Label 'ROM'
$resolvedInstaller = Resolve-RequiredFile -Path $InstallerPath -Label 'Installer'

$romExtension = [System.IO.Path]::GetExtension($resolvedRom).ToLowerInvariant()
if ($romExtension -notin @('.sfc', '.smc')) {
    throw "ROM must use .sfc or .smc extension; got '$romExtension'."
}

$romEvidence = Get-FileEvidence -Path $resolvedRom
if ($romEvidence.sizeBytes -le 0) {
    throw 'ROM is empty.'
}

$installerEvidence = Get-FileEvidence -Path $resolvedInstaller
if ($installerEvidence.sizeBytes -le 0) {
    throw 'Installer is empty.'
}

$signature = Get-AuthenticodeSignature -LiteralPath $resolvedInstaller
$installerEvidence.authenticodeStatus = [string]$signature.Status
$installerEvidence.signerSubject = if ($null -ne $signature.SignerCertificate) {
    $signature.SignerCertificate.Subject
} else {
    $null
}

$emulatorEvidence = $null
if (-not [string]::IsNullOrWhiteSpace($EmulatorPath)) {
    $resolvedEmulator = Resolve-RequiredFile -Path $EmulatorPath -Label 'Emulator'
    $emulatorEvidence = Get-FileEvidence -Path $resolvedEmulator
}

if ([string]::IsNullOrWhiteSpace($GitCommit)) {
    try {
        $candidateCommit = (& git rev-parse HEAD 2>$null).Trim()
        if ($LASTEXITCODE -eq 0 -and $candidateCommit -match '^[0-9a-fA-F]{40}$') {
            $GitCommit = $candidateCommit.ToLowerInvariant()
        }
    } catch {
        $GitCommit = $null
    }
}

$os = Get-CimInstance Win32_OperatingSystem
$computer = Get-CimInstance Win32_ComputerSystem

$evidence = [ordered]@{
    schemaVersion = 1
    generatedAtUtc = [DateTime]::UtcNow.ToString('o')
    gitCommit = if ([string]::IsNullOrWhiteSpace($GitCommit)) { $null } else { $GitCommit }
    windows = [ordered]@{
        caption = $os.Caption
        version = $os.Version
        buildNumber = $os.BuildNumber
        architecture = $os.OSArchitecture
        manufacturer = $computer.Manufacturer
        model = $computer.Model
    }
    installer = $installerEvidence
    rom = $romEvidence
    emulator = $emulatorEvidence
    acceptance = [ordered]@{
        automatedWindowsSourceGate = 'NOT_RECORDED'
        automatedNsisPackageGate = 'NOT_RECORDED'
        automatedInstallLaunchUninstallGate = 'NOT_RECORDED'
        loadValidation = 'NOT_RUN'
        editUndoRedo = 'NOT_RUN'
        savedRomEquivalence = 'NOT_RUN'
        ipsEquivalence = 'NOT_RUN'
        bpsEquivalence = 'NOT_RUN'
        comparisonPath = 'NOT_RUN'
        projectV2RestoreEquivalence = 'NOT_RUN'
        embeddedEmulatorCurrentRevision = 'NOT_RUN'
        externalEmulatorCurrentRevision = if ($null -eq $emulatorEvidence) { 'N/A' } else { 'NOT_RUN' }
        safeSaveRecovery = 'NOT_RUN'
        defaultUninstallDataPreservation = 'NOT_RUN'
    }
    notes = @(
        'Metadata only. ROM and emulator bytes are never copied into the evidence file.',
        'Complete the acceptance fields after following docs/WINDOWS_ACCEPTANCE.md.'
    )
}

$outputDirectory = Split-Path -Parent $OutputPath
if (-not [string]::IsNullOrWhiteSpace($outputDirectory)) {
    New-Item -ItemType Directory -Force -Path $outputDirectory | Out-Null
}

$evidence | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath $OutputPath -Encoding utf8
$resolvedOutput = (Resolve-Path -LiteralPath $OutputPath).Path

Write-Host 'Windows acceptance metadata captured.'
Write-Host "Evidence: $resolvedOutput"
Write-Host "Installer SHA-256: $($installerEvidence.sha256)"
Write-Host "Installer signature: $($installerEvidence.authenticodeStatus)"
Write-Host "ROM SHA-256: $($romEvidence.sha256)"
if ($null -ne $emulatorEvidence) {
    Write-Host "Emulator SHA-256: $($emulatorEvidence.sha256)"
}
