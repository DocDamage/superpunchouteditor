[CmdletBinding()]
param(
    [Parameter(Mandatory = $true)]
    [string]$EvidencePath,

    [Parameter(Mandatory = $true)]
    [string]$RomPath,

    [Parameter(Mandatory = $true)]
    [string]$SavedRomPath,

    [Parameter(Mandatory = $true)]
    [string]$BpsPatchedRomPath,

    [string]$IpsPatchedRomPath,

    [switch]$IpsUnsupported,

    [string]$ProjectRestoredRomPath,

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

function Get-FileEvidence {
    param(
        [Parameter(Mandatory = $true)]
        [string]$Path
    )

    $item = Get-Item -LiteralPath $Path
    $hash = Get-FileHash -LiteralPath $Path -Algorithm SHA256

    return [ordered]@{
        fileName = $item.Name
        sizeBytes = [int64]$item.Length
        sha256 = $hash.Hash.ToLowerInvariant()
    }
}

function Assert-SameArtifact {
    param(
        [Parameter(Mandatory = $true)]
        [System.Collections.IDictionary]$Expected,

        [Parameter(Mandatory = $true)]
        [System.Collections.IDictionary]$Actual,

        [Parameter(Mandatory = $true)]
        [string]$Label
    )

    if ($Expected.sizeBytes -ne $Actual.sizeBytes) {
        throw "$Label size mismatch: expected $($Expected.sizeBytes) bytes, got $($Actual.sizeBytes)."
    }

    if ($Expected.sha256 -ne $Actual.sha256) {
        throw "$Label SHA-256 mismatch: expected $($Expected.sha256), got $($Actual.sha256)."
    }
}

if ($IpsUnsupported -and -not [string]::IsNullOrWhiteSpace($IpsPatchedRomPath)) {
    throw 'Specify either -IpsPatchedRomPath or -IpsUnsupported, not both.'
}

$resolvedEvidence = Resolve-RequiredFile -Path $EvidencePath -Label 'Acceptance evidence'
$resolvedRom = Resolve-RequiredFile -Path $RomPath -Label 'Source ROM'
$resolvedSavedRom = Resolve-RequiredFile -Path $SavedRomPath -Label 'Saved ROM'
$resolvedBpsRom = Resolve-RequiredFile -Path $BpsPatchedRomPath -Label 'BPS-patched ROM'

$evidence = Get-Content -LiteralPath $resolvedEvidence -Raw | ConvertFrom-Json
if ($null -eq $evidence.rom -or $null -eq $evidence.acceptance) {
    throw 'Acceptance evidence is missing required rom/acceptance sections.'
}

$recordedRom = [ordered]@{
    fileName = [string]$evidence.rom.fileName
    sizeBytes = [int64]$evidence.rom.sizeBytes
    sha256 = ([string]$evidence.rom.sha256).ToLowerInvariant()
}
$currentRom = Get-FileEvidence -Path $resolvedRom
Assert-SameArtifact -Expected $recordedRom -Actual $currentRom -Label 'Source ROM integrity'

$savedRom = Get-FileEvidence -Path $resolvedSavedRom
$bpsRom = Get-FileEvidence -Path $resolvedBpsRom
Assert-SameArtifact -Expected $savedRom -Actual $bpsRom -Label 'BPS output equivalence'

$ipsRom = $null
if (-not [string]::IsNullOrWhiteSpace($IpsPatchedRomPath)) {
    $resolvedIpsRom = Resolve-RequiredFile -Path $IpsPatchedRomPath -Label 'IPS-patched ROM'
    $ipsRom = Get-FileEvidence -Path $resolvedIpsRom
    Assert-SameArtifact -Expected $savedRom -Actual $ipsRom -Label 'IPS output equivalence'
}

$projectRom = $null
if (-not [string]::IsNullOrWhiteSpace($ProjectRestoredRomPath)) {
    $resolvedProjectRom = Resolve-RequiredFile -Path $ProjectRestoredRomPath -Label 'Project-restored ROM'
    $projectRom = Get-FileEvidence -Path $resolvedProjectRom
    Assert-SameArtifact -Expected $savedRom -Actual $projectRom -Label 'Project-v2 restore equivalence'
}

$evidence.acceptance.savedRomEquivalence = 'PASS'
$evidence.acceptance.bpsEquivalence = 'PASS'
if ($null -ne $ipsRom) {
    $evidence.acceptance.ipsEquivalence = 'PASS'
} elseif ($IpsUnsupported) {
    $evidence.acceptance.ipsEquivalence = 'N/A'
}
if ($null -ne $projectRom) {
    $evidence.acceptance.projectV2RestoreEquivalence = 'PASS'
}

$verification = [ordered]@{
    verifiedAtUtc = [DateTime]::UtcNow.ToString('o')
    sourceRomUnchanged = $true
    sourceRom = $currentRom
    savedRom = $savedRom
    bpsPatchedRom = $bpsRom
    ipsPatchedRom = if ($null -ne $ipsRom) { $ipsRom } else { $null }
    ipsUnsupported = [bool]$IpsUnsupported
    projectRestoredRom = if ($null -ne $projectRom) { $projectRom } else { $null }
}
$evidence | Add-Member -NotePropertyName artifactVerification -NotePropertyValue $verification -Force

if ([string]::IsNullOrWhiteSpace($OutputPath)) {
    $directory = Split-Path -Parent $resolvedEvidence
    $baseName = [System.IO.Path]::GetFileNameWithoutExtension($resolvedEvidence)
    $OutputPath = Join-Path $directory "$baseName.verified.json"
}

$outputDirectory = Split-Path -Parent $OutputPath
if (-not [string]::IsNullOrWhiteSpace($outputDirectory)) {
    New-Item -ItemType Directory -Force -Path $outputDirectory | Out-Null
}

$temporaryOutput = "$OutputPath.tmp"
$evidence | ConvertTo-Json -Depth 10 | Set-Content -LiteralPath $temporaryOutput -Encoding utf8
Move-Item -LiteralPath $temporaryOutput -Destination $OutputPath -Force
$resolvedOutput = (Resolve-Path -LiteralPath $OutputPath).Path

Write-Host 'Canonical artifact verification passed.'
Write-Host "Evidence: $resolvedOutput"
Write-Host "Source ROM unchanged: $($currentRom.sha256)"
Write-Host "Canonical edited ROM: $($savedRom.sha256)"
Write-Host 'BPS output: byte-equivalent'
if ($null -ne $ipsRom) {
    Write-Host 'IPS output: byte-equivalent'
} elseif ($IpsUnsupported) {
    Write-Host 'IPS output: explicitly unsupported/N/A'
}
if ($null -ne $projectRom) {
    Write-Host 'Project-v2 restored output: byte-equivalent'
}
