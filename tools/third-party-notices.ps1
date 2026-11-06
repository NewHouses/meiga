<#
.SYNOPSIS
Generates THIRD-PARTY-NOTICES.txt for the crates that meiga ships.

.DESCRIPTION
Runs cargo-about with about.toml and tools/about.hbs. Install cargo-about with:
    cargo install --locked cargo-about --features cli

.PARAMETER OutFile
Destination file. Defaults to target\THIRD-PARTY-NOTICES.txt.
#>
[CmdletBinding()]
param(
    [string]$OutFile = (Join-Path $PSScriptRoot '..\target\THIRD-PARTY-NOTICES.txt')
)

$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest

$root = Resolve-Path (Join-Path $PSScriptRoot '..')
$OutFile = [System.IO.Path]::GetFullPath($OutFile)
New-Item -ItemType Directory -Force -Path (Split-Path $OutFile) | Out-Null

Push-Location $root
try {
    cargo about generate --workspace --locked --fail --output-file $OutFile tools/about.hbs
    if ($LASTEXITCODE -ne 0) {
        throw "cargo about failed with exit code $LASTEXITCODE"
    }
}
finally {
    Pop-Location
}

Write-Host "Wrote $OutFile"
