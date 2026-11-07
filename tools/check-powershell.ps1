<#
.SYNOPSIS
Checks that every PowerShell script in tools\ tells how to run it with
PowerShell 7 when it is started from Windows PowerShell 5.1.

.DESCRIPTION
The scripts in tools\ need PowerShell 7. Each one starts with a check that,
in Windows PowerShell 5.1, prints the command to run it with pwsh and stops.
That check only runs if Windows PowerShell 5.1 can read the whole file and
bind the parameters, so the scripts cannot use syntax that only PowerShell 7
knows, such as the ?: and ?? operators, and the default values of the
parameters cannot use $PSScriptRoot, which Windows PowerShell 5.1 leaves
empty while it binds them. For every .ps1 file under tools\, this script
checks that:

- it has the check;
- no default value of its parameters uses $PSScriptRoot, $PSCommandPath or
  $MyInvocation;
- Windows PowerShell 5.1 can parse it;
- run in Windows PowerShell 5.1, it shows the message and ends with exit
  code 1.
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
    $relative = $script.Substring($root.Length + 1)
    if (-not (Select-String -LiteralPath $script -Pattern '^if \(\$PSVersionTable\.PSVersion\.Major -lt 7\) \{$' -Quiet)) {
        $problems.Add("${relative}: it does not check the version of PowerShell.")
    }
    $ast = [System.Management.Automation.Language.Parser]::ParseFile($script, [ref]$null, [ref]$null)
    if ($ast.ParamBlock) {
        foreach ($parameter in $ast.ParamBlock.Parameters) {
            $variables = if ($parameter.DefaultValue) {
                $parameter.DefaultValue.FindAll({ $args[0] -is [System.Management.Automation.Language.VariableExpressionAst] }, $true) |
                    Where-Object { $_.VariablePath.UserPath -in 'PSScriptRoot', 'PSCommandPath', 'MyInvocation' }
            }
            foreach ($variable in $variables) {
                $problems.Add("${relative}:$($variable.Extent.StartLineNumber): the default value of $($parameter.Name) uses $($variable.Extent.Text), which is empty in Windows PowerShell 5.1: set it after the check.")
            }
        }
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

# Run every script in Windows PowerShell 5.1, all at once, to see the message.
# Scripts with mandatory parameters get values for them: without them,
# Windows PowerShell 5.1 would ask for them before the check runs.
$arguments = @{
    'measure.ps1'      = @('-Name', 'none')
    'run-scenario.ps1' = @('idle', '-Name', 'none')
}
$work = Join-Path ([IO.Path]::GetTempPath()) "meiga-check-powershell-$PID"
New-Item -ItemType Directory -Force -Path $work | Out-Null
try {
    $runs = for ($index = 0; $index -lt $scripts.Count; $index++) {
        $script = $scripts[$index]
        $list = @('-NoProfile', '-NonInteractive', '-ExecutionPolicy', 'Bypass', '-File', "`"$script`"")
        $name = Split-Path $script -Leaf
        if ($arguments.ContainsKey($name)) {
            $list += $arguments[$name]
        }
        $output = Join-Path $work "$index.txt"
        $process = Start-Process powershell.exe -ArgumentList $list -NoNewWindow -PassThru `
            -RedirectStandardOutput $output -RedirectStandardError "$output.err"
        # Without the handle, ExitCode is empty once the process has ended.
        $null = $process.Handle
        [pscustomobject]@{ Script = $script; Process = $process; Output = $output }
    }
    foreach ($run in $runs) {
        $relative = $run.Script.Substring($root.Length + 1)
        if (-not $run.Process.WaitForExit(60000)) {
            $run.Process.Kill()
            $problems.Add("In Windows PowerShell 5.1, $relative was still running after 60 s.")
            continue
        }
        $text = (Get-Content -LiteralPath $run.Output, "$($run.Output).err" -Raw) -join ''
        if ($run.Process.ExitCode -ne 1 -or $text -notmatch 'This script needs PowerShell 7\. Run it with: pwsh -File ') {
            $problems.Add("In Windows PowerShell 5.1, $relative did not show the message (exit code $($run.Process.ExitCode)): $text")
        }
    }
}
finally {
    Remove-Item -LiteralPath $work -Recurse -Force -ErrorAction SilentlyContinue
}

if ($problems.Count -gt 0) {
    $problems | ForEach-Object { Write-Host $_ }
    throw "$($problems.Count) problem(s) with the PowerShell 7 check."
}
Write-Host "OK: the $($scripts.Count) scripts ask for PowerShell 7, and Windows PowerShell 5.1 shows how to run them."
