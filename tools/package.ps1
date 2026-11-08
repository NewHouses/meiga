<#
.SYNOPSIS
Builds meiga in release mode and packages it into a zip file.

.DESCRIPTION
Creates dist\meiga-<version>-x86_64-pc-windows-msvc.zip with meiga.exe, the
ConPTY that meiga uses (conpty.dll and OpenConsole.exe, from
tools\fetch-conpty.ps1), THIRD-PARTY-NOTICES.txt, LICENSE.txt and README.md.
It also copies the ConPTY next to target\release\meiga.exe, which looks for
it in its own folder, so that the tools run meiga with its own ConPTY too.
Requires cargo-about:
    cargo install --locked cargo-about --features cli

.PARAMETER OutDir
Destination folder of the zip file. Defaults to dist\ in the repository.

.PARAMETER NoBuild
Package the existing target\release\meiga.exe without building it again.
#>
[CmdletBinding()]
param(
    [string]$OutDir,
    [switch]$NoBuild
)

# Windows PowerShell 5.1 can read this file but not run it: tell how to run
# it with PowerShell 7. tools/check-powershell.ps1 checks that this works.
if ($PSVersionTable.PSVersion.Major -lt 7) {
    $command = "pwsh -File $(Resolve-Path -Relative $PSCommandPath)"
    if ($PSBoundParameters.Count -gt 0) { $command += ' <the same parameters>' }
    Write-Host "This script needs PowerShell 7. Run it with: $command" -ForegroundColor Yellow
    exit 1
}

$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest

$root = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path
# Not in param(): there, Windows PowerShell 5.1 leaves $PSScriptRoot empty and
# fails before the check above.
if (-not $OutDir) {
    $OutDir = Join-Path $root 'dist'
}
$OutDir = $ExecutionContext.SessionState.Path.GetUnresolvedProviderPathFromPSPath($OutDir)

Push-Location $root
try {
    if (-not $NoBuild) {
        cargo build --release --locked
        if ($LASTEXITCODE -ne 0) {
            throw "cargo build failed with exit code $LASTEXITCODE"
        }
    }

    $metadata = cargo metadata --format-version 1 --no-deps --locked | ConvertFrom-Json
    if ($LASTEXITCODE -ne 0) {
        throw "cargo metadata failed with exit code $LASTEXITCODE"
    }
    $version = ($metadata.packages | Where-Object name -EQ 'meiga').version
    $exe = Join-Path $metadata.target_directory 'release\meiga.exe'
    if (-not (Test-Path -LiteralPath $exe -PathType Leaf)) {
        throw "$exe does not exist. Run the script without -NoBuild."
    }

    $name = "meiga-$version-x86_64-pc-windows-msvc"
    $staging = Join-Path $metadata.target_directory "package\$name"
    if (Test-Path -LiteralPath $staging) {
        Remove-Item -LiteralPath $staging -Recurse -Force
    }
    New-Item -ItemType Directory -Force -Path $staging | Out-Null

    Copy-Item -LiteralPath $exe -Destination $staging
    & (Join-Path $PSScriptRoot 'fetch-conpty.ps1') -Destination $staging, (Split-Path -Parent $exe)
    Copy-Item -LiteralPath (Join-Path $root 'LICENSE') -Destination (Join-Path $staging 'LICENSE.txt')
    Copy-Item -LiteralPath (Join-Path $root 'README.md') -Destination $staging
    & (Join-Path $PSScriptRoot 'third-party-notices.ps1') -OutFile (Join-Path $staging 'THIRD-PARTY-NOTICES.txt')

    New-Item -ItemType Directory -Force -Path $OutDir | Out-Null
    $zip = Join-Path $OutDir "$name.zip"
    Compress-Archive -Path (Join-Path $staging '*') -DestinationPath $zip -Force
}
finally {
    Pop-Location
}

Write-Host "Wrote $zip"
