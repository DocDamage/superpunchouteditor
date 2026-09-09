[CmdletBinding()]
param(
    [Parameter(Mandatory = $true)]
    [string]$EvidencePath,

    [Parameter(Mandatory = $true)]
    [ValidateSet(
        'automatedWindowsSourceGate',
        'automatedNsisPackageGate',
        'automatedInstallLaunchUninstallGate',
        'loadValidation',
        'editUndoRedo',
        'comparisonPath',
        'embeddedEmulatorCurrentRevision',
        'externalEmulatorCurrentRevision',
        'safeSaveRecovery',
        'defaultUninstallDataPreservation'
    )]
    [string]$Field,

    [Parameter(Mandatory = $true)]
    [ValidateSet('PASS', 'FAIL', 'N/A', 'NOT_RUN', 'NOT_RECORDED')]
    [string]$Status,

    [string]$Note,

    [string]$OutputPath
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

$automatedFields = @(
    'automatedWindowsSourceGate',
    'automatedNsisPackageGate',
    'automatedInstallLaunchUninstallGate'
)

if ($Field -in $automatedFields -and $Status -notin @('PASS', 'FAIL', 'NOT_RECORDED')) {
    throw "Automated gate '$Field' only accepts PASS, FAIL, or NOT_RECORDED."
}

if ($Field -notin $automatedFields -and $Status -eq 'NOT_RECORDED') {
    throw "Manual gate '$Field' uses NOT_RUN rather than NOT_RECORDED."
}

if ($Status -eq 'N/A' -and $Field -ne 'externalEmulatorCurrentRevision') {
    throw "Only externalEmulatorCurrentRevision may be recorded as N/A with this helper."
}

$resolvedEvidence = Resolve-RequiredFile -Path $EvidencePath -Label 'Acceptance evidence'
$evidence = Get-Content -LiteralPath $resolvedEvidence -Raw | ConvertFrom-Json
if ($null -eq $evidence.acceptance) {
    throw 'Acceptance evidence is missing the acceptance section.'
}

if ($null -eq $evidence.acceptance.PSObject.Properties[$Field]) {
    throw "Acceptance evidence does not contain field '$Field'."
}

$evidence.acceptance.$Field = $Status

$history = @()
if ($null -ne $evidence.PSObject.Properties['manualEvidence']) {
    $history = @($evidence.manualEvidence)
}
$history += [ordered]@{
    recordedAtUtc = [DateTime]::UtcNow.ToString('o')
    field = $Field
    status = $Status
    note = if ([string]::IsNullOrWhiteSpace($Note)) { $null } else { $Note }
}
$evidence | Add-Member -NotePropertyName manualEvidence -NotePropertyValue $history -Force

if ([string]::IsNullOrWhiteSpace($OutputPath)) {
    $OutputPath = $resolvedEvidence
}

$outputDirectory = Split-Path -Parent $OutputPath
if (-not [string]::IsNullOrWhiteSpace($outputDirectory)) {
    New-Item -ItemType Directory -Force -Path $outputDirectory | Out-Null
}

$temporaryOutput = "$OutputPath.tmp"
$evidence | ConvertTo-Json -Depth 10 | Set-Content -LiteralPath $temporaryOutput -Encoding utf8
Move-Item -LiteralPath $temporaryOutput -Destination $OutputPath -Force
$resolvedOutput = (Resolve-Path -LiteralPath $OutputPath).Path

Write-Host "Recorded $Field = $Status"
Write-Host "Evidence: $resolvedOutput"
