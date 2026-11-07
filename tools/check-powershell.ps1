<#
.SYNOPSIS
Checks that every PowerShell script in tools\ tells how to run it with
PowerShell 7 when it is started from Windows PowerShell 5.1.

.DESCRIPTION
The scripts in tools\ need PowerShell 7. Each one starts with a check that,
in Windows PowerShell 5.1, prints the command to run it with pwsh and stops.
That check only runs if Windows PowerShell 5.1 can read the whole file, so
the scripts cannot use syntax that only PowerShell 7 knows, such as the ?:
and ?? operators. For every .ps1 file under tools\, this script checks that:

- it has the check;
- Windows PowerShell 5.1 can parse it.

Then it runs check-docs.ps1 in Windows PowerShell 5.1 to see the message.
#>
[CmdletBinding()]
param()

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
$scripts = @(Get-ChildItem -LiteralPath (Join-Path $root 'tools') -Recurse -Filter '*.ps1' -File | ForEach-Object FullName)
$problems = [System.Collections.Generic.List[string]]::new()

foreach ($script in $scripts) {
    if (-not (Select-String -LiteralPath $script -Pattern '^if \(\$PSVersionTable\.PSVersion\.Major -lt 7\) \{$' -Quiet)) {
        $problems.Add("$($script.Substring($root.Length + 1)): it does not check the version of PowerShell.")
    }
}

# Parse every script with the parser of Windows PowerShell 5.1. When a script
# block is passed to powershell.exe, PowerShell sends it encoded, with its
# arguments, and returns the output as objects.
$parse = {
    param([string[]]$Paths)
    foreach ($path in $Paths) {
        $errors = $null
        $null = [System.Management.Automation.Language.Parser]::ParseFile($path, [ref]$null, [ref]$errors)
        foreach ($parseError in $errors) {
            '{0}:{1}: {2}' -f $path, $parseError.Extent.StartLineNumber, $parseError.Message
        }
    }
}
$parseErrors = @(powershell.exe -NoProfile -NonInteractive -Command $parse -args (, $scripts) 2>$null)
if ($LASTEXITCODE -ne 0) {
    throw "powershell.exe failed with exit code $LASTEXITCODE"
}
foreach ($parseError in $parseErrors) {
    $problems.Add("Windows PowerShell 5.1 cannot parse $($parseError.Substring($root.Length + 1))")
}

# Run one script in Windows PowerShell 5.1 to see the message.
$output = powershell.exe -NoProfile -NonInteractive -ExecutionPolicy Bypass -File (Join-Path $PSScriptRoot 'check-docs.ps1') 2>&1 | Out-String
if ($LASTEXITCODE -ne 1 -or $output -notmatch 'This script needs PowerShell 7\. Run it with: pwsh -File ') {
    $problems.Add("In Windows PowerShell 5.1, check-docs.ps1 did not show the message (exit code $LASTEXITCODE): $output")
}

if ($problems.Count -gt 0) {
    $problems | ForEach-Object { Write-Host $_ }
    throw "$($problems.Count) problem(s) with the PowerShell 7 check."
}
Write-Host "OK: the $($scripts.Count) scripts ask for PowerShell 7, and Windows PowerShell 5.1 can read them."
