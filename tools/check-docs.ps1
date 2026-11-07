<#
.SYNOPSIS
Checks the relative links of the Markdown documentation.

.DESCRIPTION
For every relative link in README.md and docs\**\*.md, checks that the target
exists with exactly the same letter case, because paths on GitHub are
case-sensitive, and, when the link has an anchor (#section), that the target
has a heading with that anchor. Anchors are computed like GitHub does.
External links (http, https, mailto...) are not checked.

.PARAMETER Path
Markdown files to check. Defaults to README.md and every file under docs\.
#>
[CmdletBinding()]
param(
    [string[]]$Path
)

$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest

$root = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path
if ($Path) {
    $Path = $Path | ForEach-Object { $ExecutionContext.SessionState.Path.GetUnresolvedProviderPathFromPSPath($_) }
}
else {
    $Path = @(Join-Path $root 'README.md') +
        @(Get-ChildItem -LiteralPath (Join-Path $root 'docs') -Recurse -Filter '*.md' -File | ForEach-Object FullName)
}

$fence = '^\s*(```|~~~)'

# GitHub's heading anchors: lowercase, punctuation removed and spaces replaced
# with hyphens. Letters with accents are kept.
function ConvertTo-Anchor([string]$heading) {
    $text = ($heading -replace '[`*]', '').ToLowerInvariant()
    ($text -replace '[^\p{L}\p{M}\p{N}\p{Pc}\- ]', '') -replace ' ', '-'
}

function Get-Anchor([string]$file) {
    $anchors = [System.Collections.Generic.HashSet[string]]::new()
    $seen = @{}
    $inFence = $false
    foreach ($line in [System.IO.File]::ReadAllLines($file)) {
        if ($line -match $fence) { $inFence = -not $inFence; continue }
        if ($inFence -or $line -notmatch '^#{1,6}\s+(.+?)(\s+#+)?\s*$') { continue }
        $anchor = ConvertTo-Anchor $Matches[1]
        # Repeated headings get a numeric suffix: name, name-1, name-2...
        if ($seen.ContainsKey($anchor)) {
            $seen[$anchor]++
            $anchor = "$anchor-$($seen[$anchor])"
        }
        else {
            $seen[$anchor] = 0
        }
        [void]$anchors.Add($anchor)
    }
    , $anchors
}

# Resolves a relative path one segment at a time, so that a link whose letter
# case differs from the file name fails here as it would on GitHub.
function Resolve-ExactPath([string]$directory, [string]$relative) {
    $current = $directory
    foreach ($segment in $relative -split '[\\/]') {
        if ($segment -in '', '.') { continue }
        if ($segment -eq '..') {
            $current = [System.IO.Path]::GetDirectoryName($current)
            continue
        }
        $entry = Get-ChildItem -LiteralPath $current -Force -ErrorAction SilentlyContinue |
            Where-Object Name -CEQ $segment
        if (-not $entry) { return $null }
        $current = $entry.FullName
    }
    $current
}

$problems = [System.Collections.Generic.List[string]]::new()
$anchorCache = @{}
$checked = 0

foreach ($file in $Path) {
    $directory = [System.IO.Path]::GetDirectoryName($file)
    $name = [System.IO.Path]::GetRelativePath($root, $file)
    $inFence = $false
    $number = 0
    foreach ($line in [System.IO.File]::ReadAllLines($file)) {
        $number++
        if ($line -match $fence) { $inFence = -not $inFence; continue }
        if ($inFence) { continue }

        # Links inside inline code are examples, not links.
        $prose = $line -replace '`[^`]*`', ''
        foreach ($match in [regex]::Matches($prose, '\]\(([^)\s]+)\)')) {
            $target = $match.Groups[1].Value
            if ($target -match '^[a-z][a-z0-9+.-]*:') { continue }
            $checked++

            $pathPart, $anchor = $target -split '#', 2
            $destination = if ($pathPart) {
                Resolve-ExactPath $directory ([uri]::UnescapeDataString($pathPart))
            }
            else {
                $file
            }
            if (-not $destination) {
                $problems.Add("${name}:${number}: target not found (check the letter case): $target")
                continue
            }
            if ($anchor -and (Test-Path -LiteralPath $destination -PathType Leaf)) {
                if (-not $anchorCache.ContainsKey($destination)) {
                    $anchorCache[$destination] = Get-Anchor $destination
                }
                if (-not $anchorCache[$destination].Contains([uri]::UnescapeDataString($anchor))) {
                    $problems.Add("${name}:${number}: anchor not found: $target")
                }
            }
        }
    }
}

if ($problems.Count -gt 0) {
    $problems | ForEach-Object { Write-Host $_ }
    throw "$($problems.Count) broken link(s) in the documentation."
}

Write-Host "OK: $checked relative links checked in $(@($Path).Count) files."
