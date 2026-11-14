# Retake README screenshots with headless Edge against the local rig server.
# Prereq: rig vite server running on 127.0.0.1:31421
#   pnpm exec vite --config scripts/screenshot-rig/vite.config.ts
# Usage: pwsh scripts/screenshot-rig/shoot.ps1 [-Base <url>] [-Out <dir>]
param(
    [string]$Base = "http://127.0.0.1:31421",
    [string]$Out = (Join-Path $PSScriptRoot "output")
)

$ErrorActionPreference = "Stop"
$edge = @(
    "C:\Program Files (x86)\Microsoft\Edge\Application\msedge.exe",
    "C:\Program Files\Microsoft\Edge\Application\msedge.exe"
) | Where-Object { Test-Path $_ } | Select-Object -First 1
if (-not $edge) { throw "msedge.exe not found" }

New-Item -ItemType Directory -Force -Path $Out | Out-Null
$profileDir = Join-Path $env:TEMP "jedi-rig-edge-profile"
if (Test-Path $profileDir) { Remove-Item -Recurse -Force $profileDir }

$shots = @(
    @{ Route = "/chat?rigAction=chat-hero"; Name = "ai-chat" },
    @{ Route = "/skills";     Name = "skills" },
    @{ Route = "/hosts";      Name = "hosts" },
    @{ Route = "/wallpapers"; Name = "wallpapers" },
    @{ Route = "/podcast";    Name = "podcast" },
    @{ Route = "/screenshot"; Name = "screenshot-tool" }
)

function Invoke-Shot {
    param([string]$Url, [string]$Target, [int]$TimeoutSec = 60)
    if (Test-Path $Target) { Remove-Item -Force $Target }
    # A per-shot profile avoids Chromium's process-singleton join, which makes
    # a second headless launch exit silently when another instance lingers.
    $shotProfile = "$profileDir-$([IO.Path]::GetFileNameWithoutExtension($Target))"
    Remove-Item -Recurse -Force $shotProfile -ErrorAction SilentlyContinue
    Start-Process -FilePath $edge -ArgumentList @(
        '--headless',
        '--disable-gpu',
        '--hide-scrollbars',
        '--force-device-scale-factor=2',
        '--window-size=1600,1000',
        '--virtual-time-budget=15000',
        '--no-first-run',
        "--user-data-dir=$shotProfile",
        "--screenshot=$Target",
        $Url
    )
    # msedge.exe detaches immediately; wait for the PNG to land.
    $deadline = (Get-Date).AddSeconds($TimeoutSec)
    while ((Get-Date) -lt $deadline -and -not (Test-Path $Target)) {
        Start-Sleep -Milliseconds 600
    }
    if (Test-Path $Target) {
        Start-Sleep -Milliseconds 800   # let Chromium finish writing
        Write-Host ("   ok  {0}  ({1} bytes)" -f (Split-Path $Target -Leaf), (Get-Item $Target).Length)
    } else {
        Write-Warning "   TIMEOUT: $Target"
    }
    Get-CimInstance Win32_Process -Filter "Name='msedge.exe'" |
        Where-Object { $_.CommandLine -match 'jedi-rig-edge-profile' } |
        ForEach-Object { Stop-Process -Id $_.ProcessId -Force -ErrorAction SilentlyContinue }
    Remove-Item -Recurse -Force $shotProfile -ErrorAction SilentlyContinue
}

foreach ($s in $shots) {
    $target = Join-Path $Out ("{0}.png" -f $s.Name)
    Write-Host ("-> {0}" -f $s.Route)
    Invoke-Shot -Url "$Base$($s.Route)" -Target $target
}

Write-Host ""
Get-ChildItem $Out -Filter *.png | Format-Table Name, Length -AutoSize
