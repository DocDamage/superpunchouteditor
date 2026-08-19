[CmdletBinding()]
param(
    [Parameter(Mandatory = $true)]
    [string]$EvidencePath,

    [string]$OutputPath,

    [switch]$RequireCompleteLocalAcceptance,

    [switch]$RequireSignedInstaller
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

function Get-Status {
    param(
        [Parameter(Mandatory = $true)]
        [object]$Acceptance,

        [Parameter(Mandatory = $true)]
        [string]$Field
    )

    if ($null -eq $Acceptance.PSObject.Properties[$Field]) {
        return 'MISSING'
    }

    $value = [string]$Acceptance.$Field
    if ([string]::IsNullOrWhiteSpace($value)) {
        return 'MISSING'
    }

    return $value
}

function Escape-Markdown {
    param([AllowNull()][string]$Text)

    if ($null -eq $Text) {
        return ''
    }

    return $Text.Replace('|', '\|').Replace("`r", '').Replace("`n", '<br>')
}

$resolvedEvidence = Resolve-RequiredFile -Path $EvidencePath -Label 'Acceptance evidence'
$evidence = Get-Content -LiteralPath $resolvedEvidence -Raw | ConvertFrom-Json

if ($null -eq $evidence.acceptance -or $null -eq $evidence.rom -or $null -eq $evidence.installer) {
    throw 'Acceptance evidence is missing required acceptance/rom/installer sections.'
}

$requirements = @(
    [ordered]@{ label = 'Automated Windows source gate'; field = 'automatedWindowsSourceGate'; allowed = @('PASS') },
    [ordered]@{ label = 'Automated NSIS package gate'; field = 'automatedNsisPackageGate'; allowed = @('PASS') },
    [ordered]@{ label = 'Automated install/launch/uninstall gate'; field = 'automatedInstallLaunchUninstallGate'; allowed = @('PASS') },
    [ordered]@{ label = 'Load/validation'; field = 'loadValidation'; allowed = @('PASS') },
    [ordered]@{ label = 'Edit + undo/redo'; field = 'editUndoRedo'; allowed = @('PASS') },
    [ordered]@{ label = 'Saved-ROM equivalence'; field = 'savedRomEquivalence'; allowed = @('PASS') },
    [ordered]@{ label = 'IPS equivalence'; field = 'ipsEquivalence'; allowed = @('PASS', 'N/A') },
    [ordered]@{ label = 'BPS equivalence'; field = 'bpsEquivalence'; allowed = @('PASS') },
    [ordered]@{ label = 'Comparison path'; field = 'comparisonPath'; allowed = @('PASS') },
    [ordered]@{ label = 'Project-v2 restore equivalence'; field = 'projectV2RestoreEquivalence'; allowed = @('PASS') },
    [ordered]@{ label = 'Embedded emulator current revision'; field = 'embeddedEmulatorCurrentRevision'; allowed = @('PASS') },
    [ordered]@{ label = 'External emulator current revision'; field = 'externalEmulatorCurrentRevision'; allowed = @('PASS', 'N/A') },
    [ordered]@{ label = 'Safe-save/recovery'; field = 'safeSaveRecovery'; allowed = @('PASS') },
    [ordered]@{ label = 'Default-uninstall data preservation'; field = 'defaultUninstallDataPreservation'; allowed = @('PASS') }
)

$rows = @()
$localComplete = $true
foreach ($requirement in $requirements) {
    $status = Get-Status -Acceptance $evidence.acceptance -Field $requirement.field
    $passed = $status -in $requirement.allowed
    if (-not $passed) {
        $localComplete = $false
    }

    $rows += [ordered]@{
        label = $requirement.label
        status = $status
        result = if ($passed) { 'OK' } else { 'BLOCKED' }
    }
}

$sourceRomUnchanged = $false
if ($null -ne $evidence.PSObject.Properties['artifactVerification']) {
    $sourceRomUnchanged = [bool]$evidence.artifactVerification.sourceRomUnchanged
}
if (-not $sourceRomUnchanged) {
    $localComplete = $false
}

$signatureStatus = if ($null -ne $evidence.installer.PSObject.Properties['authenticodeStatus']) {
    [string]$evidence.installer.authenticodeStatus
} else {
    'NOT_RECORDED'
}
$installerSigned = $signatureStatus -eq 'Valid'

$localStatus = if ($localComplete) { 'PASS' } else { 'INCOMPLETE' }
$signingStatus = if ($installerSigned) { 'PASS' } else { 'INCOMPLETE' }

$lines = @(
    '# Windows Acceptance Evidence Summary',
    '',
    "- Local canonical acceptance: **$localStatus**",
    "- Installer Authenticode: **$signatureStatus**",
    '- Tauri updater signature/public-key match: verified by the tagged production release workflow, not by this local evidence file.',
    '',
    '## Candidate metadata',
    '',
    "- Git commit: `$(Escape-Markdown ([string]$evidence.gitCommit))`",
    "- Installer: $(Escape-Markdown ([string]$evidence.installer.fileName))",
    "- Installer SHA-256: `$(Escape-Markdown ([string]$evidence.installer.sha256))`",
    "- ROM: $(Escape-Markdown ([string]$evidence.rom.fileName))",
    "- ROM size: $([int64]$evidence.rom.sizeBytes) bytes",
    "- ROM SHA-256: `$(Escape-Markdown ([string]$evidence.rom.sha256))`",
    "- Source ROM unchanged since preflight: **$(if ($sourceRomUnchanged) { 'PASS' } else { 'NOT VERIFIED' })**",
    '',
    '## Acceptance matrix',
    '',
    '| Gate | Status | Result |',
    '|---|---|---|'
)

foreach ($row in $rows) {
    $lines += "| $(Escape-Markdown $row.label) | $(Escape-Markdown $row.status) | $($row.result) |"
}

if ($null -ne $evidence.emulator) {
    $lines += ''
    $lines += '## External emulator metadata'
    $lines += ''
    $lines += "- Emulator: $(Escape-Markdown ([string]$evidence.emulator.fileName))"
    $lines += "- Emulator SHA-256: `$(Escape-Markdown ([string]$evidence.emulator.sha256))`"
}

if ($null -ne $evidence.PSObject.Properties['manualEvidence'] -and @($evidence.manualEvidence).Count -gt 0) {
    $lines += ''
    $lines += '## Manual evidence log'
    $lines += ''
    $lines += '| Recorded (UTC) | Field | Status | Note |'
    $lines += '|---|---|---|---|'
    foreach ($entry in @($evidence.manualEvidence)) {
        $lines += "| $(Escape-Markdown ([string]$entry.recordedAtUtc)) | $(Escape-Markdown ([string]$entry.field)) | $(Escape-Markdown ([string]$entry.status)) | $(Escape-Markdown ([string]$entry.note)) |"
    }
}

$lines += ''
$lines += '## Release-readiness interpretation'
$lines += ''
if ($localComplete) {
    $lines += '- Local real-ROM canonical-output acceptance is complete.'
} else {
    $lines += '- Local real-ROM canonical-output acceptance still has blocked or unrecorded gates.'
}
if ($installerSigned) {
    $lines += '- Installer Authenticode is valid.'
} else {
    $lines += '- Installer Authenticode is not recorded as Valid; this evidence is not sufficient for a signed stable release.'
}
$lines += '- Production Tauri updater signing remains authoritative in `.github/workflows/release.yml`.'

if ([string]::IsNullOrWhiteSpace($OutputPath)) {
    $directory = Split-Path -Parent $resolvedEvidence
    $baseName = [System.IO.Path]::GetFileNameWithoutExtension($resolvedEvidence)
    $OutputPath = Join-Path $directory "$baseName.summary.md"
}

$outputDirectory = Split-Path -Parent $OutputPath
if (-not [string]::IsNullOrWhiteSpace($outputDirectory)) {
    New-Item -ItemType Directory -Force -Path $outputDirectory | Out-Null
}

$lines | Set-Content -LiteralPath $OutputPath -Encoding utf8
$resolvedOutput = (Resolve-Path -LiteralPath $OutputPath).Path

Write-Host "Acceptance summary: $resolvedOutput"
Write-Host "Local canonical acceptance: $localStatus"
Write-Host "Installer signing: $signingStatus ($signatureStatus)"

$failures = @()
if ($RequireCompleteLocalAcceptance -and -not $localComplete) {
    $failures += 'local canonical acceptance is incomplete'
}
if ($RequireSignedInstaller -and -not $installerSigned) {
    $failures += "installer Authenticode is '$signatureStatus' instead of Valid"
}

if ($failures.Count -gt 0) {
    throw "Acceptance requirements not met: $($failures -join '; ')."
}
